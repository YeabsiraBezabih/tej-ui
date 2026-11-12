//! Toggle switch control (shadcn `Switch`).

use std::sync::Arc;
use tej_core::Signal;
use tej_hal::Color;
use tej_layout::{Rect, Size};
use tej_view::View;

/// Retained state for a mounted Switch widget.
#[derive(Debug, Default)]
pub struct SwitchState {
    pub bounds: Rect,
    pub is_checked: bool,
    pub thumb_offset_x: f32,
}

/// Two-state toggle switch control.
#[derive(Clone)]
pub struct Switch {
    checked: Signal<bool>,
    on_change: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    disabled: bool,
    active_color: Color,
    inactive_color: Color,
}

impl std::fmt::Debug for Switch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Switch")
            .field("checked", &self.checked.get())
            .field("disabled", &self.disabled)
            .finish()
    }
}

impl Switch {
    /// Creates a new switch bound to a reactive boolean signal.
    #[must_use]
    pub fn new(checked: Signal<bool>) -> Self {
        Self {
            checked,
            on_change: None,
            disabled: false,
            active_color: Color::rgb(37, 99, 235), // Primary blue
            inactive_color: Color::rgb(229, 231, 235), // Neutral gray
        }
    }

    /// Callback invoked when toggle state changes.
    #[must_use]
    pub fn on_change<F>(mut self, cb: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(cb));
        self
    }

    /// Disables user interaction.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Primary active color when switch is checked.
    #[must_use]
    pub const fn active_color(&self) -> Color {
        self.active_color
    }

    /// Background color when switch is inactive.
    #[must_use]
    pub const fn inactive_color(&self) -> Color {
        self.inactive_color
    }

    /// Toggles the switch state.
    pub fn toggle(&self) {
        if !self.disabled {
            let next = !self.checked.get();
            self.checked.set(next);
            if let Some(cb) = &self.on_change {
                cb(next);
            }
        }
    }
}

impl View for Switch {
    type State = SwitchState;

    fn build(&self) -> Self::State {
        let is_checked = self.checked.get();
        SwitchState {
            bounds: Rect::from_origin_size(tej_layout::Point::ZERO, Size::new(44.0, 24.0)),
            is_checked,
            thumb_offset_x: if is_checked { 20.0 } else { 2.0 },
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.is_checked = self.checked.get();
        state.thumb_offset_x = if state.is_checked { 20.0 } else { 2.0 };
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
