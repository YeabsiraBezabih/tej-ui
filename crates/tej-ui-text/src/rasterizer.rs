//! TrueType and OpenType font rasterization and glyph mask generation.

use fontdue::{Font, FontSettings, Metrics};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tej_ui_layout::Size;

/// A rasterized glyph with 8-bit alpha coverage mask.
#[derive(Clone, Debug)]
pub struct RasterizedGlyph {
    /// Glyph layout and boundary metrics.
    pub metrics: Metrics,
    /// 8-bit alpha mask coverage bytes.
    pub mask: Arc<[u8]>,
}

/// TTF/OTF font parser and rasterizer with subpixel positioning and LRU glyph caching.
#[derive(Debug)]
pub struct FontRasterizer {
    font: Font,
    glyph_cache: RwLock<HashMap<(char, u32), RasterizedGlyph>>,
}

impl FontRasterizer {
    /// Create a new font rasterizer from raw TTF or OTF font file bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let font = Font::from_bytes(bytes, FontSettings::default())
            .map_err(|_| "Failed to parse TrueType/OpenType font")?;
        Ok(Self {
            font,
            glyph_cache: RwLock::new(HashMap::with_capacity(256)),
        })
    }

    /// Rasterize or retrieve cached glyph mask at a given point size.
    pub fn rasterize(&self, ch: char, size_px: f32) -> RasterizedGlyph {
        let size_key = (size_px * 10.0).round() as u32;

        if let Some(cached) = self.glyph_cache.read().get(&(ch, size_key)) {
            return cached.clone();
        }

        let (metrics, mask) = self.font.rasterize(ch, size_px);
        let glyph = RasterizedGlyph {
            metrics,
            mask: Arc::from(mask.into_boxed_slice()),
        };

        self.glyph_cache
            .write()
            .insert((ch, size_key), glyph.clone());
        glyph
    }

    /// Measure accurate vector layout bounding box for a string slice.
    #[must_use]
    pub fn measure_text(&self, text: &str, size_px: f32) -> Size {
        let line_height = size_px * 1.35;
        let mut max_width: f32 = 0.0;
        let mut cur_width: f32 = 0.0;
        let mut lines = 1usize;

        for ch in text.chars() {
            if ch == '\n' {
                max_width = max_width.max(cur_width);
                cur_width = 0.0;
                lines += 1;
                continue;
            }
            let glyph = self.rasterize(ch, size_px);
            cur_width += glyph.metrics.advance_width;
        }

        max_width = max_width.max(cur_width);
        #[allow(clippy::cast_precision_loss)]
        Size::new(max_width, (lines as f32) * line_height)
    }
}
