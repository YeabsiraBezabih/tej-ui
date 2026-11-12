//! `tej-3d`: Hardware-accelerated 3D graphics, camera projection, and scene graph for `tej-ui`.
//!
//! Provides a Three.js-inspired 3D pipeline including perspective cameras, procedural
//! geometry (Cube, Sphere), lighting, standard materials, interactive orbit controls,
//! and a `Canvas3D` view widget that embeds seamlessly into 2D UI layouts.

pub mod camera;
pub mod controls;
pub mod material;
pub mod math;
pub mod mesh;
pub mod renderer;
pub mod scene;
pub mod view;

pub use camera::{PerspectiveCamera, Ray};
pub use controls::OrbitControls;
pub use material::{Light, LightKind, StandardMaterial};
pub use math::{Mat4, Vec3};
pub use mesh::{Mesh, Vertex};
pub use renderer::{GpuTextureDescriptor, GpuTextureFormat, Rasterizer3D, TextureHandle};
pub use scene::{Object3D, Scene, Transform3D};
pub use view::{Canvas3D, Canvas3DState};
