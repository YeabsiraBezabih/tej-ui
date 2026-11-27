use crate::font::SystemFont;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tej_ui_layout::Size;

/// Zero-copy text measurement engine with thread-safe content-hashed caching.
#[derive(Debug)]
pub struct TextMeasurer {
    cache: RwLock<HashMap<(u64, u32), Size>>,
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
        if text.is_empty() {
            return Size::new(0.0, font_size * 1.2);
        }

        // Content hash over (text, font_size.to_bits()) to prevent ABA pointer-reuse cache poisoning
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        font_size.to_bits().hash(&mut hasher);
        let hash_key = hasher.finish();
        let font_key = font.cache_id() ^ font_size.to_bits();

        if let Some(&cached) = self.cache.read().get(&(hash_key, font_key)) {
            return cached;
        }

        let char_count = text.chars().count();
        let avg_width = font_size * font.avg_char_width_ratio();
        let computed_width = (char_count as f32) * avg_width;
        let line_height = font_size * 1.35;

        let size = Size::new(computed_width, line_height);

        // Store in cache
        let mut w = self.cache.write();
        if w.len() > 2048 {
            w.clear();
        }
        w.insert((hash_key, font_key), size);
        size
    }
}
