//! Continuous range slider input (shadcn `Slider`).

use std::sync::Arc;
use tej_core::Signal;
use tej_layout::{Point, Rect, Size};
use tej_view::View;

/// Retained state for a mounted Slider.
#[derive(Debug, Default)]
pub struct SliderState {
    pub bounds: Rect,
    pub current_value: f32,
    pub normalized_ratio: f32,
}

/// Horizontal range slider widget.
#[derive(Clone)]
pub struct Slider {
    value: Signal<f32>,
    min: f32,
    max: f32,
    step: f32,
    on_change: Option<Arc<dyn Fn(f32) + Send + Sync>>,
    disabled: bool,
}

impl std::fmt::Debug for Slider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slider")
            .field("value", &self.value.get())
            .field("min", &self.min)
            .field("max", &self.max)
            .finish()
    }
}

impl Slider {
    /// Creates a new slider bound to a reactive signal with min and max bounds.
    #[must_use]
    pub fn new(value: Signal<f32>, min: f32, max: f32) -> Self {
        Self {
            value,
            min,
            max,
            step: 1.0,
            on_change: None,
            disabled: false,
        }
    }

    /// Sets step increment.
    #[must_use]
    pub const fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Sets on_change callback.
    #[must_use]
    pub fn on_change<F>(mut self, cb: F) -> Self
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(cb));
        self
    }

    /// Updates slider value from a normalized click/drag ratio [0.0 .. 1.0].
    pub fn set_normalized(&self, ratio: f32) {
        if self.disabled {
            return;
        }
        let clamped = ratio.clamp(0.0, 1.0);
        let raw = self.min + clamped * (self.max - self.min);
        let stepped = (raw / self.step).round() * self.step;
        let final_val = stepped.clamp(self.min, self.max);
        self.value.set(final_val);
        if let Some(cb) = &self.on_change {
            cb(final_val);
        }
    }
}

impl View for Slider {
    type State = SliderState;

    fn build(&self) -> Self::State {
        let val = self.value.get();
        let range = (self.max - self.min).max(0.0001);
        let ratio = ((val - self.min) / range).clamp(0.0, 1.0);

        SliderState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(200.0, 20.0)),
            current_value: val,
            normalized_ratio: ratio,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let val = self.value.get();
        let range = (self.max - self.min).max(0.0001);
        state.current_value = val;
        state.normalized_ratio = ((val - self.min) / range).clamp(0.0, 1.0);
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
