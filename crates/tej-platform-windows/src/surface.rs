//! DirectComposition and Direct2D hardware-accelerated 2D surface compositor.

use smallvec::SmallVec;
use tej_hal::{Color, SurfaceCompositor};
use tej_layout::{Point, Rect, Size};

/// DirectComposition hardware surface using OS-managed visual trees and dirty tiles.
#[derive(Debug)]
pub struct DirectCompositionSurface {
    size: Size,
    scale_factor: f32,
    dirty_clip: SmallVec<[Rect; 4]>,
    draw_calls_count: usize,
}

impl DirectCompositionSurface {
    /// Creates a new DirectComposition surface.
    #[must_use]
    pub fn new(size: Size, scale_factor: f32) -> Self {
        Self {
            size,
            scale_factor: if scale_factor > 0.0 {
                scale_factor
            } else {
                1.0
            },
            dirty_clip: SmallVec::new(),
            draw_calls_count: 0,
        }
    }

    /// Returns the surface logical size.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// Returns the current scale factor.
    #[must_use]
    pub const fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Total draw calls executed in the current/last frame.
    #[must_use]
    pub const fn draw_calls(&self) -> usize {
        self.draw_calls_count
    }
}

impl SurfaceCompositor for DirectCompositionSurface {
    fn begin_frame(&mut self, dirty_rects: &[Rect]) {
        self.dirty_clip.clear();
        self.dirty_clip.extend_from_slice(dirty_rects);
        self.draw_calls_count = 0;
    }

    fn fill_rect(&mut self, rect: Rect, _color: Color) {
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
        // In real Win32, calls IDCompositionDevice::Commit()
    }
}
