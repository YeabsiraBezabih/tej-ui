//! Gauges, Activity Rings (concentric arcs), and Dot-Matrix Displays.
//! Styled with Tej's high-visibility progress rings and status indicators.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A concentric ring in an Activity Rings widget (Apple Watch style).
#[derive(Debug, Clone, PartialEq)]
pub struct RingMetric {
    pub name: String,
    pub current: f32,
    pub target: f32,
    pub color: Color,
    pub background_color: Color,
}

/// State for concentric activity rings.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActivityRingsState {
    pub rings: Vec<RingMetric>,
}

impl ActivityRingsState {
    #[must_use]
    pub fn new(rings: Vec<RingMetric>) -> Self {
        Self { rings }
    }
}

/// A 5x7 or 8x8 dot-matrix LED readout display.
#[derive(Debug, Clone, PartialEq)]
pub struct DotMatrixDisplay {
    pub bounds: Rect,
    pub rows: usize,
    pub cols: usize,
    pub dot_size: f32,
    pub dot_gap: f32,
    pub active_color: Color,
    pub inactive_color: Color,
}

impl Default for DotMatrixDisplay {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            rows: 7,
            cols: 20,
            dot_size: 3.0,
            dot_gap: 1.5,
            active_color: Color::from_rgb_u32(0x007F_56D9), // Tej Brand 600
            inactive_color: Color::from_rgb_u32(0x001D_2939), // Tej Gray 800
        }
    }
}

/// Semi-circular Radial Gauge.
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGauge {
    pub bounds: Rect,
    pub min_value: f32,
    pub max_value: f32,
    pub current_value: f32,
    pub thickness: f32,
    pub track_color: Color,
    pub fill_color: Color,
}

impl Default for RadialGauge {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            min_value: 0.0,
            max_value: 100.0,
            current_value: 0.0,
            thickness: 12.0,
            track_color: Color::from_rgb_u32(0x00EA_ECF0), // Tej Gray 200
            fill_color: Color::from_rgb_u32(0x007F_56D9),  // Tej Brand 600
        }
    }
}
