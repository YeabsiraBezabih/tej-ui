//! Blender asset parsing and Nanite meshlet ingestion pipeline.

pub mod meshlet_packer;
pub mod obj_loader;
pub mod suzanne;

pub use meshlet_packer::{MeshletPacker, PackedMeshlet};
pub use obj_loader::{load_obj_from_file, parse_obj};
pub use suzanne::BlenderMesh;
