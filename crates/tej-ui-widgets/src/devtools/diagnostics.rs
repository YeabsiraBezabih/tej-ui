//! Developer diagnostics: Problem Panel, Key-Value Environment Editor, Uptime Bar, and Git Blame Gutter.
//! Styled with Tej's status indicators and data tables.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// Severity of a compiler diagnostic or code problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// A compiler or linter diagnostic item.
#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosticProblem {
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub rule_id: Option<String>,
}

/// Daily uptime status bar (e.g., 90-day uptime strip).
#[derive(Debug, Clone, PartialEq)]
pub struct UptimeDay {
    pub date_str: String,
    pub uptime_percentage: f32, // 100.0 = perfect
    pub incident_count: usize,
}

/// Key-Value configuration row.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyValueEntry {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}

/// Tej styled Uptime Bar component.
#[derive(Debug, Clone, PartialEq)]
pub struct UptimeBar {
    pub bounds: Rect,
    pub day_bar_width: f32,
    pub day_bar_gap: f32,
    pub corner_radius: f32,
}

impl Default for UptimeBar {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            day_bar_width: 3.0,
            day_bar_gap: 2.0,
            corner_radius: 2.0,
        }
    }
}

impl UptimeBar {
    #[must_use]
    pub fn color_for_uptime(&self, percentage: f32) -> Color {
        if percentage >= 99.9 {
            Color::from_rgb_u32(0x0012_B76A) // Tej Success 500
        } else if percentage >= 95.0 {
            Color::from_rgb_u32(0x00F7_9009) // Tej Warning 500
        } else {
            Color::from_rgb_u32(0x00F0_4438) // Tej Error 500
        }
    }
}
