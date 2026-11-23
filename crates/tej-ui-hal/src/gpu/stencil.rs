//! Hardware depth-stencil buffer target and arbitrary non-rectangular clip mask support.

use crate::gpu::context::GpuContext;
use crate::gpu::types::GpuError;

/// Hardware stencil operation mode for complex clipping paths (e.g. CSS `clip-path`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StencilMode {
    /// Stencil testing disabled; standard color output.
    Disabled,
    /// Writes reference mask into stencil buffer without outputting color.
    WriteMask(u32),
    /// Tests fragments against stencil reference; only matching fragments pass.
    TestMask(u32),
}

/// Depth-stencil texture target maintaining hardware stencil planes across frame resizes.
#[derive(Debug)]
pub struct DepthStencilTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}

impl DepthStencilTarget {
    pub const DEFAULT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;

    /// Constructs a new depth-stencil target.
    pub fn new(ctx: &GpuContext, width: u32, height: u32) -> Result<Self, GpuError> {
        if width == 0 || height == 0 {
            return Err(GpuError::InvalidDimensions { width, height });
        }

        let format = Self::DEFAULT_FORMAT;
        let texture = ctx.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("tej_depth_stencil_target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self {
            texture,
            view,
            width,
            height,
            format,
        })
    }

    /// Resize depth-stencil target if dimensions have changed.
    pub fn resize(&mut self, ctx: &GpuContext, width: u32, height: u32) -> Result<(), GpuError> {
        if self.width == width && self.height == height {
            return Ok(());
        }
        *self = Self::new(ctx, width, height)?;
        Ok(())
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
