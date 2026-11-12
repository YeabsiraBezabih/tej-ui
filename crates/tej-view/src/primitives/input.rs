use crate::view::View;
use std::fmt;
use std::sync::Arc;
use tej_core::Signal;

pub type TextChangeHandler = Arc<dyn Fn(&str) + Send + Sync>;

/// Single-line text input widget bound to a reactive `Signal<String>`.
#[derive(Clone)]
pub struct Input {
    text: Signal<String>,
    placeholder: String,
    on_change: Option<TextChangeHandler>,
}

impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Input")
            .field("text", &self.text)
            .field("placeholder", &self.placeholder)
            .finish()
    }
}

/// State retained by a mounted Input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputState {
    pub current_text: String,
    pub placeholder: String,
}

impl Input {
    /// Create an input bound to `text` signal.
    pub fn new(text: Signal<String>) -> Self {
        Self {
            text,
            placeholder: String::new(),
            on_change: None,
        }
    }

    /// Set placeholder text.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Register text change listener.
    pub fn on_change<F: Fn(&str) + Send + Sync + 'static>(mut self, handler: F) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }
}

impl View for Input {
    type State = InputState;

    fn build(&self) -> Self::State {
        InputState {
            current_text: self.text.get(),
            placeholder: self.placeholder.clone(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let current = self.text.get();
        if state.current_text != current {
            state.current_text = current;
        }
        state.placeholder.clone_from(&self.placeholder);
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        let current = self.text.get();
        if state.current_text != current {
            state.current_text = current;
        }
    }
}

/// Specialized SearchBar alias.
pub type SearchBar = Input;
