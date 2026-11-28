//! Extended inputs: Scrub Field (drag-to-scrub number), Money Field, OTP Input, and Tag Input.
//! Styled with Tej's input borders, brand focus rings, and tag pill badges.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// State for a drag-to-scrub numerical input field.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrubFieldState {
    pub value: f32,
    pub step: f32,
    pub min: f32,
    pub max: f32,
    pub is_dragging: bool,
    pub drag_start_x: f32,
    pub start_value: f32,
}

impl Default for ScrubFieldState {
    fn default() -> Self {
        Self {
            value: 0.0,
            step: 1.0,
            min: -1e9,
            max: 1e9,
            is_dragging: false,
            drag_start_x: 0.0,
            start_value: 0.0,
        }
    }
}

impl ScrubFieldState {
    #[must_use]
    pub fn new(val: f32, min: f32, max: f32, step: f32) -> Self {
        Self {
            value: val,
            min,
            max,
            step,
            ..Default::default()
        }
    }

    pub fn on_drag_delta(&mut self, dx: f32) {
        if self.is_dragging {
            let delta_units = (dx / 5.0) * self.step;
            self.value = (self.start_value + delta_units).clamp(self.min, self.max);
        }
    }
}

/// State for multi-cell One-Time Password (OTP) verification input.
#[derive(Debug, Clone, PartialEq)]
pub struct OtpInputState {
    pub digits: Vec<Option<char>>,
    pub length: usize,
    pub active_index: usize,
}

impl OtpInputState {
    #[must_use]
    pub fn new(length: usize) -> Self {
        Self {
            digits: vec![None; length],
            length,
            active_index: 0,
        }
    }

    pub fn input_digit(&mut self, c: char) {
        if c.is_ascii_digit() && self.active_index < self.length {
            self.digits[self.active_index] = Some(c);
            if self.active_index + 1 < self.length {
                self.active_index += 1;
            }
        }
    }

    pub fn backspace(&mut self) {
        if self.digits[self.active_index].is_some() {
            self.digits[self.active_index] = None;
        } else if self.active_index > 0 {
            self.active_index -= 1;
            self.digits[self.active_index] = None;
        }
    }

    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.digits.iter().all(std::option::Option::is_some)
    }

    #[must_use]
    pub fn code_string(&self) -> String {
        self.digits.iter().filter_map(|&d| d).collect()
    }
}

/// Tej styled OTP Input component.
#[derive(Debug, Clone, PartialEq)]
pub struct OtpInput {
    pub bounds: Rect,
    pub cell_size: f32,
    pub cell_gap: f32,
    pub background: Color,
    pub border_color: Color,
    pub focus_border_color: Color,
    pub corner_radius: f32,
}

impl Default for OtpInput {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            cell_size: 48.0,
            cell_gap: 8.0,
            background: Color::WHITE,
            border_color: Color::from_rgb_u32(0x00D0_D5DD), // Tej Gray 300
            focus_border_color: Color::from_rgb_u32(0x007F_56D9), // Tej Brand 600
            corner_radius: 8.0,
        }
    }
}
