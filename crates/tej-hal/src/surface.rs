use std::fmt::Debug;
use tej_layout::{Point, Rect};

/// Color representation with 8-bit RGBA channels.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    #[inline]
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[inline]
    #[must_use]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Abstract 2D graphics hardware compositor for Tier 2 custom views.
pub trait SurfaceCompositor: Debug + Send + Sync {
    /// Begin frame rasterization for specified dirty rectangles.
    fn begin_frame(&mut self, dirty_rects: &[Rect]);

    /// Fill a rectangle with solid color.
    fn fill_rect(&mut self, rect: Rect, color: Color);

    /// Fill a rounded rectangle with corner radius.
    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: Color);

    /// Draw a line between two points.
    fn draw_line(&mut self, p1: Point, p2: Point, stroke_width: f32, color: Color);

    /// Draw text glyph run at origin.
    fn draw_text(&mut self, text: &str, origin: Point, font_size: f32, color: Color);

    /// Conclude frame and present to window compositor.
    fn end_frame(&mut self);
}
