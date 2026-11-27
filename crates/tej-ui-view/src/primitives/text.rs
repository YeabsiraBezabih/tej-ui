use crate::view::View;
use std::sync::OnceLock;
use tej_ui_core::Signal;
use tej_ui_hal::Color;
use tej_ui_text::{SystemFont, TextMeasurer, wrap_text};

static TEXT_MEASURER: OnceLock<TextMeasurer> = OnceLock::new();

fn shared_measurer() -> &'static TextMeasurer {
    TEXT_MEASURER.get_or_init(TextMeasurer::new)
}

/// Text content source: either a static string or a reactive `Signal<String>`.
#[derive(Debug, Clone)]
pub enum TextSource {
    Static(String),
    Reactive(Signal<String>),
}

/// Text widget displaying static or reactive string data with real text measurement
/// and optional multi-line word wrapping.
#[derive(Debug, Clone)]
pub struct Text {
    source: TextSource,
    color: Option<Color>,
    max_width: Option<f32>,
    font_size: f32,
}

/// State retained by a mounted `Text` widget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextState {
    pub current_text: String,
    pub lines: Vec<String>,
    pub node: u32,
}

impl Text {
    /// Default body font size in logical pixels used for measurement when not overridden.
    pub const DEFAULT_FONT_SIZE: f32 = 14.0;

    /// Create a static text widget.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            source: TextSource::Static(text.into()),
            color: None,
            max_width: None,
            font_size: Self::DEFAULT_FONT_SIZE,
        }
    }

    /// Set foreground text color.
    #[must_use]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set maximum width constraint for multi-line word wrapping.
    #[must_use]
    pub const fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    /// Set explicit font size in logical pixels.
    #[must_use]
    pub const fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// Create a reactive text widget bound to a signal.
    pub fn reactive(signal: Signal<String>) -> Self {
        Self {
            source: TextSource::Reactive(signal),
            color: None,
            max_width: None,
            font_size: Self::DEFAULT_FONT_SIZE,
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
            lines: vec![text.clone()],
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
        let measurer = shared_measurer();
        let (width, height, lines) = if let Some(max_w) = self.max_width {
            let wrapped = wrap_text(&state.current_text, max_w, self.font_size, measurer);
            (wrapped.width, wrapped.height, wrapped.lines)
        } else if state.current_text.contains('\n') {
            let wrapped = wrap_text(&state.current_text, f32::INFINITY, self.font_size, measurer);
            (wrapped.width, wrapped.height, wrapped.lines)
        } else {
            let size =
                measurer.measure_slice(&state.current_text, SystemFont::Body, self.font_size);
            (size.width, size.height, vec![state.current_text.clone()])
        };

        state.lines = lines;
        let style = tej_ui_layout::LayoutStyle {
            width: tej_ui_layout::Dimension::Points(width),
            height: tej_ui_layout::Dimension::Points(height),
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
        let font_size = if (self.font_size - Self::DEFAULT_FONT_SIZE).abs() > f32::EPSILON {
            self.font_size
        } else {
            ctx.theme.typography.body.font_size
        };
        let line_height = font_size * 1.35;

        if state.lines.len() > 1 {
            for (idx, line) in state.lines.iter().enumerate() {
                let origin = tej_ui_layout::Point::new(
                    bounds.origin.x,
                    bounds.origin.y + (idx as f32) * line_height,
                );
                ctx.draw_text(line, origin, font_size, color);
            }
        } else {
            let line = state
                .lines
                .first()
                .map_or(state.current_text.as_str(), String::as_str);
            ctx.draw_text(line, bounds.origin, font_size, color);
        }
    }

    fn accessibility(
        &self,
        state: &Self::State,
        arena: &tej_ui_layout::LayoutArena,
        tree: &mut tej_ui_a11y::A11yTree,
        parent: Option<tej_ui_a11y::A11yId>,
    ) {
        let id = tree.allocate_id();
        let bounds = arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(tej_ui_layout::Rect::ZERO);
        let node = tej_ui_a11y::A11yNode::new(id, tej_ui_a11y::A11yRole::Label)
            .with_label(state.current_text.clone())
            .with_bounds(bounds);
        tree.insert_node(node);
        if let Some(parent_id) = parent {
            if let Some(p) = tree.get_node_mut(parent_id) {
                p.children.push(id);
            }
            tree.mark_dirty(parent_id);
        }
    }
}
