//! Select dropdown picker (shadcn `Select`).

use std::sync::Arc;
use tej_core::Signal;
use tej_layout::{Point, Rect, Size};
use tej_view::View;

/// An option item inside a Select dropdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOption<T: Clone + PartialEq> {
    pub value: T,
    pub label: String,
}

impl<T: Clone + PartialEq> SelectOption<T> {
    #[must_use]
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
        }
    }
}

/// Retained state for a mounted Select.
#[derive(Debug, Default)]
pub struct SelectState {
    pub bounds: Rect,
    pub is_open: bool,
    pub selected_label: String,
}

/// Callback invoked when selected option changes.
pub type SelectCallback<T> = Option<Arc<dyn Fn(&T) + Send + Sync>>;

/// Dropdown select component with trigger and options menu.
#[derive(Clone)]
pub struct Select<T: Clone + PartialEq + Send + Sync + 'static> {
    selected: Signal<T>,
    is_open: Signal<bool>,
    options: Vec<SelectOption<T>>,
    placeholder: String,
    on_change: SelectCallback<T>,
}

impl<T: Clone + PartialEq + Send + Sync + std::fmt::Debug + 'static> std::fmt::Debug for Select<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Select")
            .field("selected", &self.selected.get())
            .field("is_open", &self.is_open.get())
            .field("options_count", &self.options.len())
            .finish()
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Select<T> {
    /// Creates a new Select component.
    #[must_use]
    pub fn new(
        selected: Signal<T>,
        is_open: Signal<bool>,
        options: Vec<SelectOption<T>>,
        placeholder: impl Into<String>,
    ) -> Self {
        Self {
            selected,
            is_open,
            options,
            placeholder: placeholder.into(),
            on_change: None,
        }
    }

    /// Selects an option by index and closes dropdown.
    pub fn choose_index(&self, index: usize) {
        if let Some(opt) = self.options.get(index) {
            self.selected.set(opt.value.clone());
            self.is_open.set(false);
            if let Some(cb) = &self.on_change {
                cb(&opt.value);
            }
        }
    }

    /// Toggles the dropdown menu.
    pub fn toggle_open(&self) {
        self.is_open.set(!self.is_open.get());
    }
}

impl<T: Clone + PartialEq + Send + Sync + std::fmt::Debug + 'static> View for Select<T> {
    type State = SelectState;

    fn build(&self) -> Self::State {
        let val = self.selected.get();
        let selected_label = self
            .options
            .iter()
            .find(|o| o.value == val)
            .map(|o| o.label.clone())
            .unwrap_or_else(|| self.placeholder.clone());

        SelectState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(180.0, 36.0)),
            is_open: self.is_open.get(),
            selected_label,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let val = self.selected.get();
        state.selected_label = self
            .options
            .iter()
            .find(|o| o.value == val)
            .map(|o| o.label.clone())
            .unwrap_or_else(|| self.placeholder.clone());
        state.is_open = self.is_open.get();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
