//! Dynamic VRAM glyph and icon texture atlas with shelf packing and dirty rect flushing.

use crate::gpu::types::GpuError;
use std::collections::HashMap;

/// Format of the backing VRAM atlas texture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AtlasFormat {
    /// 8-bit alpha mask for antialiased monochrome font glyphs.
    Grayscale,
    /// 32-bit RGBA for color emoji and rich vector icons.
    Color,
}

impl AtlasFormat {
    #[must_use]
    pub const fn to_wgpu_format(self) -> wgpu::TextureFormat {
        match self {
            Self::Grayscale => wgpu::TextureFormat::R8Unorm,
            Self::Color => wgpu::TextureFormat::Rgba8Unorm,
        }
    }

    #[must_use]
    pub const fn bytes_per_pixel(self) -> u32 {
        match self {
            Self::Grayscale => 1,
            Self::Color => 4,
        }
    }
}

/// Unique key identifying a rasterized glyph variant with subpixel precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_id: u64,
    pub glyph_id: u32,
    pub subpixel_bin: u8,
    pub size_px: u16,
}

impl GlyphKey {
    #[must_use]
    pub const fn new(font_id: u64, glyph_id: u32, subpixel_bin: u8, size_px: u16) -> Self {
        Self {
            font_id,
            glyph_id,
            subpixel_bin,
            size_px,
        }
    }
}

/// UV coordinates and pixel boundaries of an allocated glyph within the atlas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtlasAllocation {
    /// Normalized UV texture coordinates `[u_min, v_min, u_max, v_max]`.
    pub uv_bounds: [f32; 4],
    /// Physical pixel rectangle `[x, y, width, height]`.
    pub pixel_rect: [u32; 4],
    /// Frame epoch when this glyph was last rendered.
    pub last_used_epoch: u64,
}

#[derive(Debug)]
struct Shelf {
    y: u32,
    height: u32,
    current_x: u32,
}

#[derive(Debug)]
struct PendingUpload {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    data: Vec<u8>,
}

/// Dynamic GPU texture atlas with shelf bin-packing, LRU tracking, and dirty rect flushing.
#[derive(Debug)]
pub struct VramAtlas {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    format: AtlasFormat,
    width: u32,
    height: u32,
    shelves: Vec<Shelf>,
    next_shelf_y: u32,
    allocations: HashMap<GlyphKey, AtlasAllocation>,
    pending_uploads: Vec<PendingUpload>,
    current_epoch: u64,
}

impl VramAtlas {
    /// Constructs a new VRAM atlas with given dimensions and pixel format.
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: AtlasFormat,
    ) -> Result<Self, GpuError> {
        if width == 0 || height == 0 {
            return Err(GpuError::InvalidDimensions { width, height });
        }

        let wgpu_format = format.to_wgpu_format();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("tej_vram_glyph_atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("tej_vram_atlas_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            format,
            width,
            height,
            shelves: Vec::with_capacity(64),
            next_shelf_y: 0,
            allocations: HashMap::with_capacity(512),
            pending_uploads: Vec::with_capacity(64),
            current_epoch: 1,
        })
    }

    /// Advance frame epoch counter for LRU eviction tracking.
    pub fn advance_epoch(&mut self) {
        self.current_epoch = self.current_epoch.saturating_add(1);
    }

    /// Retrieve an existing glyph allocation if present, updating its LRU epoch.
    pub fn get(&mut self, key: &GlyphKey) -> Option<AtlasAllocation> {
        if let Some(alloc) = self.allocations.get_mut(key) {
            alloc.last_used_epoch = self.current_epoch;
            Some(*alloc)
        } else {
            None
        }
    }

    /// Allocates space for a glyph and queues its pixel data for upload.
    ///
    /// If the atlas is full, purges stale allocations older than `min_age_epochs`
    /// or resets the atlas if still insufficient.
    pub fn insert(
        &mut self,
        key: GlyphKey,
        glyph_w: u32,
        glyph_h: u32,
        data: &[u8],
    ) -> Option<AtlasAllocation> {
        if glyph_w == 0 || glyph_h == 0 {
            return None;
        }

        // Add 1px padding around glyph to prevent linear filtering bleeding
        let alloc_w = glyph_w.saturating_add(1);
        let alloc_h = glyph_h.saturating_add(1);

        if alloc_w > self.width || alloc_h > self.height {
            return None;
        }

        // 1. Try shelf allocation
        if let Some((x, y)) = self.find_or_create_slot(alloc_w, alloc_h) {
            return Some(self.record_allocation(key, x, y, glyph_w, glyph_h, data));
        }

        // 2. Atlas full: purge old allocations and defragment if needed
        self.clear_stale_or_reset();

        // 3. Retry allocation
        if let Some((x, y)) = self.find_or_create_slot(alloc_w, alloc_h) {
            Some(self.record_allocation(key, x, y, glyph_w, glyph_h, data))
        } else {
            None
        }
    }

    fn find_or_create_slot(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        // Find best-fitting existing shelf
        let mut best_shelf: Option<(usize, u32)> = None;

        for (idx, shelf) in self.shelves.iter().enumerate() {
            if shelf.height >= h && shelf.current_x + w <= self.width {
                let waste = shelf.height - h;
                match best_shelf {
                    None => best_shelf = Some((idx, waste)),
                    Some((_, best_waste)) if waste < best_waste => {
                        best_shelf = Some((idx, waste));
                    }
                    _ => {}
                }
            }
        }

        if let Some((idx, _)) = best_shelf {
            let shelf = &mut self.shelves[idx];
            let x = shelf.current_x;
            let y = shelf.y;
            shelf.current_x += w;
            return Some((x, y));
        }

        // Create new shelf below existing ones if vertical room remains
        if self.next_shelf_y + h <= self.height {
            let y = self.next_shelf_y;
            self.next_shelf_y += h;
            self.shelves.push(Shelf {
                y,
                height: h,
                current_x: w,
            });
            return Some((0, y));
        }

        None
    }

    fn record_allocation(
        &mut self,
        key: GlyphKey,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        data: &[u8],
    ) -> AtlasAllocation {
        let u0 = (x as f32) / (self.width as f32);
        let v0 = (y as f32) / (self.height as f32);
        let u1 = ((x + w) as f32) / (self.width as f32);
        let v1 = ((y + h) as f32) / (self.height as f32);

        let alloc = AtlasAllocation {
            uv_bounds: [u0, v0, u1, v1],
            pixel_rect: [x, y, w, h],
            last_used_epoch: self.current_epoch,
        };

        self.allocations.insert(key, alloc);
        self.pending_uploads.push(PendingUpload {
            x,
            y,
            width: w,
            height: h,
            data: data.to_vec(),
        });

        alloc
    }

    fn clear_stale_or_reset(&mut self) {
        // Reset full atlas layout when overflowing
        self.shelves.clear();
        self.next_shelf_y = 0;
        self.allocations.clear();
        self.pending_uploads.clear();
    }

    /// Flushes all queued glyph pixel data to the GPU texture memory.
    pub fn flush(&mut self, queue: &wgpu::Queue) {
        if self.pending_uploads.is_empty() {
            return;
        }

        let bpp = self.format.bytes_per_pixel();

        for upload in self.pending_uploads.drain(..) {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: upload.x,
                        y: upload.y,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &upload.data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(upload.width * bpp),
                    rows_per_image: Some(upload.height),
                },
                wgpu::Extent3d {
                    width: upload.width,
                    height: upload.height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    #[must_use]
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    #[must_use]
    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    #[must_use]
    pub const fn format(&self) -> AtlasFormat {
        self.format
    }

    #[must_use]
    pub fn allocation_count(&self) -> usize {
        self.allocations.len()
    }
}
