//! Hardware-accelerated GPU subsystem based on WGPU.

pub mod atlas;
pub mod bridge_3d;
pub mod buffer_ring;
pub mod clip_stack;
pub mod compositor;
pub mod context;
pub mod image_pipeline;
pub mod quad_pipeline;
pub mod stencil;
pub mod surface;
pub mod text_pipeline;
pub mod types;
pub mod vector;

pub use atlas::{AtlasAllocation, AtlasFormat, GlyphKey, VramAtlas};
pub use bridge_3d::GpuTextureBridge;
pub use buffer_ring::{DrawBatch, InstanceBufferPool};
pub use clip_stack::{ClipEntry, ClipStack};
pub use compositor::{FrameStats, GpuCompositor};
pub use context::{AdapterCapabilities, GpuContext};
pub use image_pipeline::{
    IMAGE_SHADER_WGSL, ImageBatch, ImageBufferPool, ImageInstance, ImagePipeline,
};
pub use stencil::{DepthStencilTarget, StencilMode};
pub use surface::{HeadlessTarget, RenderTarget, WindowSurface};
pub use text_pipeline::{GlyphInstance, GlyphKind, TextBatch, TextPipeline};
pub use types::{
    BackendConfig, GpuError, PremulColor, PrimitiveKind, QuadInstance, ScissorRect, ViewportUniform,
};
pub use vector::{
    Canvas2DContext, FillRule, PathBuilder, PathTessellator, SvgParseError, SvgPathParser,
    TessellatedMesh, Transform2D, VectorPath, VectorPipeline, VectorVertex,
};
