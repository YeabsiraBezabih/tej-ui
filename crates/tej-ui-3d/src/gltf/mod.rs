//! glTF 2.0 and binary (`.glb`) asset ingestion.

pub mod loader;

pub use loader::{
    CHUNK_TYPE_BIN, CHUNK_TYPE_JSON, GLB_MAGIC, GLB_VERSION_2, GlbError, GlbHeader, GlbPackage,
};
