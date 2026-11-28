use crate::font::SystemFont;
use crate::font_db::{FontFamilyKind, FontRegistry};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tej_ui_layout::Size;

/// Zero-copy text measurement engine with thread-safe content-hashed caching.
#[derive(Debug)]
pub struct TextMeasurer {
    cache: RwLock<HashMap<(u64, u32, FontFamilyKind, u16), Size>>,
}

impl Default for TextMeasurer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextMeasurer {
    /// Create a new text measurer instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::with_capacity(256)),
        }
    }

    /// Measure bounding size of string slice without copying the underlying bytes.
    pub fn measure_slice(&self, text: &str, font: SystemFont, font_size: f32) -> Size {
        let family_kind = match font {
            SystemFont::Monospace => FontFamilyKind::Monospace,
            _ => FontFamilyKind::SansSerif,
        };
        let weight = match font {
            SystemFont::Headline => 700,
            SystemFont::Title => 600,
            _ => 400,
        };
        self.measure_styled(text, font_size, family_kind, weight)
    }

    /// Measure bounding size of a styled string slice using [`FontRegistry::global()`] with
    /// content-hashed caching keyed by `(text_hash, font_size.to_bits(), family_kind, weight)`.
    pub fn measure_styled(
        &self,
        text: &str,
        font_size: f32,
        family_kind: FontFamilyKind,
        weight: u16,
    ) -> Size {
        if text.is_empty() {
            let safe_size = if font_size.is_finite() && font_size > 0.0 {
                font_size
            } else {
                1.0
            };
            return Size::new(0.0, safe_size * 1.35);
        }

        // Content hash over text to prevent ABA pointer-reuse cache poisoning.
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let text_hash = hasher.finish();
        let cache_key = (text_hash, font_size.to_bits(), family_kind, weight);

        if let Some(&cached) = self.cache.read().get(&cache_key) {
            return cached;
        }

        let size = FontRegistry::global().measure_text(text, font_size, family_kind, weight);

        // Store in bounded cache
        let mut w = self.cache.write();
        if w.len() > 2048 {
            w.clear();
        }
        w.insert(cache_key, size);
        size
    }
}
