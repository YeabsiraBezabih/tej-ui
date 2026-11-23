//! Hover helper tooltip popup (shadcn `Tooltip`).

use tej_ui_core::Signal;
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::View;

/// Retained state for a mounted Tooltip.
#[derive(Debug, Default)]
pub struct TooltipState {
    pub bounds: Rect,
    pub is_visible: bool,
    pub text: String,
}

/// Floating tooltip communicating context on hover.
#[derive(Debug, Clone)]
pub struct Tooltip {
    text: String,
    is_visible: Signal<bool>,
}

impl Tooltip {
    /// Creates a tooltip with text content and visibility signal.
    #[must_use]
    pub fn new(text: impl Into<String>, is_visible: Signal<bool>) -> Self {
        Self {
            text: text.into(),
            is_visible,
        }
    }

    /// Sets tooltip visibility.
    pub fn set_visible(&self, visible: bool) {
        self.is_visible.set(visible);
    }
}

impl View for Tooltip {
    type State = TooltipState;

    fn build(&self) -> Self::State {
        let width = (self.text.len() as f32) * 7.0 + 16.0;
        TooltipState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(width, 28.0)),
            is_visible: self.is_visible.get(),
            text: self.text.clone(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.is_visible = self.is_visible.get();
        state.text = self.text.clone();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
