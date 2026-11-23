//! Split buttons, status buttons, and hold-to-confirm buttons.
//! Styled with Tej's primary brand fills, chevron split dividers, and hold progress meters.

use std::time::Duration;
use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// State for a hold-to-confirm button (preventing accidental destructive actions).
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmButtonState {
    pub label: String,
    pub is_holding: bool,
    pub hold_duration: Duration,
    pub required_hold: Duration,
    pub is_confirmed: bool,
}

impl Default for ConfirmButtonState {
    fn default() -> Self {
        Self {
            label: "Hold to Confirm".to_string(),
            is_holding: false,
            hold_duration: Duration::ZERO,
            required_hold: Duration::from_millis(1500),
            is_confirmed: false,
        }
    }
}

impl ConfirmButtonState {
    #[must_use]
    pub fn new(label: impl Into<String>, required: Duration) -> Self {
        Self {
            label: label.into(),
            required_hold: required,
            ..Default::default()
        }
    }

    pub fn update_hold(&mut self, delta: Duration) -> bool {
        if self.is_holding && !self.is_confirmed {
            self.hold_duration += delta;
            if self.hold_duration >= self.required_hold {
                self.is_confirmed = true;
                return true;
            }
        }
        false
    }

    pub fn reset(&mut self) {
        self.is_holding = false;
        self.hold_duration = Duration::ZERO;
        self.is_confirmed = false;
    }

    #[must_use]
    pub fn progress_ratio(&self) -> f32 {
        (self.hold_duration.as_secs_f32() / self.required_hold.as_secs_f32()).clamp(0.0, 1.0)
    }
}

/// Tej styled Split Button component.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitButton {
    pub bounds: Rect,
    pub primary_label: String,
    pub background: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub corner_radius: f32,
    pub is_menu_open: bool,
}

impl Default for SplitButton {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            primary_label: "Save".to_string(),
            background: Color::from_rgb_u32(0x7F56D9), // Tej Brand 600
            border_color: Color::from_rgb_u32(0x6941C6), // Tej Brand 700
            text_color: Color::WHITE,
            corner_radius: 8.0,
            is_menu_open: false,
        }
    }
}
