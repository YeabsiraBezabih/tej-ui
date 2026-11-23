//! Spotlight interactive cards and KPI Metric trend tiles.
//! Styled with Tej's trend pills (positive green / negative red) and subtle elevation cards.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A KPI metric trend direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricTrend {
    Positive,
    Negative,
    Neutral,
}

/// State for a high-density Metric KPI card.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricCardState {
    pub title: String,
    pub current_value: String,
    pub delta_percentage: f32,
    pub trend: MetricTrend,
    pub sparkline_points: Vec<f32>,
}

impl MetricCardState {
    #[must_use]
    pub fn new(title: impl Into<String>, value: impl Into<String>, delta: f32) -> Self {
        let trend = if delta > 0.0 {
            MetricTrend::Positive
        } else if delta < 0.0 {
            MetricTrend::Negative
        } else {
            MetricTrend::Neutral
        };
        Self {
            title: title.into(),
            current_value: value.into(),
            delta_percentage: delta,
            trend,
            sparkline_points: Vec::new(),
        }
    }
}

/// Tej styled Spotlight Card with cursor-proximity border glow.
#[derive(Debug, Clone, PartialEq)]
pub struct SpotlightCard {
    pub bounds: Rect,
    pub background: Color,
    pub border_color: Color,
    pub spotlight_color: Color,
    pub spotlight_radius: f32,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub corner_radius: f32,
}

impl Default for SpotlightCard {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            background: Color::WHITE,
            border_color: Color::from_rgb_u32(0xEAECF0), // Tej Gray 200
            spotlight_color: Color::from_rgb_u32(0x7F56D9), // Tej Brand 600
            spotlight_radius: 120.0,
            cursor_x: 0.0,
            cursor_y: 0.0,
            corner_radius: 12.0, // Tej rounded-xl
        }
    }
}
