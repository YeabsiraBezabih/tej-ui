//! Vector path tessellation, immediate-mode 2D canvas, and GPU vector render pipeline.

pub mod canvas_context;
pub mod path;
pub mod pipeline;
pub mod tessellator;

pub use canvas_context::{Canvas2DContext, FillRule, Transform2D};
pub use path::{PathBuilder, SvgParseError, SvgPathParser, VectorPath};
pub use pipeline::VectorPipeline;
pub use tessellator::{PathTessellator, TessellatedMesh, VectorVertex};
