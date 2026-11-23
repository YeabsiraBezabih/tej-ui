//! Task completion progress indicator (shadcn `Progress`).

use tej_ui_core::Signal;
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::View;

/// Retained state for a mounted Progress bar.
#[derive(Debug, Default)]
pub struct ProgressState {
    pub bounds: Rect,
    pub percentage: f32,
}

/// Horizontal progress bar reflecting task completion [0.0 .. 100.0]%.
#[derive(Debug, Clone)]
pub struct Progress {
    percentage: Signal<f32>,
    height: f32,
}

impl Progress {
    /// Creates a progress bar bound to a reactive completion percentage signal.
    #[must_use]
    pub const fn new(percentage: Signal<f32>) -> Self {
        Self {
            percentage,
            height: 8.0,
        }
    }

    /// Sets bar thickness.
    #[must_use]
    pub const fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl View for Progress {
    type State = ProgressState;

    fn build(&self) -> Self::State {
        let pct = self.percentage.get().clamp(0.0, 100.0);
        ProgressState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(240.0, self.height)),
            percentage: pct,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.percentage = self.percentage.get().clamp(0.0, 100.0);
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
