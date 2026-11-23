//! Distributed OpenTelemetry-style trace waterfall and network timing breakdown.
//! Styled with Tej's structured row spacing, hierarchical guides, and phase badges.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A distinct timing phase within a network request or distributed trace.
#[derive(Debug, Clone, PartialEq)]
pub struct TimingPhase {
    pub name: &'static str,
    pub start_offset_ms: f32,
    pub duration_ms: f32,
    pub color: Color,
}

/// A span entry inside a trace waterfall.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceSpan {
    pub id: String,
    pub parent_id: Option<String>,
    pub service_name: String,
    pub operation_name: String,
    pub start_time_ms: f32,
    pub duration_ms: f32,
    pub phases: Vec<TimingPhase>,
    pub has_error: bool,
}

/// State for the trace waterfall component.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TraceWaterfallState {
    pub spans: Vec<TraceSpan>,
    pub total_trace_time_ms: f32,
    pub expanded_span_id: Option<String>,
    pub selected_span_id: Option<String>,
}

impl TraceWaterfallState {
    #[must_use]
    pub fn new(spans: Vec<TraceSpan>, total_time_ms: f32) -> Self {
        Self {
            spans,
            total_trace_time_ms: total_time_ms,
            expanded_span_id: None,
            selected_span_id: None,
        }
    }
}

/// Tej styled Trace Waterfall component.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceWaterfall {
    pub bounds: Rect,
    pub row_height: f32,
    pub background: Color,
    pub border_color: Color,
    pub corner_radius: f32,
}

impl Default for TraceWaterfall {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            row_height: 32.0,
            background: Color::WHITE,
            border_color: Color::from_rgb_u32(0xEAECF0), // Tej Gray 200
            corner_radius: 8.0,
        }
    }
}
