use crate::font::SystemFont;
use parking_lot::RwLock;
use std::collections::HashMap;
use tej_ui_layout::Size;

/// Zero-copy text measurement engine with thread-safe caching.
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
        let text_len = text.len();
        if text_len == 0 {
            return Size::new(0.0, font_size * 1.2);
        }

        // Fast hash of string slice pointer + length + font to avoid allocating cache keys
        let hash_key = (text.as_ptr() as u64) ^ ((text_len as u64) << 32);
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
