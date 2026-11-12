//! Visual theme color tokens and geometric layout calculations.

use tej_hal::Color;
use tej_layout::Rect;

pub const COLOR_BG: Color = Color::rgb(30, 30, 46); // #1e1e2e
pub const COLOR_SIDEBAR: Color = Color::rgb(24, 24, 37); // #181825
pub const COLOR_HEADER: Color = Color::rgb(17, 17, 27); // #11111b
pub const COLOR_CARD: Color = Color::rgb(36, 37, 52); // #242534
pub const COLOR_HOVER: Color = Color::rgb(45, 47, 66); // #2d2f42
pub const COLOR_ACTIVE: Color = Color::rgb(49, 50, 68); // #313244
pub const COLOR_BORDER: Color = Color::rgb(49, 50, 68); // #313244
pub const COLOR_TEXT: Color = Color::rgb(205, 214, 244); // #cdd6f4
pub const COLOR_TEXT_MUTED: Color = Color::rgb(166, 173, 200); // #a6adc8
pub const COLOR_TEXT_DIM: Color = Color::rgb(108, 112, 134); // #6c7086
pub const COLOR_ACCENT: Color = Color::rgb(137, 180, 250); // #89b4fa
pub const COLOR_GREEN: Color = Color::rgb(166, 227, 161); // #a6e3a1
pub const COLOR_YELLOW: Color = Color::rgb(249, 226, 175); // #f9e2af

/// Dynamically compute responsive tab bounding boxes based on active window width.
#[must_use]
pub fn calculate_tab_rects(win_w: f32) -> [Rect; 3] {
    let start_x = if win_w < 550.0 {
        44.0
    } else if win_w < 750.0 {
        100.0
    } else {
        180.0
    };
    let available = (win_w - start_x - 16.0).max(90.0);
    let tab_w = ((available - 12.0) / 3.0).clamp(30.0, 180.0);
    [
        Rect::new(start_x, 4.0, tab_w, 30.0),
        Rect::new(start_x + tab_w + 6.0, 4.0, tab_w, 30.0),
        Rect::new(start_x + (tab_w + 6.0) * 2.0, 4.0, tab_w, 30.0),
    ]
}
