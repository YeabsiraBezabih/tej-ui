use crate::view::View;
use std::fmt;
use std::sync::Arc;
use tej_ui_core::Signal;

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
    pub node: u32,
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
            node: 0,
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

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let style = tej_ui_layout::LayoutStyle {
            width: tej_ui_layout::Dimension::Points(240.0),
            height: tej_ui_layout::Dimension::Points(36.0),
            padding: tej_ui_layout::Insets::symmetric(6.0, 12.0),
            ..Default::default()
        };
        let node = arena.allocate_node(style);
        state.node = node;
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut crate::paint::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(tej_ui_layout::Rect::ZERO);
        let bg = ctx.theme.colors.input;
        let radius = ctx.theme.radii.sm;

        if ctx.is_focused(state.node) {
            let ring = tej_ui_layout::Rect::new(
                bounds.origin.x - 2.0,
                bounds.origin.y - 2.0,
                bounds.size.width + 4.0,
                bounds.size.height + 4.0,
            );
            ctx.fill_rounded_rect(ring, radius + 2.0, ctx.theme.colors.ring);
        }

        ctx.fill_rounded_rect(bounds, radius, bg);

        let font_size = ctx.theme.typography.body_small.font_size;
        let origin = tej_ui_layout::Point::new(bounds.origin.x + 8.0, bounds.origin.y + 8.0);
        if !state.current_text.is_empty() {
            ctx.draw_text(
                &state.current_text,
                origin,
                font_size,
                ctx.theme.colors.foreground,
            );
        } else if !state.placeholder.is_empty() {
            ctx.draw_text(
                &state.placeholder,
                origin,
                font_size,
                ctx.theme.colors.muted_foreground,
            );
        }
    }

    fn register_events(
        &self,
        state: &Self::State,
        dispatcher: &mut crate::events::EventDispatcher,
    ) {
        let sig = self.text;
        let on_change = self.on_change.clone();
        dispatcher.register_click(state.node, Arc::new(|| {}));
        dispatcher.register_text_input(
            state.node,
            Arc::new(move |text| {
                let mut current = sig.get();
                current.push_str(text);
                sig.set(current.clone());
                if let Some(h) = &on_change {
                    h(&current);
                }
            }),
        );
        let on_change_del = self.on_change.clone();
        dispatcher.register_key_down(
            state.node,
            Arc::new(move |code| {
                if code == 8 {
                    // Backspace
                    let mut current = sig.get();
                    if !current.is_empty() {
                        current.pop();
                        sig.set(current.clone());
                        if let Some(h) = &on_change_del {
                            h(&current);
                        }
                    }
                }
            }),
        );
    }
}

/// Specialized SearchBar alias.
pub type SearchBar = Input;
