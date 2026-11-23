//! Wayland and X11 Client-Side Decorations (CSD) and GNOME / libadwaita styling.

use tej_ui_hal::Color;
use tej_ui_layout::{Point, Rect};

/// Window decoration mode negotiated with the Wayland compositor (`xdg_decoration_manager_v1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowDecorationMode {
    /// Server-side decorations drawn by the Wayland compositor or X11 window manager.
    ServerSide,
    /// Client-side decorations drawn by tej-ui (Adwaita / GTK4 headerbar style).
    #[default]
    ClientSide,
}

/// Libadwaita / GTK4 style headerbar metrics and button placement.
#[derive(Debug, Clone)]
pub struct AdwaitaHeaderBar {
    width: f32,
    height: f32,
    close_button_rect: Rect,
    corner_radius: f32,
    mode: WindowDecorationMode,
}

impl AdwaitaHeaderBar {
    /// Creates an Adwaita headerbar with standard 46px height and 12px top corner radius.
    pub fn new(window_width: f32) -> Self {
        let height = 46.0;
        let btn_size = 24.0;
        let right_padding = 12.0;

        let close_rect = Rect::new(
            window_width - right_padding - btn_size,
            (height - btn_size) * 0.5,
            btn_size,
            btn_size,
        );

        Self {
            width: window_width,
            height,
            close_button_rect: close_rect,
            corner_radius: 12.0,
            mode: WindowDecorationMode::ClientSide,
        }
    }

    /// Evaluates if a click hits the GNOME circular close button.
    pub fn is_close_button_hit(&self, point: Point) -> bool {
        self.close_button_rect.contains(point)
    }

    /// True if click is in the headerbar drag area.
    pub fn is_drag_area(&self, point: Point) -> bool {
        point.y <= self.height && !self.is_close_button_hit(point)
    }

    /// Colors for the dark GNOME Adwaita headerbar.
    pub fn adwaita_dark_colors(&self) -> (Color, Color) {
        (
            Color::rgb(36, 36, 36),    // Background headerbar
            Color::rgb(255, 255, 255), // Title and icon foreground
        )
    }

    pub fn corner_radius(&self) -> f32 {
        self.corner_radius
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn mode(&self) -> WindowDecorationMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: WindowDecorationMode) {
        self.mode = mode;
    }
}

impl Default for AdwaitaHeaderBar {
    fn default() -> Self {
        Self::new(800.0)
    }
}
