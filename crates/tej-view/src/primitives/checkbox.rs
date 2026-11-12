use crate::view::View;
use std::fmt;
use std::sync::Arc;
use tej_core::Signal;

pub type ToggleHandler = Arc<dyn Fn(bool) + Send + Sync>;

/// Two-state checkbox widget bound to a reactive `Signal<bool>`.
#[derive(Clone)]
pub struct Checkbox {
    checked: Signal<bool>,
    on_toggle: Option<ToggleHandler>,
}

impl fmt::Debug for Checkbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Checkbox")
            .field("checked", &self.checked)
            .field("has_toggle_handler", &self.on_toggle.is_some())
            .finish()
    }
}

/// State retained by a mounted Checkbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckboxState {
    pub is_checked: bool,
}

impl Checkbox {
    /// Create a checkbox bound to `checked` signal.
    pub fn new(checked: Signal<bool>) -> Self {
        Self {
            checked,
            on_toggle: None,
        }
    }

    /// Register a toggle event callback.
    pub fn on_toggle<F: Fn(bool) + Send + Sync + 'static>(mut self, handler: F) -> Self {
        self.on_toggle = Some(Arc::new(handler));
        self
    }
}

impl View for Checkbox {
    type State = CheckboxState;

    fn build(&self) -> Self::State {
        CheckboxState {
            is_checked: self.checked.get(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let current = self.checked.get();
        if state.is_checked != current {
            state.is_checked = current;
        }
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        let current = self.checked.get();
        if state.is_checked != current {
            state.is_checked = current;
        }
    }
}
