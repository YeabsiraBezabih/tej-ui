//! Collapsible reasoning / thinking drawer for LLM and agentic workflows.
//! Styled with Tej's sleek typography, muted card backgrounds, and progress indicators.

use std::time::Duration;
use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// Visual and execution state of an AI reasoning/thinking drawer.
#[derive(Debug, Clone, PartialEq)]
pub struct ReasoningDrawerState {
    pub is_expanded: bool,
    pub is_thinking: bool,
    pub elapsed: Duration,
    pub token_count: usize,
    pub thinking_content: String,
    pub summary: String,
}

impl Default for ReasoningDrawerState {
    fn default() -> Self {
        Self {
            is_expanded: false,
            is_thinking: false,
            elapsed: Duration::ZERO,
            token_count: 0,
            thinking_content: String::new(),
            summary: "Thinking process".to_string(),
        }
    }
}

impl ReasoningDrawerState {
    #[must_use]
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            ..Default::default()
        }
    }

    pub fn toggle(&mut self) {
        self.is_expanded = !self.is_expanded;
    }

    pub fn append_thought(&mut self, text: &str) {
        self.thinking_content.push_str(text);
        // Estimate ~4 chars per token
        self.token_count += text.len().div_ceil(4);
    }

    pub fn finish_thinking(&mut self, final_elapsed: Duration) {
        self.is_thinking = false;
        self.elapsed = final_elapsed;
    }
}

/// Tej styled Reasoning Drawer component.
#[derive(Debug, Clone, PartialEq)]
pub struct ReasoningDrawer {
    pub bounds: Rect,
    pub background: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub accent_color: Color,
    pub corner_radius: f32,
}

impl Default for ReasoningDrawer {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            // Tej Gray 50
            background: Color::from_rgb_u32(0x00F9_FAFB),
            // Tej Gray 200
            border_color: Color::from_rgb_u32(0x00EA_ECF0),
            // Tej Gray 700
            text_color: Color::from_rgb_u32(0x0034_4054),
            // Tej Brand 600
            accent_color: Color::from_rgb_u32(0x007F_56D9),
            corner_radius: 8.0,
        }
    }
}

impl ReasoningDrawer {
    #[must_use]
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn formatted_header(&self, state: &ReasoningDrawerState) -> String {
        if state.is_thinking {
            format!(
                "Thinking... ({} tokens, {:.1}s)",
                state.token_count,
                state.elapsed.as_secs_f32()
            )
        } else {
            format!(
                "Thought for {:.1}s ({} tokens)",
                state.elapsed.as_secs_f32(),
                state.token_count
            )
        }
    }
}
