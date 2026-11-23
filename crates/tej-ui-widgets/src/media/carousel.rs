//! Image carousels, before/after compare sliders, and swipable card stacks.
//! Styled with Tej's pagination dots, split divider handles, and elevation shadows.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// State for a before/after split image comparison slider.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageCompareState {
    pub split_position: f32, // 0.0 to 1.0 (0.5 = center)
    pub is_dragging: bool,
}

impl Default for ImageCompareState {
    fn default() -> Self {
        Self {
            split_position: 0.5,
            is_dragging: false,
        }
    }
}

impl ImageCompareState {
    pub fn update_split(&mut self, normalized_x: f32) {
        self.split_position = normalized_x.clamp(0.0, 1.0);
    }
}

/// State for an image carousel with dot pagination.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CarouselState {
    pub total_items: usize,
    pub current_index: usize,
}

impl CarouselState {
    #[must_use]
    pub fn new(total: usize) -> Self {
        Self {
            total_items: total,
            current_index: 0,
        }
    }

    pub fn next(&mut self) {
        if self.total_items > 0 {
            self.current_index = (self.current_index + 1) % self.total_items;
        }
    }

    pub fn prev(&mut self) {
        if self.total_items > 0 {
            self.current_index = if self.current_index == 0 {
                self.total_items - 1
            } else {
                self.current_index - 1
            };
        }
    }
}

/// Tej styled Image Compare component.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageCompare {
    pub bounds: Rect,
    pub divider_color: Color,
    pub handle_color: Color,
    pub handle_diameter: f32,
}

impl Default for ImageCompare {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            divider_color: Color::WHITE,
            handle_color: Color::WHITE,
            handle_diameter: 32.0,
        }
    }
}
