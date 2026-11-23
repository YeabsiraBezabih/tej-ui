//! Texture subsystem and out-of-core Virtual Texture streaming.

pub mod virtual_texture;

pub use virtual_texture::{
    VIRTUAL_TILE_PAGE_BYTES, VIRTUAL_TILE_PIXELS, VIRTUAL_TILE_SIZE, VirtualTexture,
    VirtualTextureResidentPool, VirtualTileCoord,
};
