//! Out-of-core Virtual Texture streaming backed by 4 KiB `MyDb` pages.
//!
//! Partitions gigabyte-scale textures into $64 \times 64$ BC1/BC7 compressed tiles (2 KB or 4 KB)
//! mapped directly into `MyDb` slotted pages and streamed on demand by the camera view.

use mydb_core::PageId;
use std::collections::HashMap;

/// Standard tile dimension in pixels ($64 \times 64$).
pub const VIRTUAL_TILE_SIZE: usize = 64;
/// Number of pixels in a virtual texture tile ($64 \times 64 = 4096$).
pub const VIRTUAL_TILE_PIXELS: usize = VIRTUAL_TILE_SIZE * VIRTUAL_TILE_SIZE;
/// Memory footprint of a single 4-bpp BC1/BC7 compressed tile (fits in 4 KiB `MyDb` page).
pub const VIRTUAL_TILE_PAGE_BYTES: usize = 4096;

/// 3D coordinate of a texture tile in the virtual mip pyramid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VirtualTileCoord {
    pub mip: u8,
    pub tile_x: u16,
    pub tile_y: u16,
}

impl VirtualTileCoord {
    /// Creates a tile coordinate for a specific mip level.
    #[must_use]
    pub const fn new(mip: u8, tile_x: u16, tile_y: u16) -> Self {
        Self {
            mip,
            tile_x,
            tile_y,
        }
    }
}

/// Out-of-core virtual texture descriptor mapped to `MyDb` page table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualTexture {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub max_mip: u8,
    /// Mapping from `(mip, tile_x, tile_y)` to `MyDb` PageId.
    tile_directory: HashMap<VirtualTileCoord, PageId>,
}

impl VirtualTexture {
    /// Creates a new virtual texture with specified dimensions.
    #[must_use]
    pub fn new(id: u64, width: u32, height: u32) -> Self {
        let max_dim = width.max(height);
        let max_mip = (32 - max_dim.leading_zeros()).saturating_sub(6) as u8;
        Self {
            id,
            width,
            height,
            max_mip,
            tile_directory: HashMap::new(),
        }
    }

    /// Number of horizontal and vertical tiles for a given mip level.
    #[must_use]
    pub fn tile_dimensions(&self, mip: u8) -> (usize, usize) {
        let mip_w = (self.width >> mip).max(1) as usize;
        let mip_h = (self.height >> mip).max(1) as usize;
        (
            mip_w.div_ceil(VIRTUAL_TILE_SIZE),
            mip_h.div_ceil(VIRTUAL_TILE_SIZE),
        )
    }

    /// Registers a `MyDb` page ID for a virtual texture tile.
    pub fn register_tile_page(&mut self, coord: VirtualTileCoord, page_id: PageId) {
        self.tile_directory.insert(coord, page_id);
    }

    /// Looks up the `MyDb` page ID for a virtual texture tile.
    #[must_use]
    pub fn get_tile_page(&self, coord: &VirtualTileCoord) -> Option<PageId> {
        self.tile_directory.get(coord).copied()
    }

    /// Maps normalized UV coordinates `[0.0, 1.0]^2` to a tile coordinate at `mip`.
    #[must_use]
    pub fn uv_to_tile(&self, u: f32, v: f32, mip: u8) -> VirtualTileCoord {
        let (tiles_x, tiles_y) = self.tile_dimensions(mip);
        let u_clamped = u.fract().abs();
        let v_clamped = v.fract().abs();

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let tx =
            ((u_clamped * tiles_x as f32).floor() as u16).min(tiles_x.saturating_sub(1) as u16);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let ty =
            ((v_clamped * tiles_y as f32).floor() as u16).min(tiles_y.saturating_sub(1) as u16);

        VirtualTileCoord::new(mip, tx, ty)
    }

    /// Total number of tiles mapped in the virtual texture.
    #[must_use]
    pub fn mapped_tiles_count(&self) -> usize {
        self.tile_directory.len()
    }
}

/// Resident memory cache for active virtual texture tiles.
#[derive(Debug, Clone)]
pub struct VirtualTextureResidentPool {
    pub max_resident_tiles: usize,
    tiles: HashMap<VirtualTileCoord, Vec<u8>>,
}

impl VirtualTextureResidentPool {
    /// Creates a pool with maximum resident tile capacity.
    #[must_use]
    pub fn new(max_resident_tiles: usize) -> Self {
        Self {
            max_resident_tiles,
            tiles: HashMap::with_capacity(max_resident_tiles),
        }
    }

    /// Inserts or updates a resident tile payload.
    pub fn insert_tile(&mut self, coord: VirtualTileCoord, payload: Vec<u8>) {
        if self.tiles.len() >= self.max_resident_tiles {
            // Evict oldest entry
            if let Some(key) = self.tiles.keys().next().copied() {
                self.tiles.remove(&key);
            }
        }
        self.tiles.insert(coord, payload);
    }

    /// Retrieves a reference to a cached resident tile payload.
    #[must_use]
    pub fn get_tile(&self, coord: &VirtualTileCoord) -> Option<&[u8]> {
        self.tiles.get(coord).map(Vec::as_slice)
    }

    /// Checks if a tile is resident in memory.
    #[must_use]
    pub fn is_resident(&self, coord: &VirtualTileCoord) -> bool {
        self.tiles.contains_key(coord)
    }
}
