//! 2D surface rasterizer for Linux platform backends.

use crate::font_engine::FontEngine;
use smallvec::SmallVec;
use std::sync::Arc;
use tej_hal::{Color, SurfaceCompositor};
use tej_layout::{Point, Rect, Size};

/// 2D surface rasterizer for Linux platform backends with 32-bit pixel buffer.
#[derive(Debug)]
pub struct LinuxSurface {
    size: Size,
    width: usize,
    height: usize,
    pixels: Vec<u32>,
    dirty_clip: SmallVec<[Rect; 4]>,
    draw_calls_count: usize,
    font_engine: Arc<FontEngine>,
}

impl LinuxSurface {
    /// Create a surface for a given window size with system font discovery.
    pub fn new(size: Size) -> Self {
        Self::with_font_engine(size, Arc::new(FontEngine::load_system_or_fallback()))
    }

    /// Create a surface with an explicit font engine.
    pub fn with_font_engine(size: Size, font_engine: Arc<FontEngine>) -> Self {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let width = (size.width.max(1.0) as usize).min(8192);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let height = (size.height.max(1.0) as usize).min(8192);
        let pixels = vec![0xFF1E_1E24; width * height]; // Default dark slate background

        Self {
            size,
            width,
            height,
            pixels,
            dirty_clip: SmallVec::new(),
            draw_calls_count: 0,
            font_engine,
        }
    }

    /// Surface logical dimensions.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// Buffer pixel width.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Buffer pixel height.
    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Direct reference to 32-bit ARGB/XRGB pixel buffer.
    #[must_use]
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Clear buffer with uniform color.
    pub fn clear(&mut self, color: Color) {
        let px = pack_color(color);
        self.pixels.fill(px);
    }

    /// Number of draw calls executed in the current/last frame.
    #[must_use]
    pub const fn draw_calls(&self) -> usize {
        self.draw_calls_count
    }
}

#[inline]
const fn pack_color(c: Color) -> u32 {
    ((c.a as u32) << 24) | ((c.r as u32) << 16) | ((c.g as u32) << 8) | (c.b as u32)
}

#[inline]
fn blend_pixel(bg: u32, fg_color: Color, alpha: u8) -> u32 {
    if alpha == 255 {
        return pack_color(fg_color);
    }
    let a = alpha as u32;
    let inv_a = 255 - a;
    let bg_r = (bg >> 16) & 0xff;
    let bg_g = (bg >> 8) & 0xff;
    let bg_b = bg & 0xff;
    let r = (fg_color.r as u32 * a + bg_r * inv_a) / 255;
    let g = (fg_color.g as u32 * a + bg_g * inv_a) / 255;
    let b = (fg_color.b as u32 * a + bg_b * inv_a) / 255;
    0xff00_0000 | (r << 16) | (g << 8) | b
}

impl SurfaceCompositor for LinuxSurface {
    fn begin_frame(&mut self, dirty_rects: &[Rect]) {
        self.dirty_clip.clear();
        self.dirty_clip.extend_from_slice(dirty_rects);
        self.draw_calls_count = 0;
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        // Only process if rect intersects with dirty clip regions
        if !self.dirty_clip.is_empty()
            && !self
                .dirty_clip
                .iter()
                .any(|c| c.intersection(&rect).is_some())
        {
            return;
        }

        self.draw_calls_count += 1;
        let px = pack_color(color);

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let x0 = (rect.origin.x.max(0.0) as usize).min(self.width);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let y0 = (rect.origin.y.max(0.0) as usize).min(self.height);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let x1 = ((rect.origin.x + rect.size.width).max(0.0) as usize).min(self.width);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let y1 = ((rect.origin.y + rect.size.height).max(0.0) as usize).min(self.height);

        for y in y0..y1 {
            let row_start = y * self.width;
            for x in x0..x1 {
                self.pixels[row_start + x] = px;
            }
        }
    }

    fn fill_rounded_rect(&mut self, rect: Rect, _radius: f32, color: Color) {
        // Fallback to fill_rect in software buffer
        self.fill_rect(rect, color);
    }

    fn draw_line(&mut self, p1: Point, p2: Point, stroke_width: f32, color: Color) {
        self.draw_calls_count += 1;
        let px = pack_color(color);

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let x = (p1.x.min(p2.x).max(0.0) as usize).min(self.width);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let y = (p1.y.min(p2.y).max(0.0) as usize).min(self.height);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let w = (stroke_width.max(1.0) as usize).min(self.width.saturating_sub(x));

        for offset in 0..w {
            if y < self.height && (x + offset) < self.width {
                self.pixels[y * self.width + x + offset] = px;
            }
        }
    }

    fn draw_text(&mut self, text: &str, origin: Point, font_size: f32, color: Color) {
        self.draw_calls_count += 1;

        if self.font_engine.has_vector_font() {
            let line_height = font_size * 1.35;
            let mut cur_x = origin.x;
            let mut cur_y = origin.y;
            let mut baseline = cur_y + font_size * 0.8;

            for ch in text.chars() {
                if ch == '\n' {
                    cur_x = origin.x;
                    cur_y += line_height;
                    baseline = cur_y + font_size * 0.8;
                    continue;
                }
                if let Some(glyph) = self.font_engine.get_glyph(ch, font_size) {
                    #[allow(clippy::cast_possible_truncation)]
                    let start_x = (cur_x + glyph.xmin as f32).round() as i32;
                    #[allow(clippy::cast_possible_truncation)]
                    let start_y =
                        (baseline - glyph.ymin as f32 - glyph.height as f32).round() as i32;

                    for row in 0..glyph.height {
                        let y = start_y + row as i32;
                        if y < 0 || (y as usize) >= self.height {
                            continue;
                        }
                        let row_offset = (y as usize) * self.width;
                        let bmp_row = row * glyph.width;

                        for col in 0..glyph.width {
                            let x = start_x + col as i32;
                            if x < 0 || (x as usize) >= self.width {
                                continue;
                            }
                            let alpha = glyph.bitmap[bmp_row + col];
                            if alpha > 0 {
                                let idx = row_offset + (x as usize);
                                self.pixels[idx] = blend_pixel(self.pixels[idx], color, alpha);
                            }
                        }
                    }
                    cur_x += glyph.advance_width;
                    if cur_x >= self.width as f32 {
                        break;
                    }
                }
            }
            return;
        }

        let px = pack_color(color);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let scale = (font_size / 8.0).round().max(1.0) as usize;
        let char_advance = 8 * scale;
        let line_height = 10 * scale;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mut cur_x = origin.x.max(0.0) as usize;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mut cur_y = origin.y.max(0.0) as usize;

        for ch in text.chars() {
            if ch == '\n' {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                {
                    cur_x = origin.x.max(0.0) as usize;
                }
                cur_y += line_height;
                continue;
            }
            if let Some(glyph) = crate::font_table::get_glyph_8x8(ch) {
                for (row_idx, &row_byte) in glyph.iter().enumerate() {
                    for col_idx in 0..8 {
                        if (row_byte & (0x80 >> col_idx)) != 0 {
                            for dy in 0..scale {
                                for dx in 0..scale {
                                    let px_x = cur_x + col_idx * scale + dx;
                                    let px_y = cur_y + row_idx * scale + dy;
                                    if px_x < self.width && px_y < self.height {
                                        self.pixels[px_y * self.width + px_x] = px;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            cur_x += char_advance;
            if cur_x >= self.width {
                break;
            }
        }
    }

    fn end_frame(&mut self) {
        self.dirty_clip.clear();
    }
}
