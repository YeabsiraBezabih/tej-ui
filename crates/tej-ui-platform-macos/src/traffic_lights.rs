//! macOS Traffic Light buttons (Close, Minimize, Zoom) and unified titlebar geometry.

use tej_ui_hal::Color;
use tej_ui_layout::{Point, Rect};

/// Hover state across macOS traffic light controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HoveredTrafficLight {
    #[default]
    None,
    Close,
    Minimize,
    Zoom,
}

/// Standard AppKit window traffic light buttons geometry and hover coordinator.
#[derive(Debug, Clone)]
pub struct TrafficLightsController {
    close_rect: Rect,
    minimize_rect: Rect,
    zoom_rect: Rect,
    hovered: HoveredTrafficLight,
    is_active_window: bool,
}

impl TrafficLightsController {
    /// Standard macOS AppKit placement (14pt button diameter, 8pt gap, 20pt left margin, 14pt top margin).
    pub fn new() -> Self {
        let size = 14.0;
        let gap = 8.0;
        let left = 20.0;
        let top = 14.0;

        let close_rect = Rect::new(left, top, size, size);
        let minimize_rect = Rect::new(left + size + gap, top, size, size);
        let zoom_rect = Rect::new(left + (size + gap) * 2.0, top, size, size);

        Self {
            close_rect,
            minimize_rect,
            zoom_rect,
            hovered: HoveredTrafficLight::None,
            is_active_window: true,
        }
    }

    /// Evaluates cursor position to update hover state.
    pub fn update_hover(&mut self, cursor: Point) -> HoveredTrafficLight {
        if self.close_rect.contains(cursor) {
            self.hovered = HoveredTrafficLight::Close;
        } else if self.minimize_rect.contains(cursor) {
            self.hovered = HoveredTrafficLight::Minimize;
        } else if self.zoom_rect.contains(cursor) {
            self.hovered = HoveredTrafficLight::Zoom;
        } else {
            self.hovered = HoveredTrafficLight::None;
        }
        self.hovered
    }

    /// Returns the authentic macOS button colors based on window active state.
    pub fn button_colors(&self) -> (Color, Color, Color) {
        if !self.is_active_window {
            // Disabled gray when window is inactive
            let inactive_gray = Color::rgb(70, 70, 70);
            return (inactive_gray, inactive_gray, inactive_gray);
        }

        (
            Color::rgb(255, 95, 86),  // Close: macOS Red (#FF5F56)
            Color::rgb(255, 189, 46), // Minimize: macOS Yellow (#FFBD2E)
            Color::rgb(39, 201, 63),  // Zoom: macOS Green (#27C93F)
        )
    }

    /// Marks the window as active so traffic lights render in color.
    pub fn activate_window(&mut self) {
        self.is_active_window = true;
    }

    /// Marks the window as inactive so traffic lights render in grayscale.
    pub fn deactivate_window(&mut self) {
        self.is_active_window = false;
    }

    pub fn hovered(&self) -> HoveredTrafficLight {
        self.hovered
    }

    pub fn close_rect(&self) -> Rect {
        self.close_rect
    }

    pub fn minimize_rect(&self) -> Rect {
        self.minimize_rect
    }

    pub fn zoom_rect(&self) -> Rect {
        self.zoom_rect
    }
}

impl Default for TrafficLightsController {
    fn default() -> Self {
        Self::new()
    }
}
