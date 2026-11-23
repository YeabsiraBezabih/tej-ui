//! `tej-3d`: Hardware-accelerated 3D graphics, camera projection, and scene graph for `tej-ui`.
//!
//! Provides a Three.js-inspired 3D pipeline including perspective cameras, procedural
//! geometry (Cube, Sphere), lighting, standard materials, interactive orbit controls,
//! Nanite out-of-core meshlet streaming backed by `MyDb`, and a `Canvas3D` view widget.

pub mod blender;
pub mod camera;
pub mod controls;
pub mod gltf;
pub mod material;
pub mod math;
pub mod mesh;
pub mod nanite;
pub mod rasterizer;
pub mod renderer;
pub mod scene;
pub mod texture;
pub mod view;

pub use blender::{BlenderMesh, MeshletPacker, PackedMeshlet, load_obj_from_file, parse_obj};
pub use camera::{PerspectiveCamera, Ray};
pub use controls::{CameraPreset, OrbitControls};
pub use gltf::{
    CHUNK_TYPE_BIN, CHUNK_TYPE_JSON, GLB_MAGIC, GLB_VERSION_2, GlbError, GlbHeader, GlbPackage,
};
pub use material::{
    Light, LightKind, PbrMaterialParams, SphericalHarmonics9, StandardMaterial, pbr_shade_fragment,
};
pub use math::{Mat4, Vec3};
pub use mesh::{Mesh, Vertex};
pub use nanite::{
    GpuBufferAllocation, GpuMeshletRing, GpuStagingUploader, HIZ_CELLS0, HIZ_CELLS1, HIZ_CELLS2,
    HIZ_H0, HIZ_H1, HIZ_H2, HIZ_W0, HIZ_W1, HIZ_W2, HiZBuffer, MeshSanitizerError, MockGpuUploader,
    NaniteFrameStats, NaniteGizmoManager, NaniteRenderEngine, NaniteStreamer, PackedMeshletVertex,
    ProjectedTriangle, QuantizedMeshlet, cluster_debug_color, decode_oct_normal,
    dequantize_position, encode_oct_normal, pick_cluster, quantize_position,
    sanitize_mesh_geometry, screen_point_to_ray,
};
pub use rasterizer::{
    ClipVertex, ClippedTriangles, EdgeEquation, PerspectiveBarycentrics, ResolvedTriangle,
    ShadingContext, ShadowMap, TileBounds, TileGrid, TileVisBuffer, VisBuffer, VisPixel,
    clip_triangle_near_plane, resolve_tile, shade_fragment,
};
pub use renderer::{GpuTextureDescriptor, GpuTextureFormat, Rasterizer3D, TextureHandle};
pub use scene::{
    InstanceData, InstancedMesh, NO_PARENT, Object3D, Scene, SceneGraphSoA, Transform3D,
};
pub use texture::{
    VIRTUAL_TILE_PAGE_BYTES, VIRTUAL_TILE_PIXELS, VIRTUAL_TILE_SIZE, VirtualTexture,
    VirtualTextureResidentPool, VirtualTileCoord,
};
pub use view::{Canvas3D, Canvas3DState};
