use smallvec::SmallVec;
use tej_hal::{Color, SurfaceCompositor};
use tej_layout::{Point, Rect, Size};

/// 2D surface rasterizer for Linux platform backends.
#[derive(Debug)]
pub struct LinuxSurface {
    size: Size,
    dirty_clip: SmallVec<[Rect; 4]>,
    draw_calls_count: usize,
}

impl LinuxSurface {
    /// Create a surface for a given window size.
    pub fn new(size: Size) -> Self {
        Self {
            size,
            dirty_clip: SmallVec::new(),
            draw_calls_count: 0,
        }
    }

    /// Get surface size.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// Number of draw calls executed in the current/last frame.
    #[must_use]
    pub const fn draw_calls(&self) -> usize {
        self.draw_calls_count
    }
}

impl SurfaceCompositor for LinuxSurface {
    fn begin_frame(&mut self, dirty_rects: &[Rect]) {
        self.dirty_clip.clear();
        self.dirty_clip.extend_from_slice(dirty_rects);
        self.draw_calls_count = 0;
    }

    fn fill_rect(&mut self, rect: Rect, _color: Color) {
        // Only process if rect intersects with dirty clip regions
        if self.dirty_clip.is_empty()
            || self
                .dirty_clip
                .iter()
                .any(|c| c.intersection(&rect).is_some())
        {
            self.draw_calls_count += 1;
        }
    }

    fn fill_rounded_rect(&mut self, rect: Rect, _radius: f32, _color: Color) {
        if self.dirty_clip.is_empty()
            || self
                .dirty_clip
                .iter()
                .any(|c| c.intersection(&rect).is_some())
        {
            self.draw_calls_count += 1;
        }
    }

    fn draw_line(&mut self, _p1: Point, _p2: Point, _stroke_width: f32, _color: Color) {
        self.draw_calls_count += 1;
    }

    fn draw_text(&mut self, _text: &str, _origin: Point, _font_size: f32, _color: Color) {
        self.draw_calls_count += 1;
    }

    fn end_frame(&mut self) {
        self.dirty_clip.clear();
    }
}
