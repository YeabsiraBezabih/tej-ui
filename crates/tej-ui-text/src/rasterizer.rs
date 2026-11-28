//! TrueType and OpenType font rasterization and glyph mask generation.

use fontdue::{Font, FontSettings, Metrics};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tej_ui_layout::Size;

/// Maximum number of rasterized glyph masks retained in the in-memory cache.
const MAX_GLYPH_CACHE_ENTRIES: usize = 4096;

/// A rasterized glyph with 8-bit alpha coverage mask.
#[derive(Clone, Debug)]
pub struct RasterizedGlyph {
    /// Glyph layout and boundary metrics.
    pub metrics: Metrics,
    /// 8-bit alpha mask coverage bytes.
    pub mask: Arc<[u8]>,
}

/// TTF/OTF font parser and rasterizer with subpixel positioning and bounded glyph caching.
#[derive(Debug)]
pub struct FontRasterizer {
    font: Font,
    glyph_cache: RwLock<HashMap<(char, u32), RasterizedGlyph>>,
}

impl FontRasterizer {
    /// Create a new font rasterizer from raw TTF or OTF font file bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        Self::from_bytes_with_index(bytes, 0)
    }

    /// Create a new font rasterizer from raw TTF, OTF, or TTC bytes at a specific collection index.
    pub fn from_bytes_with_index(
        bytes: &[u8],
        collection_index: u32,
    ) -> Result<Self, &'static str> {
        let settings = FontSettings {
            collection_index,
            ..FontSettings::default()
        };
        let font = Font::from_bytes(bytes, settings)
            .map_err(|_| "Failed to parse TrueType/OpenType font")?;
        Ok(Self {
            font,
            glyph_cache: RwLock::new(HashMap::with_capacity(256)),
        })
    }

    /// Current number of cached glyph entries.
    #[must_use]
    pub fn cache_len(&self) -> usize {
        self.glyph_cache.read().len()
    }

    /// Sanitize pixel size, clamping non-positive or non-finite values to `1.0`.
    #[inline]
    fn sanitize_size_px(size_px: f32) -> f32 {
        if size_px.is_finite() && size_px > 0.0 {
            size_px
        } else {
            1.0
        }
    }

    /// Return horizontal line metrics `(ascent, descent, new_line_size)` for `size_px`.
    ///
    /// Falls back to `(safe * 0.8, -safe * 0.2, safe * 1.35)` if the font does not provide
    /// horizontal line metrics.
    #[must_use]
    pub fn horizontal_line_metrics(&self, size_px: f32) -> (f32, f32, f32) {
        let safe = Self::sanitize_size_px(size_px);
        self.font
            .horizontal_line_metrics(safe)
            .map_or((safe * 0.8, -safe * 0.2, safe * 1.35), |m| {
                (m.ascent, m.descent, m.new_line_size)
            })
    }

    /// Rasterize or retrieve cached glyph mask at a given point size.
    pub fn rasterize(&self, ch: char, size_px: f32) -> RasterizedGlyph {
        let safe_size = Self::sanitize_size_px(size_px);
        let size_key = (safe_size * 10.0).round() as u32;

        if let Some(cached) = self.glyph_cache.read().get(&(ch, size_key)) {
            return cached.clone();
        }

        let (metrics, mask) = self.font.rasterize(ch, safe_size);
        let glyph = RasterizedGlyph {
            metrics,
            mask: Arc::from(mask.into_boxed_slice()),
        };

        let mut cache = self.glyph_cache.write();
        if cache.len() >= MAX_GLYPH_CACHE_ENTRIES {
            cache.clear();
        }
        cache.insert((ch, size_key), glyph.clone());
        glyph
    }

    /// Measure accurate vector layout bounding box for a string slice.
    #[must_use]
    pub fn measure_text(&self, text: &str, size_px: f32) -> Size {
        let safe_size = Self::sanitize_size_px(size_px);
        let line_height = safe_size * 1.35;
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
            let glyph = self.rasterize(ch, safe_size);
            cur_width += glyph.metrics.advance_width;
        }

        max_width = max_width.max(cur_width);
        #[allow(clippy::cast_precision_loss)]
        Size::new(max_width, (lines as f32) * line_height)
    }
}
