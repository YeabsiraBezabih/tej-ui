//! Rotary knobs, audio level meters, and radial dials.
//! Styled with Tej's high-contrast indicators, pointer notches, and dB level tracks.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// State for a rotary dial knob.
#[derive(Debug, Clone, PartialEq)]
pub struct KnobState {
    pub value: f32, // 0.0 to 1.0
    pub min_angle_rad: f32,
    pub max_angle_rad: f32,
    pub is_dragging: bool,
    pub drag_start_y: f32,
    pub start_value: f32,
}

impl Default for KnobState {
    fn default() -> Self {
        Self {
            value: 0.5,
            min_angle_rad: -2.356, // -135 degrees
            max_angle_rad: 2.356,  // +135 degrees
            is_dragging: false,
            drag_start_y: 0.0,
            start_value: 0.5,
        }
    }
}

impl KnobState {
    #[must_use]
    pub fn current_angle_rad(&self) -> f32 {
        self.min_angle_rad + self.value * (self.max_angle_rad - self.min_angle_rad)
    }

    pub fn on_drag_delta_y(&mut self, dy: f32) {
        if self.is_dragging {
            // Dragging upwards increases value
            let delta = -dy / 100.0;
            self.value = (self.start_value + delta).clamp(0.0, 1.0);
        }
    }
}

/// Tej styled Rotary Knob component.
#[derive(Debug, Clone, PartialEq)]
pub struct RotaryKnob {
    pub bounds: Rect,
    pub diameter: f32,
    pub body_color: Color,
    pub indicator_color: Color,
    pub track_color: Color,
}

impl Default for RotaryKnob {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            diameter: 44.0,
            body_color: Color::from_rgb_u32(0x0010_1828), // Tej Gray 900
            indicator_color: Color::from_rgb_u32(0x007F_56D9), // Tej Brand 600
            track_color: Color::from_rgb_u32(0x00EA_ECF0), // Tej Gray 200
        }
    }
}
