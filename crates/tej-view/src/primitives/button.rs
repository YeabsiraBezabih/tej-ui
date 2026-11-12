use crate::view::View;
use std::fmt::{self, Debug};
use std::sync::Arc;

/// Click callback type.
pub type ClickHandler = Arc<dyn Fn() + Send + Sync>;

/// Interactive push button widget.
#[derive(Clone)]
pub struct Button<V: View> {
    child: V,
    on_click: Option<ClickHandler>,
}

impl<V: View> fmt::Debug for Button<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Button")
            .field("child", &self.child)
            .field("has_click_handler", &self.on_click.is_some())
            .finish()
    }
}

/// State retained by a mounted Button.
pub struct ButtonState<S: Debug> {
    pub child_state: S,
    pub on_click: Option<ClickHandler>,
}

impl<S: Debug> fmt::Debug for ButtonState<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ButtonState")
            .field("child_state", &self.child_state)
            .field("has_click_handler", &self.on_click.is_some())
            .finish()
    }
}

impl<V: View> Button<V> {
    /// Create a button wrapping child view `child`.
    pub fn new(child: V) -> Self {
        Self {
            child,
            on_click: None,
        }
    }

    /// Register a click event callback.
    pub fn on_click<F: Fn() + Send + Sync + 'static>(mut self, handler: F) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }
}

impl<V: View> View for Button<V> {
    type State = ButtonState<V::State>;

    fn build(&self) -> Self::State {
        ButtonState {
            child_state: self.child.build(),
            on_click: self.on_click.clone(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        self.child.rebuild(&mut state.child_state);
        state.on_click = self.on_click.clone();
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        self.child.patch(&mut state.child_state, dirty_bits);
    }
}
