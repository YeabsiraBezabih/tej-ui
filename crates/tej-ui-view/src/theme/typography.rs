//! Typography scale and text styles for application themes.

/// Typography specification for a text role.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub font_weight: u16,
}

impl TextStyle {
    /// Constructs a new text style.
    #[must_use]
    pub const fn new(font_size: f32, line_height: f32, font_weight: u16) -> Self {
        Self {
            font_size,
            line_height,
            letter_spacing: 0.0,
            font_weight,
        }
    }
}

/// Standardized typography scale from display titles down to code and captions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypographyTokens {
    pub display: TextStyle,
    pub h1: TextStyle,
    pub h2: TextStyle,
    pub h3: TextStyle,
    pub body: TextStyle,
    pub body_small: TextStyle,
    pub caption: TextStyle,
    pub code: TextStyle,
}

impl Default for TypographyTokens {
    fn default() -> Self {
        Self {
            display: TextStyle::new(32.0, 40.0, 700),
            h1: TextStyle::new(24.0, 32.0, 600),
            h2: TextStyle::new(20.0, 28.0, 600),
            h3: TextStyle::new(16.0, 24.0, 600),
            body: TextStyle::new(14.0, 20.0, 400),
            body_small: TextStyle::new(12.0, 16.0, 400),
            caption: TextStyle::new(11.0, 14.0, 400),
            code: TextStyle::new(13.0, 18.0, 400),
        }
    }
}
