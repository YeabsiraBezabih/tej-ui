//! CI/CD Pipeline DAG and stage graph visualizer.
//! Styled with Tej status badges and connected node pipelines.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// Execution status of a pipeline stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStatus {
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
}

/// A stage in a CI/CD build or deployment pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineStage {
    pub name: String,
    pub status: PipelineStatus,
    pub duration_seconds: u32,
    pub steps_total: usize,
    pub steps_passed: usize,
}

/// State for the CI/CD pipeline graph.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PipelineState {
    pub stages: Vec<PipelineStage>,
    pub selected_stage_index: Option<usize>,
}

impl PipelineState {
    #[must_use]
    pub fn new(stages: Vec<PipelineStage>) -> Self {
        Self {
            stages,
            selected_stage_index: None,
        }
    }
}

/// Tej styled Pipeline component.
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineGraph {
    pub bounds: Rect,
    pub stage_width: f32,
    pub stage_height: f32,
    pub background: Color,
    pub border_color: Color,
    pub corner_radius: f32,
}

impl Default for PipelineGraph {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            stage_width: 140.0,
            stage_height: 64.0,
            background: Color::WHITE,
            border_color: Color::from_rgb_u32(0xEAECF0), // Tej Gray 200
            corner_radius: 8.0,
        }
    }
}
