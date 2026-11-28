//! Visual regression & deterministic golden-image snapshot comparison engine.

use std::fmt::Debug;

/// 32-bit RGBA software framebuffer snapshot.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ImageBuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

impl ImageBuffer {
    /// Create a new blank image buffer filled with transparent pixels.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height) as usize],
        }
    }

    /// Construct from raw 32-bit pixel slice.
    #[must_use]
    pub fn from_pixels(width: u32, height: u32, pixels: Vec<u32>) -> Self {
        Self {
            width,
            height,
            pixels,
        }
    }

    /// Read pixel at `(x, y)` coordinate.
    #[must_use]
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<u32> {
        if x < self.width && y < self.height {
            Some(self.pixels[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Set pixel at `(x, y)` coordinate.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize] = color;
        }
    }

    /// Encode buffer into Netpbm PPM (P6 binary or P3 text) for zero-dependency inspection and export.
    #[must_use]
    pub fn to_ppm_string(&self) -> String {
        use std::fmt::Write;
        let mut out = format!("P3\n{} {}\n255\n", self.width, self.height);
        for pixel in &self.pixels {
            let r = (pixel >> 16) & 0xFF;
            let g = (pixel >> 8) & 0xFF;
            let b = pixel & 0xFF;
            let _ = write!(out, "{r} {g} {b} ");
        }
        out
    }
}

/// Comparison analysis between two image snapshots.
#[derive(Clone, PartialEq, Debug)]
pub struct SnapshotDiffResult {
    /// Whether the images matched within allowed thresholds.
    pub is_match: bool,
    /// Total count of differing pixels.
    pub different_pixels: usize,
    /// Fraction of differing pixels relative to total pixel count.
    pub difference_percentage: f32,
    /// Maximum color channel difference observed (0..=255).
    pub max_channel_diff: u8,
    /// Visual diff mask with mismatched pixels highlighted in red (0xFFFF0000).
    pub diff_mask: ImageBuffer,
}

/// Visual snapshot testing comparator with per-channel color tolerance.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SnapshotComparator {
    /// Allowed channel difference per R, G, B component (0 = strict exact match).
    pub channel_tolerance: u8,
    /// Maximum allowed percentage of differing pixels (0.0 = zero tolerance).
    pub max_diff_percentage: f32,
}

impl Default for SnapshotComparator {
    fn default() -> Self {
        Self {
            channel_tolerance: 2,
            max_diff_percentage: 0.001, // 0.1% threshold
        }
    }
}

impl SnapshotComparator {
    /// Create a strict exact-match comparator.
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            channel_tolerance: 0,
            max_diff_percentage: 0.0,
        }
    }

    /// Compare `actual` rendered snapshot against `expected` golden baseline.
    #[must_use]
    pub fn compare(&self, actual: &ImageBuffer, expected: &ImageBuffer) -> SnapshotDiffResult {
        let width = actual.width.min(expected.width);
        let height = actual.height.min(expected.height);
        let total_pixels = (actual.width * actual.height) as usize;

        let mut diff_mask = ImageBuffer::new(actual.width, actual.height);
        let mut different_pixels = 0;
        let mut max_channel_diff: u8 = 0;

        if actual.width != expected.width || actual.height != expected.height {
            different_pixels = total_pixels;
        }

        for y in 0..height {
            for x in 0..width {
                let p1 = actual.get_pixel(x, y).unwrap_or(0);
                let p2 = expected.get_pixel(x, y).unwrap_or(0);

                let r1 = ((p1 >> 16) & 0xFF) as u8;
                let g1 = ((p1 >> 8) & 0xFF) as u8;
                let b1 = (p1 & 0xFF) as u8;
                let a1 = ((p1 >> 24) & 0xFF) as u8;

                let r2 = ((p2 >> 16) & 0xFF) as u8;
                let g2 = ((p2 >> 8) & 0xFF) as u8;
                let b2 = (p2 & 0xFF) as u8;
                let a2 = ((p2 >> 24) & 0xFF) as u8;

                let dr = (i16::from(r1) - i16::from(r2)).unsigned_abs() as u8;
                let dg = (i16::from(g1) - i16::from(g2)).unsigned_abs() as u8;
                let db = (i16::from(b1) - i16::from(b2)).unsigned_abs() as u8;
                let da = (i16::from(a1) - i16::from(a2)).unsigned_abs() as u8;

                let max_d = dr.max(dg).max(db).max(da);
                if max_d > max_channel_diff {
                    max_channel_diff = max_d;
                }

                if max_d > self.channel_tolerance {
                    different_pixels += 1;
                    diff_mask.set_pixel(x, y, 0xFFFF_0000); // Red highlight for failure
                } else {
                    diff_mask.set_pixel(x, y, p1);
                }
            }
        }

        let diff_px = f64::from(u32::try_from(different_pixels).unwrap_or(u32::MAX));
        let total_px = f64::from(u32::try_from(total_pixels.max(1)).unwrap_or(u32::MAX));
        let difference_percentage = (diff_px / total_px) as f32;
        let is_match = difference_percentage <= self.max_diff_percentage;

        SnapshotDiffResult {
            is_match,
            different_pixels,
            difference_percentage,
            max_channel_diff,
            diff_mask,
        }
    }
}
