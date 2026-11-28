//! Agent Plan card displaying task decomposition, DAG step sequencing, and execution progress.
//! Formatted with Tej step indicators and milestone progress tracks.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// Status of an individual task step in an agent plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Todo,
    Running,
    Done,
    Skipped,
    Failed,
}

/// An individual task step inside an agent's plan DAG.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanStep {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub status: StepStatus,
}

/// Complete plan state managed by an autonomous agent.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AgentPlanState {
    pub title: String,
    pub steps: Vec<PlanStep>,
    pub active_step_index: usize,
}

impl AgentPlanState {
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            steps: Vec::new(),
            active_step_index: 0,
        }
    }

    pub fn add_step(&mut self, title: impl Into<String>, desc: impl Into<String>) -> u32 {
        let id = self.steps.len() as u32 + 1;
        self.steps.push(PlanStep {
            id,
            title: title.into(),
            description: desc.into(),
            status: if self.steps.is_empty() {
                StepStatus::Running
            } else {
                StepStatus::Todo
            },
        });
        id
    }

    pub fn complete_current_step(&mut self) {
        if self.active_step_index < self.steps.len() {
            self.steps[self.active_step_index].status = StepStatus::Done;
            self.active_step_index += 1;
            if self.active_step_index < self.steps.len() {
                self.steps[self.active_step_index].status = StepStatus::Running;
            }
        }
    }

    #[must_use]
    pub fn progress_ratio(&self) -> f32 {
        if self.steps.is_empty() {
            return 1.0;
        }
        let completed = self
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::Done || s.status == StepStatus::Skipped)
            .count();
        crate::count_to_f32(completed) / crate::count_to_f32(self.steps.len())
    }
}

/// Tej styled Agent Plan Card.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentPlanCard {
    pub bounds: Rect,
    pub background: Color,
    pub border_color: Color,
    pub corner_radius: f32,
}

impl Default for AgentPlanCard {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            background: Color::WHITE,
            border_color: Color::from_rgb_u32(0x00EA_ECF0), // Tej Gray 200
            corner_radius: 12.0,                            // Tej rounded-xl
        }
    }
}
