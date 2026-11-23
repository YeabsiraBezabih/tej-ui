//! System font fallback chains and emoji font resolution.

use crate::font::SystemFont;

/// Font kind categorization for fallback routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontCategory {
    /// Standard alphanumeric text (Latin, Cyrillic, Greek).
    Primary,
    /// Complex non-Latin scripts (Arabic, Devanagari, Hebrew, Thai).
    ComplexScript,
    /// East Asian ideographs (CJK Unified Ideographs, Kana, Hangul).
    Cjk,
    /// Full-color or monochrome emoji glyphs.
    Emoji,
    /// Symbol / Dingbat font.
    Symbols,
}

/// Fallback chain that resolves characters across primary and secondary system fonts.
#[derive(Debug, Clone)]
pub struct FontFallbackChain {
    /// Primary default font.
    pub primary: SystemFont,
    /// CJK system fallback font.
    pub cjk_fallback: SystemFont,
    /// Arabic/Hebrew RTL fallback font.
    pub rtl_fallback: SystemFont,
    /// Color emoji fallback font.
    pub emoji_fallback: SystemFont,
}

impl Default for FontFallbackChain {
    fn default() -> Self {
        Self {
            primary: SystemFont::system_ui(),
            cjk_fallback: SystemFont::named("Noto Sans CJK SC"),
            rtl_fallback: SystemFont::named("Noto Sans Arabic"),
            emoji_fallback: SystemFont::named("Noto Color Emoji"),
        }
    }
}

impl FontFallbackChain {
    /// Creates a fallback chain with the given primary font.
    #[must_use]
    pub const fn new(primary: SystemFont) -> Self {
        Self {
            primary,
            cjk_fallback: SystemFont::named("Noto Sans CJK SC"),
            rtl_fallback: SystemFont::named("Noto Sans Arabic"),
            emoji_fallback: SystemFont::named("Noto Color Emoji"),
        }
    }

    /// Classifies a character into its appropriate font category.
    #[must_use]
    pub const fn categorize_char(c: char) -> FontCategory {
        let u = c as u32;

        // Emoji & Pictographs
        if Self::is_emoji(c) {
            return FontCategory::Emoji;
        }

        // CJK Unified Ideographs & Hangul
        if (u >= 0x2E80 && u <= 0x9FFF) || (u >= 0xAC00 && u <= 0xD7AF) {
            return FontCategory::Cjk;
        }

        // Arabic, Hebrew, Devanagari, Thai
        if (u >= 0x0590 && u <= 0x0E7F) || (u >= 0xFB1D && u <= 0xFEFF) {
            return FontCategory::ComplexScript;
        }

        FontCategory::Primary
    }

    /// Checks if a character is a Unicode Emoji or Pictograph.
    #[must_use]
    pub const fn is_emoji(c: char) -> bool {
        let u = c as u32;
        // Miscellaneous Symbols and Pictographs: 1F300 - 1F5FF
        // Emoticons: 1F600 - 1F64F
        // Transport and Map: 1F680 - 1F6FF
        // Supplemental Symbols and Pictographs: 1F900 - 1F9FF
        // Symbols and Pictographs Extended-A: 1FA70 - 1FAFF
        // Dingbats: 2700 - 27BF
        (u >= 0x1F300 && u <= 0x1F64F)
            || (u >= 0x1F680 && u <= 0x1F6FF)
            || (u >= 0x1F900 && u <= 0x1FAFF)
            || (u >= 0x2600 && u <= 0x27BF)
    }

    /// Resolves the appropriate system font for a specific character.
    #[must_use]
    pub const fn font_for_char(&self, c: char) -> &SystemFont {
        match Self::categorize_char(c) {
            FontCategory::Emoji => &self.emoji_fallback,
            FontCategory::Cjk => &self.cjk_fallback,
            FontCategory::ComplexScript => &self.rtl_fallback,
            FontCategory::Primary | FontCategory::Symbols => &self.primary,
        }
    }
}
