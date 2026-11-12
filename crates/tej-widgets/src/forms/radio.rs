//! Radio group selector (shadcn `RadioGroup`).

use std::sync::Arc;
use tej_core::Signal;
use tej_layout::{Point, Rect, Size};
use tej_view::View;

/// An option item within a radio group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioItem<T: Clone + PartialEq> {
    pub value: T,
    pub label: String,
    pub disabled: bool,
}

impl<T: Clone + PartialEq> RadioItem<T> {
    #[must_use]
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            disabled: false,
        }
    }
}

/// Retained state for a mounted RadioGroup.
#[derive(Debug, Default)]
pub struct RadioGroupState {
    pub bounds: Rect,
    pub selected_index: Option<usize>,
}

/// Callback invoked when selected radio option changes.
pub type RadioCallback<T> = Option<Arc<dyn Fn(&T) + Send + Sync>>;

/// Radio group single-selection container.
#[derive(Clone)]
pub struct RadioGroup<T: Clone + PartialEq + Send + Sync + 'static> {
    selected: Signal<T>,
    items: Vec<RadioItem<T>>,
    on_change: RadioCallback<T>,
}

impl<T: Clone + PartialEq + Send + Sync + std::fmt::Debug + 'static> std::fmt::Debug
    for RadioGroup<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RadioGroup")
            .field("selected", &self.selected.get())
            .field("items_count", &self.items.len())
            .finish()
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> RadioGroup<T> {
    /// Creates a new radio group bound to the selected signal.
    #[must_use]
    pub fn new(selected: Signal<T>, items: Vec<RadioItem<T>>) -> Self {
        Self {
            selected,
            items,
            on_change: None,
        }
    }

    /// Sets selection by index.
    pub fn select_index(&self, index: usize) {
        if let Some(item) = self.items.get(index) {
            if !item.disabled {
                self.selected.set(item.value.clone());
                if let Some(cb) = &self.on_change {
                    cb(&item.value);
                }
            }
        }
    }
}

impl<T: Clone + PartialEq + Send + Sync + std::fmt::Debug + 'static> View for RadioGroup<T> {
    type State = RadioGroupState;

    fn build(&self) -> Self::State {
        let current = self.selected.get();
        let selected_index = self.items.iter().position(|it| it.value == current);
        RadioGroupState {
            bounds: Rect::from_origin_size(
                Point::ZERO,
                Size::new(200.0, (self.items.len() as f32) * 28.0),
            ),
            selected_index,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let current = self.selected.get();
        state.selected_index = self.items.iter().position(|it| it.value == current);
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
