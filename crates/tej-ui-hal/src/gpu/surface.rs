//! Target surfaces for rendering: offscreen headless buffers and OS window surfaces.

use crate::gpu::context::GpuContext;
use crate::gpu::types::GpuError;

/// Offscreen target supporting GPU rendering and zero-copy readback into host memory.
#[derive(Debug)]
pub struct HeadlessTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    readback_buffer: wgpu::Buffer,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    padded_bytes_per_row: u32,
}

impl HeadlessTarget {
    /// Create a new offscreen headless target.
    pub fn new(
        ctx: &GpuContext,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Result<Self, GpuError> {
        if width == 0 || height == 0 {
            return Err(GpuError::InvalidDimensions { width, height });
        }

        let unpadded_bytes = width.saturating_mul(4);
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded_bytes.div_ceil(align) * align;

        let texture = ctx.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("tej_headless_texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let buffer_size = u64::from(padded_bytes_per_row).saturating_mul(u64::from(height));
        let readback_buffer = ctx.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("tej_headless_readback"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Ok(Self {
            texture,
            view,
            readback_buffer,
            width,
            height,
            format,
            padded_bytes_per_row,
        })
    }

    /// Resize the offscreen target. No-op if dimensions match current size.
    pub fn resize(&mut self, ctx: &GpuContext, width: u32, height: u32) -> Result<(), GpuError> {
        if self.width == width && self.height == height {
            return Ok(());
        }
        *self = Self::new(ctx, width, height, self.format)?;
        Ok(())
    }

    /// Reads back RGBA8 pixel data from the GPU into a reused CPU buffer.
    pub fn readback_rgba(&self, ctx: &GpuContext, out: &mut Vec<u8>) -> Result<(), GpuError> {
        let mut encoder = ctx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("tej_readback_encoder"),
            });

        let copy_size = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            copy_size,
        );

        ctx.queue().submit(Some(encoder.finish()));

        let slice = self.readback_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });

        ctx.device().poll(wgpu::Maintain::wait()).panic_on_timeout();

        rx.recv()
            .map_err(|e| GpuError::ReadbackFailed(e.to_string()))?
            .map_err(|e| GpuError::ReadbackFailed(e.to_string()))?;

        let mapped = slice.get_mapped_range();
        let unpadded_row_size = (self.width as usize).saturating_mul(4);
        let total_bytes = unpadded_row_size.saturating_mul(self.height as usize);

        out.clear();
        out.reserve(total_bytes);

        let padded_row = self.padded_bytes_per_row as usize;
        for y in 0..self.height as usize {
            let start = y * padded_row;
            let end = start + unpadded_row_size;
            out.extend_from_slice(&mapped[start..end]);
        }

        drop(mapped);
        self.readback_buffer.unmap();
        Ok(())
    }

    #[must_use]
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
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

/// OS window surface backed by a hardware swapchain.
#[derive(Debug)]
pub struct WindowSurface<'window> {
    surface: wgpu::Surface<'window>,
    config: wgpu::SurfaceConfiguration,
}

impl<'window> WindowSurface<'window> {
    /// Create a new window surface configured for display.
    pub fn new<W>(
        ctx: &GpuContext,
        window: W,
        width: u32,
        height: u32,
        present_mode: wgpu::PresentMode,
    ) -> Result<Self, GpuError>
    where
        W: Into<wgpu::SurfaceTarget<'window>>,
    {
        let surface = ctx
            .instance()
            .create_surface(window)
            .map_err(|e| GpuError::SurfaceCreationFailed(e.to_string()))?;

        let caps = surface.get_capabilities(ctx.adapter());
        let format = caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or_else(|| {
                caps.formats
                    .first()
                    .copied()
                    .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb)
            });

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
        };

        surface.configure(ctx.device(), &config);

        Ok(Self { surface, config })
    }

    /// Resize window surface and reconfigure swapchain.
    pub fn resize(&mut self, ctx: &GpuContext, width: u32, height: u32) {
        if self.config.width == width && self.config.height == height {
            return;
        }
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(ctx.device(), &self.config);
    }

    /// Acquires the current frame surface texture, recovering once from `Lost`/`Outdated`.
    pub fn acquire_frame(&mut self, ctx: &GpuContext) -> Result<wgpu::SurfaceTexture, GpuError> {
        match self.surface.get_current_texture() {
            Ok(frame) => Ok(frame),
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(ctx.device(), &self.config);
                self.surface
                    .get_current_texture()
                    .map_err(|e| GpuError::SurfaceAcquireFailed(e.to_string()))
            }
            Err(e) => Err(GpuError::SurfaceAcquireFailed(e.to_string())),
        }
    }

    #[must_use]
    pub const fn width(&self) -> u32 {
        self.config.width
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.config.height
    }

    #[must_use]
    pub const fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }
}

/// Unified render target abstraction for windowed or offscreen presentation.
#[derive(Debug)]
pub enum RenderTarget<'window> {
    Window(WindowSurface<'window>),
    Headless(HeadlessTarget),
}

impl RenderTarget<'_> {
    #[must_use]
    pub const fn width(&self) -> u32 {
        match self {
            Self::Window(w) => w.width(),
            Self::Headless(h) => h.width(),
        }
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        match self {
            Self::Window(w) => w.height(),
            Self::Headless(h) => h.height(),
        }
    }

    #[must_use]
    pub const fn format(&self) -> wgpu::TextureFormat {
        match self {
            Self::Window(w) => w.format(),
            Self::Headless(h) => h.format(),
        }
    }

    pub fn resize(&mut self, ctx: &GpuContext, width: u32, height: u32) -> Result<(), GpuError> {
        match self {
            Self::Window(w) => {
                w.resize(ctx, width, height);
                Ok(())
            }
            Self::Headless(h) => h.resize(ctx, width, height),
        }
    }
}
