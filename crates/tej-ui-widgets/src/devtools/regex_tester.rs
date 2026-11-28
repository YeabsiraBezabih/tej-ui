//! Interactive regular expression tester with live capture group highlighting.
//! Styled with Tej's monospaced inputs, syntax color fills, and match tag badges.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A matched range with its matched string and group captures.
#[derive(Debug, Clone, PartialEq)]
pub struct RegexMatch {
    pub start: usize,
    pub end: usize,
    pub full_text: String,
    pub groups: Vec<String>,
}

/// State for the interactive regex tester.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RegexTesterState {
    pub pattern: String,
    pub test_text: String,
    pub is_case_insensitive: bool,
    pub is_multiline: bool,
    pub matches: Vec<RegexMatch>,
    pub error_message: Option<String>,
}

impl RegexTesterState {
    #[must_use]
    pub fn new(pattern: impl Into<String>, test_text: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            test_text: test_text.into(),
            is_case_insensitive: false,
            is_multiline: false,
            matches: Vec::new(),
            error_message: None,
        }
    }
}

/// Tej styled Regex Tester component.
#[derive(Debug, Clone, PartialEq)]
pub struct RegexTester {
    pub bounds: Rect,
    pub background: Color,
    pub border_color: Color,
    pub corner_radius: f32,
}

impl Default for RegexTester {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            background: Color::WHITE,
            border_color: Color::from_rgb_u32(0x00D0_D5DD), // Tej Gray 300
            corner_radius: 8.0,
        }
    }
}
