//! Tool invocation tray and execution status block for agent workflows.
//! Styled with Tej status badges and structured JSON parameter inspectors.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// Execution status of an agent tool call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolExecutionStatus {
    Pending,
    Executing,
    Success,
    Failed,
}

/// State container for an individual tool invocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCallState {
    pub tool_name: String,
    pub call_id: String,
    pub arguments_json: String,
    pub output_result: Option<String>,
    pub status: ToolExecutionStatus,
    pub execution_duration_ms: u64,
    pub is_expanded: bool,
}

impl ToolCallState {
    #[must_use]
    pub fn new(
        tool_name: impl Into<String>,
        call_id: impl Into<String>,
        args: impl Into<String>,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            call_id: call_id.into(),
            arguments_json: args.into(),
            output_result: None,
            status: ToolExecutionStatus::Pending,
            execution_duration_ms: 0,
            is_expanded: false,
        }
    }

    pub fn set_success(&mut self, result: impl Into<String>, duration_ms: u64) {
        self.output_result = Some(result.into());
        self.status = ToolExecutionStatus::Success;
        self.execution_duration_ms = duration_ms;
    }

    pub fn set_failed(&mut self, error: impl Into<String>, duration_ms: u64) {
        self.output_result = Some(error.into());
        self.status = ToolExecutionStatus::Failed;
        self.execution_duration_ms = duration_ms;
    }

    pub fn toggle_expand(&mut self) {
        self.is_expanded = !self.is_expanded;
    }
}

/// Tej styled Tool Call Tray component.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCallTray {
    pub bounds: Rect,
    pub background: Color,
    pub border_color: Color,
    pub corner_radius: f32,
}

impl Default for ToolCallTray {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            background: Color::WHITE,
            border_color: Color::from_rgb_u32(0xEAECF0), // Tej Gray 200
            corner_radius: 8.0,
        }
    }
}

impl ToolCallTray {
    #[must_use]
    pub fn status_color(&self, status: ToolExecutionStatus) -> Color {
        match status {
            ToolExecutionStatus::Pending => Color::from_rgb_u32(0x98A2B3), // Gray 400
            ToolExecutionStatus::Executing => Color::from_rgb_u32(0x7F56D9), // Brand 600
            ToolExecutionStatus::Success => Color::from_rgb_u32(0x12B76A), // Success 500
            ToolExecutionStatus::Failed => Color::from_rgb_u32(0xF04438),  // Error 500
        }
    }

    #[must_use]
    pub fn badge_label(&self, status: ToolExecutionStatus) -> &'static str {
        match status {
            ToolExecutionStatus::Pending => "Queued",
            ToolExecutionStatus::Executing => "Running",
            ToolExecutionStatus::Success => "Completed",
            ToolExecutionStatus::Failed => "Failed",
        }
    }
}
