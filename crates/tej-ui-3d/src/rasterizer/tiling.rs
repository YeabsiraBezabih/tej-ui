//! $16 \times 16$ L1-cache resident tile binning and incremental edge evaluation.
//!
//! A single tile's Visibility Buffer occupies $16 \times 16 \times 8 = 2048$ bytes (2 KB),
//! staying 100% inside the CPU L1 data cache (32 KB) with zero RAM thrashing.

use super::vis_buffer::{VisBuffer, VisPixel};

/// Tile dimension in pixels (16x16).
pub const TILE_SIZE: usize = 16;
/// Number of pixels per tile ($16 \times 16 = 256$).
pub const TILE_PIXELS: usize = TILE_SIZE * TILE_SIZE;
/// Memory footprint of a single tile visibility buffer in bytes (2048 bytes).
pub const TILE_VIS_BUFFER_BYTES: usize = TILE_PIXELS * size_of::<VisPixel>();

/// 2D tile coordinate in the screen tile grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileCoord {
    pub x: usize,
    pub y: usize,
}

/// Clamped pixel boundaries of a tile in screen space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileBounds {
    pub min_x: usize,
    pub max_x: usize,
    pub min_y: usize,
    pub max_y: usize,
}

impl TileBounds {
    /// Width of this tile in pixels (at most `TILE_SIZE`).
    #[inline]
    #[must_use]
    pub fn width(&self) -> usize {
        (self.max_x - self.min_x) + 1
    }

    /// Height of this tile in pixels (at most `TILE_SIZE`).
    #[inline]
    #[must_use]
    pub fn height(&self) -> usize {
        (self.max_y - self.min_y) + 1
    }

    /// Tests if a screen-space pixel coordinate falls within this tile.
    #[inline]
    #[must_use]
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }
}

/// Viewport partitioned into $16 \times 16$ pixel tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileGrid {
    pub width: usize,
    pub height: usize,
    pub tiles_x: usize,
    pub tiles_y: usize,
}

impl TileGrid {
    /// Constructs a tile grid for a given viewport size.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let tiles_x = if width == 0 {
            0
        } else {
            width.div_ceil(TILE_SIZE)
        };
        let tiles_y = if height == 0 {
            0
        } else {
            height.div_ceil(TILE_SIZE)
        };
        Self {
            width,
            height,
            tiles_x,
            tiles_y,
        }
    }

    /// Total number of tiles covering the viewport.
    #[inline]
    #[must_use]
    pub fn total_tiles(&self) -> usize {
        self.tiles_x * self.tiles_y
    }

    /// Calculates pixel boundaries for tile `(tx, ty)`, clamped to viewport dimensions.
    #[must_use]
    pub fn tile_bounds(&self, tx: usize, ty: usize) -> Option<TileBounds> {
        if tx >= self.tiles_x || ty >= self.tiles_y || self.width == 0 || self.height == 0 {
            return None;
        }

        let min_x = tx * TILE_SIZE;
        let min_y = ty * TILE_SIZE;
        let max_x = (min_x + TILE_SIZE - 1).min(self.width - 1);
        let max_y = (min_y + TILE_SIZE - 1).min(self.height - 1);

        Some(TileBounds {
            min_x,
            max_x,
            min_y,
            max_y,
        })
    }

    /// Maps a screen-space bounding box `[min_x, min_y, max_x, max_y]` to overlapping tile coordinates.
    ///
    /// Returns `Some((min_tx, min_ty, max_tx, max_ty))` or `None` if completely outside viewport.
    #[must_use]
    pub fn clamp_bbox_to_tiles(
        &self,
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
    ) -> Option<(usize, usize, usize, usize)> {
        if self.width == 0 || self.height == 0 {
            return None;
        }

        let screen_w = (self.width - 1) as f32;
        let screen_h = (self.height - 1) as f32;

        if max_x < 0.0 || max_y < 0.0 || min_x > screen_w || min_y > screen_h {
            return None;
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let c_min_x = (min_x.max(0.0).floor() as usize).min(self.width - 1);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let c_max_x = (max_x.max(0.0).ceil() as usize).min(self.width - 1);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let c_min_y = (min_y.max(0.0).floor() as usize).min(self.height - 1);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let c_max_y = (max_y.max(0.0).ceil() as usize).min(self.height - 1);

        let min_tx = c_min_x / TILE_SIZE;
        let max_tx = (c_max_x / TILE_SIZE).min(self.tiles_x.saturating_sub(1));
        let min_ty = c_min_y / TILE_SIZE;
        let max_ty = (c_max_y / TILE_SIZE).min(self.tiles_y.saturating_sub(1));

        Some((min_tx, min_ty, max_tx, max_ty))
    }
}

/// 2 KB L1-cache resident Visibility Buffer for a single $16 \times 16$ tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileVisBuffer {
    pub pixels: [VisPixel; TILE_PIXELS],
}

impl Default for TileVisBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TileVisBuffer {
    /// Creates an empty tile buffer (2 KB on stack or thread-local).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pixels: [VisPixel::EMPTY; TILE_PIXELS],
        }
    }

    /// Resets all pixels in this tile to empty.
    #[inline]
    pub fn clear(&mut self) {
        self.pixels.fill(VisPixel::EMPTY);
    }

    /// Retrieves the pixel at local coordinates `(local_x, local_y)` inside the tile `[0..16)`.
    #[inline]
    #[must_use]
    pub fn get_pixel(&self, local_x: usize, local_y: usize) -> VisPixel {
        if local_x < TILE_SIZE && local_y < TILE_SIZE {
            self.pixels[local_y * TILE_SIZE + local_x]
        } else {
            VisPixel::EMPTY
        }
    }

    /// Updates the local pixel if the incoming depth is closer (Reversed-Z).
    #[inline]
    pub fn update_if_closer(
        &mut self,
        local_x: usize,
        local_y: usize,
        depth: f32,
        slot: u32,
        tri_id: u16,
    ) -> bool {
        if local_x < TILE_SIZE && local_y < TILE_SIZE {
            let idx = local_y * TILE_SIZE + local_x;
            self.pixels[idx].update_if_closer(depth, slot, tri_id)
        } else {
            false
        }
    }

    /// Blits the tile contents into the global visibility buffer.
    pub fn blit_to_global(&self, bounds: TileBounds, global_buffer: &mut VisBuffer) {
        let global_w = global_buffer.width;
        let global_slice = global_buffer.as_mut_slice();

        for y in bounds.min_y..=bounds.max_y {
            let local_y = y - bounds.min_y;
            let global_row = y * global_w;
            let local_row = local_y * TILE_SIZE;

            for x in bounds.min_x..=bounds.max_x {
                let local_x = x - bounds.min_x;
                let local_pixel = self.pixels[local_row + local_x];
                let global_idx = global_row + x;
                if local_pixel.depth_bits() > global_slice[global_idx].depth_bits() {
                    global_slice[global_idx] = local_pixel;
                }
            }
        }
    }
}

/// Linear 2D edge equation $E(x, y) = A \cdot x + B \cdot y + C$ for incremental stepping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeEquation {
    pub a: f32,
    pub b: f32,
    pub c: f32,
}

impl EdgeEquation {
    /// Builds an edge equation from directed edge $P_0 \to P_1$.
    #[must_use]
    pub fn from_points(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        let a = y0 - y1;
        let b = x1 - x0;
        let c = x0 * y1 - x1 * y0;
        Self { a, b, c }
    }

    /// Evaluates edge equation at pixel center `(x, y)`.
    #[inline]
    #[must_use]
    pub fn eval(&self, x: f32, y: f32) -> f32 {
        self.a * x + self.b * y + self.c
    }

    /// Incremental step when moving $+1$ pixel horizontally.
    #[inline]
    #[must_use]
    pub fn step_x(&self) -> f32 {
        self.a
    }

    /// Incremental step when moving $+1$ pixel vertically.
    #[inline]
    #[must_use]
    pub fn step_y(&self) -> f32 {
        self.b
    }
}
