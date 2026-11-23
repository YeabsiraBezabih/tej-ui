//! 2D surface rasterizer for Linux platform backends.

use crate::font_engine::FontEngine;
use smallvec::SmallVec;
use std::sync::Arc;
use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{Point, Rect, Size};

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
    /// Creates a native surface from logical bounds.
    #[must_use]
    pub fn from_size(size: Size) -> Self {
        Self::new(size)
    }

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
        let pixels = vec![0xFFEF_EEEA; width * height]; // Default warm sand canvas background

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

    /// Resize surface buffer to match the active window dimensions.
    pub fn resize(&mut self, new_size: Size) {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let width = (new_size.width.max(1.0) as usize).min(8192);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let height = (new_size.height.max(1.0) as usize).min(8192);

        if self.width == width && self.height == height {
            return;
        }

        self.size = new_size;
        self.width = width;
        self.height = height;
        self.pixels = vec![0xFFEF_EEEA; width * height];
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
        self.pixels.fill(0xFFEF_EEEA);
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        if color.a == 0 {
            return;
        }
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

        if color.a == 255 {
            for y in y0..y1 {
                let row_start = y * self.width;
                for x in x0..x1 {
                    self.pixels[row_start + x] = px;
                }
            }
        } else {
            for y in y0..y1 {
                let row_start = y * self.width;
                for x in x0..x1 {
                    let idx = row_start + x;
                    self.pixels[idx] = blend_pixel(self.pixels[idx], color, color.a);
                }
            }
        }
    }

    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: Color) {
        if color.a == 0 {
            return;
        }
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

        let r = radius
            .min(rect.size.width * 0.5)
            .min(rect.size.height * 0.5)
            .max(0.0);
        if r < 1.0 {
            if color.a == 255 {
                for y in y0..y1 {
                    let row_start = y * self.width;
                    for x in x0..x1 {
                        self.pixels[row_start + x] = px;
                    }
                }
            } else {
                for y in y0..y1 {
                    let row_start = y * self.width;
                    for x in x0..x1 {
                        let idx = row_start + x;
                        self.pixels[idx] = blend_pixel(self.pixels[idx], color, color.a);
                    }
                }
            }
            return;
        }

        let r2 = r * r;
        let rx0 = rect.origin.x + r;
        let rx1 = rect.origin.x + rect.size.width - r;
        let ry0 = rect.origin.y + r;
        let ry1 = rect.origin.y + rect.size.height - r;

        for y in y0..y1 {
            let py = y as f32 + 0.5;
            let row_start = y * self.width;
            for x in x0..x1 {
                let px_f = x as f32 + 0.5;
                if px_f < rx0 && py < ry0 {
                    let dx = rx0 - px_f;
                    let dy = ry0 - py;
                    if dx * dx + dy * dy > r2 {
                        continue;
                    }
                } else if px_f > rx1 && py < ry0 {
                    let dx = px_f - rx1;
                    let dy = ry0 - py;
                    if dx * dx + dy * dy > r2 {
                        continue;
                    }
                } else if px_f < rx0 && py > ry1 {
                    let dx = rx0 - px_f;
                    let dy = py - ry1;
                    if dx * dx + dy * dy > r2 {
                        continue;
                    }
                } else if px_f > rx1 && py > ry1 {
                    let dx = px_f - rx1;
                    let dy = py - ry1;
                    if dx * dx + dy * dy > r2 {
                        continue;
                    }
                }
                let idx = row_start + x;
                if color.a == 255 {
                    self.pixels[idx] = px;
                } else {
                    self.pixels[idx] = blend_pixel(self.pixels[idx], color, color.a);
                }
            }
        }
    }

    fn draw_line(&mut self, p1: Point, p2: Point, stroke_width: f32, color: Color) {
        self.draw_calls_count += 1;
        if color.a == 0 || stroke_width <= 0.0 {
            return;
        }

        let radius = (stroke_width * 0.5).max(0.5);
        let d_max = radius + 0.5;
        let d_min = (radius - 0.5).max(0.0);

        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let min_x = ((p1.x.min(p2.x) - d_max).floor() as i32).max(0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let max_x = ((p1.x.max(p2.x) + d_max).ceil() as i32).min((self.width as i32) - 1);
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let min_y = ((p1.y.min(p2.y) - d_max).floor() as i32).max(0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let max_y = ((p1.y.max(p2.y) + d_max).ceil() as i32).min((self.height as i32) - 1);

        if min_x > max_x || min_y > max_y {
            return;
        }

        let vx = p2.x - p1.x;
        let vy = p2.y - p1.y;
        let seg_len_sq = vx * vx + vy * vy;

        for py in min_y..=max_y {
            let y_center = py as f32 + 0.5;
            let row_offset = (py as usize) * self.width;

            for px in min_x..=max_x {
                let x_center = px as f32 + 0.5;

                let dist = if seg_len_sq < 1e-4 {
                    let dx = x_center - p1.x;
                    let dy = y_center - p1.y;
                    (dx * dx + dy * dy).sqrt()
                } else {
                    let wx = x_center - p1.x;
                    let wy = y_center - p1.y;
                    let t = ((wx * vx + wy * vy) / seg_len_sq).clamp(0.0, 1.0);
                    let proj_x = p1.x + t * vx;
                    let proj_y = p1.y + t * vy;
                    let dx = x_center - proj_x;
                    let dy = y_center - proj_y;
                    (dx * dx + dy * dy).sqrt()
                };

                if dist < d_max {
                    let coverage = if dist <= d_min {
                        1.0
                    } else {
                        (d_max - dist) / (d_max - d_min)
                    };

                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let eff_alpha = ((color.a as f32) * coverage).round() as u8;
                    if eff_alpha > 0 {
                        let idx = row_offset + (px as usize);
                        self.pixels[idx] = blend_pixel(self.pixels[idx], color, eff_alpha);
                    }
                }
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
                let target_glyph = self.font_engine.get_glyph(ch, font_size).or_else(|| {
                    let fallback = match ch {
                        '●' | '•' => '*',
                        '▲' => '^',
                        '▾' | '▼' => 'v',
                        '↗' | '→' => '>',
                        '↵' => '<',
                        '⌖' => '+',
                        '⌘' => '#',
                        '–' | '—' => '-',
                        other => other,
                    };
                    self.font_engine.get_glyph(fallback, font_size)
                });
                if let Some(glyph) = target_glyph {
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
            let normalized_ch = match ch {
                '●' | '•' => '*',
                '▲' => '^',
                '▾' | '▼' => 'v',
                '↗' | '→' => '>',
                '↵' => '<',
                '⌖' => '+',
                '⌘' => '#',
                '–' | '—' => '-',
                other => other,
            };
            if let Some(glyph) = crate::font_table::get_glyph_8x8(normalized_ch) {
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

    fn blit_pixels(&mut self, origin: Point, width: usize, height: usize, pixels: &[u32]) {
        if width == 0 || height == 0 || pixels.is_empty() {
            return;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let start_x = (origin.x.max(0.0) as usize).min(self.width);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let start_y = (origin.y.max(0.0) as usize).min(self.height);
        let end_x = (start_x + width).min(self.width);
        let end_y = (start_y + height).min(self.height);

        for y in start_y..end_y {
            let src_y = y - start_y;
            let dst_offset = y * self.width + start_x;
            let src_offset = src_y * width;
            let copy_w = end_x - start_x;
            if src_offset + copy_w <= pixels.len() && dst_offset + copy_w <= self.pixels.len() {
                self.pixels[dst_offset..dst_offset + copy_w]
                    .copy_from_slice(&pixels[src_offset..src_offset + copy_w]);
            }
        }
    }

    fn end_frame(&mut self) {
        self.dirty_clip.clear();
    }
}
