//! Extended typography options, numeric variants, and caret styles.

use tej_ui_hal::Color;

/// Text wrapping strategy across multi-line blocks.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TextWrap {
    /// Standard greedy line breaking.
    #[default]
    Wrap,
    /// Disables line wrapping.
    NoWrap,
    /// Balances line lengths across headings to avoid uneven lines.
    Balance,
    /// Prevents orphaned single words on trailing lines in editorial copy.
    Pretty,
}

/// OpenType numeric figure variants (crucial for data grids and tables).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FontVariantNumeric {
    /// Default proportional numbers.
    #[default]
    Normal,
    /// Fixed-width numbers so digits align vertically across table columns.
    TabularNums,
    /// Variable-width proportional numbers.
    ProportionalNums,
    /// Old-style lowercase numbers with ascenders and descenders.
    OldstyleNums,
    /// Lining uppercase numbers aligning with capital letters.
    LiningNums,
    /// Numbers rendered with a diagonal slash through zero.
    SlashedZero,
}

/// Advanced text underline configuration.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct TextUnderlineStyle {
    /// Vertical offset pushing underline below glyph descenders.
    pub offset: f32,
    /// Stroke thickness of the underline.
    pub thickness: f32,
    /// Independent color for the underline stroke.
    pub color: Option<Color>,
}

impl TextUnderlineStyle {
    /// Creates a crisp editorial underline style.
    #[must_use]
    pub const fn new(offset: f32, thickness: f32, color: Option<Color>) -> Self {
        Self {
            offset,
            thickness,
            color,
        }
    }
}

/// Text insertion cursor (caret) styling for text inputs and editors.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct CaretStyle {
    /// Caret bar color.
    pub color: Color,
    /// Caret width in display points (typically 1.5–2.0 pt).
    pub width: f32,
}

impl Default for CaretStyle {
    fn default() -> Self {
        Self {
            color: Color::from_rgb_u32(0x7F56D9), // Tej Brand 600
            width: 2.0,
        }
    }
}
