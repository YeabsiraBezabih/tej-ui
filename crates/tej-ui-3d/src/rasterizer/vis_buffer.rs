//! 64-bit packed visibility buffer for zero-overdraw deferred shading (Unreal Nanite architecture).
//!
//! Stores depth and triangle ownership per pixel in a single 64-bit word:
//! `[32 bits: Reversed-Z Depth] | [20 bits: Meshlet Slot] | [12 bits: Triangle Index]`

/// Packed 64-bit Visibility Buffer pixel.
///
/// Bit layout:
/// - Bits `[63..32]`: Reversed-Z depth (`u32` bits of IEEE-754 positive `f32` in `[0.0, 1.0]`).
/// - Bits `[31..12]`: Meshlet / Model slot identifier (up to $2^{20} = 1,048,576$ clusters).
/// - Bits `[11..00]`: Local triangle index within the cluster (up to $2^{12} = 4,096$ triangles).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VisPixel(pub u64);

impl VisPixel {
    /// Cleared / unwritten visibility pixel.
    pub const EMPTY: Self = Self(0);

    /// Mask for the 20-bit meshlet slot.
    pub const SLOT_MASK: u64 = 0xF_FFFF;
    /// Mask for the 12-bit triangle index.
    pub const TRI_MASK: u64 = 0x0FFF;

    /// Packs depth, meshlet slot, and triangle ID into a 64-bit visibility pixel.
    #[inline]
    #[must_use]
    pub fn pack(depth: f32, meshlet_slot: u32, tri_id: u16) -> Self {
        let clamped = if depth.is_nan() || depth < 0.0 {
            0.0
        } else if depth > 1.0 {
            1.0
        } else {
            depth
        };
        let d_bits = u64::from(clamped.to_bits());
        let slot_bits = (u64::from(meshlet_slot) & Self::SLOT_MASK) << 12;
        let tri_bits = u64::from(tri_id) & Self::TRI_MASK;
        Self((d_bits << 32) | slot_bits | tri_bits)
    }

    /// Extracts the Reversed-Z depth value in range `[0.0, 1.0]`.
    #[inline]
    #[must_use]
    pub fn depth(self) -> f32 {
        let d_bits = (self.0 >> 32) as u32;
        f32::from_bits(d_bits)
    }

    /// Extracts the raw integer bits of the Reversed-Z depth.
    #[inline]
    #[must_use]
    pub fn depth_bits(self) -> u32 {
        (self.0 >> 32) as u32
    }

    /// Extracts the 20-bit meshlet/object slot index.
    #[inline]
    #[must_use]
    pub fn meshlet_slot(self) -> u32 {
        ((self.0 >> 12) & Self::SLOT_MASK) as u32
    }

    /// Extracts the 12-bit local triangle index.
    #[inline]
    #[must_use]
    pub fn tri_id(self) -> u16 {
        (self.0 & Self::TRI_MASK) as u16
    }

    /// Returns `true` if this pixel has not been covered by any triangle.
    #[inline]
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Evaluates Reversed-Z depth test and updates pixel if the incoming fragment is closer.
    ///
    /// In Reversed-Z, closer geometry has a strictly greater depth value ($1.0$ near, $0.0$ far).
    /// Returns `true` if this pixel was updated.
    #[inline]
    pub fn update_if_closer(&mut self, depth: f32, meshlet_slot: u32, tri_id: u16) -> bool {
        let clamped = if depth.is_nan() || depth < 0.0 {
            0.0
        } else if depth > 1.0 {
            1.0
        } else {
            depth
        };
        let d_bits = clamped.to_bits();
        if d_bits > self.depth_bits() {
            let slot_bits = (u64::from(meshlet_slot) & Self::SLOT_MASK) << 12;
            let tri_bits = u64::from(tri_id) & Self::TRI_MASK;
            self.0 = (u64::from(d_bits) << 32) | slot_bits | tri_bits;
            true
        } else {
            false
        }
    }
}

/// 2D Visibility Buffer storing 64-bit packed geometry IDs per pixel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisBuffer {
    pub width: usize,
    pub height: usize,
    pixels: Vec<VisPixel>,
}

impl VisBuffer {
    /// Creates a new visibility buffer with specified viewport dimensions.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            pixels: vec![VisPixel::EMPTY; size],
        }
    }

    /// Resizes the visibility buffer, reallocating memory only if size changes.
    pub fn resize(&mut self, width: usize, height: usize) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.pixels.resize(width * height, VisPixel::EMPTY);
            self.clear();
        }
    }

    /// Clears the visibility buffer to all-empty words.
    #[inline]
    pub fn clear(&mut self) {
        self.pixels.fill(VisPixel::EMPTY);
    }

    /// Total number of pixels in the visibility buffer.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.pixels.len()
    }

    /// Returns `true` if the buffer contains zero pixels.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }

    /// Read-only slice of raw 64-bit visibility pixels.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[VisPixel] {
        &self.pixels
    }

    /// Mutable slice of raw 64-bit visibility pixels.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [VisPixel] {
        &mut self.pixels
    }

    /// Retrieves the visibility pixel at `(x, y)`.
    #[inline]
    #[must_use]
    pub fn get_pixel(&self, x: usize, y: usize) -> VisPixel {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x]
        } else {
            VisPixel::EMPTY
        }
    }

    /// Resets the pixel at `(x, y)` to empty.
    #[inline]
    pub fn clear_pixel(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = VisPixel::EMPTY;
        }
    }

    /// Directly writes a winning fragment into the visibility buffer at a linear index.
    #[inline]
    pub fn write_pixel_unchecked(
        &mut self,
        idx: usize,
        depth: f32,
        meshlet_slot: u32,
        tri_id: u16,
    ) {
        if idx < self.pixels.len() {
            let clamped = if depth.is_nan() || depth < 0.0 {
                0.0
            } else if depth > 1.0 {
                1.0
            } else {
                depth
            };
            let d_bits = clamped.to_bits();
            let slot_bits = (u64::from(meshlet_slot) & VisPixel::SLOT_MASK) << 12;
            let tri_bits = u64::from(tri_id) & VisPixel::TRI_MASK;
            self.pixels[idx] = VisPixel((u64::from(d_bits) << 32) | slot_bits | tri_bits);
        }
    }

    /// Updates the pixel at `(x, y)` if the new depth is closer.
    #[inline]
    pub fn update_pixel(
        &mut self,
        x: usize,
        y: usize,
        depth: f32,
        meshlet_slot: u32,
        tri_id: u16,
    ) -> bool {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.pixels[idx].update_if_closer(depth, meshlet_slot, tri_id)
        } else {
            false
        }
    }

    /// Converts the visibility buffer into an array of `f32` depth values.
    #[must_use]
    pub fn to_depth_vec(&self) -> Vec<f32> {
        self.pixels.iter().map(|p| p.depth()).collect()
    }
}
