use crate::color::Color;
use std::fmt::Debug;
use tej_ui_layout::{Point, Rect};

pub use crate::color::{ColorLinear, ColorSpace};

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

    /// Blit a 32-bit ARGB/XRGB pixel buffer onto the compositor surface at logical origin.
    fn blit_pixels(&mut self, _origin: Point, _width: usize, _height: usize, _pixels: &[u32]) {}

    /// Conclude frame and present to window compositor.
    fn end_frame(&mut self);

    /// Resize surface backing buffer to match updated window bounds.
    fn resize(&mut self, _size: tej_ui_layout::Size) {}
}
