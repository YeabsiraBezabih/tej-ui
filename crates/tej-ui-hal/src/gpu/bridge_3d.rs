//! Zero-copy bridge for compositing 3D viewports and external GPU textures into the 2D UI tree.

use crate::gpu::context::GpuContext;
use crate::gpu::types::GpuError;

/// GPU texture bridge allowing 3D scenes or camera viewports to render directly to a shared texture.
#[derive(Debug)]
pub struct GpuTextureBridge {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}

impl GpuTextureBridge {
    pub const DEFAULT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    /// Construct a new shared 3D-to-2D GPU texture bridge.
    pub fn new(
        ctx: &GpuContext,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Result<Self, GpuError> {
        if width == 0 || height == 0 {
            return Err(GpuError::InvalidDimensions { width, height });
        }

        let texture = ctx.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("tej_3d_bridge_texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = ctx.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("tej_3d_bridge_sampler"),
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
            width,
            height,
            format,
        })
    }

    /// Resize shared texture if viewport dimensions change.
    pub fn resize(&mut self, ctx: &GpuContext, width: u32, height: u32) -> Result<(), GpuError> {
        if self.width == width && self.height == height {
            return Ok(());
        }
        *self = Self::new(ctx, width, height, self.format)?;
        Ok(())
    }

    /// Upload CPU-rendered color buffer (e.g. from software rasterizer) into the GPU texture.
    pub fn upload_rgba_pixels(&self, queue: &wgpu::Queue, pixels: &[u32]) {
        let bytes: &[u8] = bytemuck::cast_slice(pixels);
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.width * 4),
                rows_per_image: Some(self.height),
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }

    #[must_use]
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
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
    pub const fn format(&self) -> wgpu::TextureFormat {
        self.format
    }
}
