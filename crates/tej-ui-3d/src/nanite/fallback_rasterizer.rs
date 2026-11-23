//! Pure-Rust software fallback rasterizer for Nanite cluster rendering and debug visualization.

use tej_ui_hal::Color;
use tej_ui_layout::Point;

/// Generates a distinct deterministic RGB color for visualizing individual Nanite clusters.
#[must_use]
pub fn cluster_debug_color(cluster_id: u32) -> Color {
    let mut hash = cluster_id.wrapping_mul(0x9E37_79B9);
    hash ^= hash >> 16;
    let r = (100 + (hash & 0x7F)) as u8;
    let g = (100 + ((hash >> 8) & 0x7F)) as u8;
    let b = (100 + ((hash >> 16) & 0x7F)) as u8;
    Color::rgb(r, g, b)
}

/// A projected 2D triangle ready for rasterization.
#[derive(Debug, Clone, Copy)]
pub struct ProjectedTriangle {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
    pub depth: f32,
    pub color: Color,
}

impl ProjectedTriangle {
    /// Creates a projected triangle with depth and color.
    #[must_use]
    pub const fn new(p0: Point, p1: Point, p2: Point, depth: f32, color: Color) -> Self {
        Self {
            p0,
            p1,
            p2,
            depth,
            color,
        }
    }
}
