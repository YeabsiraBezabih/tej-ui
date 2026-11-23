//! Web canvas surface compositor managing pixel buffers and WebGL/WebGPU presentations.

use tej_ui_hal::Color;
use tej_ui_hal::dirty::DirtyRectTracker;
use tej_ui_hal::surface::SurfaceCompositor;
use tej_ui_layout::{Point, Rect, Size};

/// Web canvas compositor presenting rendered frames to HTML5 canvas elements.
#[derive(Debug)]
pub struct WebSurfaceCompositor {
    width: u32,
    height: u32,
    pixels: Vec<u32>,
    dirty_tracker: DirtyRectTracker,
}

impl WebSurfaceCompositor {
    /// Creates a new web canvas surface compositor with given dimensions.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize).saturating_mul(height as usize);
        Self {
            width,
            height,
            pixels: vec![0xFF00_0000; size],
            dirty_tracker: DirtyRectTracker::new(),
        }
    }

    /// Resizes the surface buffer to match canvas dimensions.
    pub fn resize_dims(&mut self, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        let size = (width as usize).saturating_mul(height as usize);
        self.pixels.resize(size, 0xFF00_0000);
        self.dirty_tracker.mark_full_dirty();
    }

    /// Access raw pixel buffer `[ARGB / RGBA]`.
    #[must_use]
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Mutable access to pixel buffer.
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.pixels
    }

    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    #[must_use]
    pub fn dirty_tracker(&self) -> &DirtyRectTracker {
        &self.dirty_tracker
    }
}

impl SurfaceCompositor for WebSurfaceCompositor {
    fn begin_frame(&mut self, _dirty_rects: &[Rect]) {
        self.dirty_tracker.reset();
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        let argb = ((u32::from(color.a)) << 24)
            | ((u32::from(color.r)) << 16)
            | ((u32::from(color.g)) << 8)
            | u32::from(color.b);
        let x0 = (rect.origin.x as usize).min(self.width as usize);
        let y0 = (rect.origin.y as usize).min(self.height as usize);
        let x1 = ((rect.origin.x + rect.size.width) as usize).min(self.width as usize);
        let y1 = ((rect.origin.y + rect.size.height) as usize).min(self.height as usize);

        for y in y0..y1 {
            let row_start = y * (self.width as usize);
            self.pixels[row_start + x0..row_start + x1].fill(argb);
        }

        self.dirty_tracker.add_rect(rect);
    }

    fn fill_rounded_rect(&mut self, rect: Rect, _radius: f32, color: Color) {
        self.fill_rect(rect, color);
    }

    fn draw_line(&mut self, p1: Point, p2: Point, _stroke_width: f32, color: Color) {
        let min_x = p1.x.min(p2.x);
        let min_y = p1.y.min(p2.y);
        let w = (p1.x - p2.x).abs().max(1.0);
        let h = (p1.y - p2.y).abs().max(1.0);
        self.fill_rect(Rect::new(min_x, min_y, w, h), color);
    }

    fn draw_text(&mut self, _text: &str, _origin: Point, _font_size: f32, _color: Color) {}

    fn blit_pixels(&mut self, origin: Point, width: usize, height: usize, pixels: &[u32]) {
        let x0 = (origin.x as usize).min(self.width as usize);
        let y0 = (origin.y as usize).min(self.height as usize);
        let copy_w = width.min((self.width as usize).saturating_sub(x0));
        let copy_h = height.min((self.height as usize).saturating_sub(y0));

        for y in 0..copy_h {
            let src_start = y * width;
            let dst_start = (y0 + y) * (self.width as usize) + x0;
            self.pixels[dst_start..dst_start + copy_w]
                .copy_from_slice(&pixels[src_start..src_start + copy_w]);
        }
    }

    fn end_frame(&mut self) {}

    fn resize(&mut self, size: Size) {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        self.resize_dims(size.width.max(1.0) as u32, size.height.max(1.0) as u32);
    }
}
