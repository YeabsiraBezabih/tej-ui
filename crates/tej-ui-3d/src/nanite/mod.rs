//! Nanite and 3D Gaussian Splatting engine backed by `MyDb`.

pub mod fallback_rasterizer;
pub mod gizmo;
pub mod hiz;
pub mod meshlet_cache;
pub mod octahedral;
pub mod picking;
pub mod quantized;
pub mod sanitizer;
pub mod streamer;
pub mod types;
pub mod viewport;

pub use fallback_rasterizer::{ProjectedTriangle, cluster_debug_color};
pub use gizmo::{GizmoMode, NaniteGizmoManager};
pub use hiz::{
    HIZ_CELLS0, HIZ_CELLS1, HIZ_CELLS2, HIZ_H0, HIZ_H1, HIZ_H2, HIZ_W0, HIZ_W1, HIZ_W2, HiZBuffer,
};
pub use meshlet_cache::GpuMeshletRing;
pub use octahedral::{decode_oct_normal, encode_oct_normal};
pub use picking::{pick_cluster, screen_point_to_ray};
pub use quantized::{
    PackedMeshletVertex, QuantizedMeshlet, dequantize_position, quantize_position,
};
pub use sanitizer::{MeshSanitizerError, sanitize_mesh_geometry};
pub use streamer::{GpuStagingUploader, MockGpuUploader, NaniteStreamer};
pub use types::{GpuBufferAllocation, NaniteFrameStats};
pub use viewport::NaniteRenderEngine;
