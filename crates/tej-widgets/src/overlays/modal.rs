use std::fmt::Debug;
use tej_core::Signal;
use tej_view::View;

/// Accessible modal dialog container with focus trapping and backdrop dimming.
#[derive(Clone, Debug)]
pub struct Modal<V: View> {
    is_open: Signal<bool>,
    content: V,
}

impl<V: View> Modal<V> {
    /// Create a modal dialog controlled by `is_open` signal.
    pub fn new(is_open: Signal<bool>, content: V) -> Self {
        Self { is_open, content }
    }
}

/// State of a mounted Modal dialog.
#[derive(Debug)]
pub struct ModalState<S: Debug> {
    pub is_open: bool,
    pub content_state: Option<S>,
}

impl<V: View> View for Modal<V> {
    type State = ModalState<V::State>;

    fn build(&self) -> Self::State {
        let open = self.is_open.get();
        let content_state = if open {
            Some(self.content.build())
        } else {
            None
        };
        ModalState {
            is_open: open,
            content_state,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let open = self.is_open.get();
        if open != state.is_open {
            state.is_open = open;
            state.content_state = if open {
                Some(self.content.build())
            } else {
                None
            };
        } else if let Some(cs) = state.content_state.as_mut() {
            self.content.rebuild(cs);
        }
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        let open = self.is_open.get();
        if open != state.is_open {
            self.rebuild(state);
        } else if let Some(cs) = state.content_state.as_mut() {
            self.content.patch(cs, dirty_bits);
        }
    }
}
