//! TrueType / OpenType font rasterizer and glyph cache for Linux platforms.
//!
//! Automatically discovers Google Fonts (e.g. Noto Sans, Adwaita, DejaVu Sans)
//! and provides antialiased 8-bit alpha mask glyph rasterization with zero-allocation steady-state caching.

use fontdue::{Font, FontSettings};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

/// Cached rasterized glyph metrics and antialiased 8-bit alpha bitmap.
#[derive(Clone, Debug)]
pub struct CachedGlyph {
    pub width: usize,
    pub height: usize,
    pub xmin: i32,
    pub ymin: i32,
    pub advance_width: f32,
    pub bitmap: Arc<[u8]>,
}

/// TTF/OTF font rasterization engine backed by `fontdue`.
#[derive(Debug)]
pub struct FontEngine {
    fonts: Vec<Font>,
    cache: Mutex<HashMap<(char, u32), CachedGlyph>>,
}

impl Default for FontEngine {
    fn default() -> Self {
        Self::load_system_or_fallback()
    }
}

impl FontEngine {
    /// Discover and load TrueType/OpenType fonts in cascading priority order.
    #[must_use]
    pub fn load_system_or_fallback() -> Self {
        let mut fonts = Vec::new();

        let candidates = [
            "assets/fonts/Inter-Regular.ttf",
            "assets/fonts/JetBrainsMono.ttf",
            "assets/fonts/NotoSansSymbols-Regular.ttf",
            "../TejStudio/assets/fonts/Inter-Regular.ttf",
            "../TejStudio/assets/fonts/JetBrainsMono.ttf",
            "../TejStudio/assets/fonts/NotoSansSymbols-Regular.ttf",
            "/usr/share/fonts/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/liberation-sans/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/TTF/OpenSans-Regular.ttf",
        ];

        for path in &candidates {
            if let Ok(bytes) = fs::read(path) {
                if let Ok(font) = Font::from_bytes(bytes, FontSettings::default()) {
                    fonts.push(font);
                }
            }
        }

        Self {
            fonts,
            cache: Mutex::new(HashMap::with_capacity(256)),
        }
    }

    /// Load font explicitly from raw TTF/OTF byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let font = Font::from_bytes(bytes, FontSettings::default())
            .map_err(|_| "Failed to parse TTF bytes")?;
        Ok(Self {
            fonts: vec![font],
            cache: Mutex::new(HashMap::with_capacity(256)),
        })
    }

    /// Check whether any vector TrueType font is actively loaded.
    #[must_use]
    pub fn has_vector_font(&self) -> bool {
        !self.fonts.is_empty()
    }

    /// Retrieve or rasterize a glyph with subpixel antialiasing at the specified point size.
    pub fn get_glyph(&self, ch: char, font_size: f32) -> Option<CachedGlyph> {
        if self.fonts.is_empty() {
            return None;
        }
        let size_key = (font_size * 10.0).round() as u32;

        // 1. Fast read from cache
        {
            let cache = self.cache.lock();
            if let Some(glyph) = cache.get(&(ch, size_key)) {
                return Some(glyph.clone());
            }
        }

        // 2. Search loaded fonts in cascade order
        for font in &self.fonts {
            let glyph_idx = font.lookup_glyph_index(ch);
            if glyph_idx != 0 || ch == ' ' || ch == '\t' {
                let (metrics, bitmap) = font.rasterize(ch, font_size);
                let cached = CachedGlyph {
                    width: metrics.width,
                    height: metrics.height,
                    xmin: metrics.xmin,
                    ymin: metrics.ymin,
                    advance_width: metrics.advance_width,
                    bitmap: Arc::from(bitmap.into_boxed_slice()),
                };

                let mut cache = self.cache.lock();
                cache.insert((ch, size_key), cached.clone());
                return Some(cached);
            }
        }

        None
    }
}
