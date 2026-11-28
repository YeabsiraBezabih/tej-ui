//! Win32 non-client area hit-testing (WM_NCHITTEST) and Windows 11 Snap Layouts.

use tej_ui_layout::{Point, Rect};

/// Win32 non-client hit-test return codes (LRESULT from WM_NCHITTEST).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum NcHitCode {
    /// In the client area.
    Client = 1,
    /// In the titlebar / caption area (allows window dragging).
    Caption = 2,
    /// Minimize button.
    MinButton = 8,
    /// Maximize button (or Windows 11 Snap Layouts flyout trigger).
    MaxButton = 9,
    /// Left resizing border.
    Left = 10,
    /// Right resizing border.
    Right = 11,
    /// Top resizing border.
    Top = 12,
    /// Top-left resizing corner.
    TopLeft = 13,
    /// Top-right resizing corner.
    TopRight = 14,
    /// Bottom resizing border.
    Bottom = 15,
    /// Bottom-left resizing corner.
    BottomLeft = 16,
    /// Bottom-right resizing corner.
    BottomRight = 17,
    /// Close button.
    Close = 20,
}

/// Custom window non-client frame layout and hit-testing engine.
#[derive(Debug, Clone)]
pub struct Win32WindowFrame {
    window_width: f32,
    window_height: f32,
    titlebar_height: f32,
    resize_border_thickness: f32,
    snap_layouts_enabled: bool,
}

impl Win32WindowFrame {
    /// Standard Windows 11 frame with 32px titlebar and 8px invisible resize border padding.
    pub fn new(window_width: f32, window_height: f32) -> Self {
        Self {
            window_width,
            window_height,
            titlebar_height: 32.0,
            resize_border_thickness: 8.0,
            snap_layouts_enabled: true,
        }
    }

    /// Updates window dimensions on resize.
    pub fn resize(&mut self, width: f32, height: f32) {
        self.window_width = width;
        self.window_height = height;
    }

    /// Evaluates WM_NCHITTEST for a given point in window-local coordinates.
    pub fn hit_test(&self, point: Point) -> NcHitCode {
        let px = point.x;
        let py = point.y;
        let border = self.resize_border_thickness;
        let win_w = self.window_width;
        let win_h = self.window_height;

        // 1. Resizing corners and borders (checked first so borders overlap titlebar edges)
        if py < border {
            if px < border {
                return NcHitCode::TopLeft;
            }
            if px > win_w - border {
                return NcHitCode::TopRight;
            }
            return NcHitCode::Top;
        }

        if py > win_h - border {
            if px < border {
                return NcHitCode::BottomLeft;
            }
            if px > win_w - border {
                return NcHitCode::BottomRight;
            }
            return NcHitCode::Bottom;
        }

        if px < border {
            return NcHitCode::Left;
        }
        if px > win_w - border {
            return NcHitCode::Right;
        }

        // 2. Titlebar and native button regions
        if py < self.titlebar_height {
            let btn_width = 46.0;

            // Close button (rightmost)
            if px >= win_w - btn_width {
                return NcHitCode::Close;
            }
            // Maximize / Snap layout button
            if px >= win_w - btn_width * 2.0 {
                return NcHitCode::MaxButton;
            }
            // Minimize button
            if px >= win_w - btn_width * 3.0 {
                return NcHitCode::MinButton;
            }

            return NcHitCode::Caption;
        }

        // 3. Client area
        NcHitCode::Client
    }

    /// True if cursor point is hovering within the maximize button triggering Windows 11 Snap Layouts.
    pub fn is_snap_layout_hover(&self, point: Point) -> bool {
        self.snap_layouts_enabled && self.hit_test(point) == NcHitCode::MaxButton
    }

    /// Titlebar bounds.
    pub fn titlebar_rect(&self) -> Rect {
        Rect::new(0.0, 0.0, self.window_width, self.titlebar_height)
    }
}
