//! Heatmap visualizations: GitHub-style 52-week Calendar Heatmap and 2D Matrix Heatmap.
//! Styled with Tej's monochromatic green/brand tints and subtle cell borders.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A cell in a 2D matrix heatmap.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatmapCell {
    pub x: usize,
    pub y: usize,
    pub value: f32, // Normalized 0.0 to 1.0
    pub label: Option<String>,
}

/// State for a 52-week contribution calendar heatmap.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CalendarHeatmapState {
    pub daily_counts: Vec<u32>, // 365 or 366 days
    pub max_count: u32,
    pub hovered_day_index: Option<usize>,
}

impl CalendarHeatmapState {
    #[must_use]
    pub fn new(daily_counts: Vec<u32>) -> Self {
        let max_count = daily_counts.iter().copied().max().unwrap_or(1).max(1);
        Self {
            daily_counts,
            max_count,
            hovered_day_index: None,
        }
    }

    #[must_use]
    pub fn intensity_for_day(&self, index: usize) -> f32 {
        if let Some(&count) = self.daily_counts.get(index) {
            (count as f32 / self.max_count as f32).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Tej styled Calendar Heatmap component.
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarHeatmap {
    pub bounds: Rect,
    pub cell_size: f32,
    pub cell_gap: f32,
    pub corner_radius: f32,
    pub empty_color: Color,
    pub max_color: Color,
}

impl Default for CalendarHeatmap {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            cell_size: 11.0,
            cell_gap: 3.0,
            corner_radius: 2.0,
            empty_color: Color::from_rgb_u32(0xEAECF0), // Tej Gray 200
            max_color: Color::from_rgb_u32(0x12B76A),   // Tej Success 500
        }
    }
}
