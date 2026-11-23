//! Custom titlebars with native window controls and snap layouts (Windows 11 Snap / macOS traffic lights).

use tej_ui_layout::{Point, Rect};

/// Hit-test regions within a custom window frame / client-side titlebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitlebarHitRegion {
    /// Draggable caption area that moves the window.
    CaptionDrag,
    /// Minimize button.
    MinimizeButton,
    /// Maximize / Restore toggle button.
    MaximizeButton,
    /// Close window button.
    CloseButton,
    /// Windows 11 Snap Layouts flyout trigger zone (usually hovering over maximize button).
    SnapLayoutZone,
    /// Non-interactive client area inside titlebar (e.g. search bar, tabs).
    ClientContent,
    /// Outside titlebar bounds.
    None,
}

/// Custom titlebar geometry and hit testing engine.
#[derive(Debug, Clone)]
pub struct CustomTitlebar {
    bounds: Rect,
    min_button: Rect,
    max_button: Rect,
    close_button: Rect,
    snap_zone_enabled: bool,
}

impl CustomTitlebar {
    /// Creates a new titlebar layout with standard dimensions.
    pub fn new(window_width: f32, titlebar_height: f32) -> Self {
        let btn_width = 46.0;
        let btn_height = titlebar_height;

        let close_rect = Rect::new(window_width - btn_width, 0.0, btn_width, btn_height);
        let max_rect = Rect::new(window_width - btn_width * 2.0, 0.0, btn_width, btn_height);
        let min_rect = Rect::new(window_width - btn_width * 3.0, 0.0, btn_width, btn_height);

        Self {
            bounds: Rect::new(0.0, 0.0, window_width, titlebar_height),
            min_button: min_rect,
            max_button: max_rect,
            close_button: close_rect,
            snap_zone_enabled: true,
        }
    }

    /// Resizes titlebar to match new window width.
    pub fn resize(&mut self, window_width: f32) {
        let btn_width = self.close_button.width();
        let btn_height = self.bounds.height();
        self.bounds = Rect::new(0.0, 0.0, window_width, btn_height);

        self.close_button = Rect::new(window_width - btn_width, 0.0, btn_width, btn_height);
        self.max_button = Rect::new(window_width - btn_width * 2.0, 0.0, btn_width, btn_height);
        self.min_button = Rect::new(window_width - btn_width * 3.0, 0.0, btn_width, btn_height);
    }

    /// Hit-tests a cursor point in window-local coordinates.
    pub fn hit_test(&self, point: Point) -> TitlebarHitRegion {
        if !self.bounds.contains(point) {
            return TitlebarHitRegion::None;
        }

        if self.close_button.contains(point) {
            return TitlebarHitRegion::CloseButton;
        }

        if self.max_button.contains(point) {
            if self.snap_zone_enabled {
                return TitlebarHitRegion::SnapLayoutZone;
            }
            return TitlebarHitRegion::MaximizeButton;
        }

        if self.min_button.contains(point) {
            return TitlebarHitRegion::MinimizeButton;
        }

        TitlebarHitRegion::CaptionDrag
    }

    /// Enables or disables snap layout hover zone.
    pub fn set_snap_zone_enabled(&mut self, enabled: bool) {
        self.snap_zone_enabled = enabled;
    }

    /// Titlebar bounds.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }
}
