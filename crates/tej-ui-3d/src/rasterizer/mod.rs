//! Rasterization subsystem providing clipping, interpolation, and depth buffering.

pub mod clipping;
pub mod interpolator;
pub mod resolve;
pub mod shadows;
pub mod tiling;
pub mod vis_buffer;

pub use clipping::{ClipVertex, ClippedTriangles, clip_triangle_near_plane};
pub use interpolator::PerspectiveBarycentrics;
pub use resolve::{ResolvedTriangle, ShadingContext, resolve_tile, shade_fragment};
pub use shadows::ShadowMap;
pub use tiling::{
    EdgeEquation, TILE_PIXELS, TILE_SIZE, TILE_VIS_BUFFER_BYTES, TileBounds, TileCoord, TileGrid,
    TileVisBuffer,
};
pub use vis_buffer::{VisBuffer, VisPixel};
