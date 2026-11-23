//! Extended visualizers: Financial Candlesticks, Conversion Funnel, Spider/Radar, and Donut Charts.
//! Styled with Tej's clean numeric labels, tinted area fills, and tooltip legends.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A OHLC candlestick data point.
#[derive(Debug, Clone, PartialEq)]
pub struct CandlestickPoint {
    pub time_str: String,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
}

/// A stage in a conversion funnel chart.
#[derive(Debug, Clone, PartialEq)]
pub struct FunnelStage {
    pub name: String,
    pub count: usize,
    pub conversion_rate: f32,
    pub color: Color,
}

/// A slice in a Donut / Pie chart.
#[derive(Debug, Clone, PartialEq)]
pub struct DonutSlice {
    pub label: String,
    pub value: f32,
    pub color: Color,
}

/// State for financial candlestick visualizer.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CandlestickState {
    pub points: Vec<CandlestickPoint>,
    pub selected_index: Option<usize>,
}

/// Tej styled Candlestick component.
#[derive(Debug, Clone, PartialEq)]
pub struct CandlestickChart {
    pub bounds: Rect,
    pub bull_color: Color,
    pub bear_color: Color,
    pub wick_thickness: f32,
    pub body_width: f32,
}

impl Default for CandlestickChart {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            bull_color: Color::from_rgb_u32(0x12B76A), // Tej Success 500
            bear_color: Color::from_rgb_u32(0xF04438), // Tej Error 500
            wick_thickness: 1.5,
            body_width: 8.0,
        }
    }
}
