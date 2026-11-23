use crate::view::View;
use tej_ui_core::Signal;
use tej_ui_hal::Color;

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
    color: Option<Color>,
}

/// State retained by a mounted `Text` widget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextState {
    pub current_text: String,
    pub node: u32,
}

impl Text {
    /// Create a static text widget.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            source: TextSource::Static(text.into()),
            color: None,
        }
    }

    /// Set foreground text color.
    #[must_use]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Create a reactive text widget bound to a signal.
    pub fn reactive(signal: Signal<String>) -> Self {
        Self {
            source: TextSource::Reactive(signal),
            color: None,
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
        TextState {
            current_text: text,
            node: 0,
        }
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

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let approx_w = (state.current_text.len() as f32) * 8.5;
        let style = tej_ui_layout::LayoutStyle {
            width: tej_ui_layout::Dimension::Points(approx_w),
            height: tej_ui_layout::Dimension::Points(20.0),
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
        let color = self.color.unwrap_or(ctx.theme.colors.foreground);
        let font_size = ctx.theme.typography.body.font_size;
        ctx.draw_text(&state.current_text, bounds.origin, font_size, color);
    }
}
