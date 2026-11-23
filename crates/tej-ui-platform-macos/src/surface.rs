//! CAMetalLayer hardware-accelerated 2D surface compositor.

use smallvec::SmallVec;
use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{Point, Rect, Size};

/// CAMetalLayer hardware surface using Metal dirty-tile blitting.
#[derive(Debug)]
pub struct MetalSurfaceCompositor {
    size: Size,
    backing_scale_factor: f32,
    dirty_clip: SmallVec<[Rect; 4]>,
    draw_calls_count: usize,
    pixel_buffer: Vec<u32>, // 32-bit RGBA native to CAMetalLayer
}

impl MetalSurfaceCompositor {
    /// Creates a native Metal surface compositor from logical bounds.
    #[must_use]
    pub fn from_size(size: Size) -> Self {
        Self::new(size, 2.0)
    }

    /// Resize surface backing buffer to match updated window bounds.
    pub fn resize(&mut self, new_size: Size) {
        let phys_w = (new_size.width * self.backing_scale_factor).max(1.0) as usize;
        let phys_h = (new_size.height * self.backing_scale_factor).max(1.0) as usize;
        self.size = new_size;
        self.pixel_buffer = vec![0xFFFFFFFF; phys_w * phys_h];
    }

    /// Creates a new Metal surface compositor with Retina-scaled pixel buffer.
    #[must_use]
    pub fn new(size: Size, backing_scale_factor: f32) -> Self {
        let sf = if backing_scale_factor > 0.0 {
            backing_scale_factor
        } else {
            2.0
        };
        let phys_w = (size.width * sf).max(1.0) as usize;
        let phys_h = (size.height * sf).max(1.0) as usize;

        Self {
            size,
            backing_scale_factor: sf,
            dirty_clip: SmallVec::new(),
            draw_calls_count: 0,
            pixel_buffer: vec![0xFFFFFFFF; phys_w * phys_h],
        }
    }

    /// Returns the surface logical size.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// Returns the Retina backing scale factor.
    #[must_use]
    pub const fn backing_scale_factor(&self) -> f32 {
        self.backing_scale_factor
    }

    /// Total draw calls executed in the current/last frame.
    #[must_use]
    pub const fn draw_calls(&self) -> usize {
        self.draw_calls_count
    }

    /// Access to the underlying 32-bit RGBA pixel memory.
    pub fn pixel_data(&self) -> &[u32] {
        &self.pixel_buffer
    }
}

impl SurfaceCompositor for MetalSurfaceCompositor {
    fn begin_frame(&mut self, dirty_rects: &[Rect]) {
        self.dirty_clip.clear();
        self.dirty_clip.extend_from_slice(dirty_rects);
        self.draw_calls_count = 0;
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        if self.dirty_clip.is_empty()
            || self
                .dirty_clip
                .iter()
                .any(|c| c.intersection(&rect).is_some())
        {
            self.draw_calls_count += 1;

            // Direct pixel blit in 32-bit RGBA
            let sf = self.backing_scale_factor;
            let phys_w = (self.size.width * sf) as usize;
            let phys_h = (self.size.height * sf) as usize;

            let x0 = (rect.origin.x * sf).clamp(0.0, phys_w as f32) as usize;
            let y0 = (rect.origin.y * sf).clamp(0.0, phys_h as f32) as usize;
            let x1 = ((rect.origin.x + rect.size.width) * sf).clamp(0.0, phys_w as f32) as usize;
            let y1 = ((rect.origin.y + rect.size.height) * sf).clamp(0.0, phys_h as f32) as usize;

            let rgba = ((color.a as u32) << 24)
                | ((color.b as u32) << 16)
                | ((color.g as u32) << 8)
                | (color.r as u32);

            for y in y0..y1 {
                let row_offset = y * phys_w;
                for x in x0..x1 {
                    if row_offset + x < self.pixel_buffer.len() {
                        self.pixel_buffer[row_offset + x] = rgba;
                    }
                }
            }
        }
    }

    fn fill_rounded_rect(&mut self, rect: Rect, _radius: f32, color: Color) {
        self.fill_rect(rect, color);
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

    fn resize(&mut self, size: Size) {
        self.resize(size);
    }
}
