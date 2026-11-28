//! Resizable master-detail split pane and dockable divider primitive.

use std::fmt::Debug;
use tej_ui_core::Signal;

/// Orientation of a split pane container.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SplitOrientation {
    /// Horizontal split: left pane and right pane side by side.
    #[default]
    Horizontal,
    /// Vertical split: top pane and bottom pane stacked vertically.
    Vertical,
}

/// State of a resizable split divider session.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SplitDividerDrag {
    pub start_coord: f32,
    pub start_ratio: f32,
    pub total_length: f32,
}

impl SplitDividerDrag {
    /// Calculate updated split ratio clamped between `min_ratio` and `max_ratio`.
    #[must_use]
    pub fn compute_ratio(&self, current_coord: f32, min_ratio: f32, max_ratio: f32) -> f32 {
        if self.total_length <= 0.0 {
            return self.start_ratio;
        }

        let delta = current_coord - self.start_coord;
        let delta_ratio = delta / self.total_length;
        (self.start_ratio + delta_ratio).clamp(min_ratio, max_ratio)
    }
}

/// Master-detail split pane controller managing resizable split ratios and panel collapse.
#[derive(Clone, Debug)]
pub struct SplitPaneController {
    pub split_ratio: Signal<f32>,
    pub orientation: SplitOrientation,
    pub min_ratio: f32,
    pub max_ratio: f32,
    pub is_collapsed: Signal<bool>,
}

impl Default for SplitPaneController {
    fn default() -> Self {
        Self::new(Signal::new(0.5))
    }
}

impl SplitPaneController {
    /// Create a new split pane controller with a reactive split ratio (0.0 to 1.0).
    #[must_use]
    pub fn new(split_ratio: Signal<f32>) -> Self {
        Self {
            split_ratio,
            orientation: SplitOrientation::Horizontal,
            min_ratio: 0.15,
            max_ratio: 0.85,
            is_collapsed: Signal::new(false),
        }
    }

    #[must_use]
    pub fn orientation(mut self, orientation: SplitOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    #[must_use]
    pub fn limits(mut self, min: f32, max: f32) -> Self {
        self.min_ratio = min.clamp(0.0, 1.0);
        self.max_ratio = max.clamp(self.min_ratio, 1.0);
        self
    }

    /// Toggle collapse state of the primary master panel.
    pub fn toggle_collapsed(&self) {
        self.is_collapsed.update(|c| *c = !*c);
    }

    /// Set explicit split ratio.
    pub fn set_ratio(&self, ratio: f32) {
        let clamped = ratio.clamp(self.min_ratio, self.max_ratio);
        self.split_ratio.set(clamped);
    }
}
