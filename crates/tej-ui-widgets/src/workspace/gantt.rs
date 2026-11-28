//! Interactive Gantt chart with task dependencies and drag-to-reschedule bars.
//! Styled with Tej's progress bars, time axis dividers, and rounded task capsules.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// A task item plotted on a Gantt timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct GanttTask {
    pub id: String,
    pub title: String,
    pub start_day: f32,
    pub duration_days: f32,
    pub progress: f32, // 0.0 to 1.0
    pub dependency_ids: Vec<String>,
    pub color: Color,
}

/// State for the Gantt chart.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GanttState {
    pub tasks: Vec<GanttTask>,
    pub total_days: f32,
    pub selected_task_id: Option<String>,
}

impl GanttState {
    #[must_use]
    pub fn new(tasks: Vec<GanttTask>, total_days: f32) -> Self {
        Self {
            tasks,
            total_days,
            selected_task_id: None,
        }
    }
}

/// Tej styled Gantt Chart component.
#[derive(Debug, Clone, PartialEq)]
pub struct GanttChart {
    pub bounds: Rect,
    pub row_height: f32,
    pub day_column_width: f32,
    pub background: Color,
    pub border_color: Color,
    pub grid_line_color: Color,
}

impl Default for GanttChart {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            row_height: 38.0,
            day_column_width: 32.0,
            background: Color::WHITE,
            border_color: Color::from_rgb_u32(0x00EA_ECF0), // Tej Gray 200
            grid_line_color: Color::from_rgb_u32(0x00F2_F4F7), // Tej Gray 100
        }
    }
}
