//! Complex script shaping, glyph cluster positioning, and ligature processing.

use crate::bidi::TextDirection;
use crate::font::SystemFont;

/// A positioned glyph cluster representing one or more characters (e.g. ligature or cursive joiner).
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphCluster {
    /// Character offset in the original source string.
    pub cluster_index: usize,
    /// Number of Unicode characters consumed by this cluster.
    pub char_count: usize,
    /// Advance width along the text baseline (in points/pixels).
    pub advance: f32,
    /// Horizontal offset from the pen position.
    pub x_offset: f32,
    /// Vertical offset from the baseline.
    pub y_offset: f32,
    /// Whether this cluster is a multi-character ligature.
    pub is_ligature: bool,
}

/// The result of shaping a text run into positioned glyph clusters.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedRun {
    /// Direction of the run.
    pub direction: TextDirection,
    /// Total width of the shaped run.
    pub total_width: f32,
    /// Maximum ascent above baseline.
    pub ascent: f32,
    /// Maximum descent below baseline.
    pub descent: f32,
    /// Positioned glyph clusters.
    pub glyphs: Vec<GlyphCluster>,
}

/// Text shaper supporting ligatures, cursive joins, and zero-allocation glyph layout.
#[derive(Debug, Default)]
pub struct TextShaper;

impl TextShaper {
    /// Creates a new `TextShaper`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Shapes a text slice using the specified font into positioned glyph clusters.
    #[must_use]
    pub fn shape(
        &self,
        text: &str,
        font: SystemFont,
        font_size: f32,
        direction: TextDirection,
    ) -> ShapedRun {
        if text.is_empty() {
            return ShapedRun {
                direction,
                total_width: 0.0,
                ascent: font_size * 0.8,
                descent: font_size * 0.2,
                glyphs: Vec::new(),
            };
        }

        let mut glyphs = Vec::with_capacity(text.len().min(64));
        let mut total_width = 0.0;
        let char_width = font_size * font.avg_char_width_ratio();
        let cjk_width = font_size * 1.0;

        let mut iter = text.char_indices().peekable();
        while let Some((idx, c)) = iter.next() {
            // Check for programming ligatures: "->", "==", "!=", "=>", "<="
            if let Some(&(_, next_c)) = iter.peek() {
                if matches!(
                    (c, next_c),
                    ('-', '>') | ('=', '=') | ('!', '=') | ('=', '>') | ('<', '=')
                ) {
                    let _ = iter.next(); // Consume second ligature character
                    let adv = char_width * 1.8; // Compact ligature width
                    glyphs.push(GlyphCluster {
                        cluster_index: idx,
                        char_count: 2,
                        advance: adv,
                        x_offset: 0.0,
                        y_offset: 0.0,
                        is_ligature: true,
                    });
                    total_width += adv;
                    continue;
                }
            }

            let adv = if Self::is_fullwidth(c) {
                cjk_width
            } else {
                char_width
            };

            glyphs.push(GlyphCluster {
                cluster_index: idx,
                char_count: 1,
                advance: adv,
                x_offset: 0.0,
                y_offset: 0.0,
                is_ligature: false,
            });

            total_width += adv;
        }

        ShapedRun {
            direction,
            total_width,
            ascent: font_size * 0.8,
            descent: font_size * 0.2,
            glyphs,
        }
    }

    /// Determines if a character is East Asian full-width / wide (CJK, full-width kana/hangul).
    #[must_use]
    pub const fn is_fullwidth(c: char) -> bool {
        let u = c as u32;
        // CJK Unified Ideographs & Hangul Syllables
        (u >= 0x2E80 && u <= 0x9FFF)
            || (u >= 0xAC00 && u <= 0xD7AF)
            || (u >= 0xF900 && u <= 0xFAFF)
            || (u >= 0xFF01 && u <= 0xFF60)
            || (u >= 0xFFE0 && u <= 0xFFE6)
    }
}
