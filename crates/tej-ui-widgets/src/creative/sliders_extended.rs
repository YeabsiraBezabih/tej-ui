//! Histogram range sliders, ratio sliders, and ruler sliders.
//! Styled with Tej's range selection fills, histogram frequency bars, and tick marks.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// State for a dual-handle histogram range slider (e.g. price or timestamp filter).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HistogramSliderState {
    pub bucket_counts: Vec<u32>,
    pub max_bucket_count: u32,
    pub range_start: f32,             // 0.0 to 1.0
    pub range_end: f32,               // 0.0 to 1.0
    pub active_handle: Option<usize>, // 0 for start, 1 for end
}

impl HistogramSliderState {
    #[must_use]
    pub fn new(bucket_counts: Vec<u32>, range_start: f32, range_end: f32) -> Self {
        let max_bucket_count = bucket_counts.iter().copied().max().unwrap_or(1).max(1);
        Self {
            bucket_counts,
            max_bucket_count,
            range_start: range_start.clamp(0.0, 1.0),
            range_end: range_end.clamp(range_start, 1.0),
            active_handle: None,
        }
    }
}

/// Tej styled Histogram Slider component.
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramSlider {
    pub bounds: Rect,
    pub bar_gap: f32,
    pub handle_diameter: f32,
    pub unselected_bar_color: Color,
    pub selected_bar_color: Color,
    pub handle_color: Color,
}

impl Default for HistogramSlider {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            bar_gap: 1.5,
            handle_diameter: 16.0,
            unselected_bar_color: Color::from_rgb_u32(0x00EA_ECF0), // Tej Gray 200
            selected_bar_color: Color::from_rgb_u32(0x007F_56D9),   // Tej Brand 600
            handle_color: Color::WHITE,
        }
    }
}
