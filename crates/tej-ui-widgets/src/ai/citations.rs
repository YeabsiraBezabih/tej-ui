//! Citation popover, source cards, and token context meter for AI responses.
//! Styled with Tej's tag badges, subtle gray borders, and progress meters.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// Source citation data reference.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceCitation {
    pub index: usize,
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Token utilization meter measuring prompt, output, and context cache.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextMeterState {
    pub current_tokens: usize,
    pub max_tokens: usize,
    pub cached_tokens: usize,
}

impl ContextMeterState {
    #[must_use]
    pub fn new(current: usize, max: usize, cached: usize) -> Self {
        Self {
            current_tokens: current,
            max_tokens: max.max(1),
            cached_tokens: cached,
        }
    }

    #[must_use]
    pub fn usage_ratio(&self) -> f32 {
        (self.current_tokens as f32 / self.max_tokens as f32).clamp(0.0, 1.0)
    }

    #[must_use]
    pub fn formatted_summary(&self) -> String {
        format!(
            "{:.1}k / {:.1}k tokens",
            self.current_tokens as f32 / 1000.0,
            self.max_tokens as f32 / 1000.0
        )
    }
}

/// Tej styled Context Meter component.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextMeter {
    pub bounds: Rect,
    pub track_color: Color,
    pub fill_color: Color,
    pub cache_color: Color,
    pub corner_radius: f32,
}

impl Default for ContextMeter {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            track_color: Color::from_rgb_u32(0xEAECF0), // Tej Gray 200
            fill_color: Color::from_rgb_u32(0x7F56D9),  // Tej Brand 600
            cache_color: Color::from_rgb_u32(0x9E77ED), // Tej Brand 500
            corner_radius: 4.0,
        }
    }
}
