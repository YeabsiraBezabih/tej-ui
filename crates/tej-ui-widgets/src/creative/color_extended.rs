//! Gradient stop sliders, Palette strips, Aspect Ratio pickers, and Seed fields.
//! Styled with Tej's color swatch borders, aspect ratio buttons, and randomizer chips.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A color stop along a linear gradient.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorStop {
    pub position: f32, // 0.0 to 1.0
    pub color: Color,
}

/// State for a multi-stop gradient editor.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GradientEditorState {
    pub stops: Vec<ColorStop>,
    pub selected_stop_index: Option<usize>,
}

impl GradientEditorState {
    #[must_use]
    pub fn new_two_stop(first: Color, second: Color) -> Self {
        Self {
            stops: vec![
                ColorStop {
                    position: 0.0,
                    color: first,
                },
                ColorStop {
                    position: 1.0,
                    color: second,
                },
            ],
            selected_stop_index: None,
        }
    }
}

/// Common aspect ratios for creative image generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspectRatioPreset {
    Square1x1,
    Landscape16x9,
    Portrait9x16,
    Classic4x3,
    Cinema21x9,
}

impl AspectRatioPreset {
    #[must_use]
    pub fn ratio(&self) -> f32 {
        match self {
            Self::Square1x1 => 1.0,
            Self::Landscape16x9 => 16.0 / 9.0,
            Self::Portrait9x16 => 9.0 / 16.0,
            Self::Classic4x3 => 4.0 / 3.0,
            Self::Cinema21x9 => 21.0 / 9.0,
        }
    }

    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Square1x1 => "1:1",
            Self::Landscape16x9 => "16:9",
            Self::Portrait9x16 => "9:16",
            Self::Classic4x3 => "4:3",
            Self::Cinema21x9 => "21:9",
        }
    }
}

/// Tej styled Aspect Ratio Picker.
#[derive(Debug, Clone, PartialEq)]
pub struct AspectRatioPicker {
    pub bounds: Rect,
    pub selected: AspectRatioPreset,
    pub background: Color,
    pub border_color: Color,
    pub active_border_color: Color,
}

impl Default for AspectRatioPicker {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            selected: AspectRatioPreset::Square1x1,
            background: Color::WHITE,
            border_color: Color::from_rgb_u32(0xD0D5DD), // Tej Gray 300
            active_border_color: Color::from_rgb_u32(0x7F56D9), // Tej Brand 600
        }
    }
}
