use crate::view::View;
use tej_core::Signal;

/// Text content source: either a static string or a reactive `Signal<String>`.
#[derive(Debug, Clone)]
pub enum TextSource {
    Static(String),
    Reactive(Signal<String>),
}

/// Text widget displaying static or reactive string data.
#[derive(Debug, Clone)]
pub struct Text {
    source: TextSource,
}

/// State retained by a mounted `Text` widget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextState {
    pub current_text: String,
}

impl Text {
    /// Create a static text widget.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            source: TextSource::Static(text.into()),
        }
    }

    /// Create a reactive text widget bound to a signal.
    pub fn reactive(signal: Signal<String>) -> Self {
        Self {
            source: TextSource::Reactive(signal),
        }
    }
}

impl View for Text {
    type State = TextState;

    fn build(&self) -> Self::State {
        let text = match &self.source {
            TextSource::Static(s) => s.clone(),
            TextSource::Reactive(sig) => sig.get(),
        };
        TextState { current_text: text }
    }

    fn rebuild(&self, state: &mut Self::State) {
        match &self.source {
            TextSource::Static(s) => {
                if state.current_text != *s {
                    state.current_text.clone_from(s);
                }
            }
            TextSource::Reactive(sig) => {
                let new_val = sig.get();
                if state.current_text != new_val {
                    state.current_text = new_val;
                }
            }
        }
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        if let TextSource::Reactive(sig) = &self.source {
            let new_val = sig.get();
            if state.current_text != new_val {
                state.current_text = new_val;
            }
        }
    }
}
