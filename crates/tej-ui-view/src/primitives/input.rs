use crate::view::View;
use std::fmt;
use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_layout::Dimension;

pub type TextChangeHandler = Arc<dyn Fn(&str) + Send + Sync>;

/// Single-line text input widget bound to a reactive `Signal<String>`.
#[derive(Clone)]
pub struct Input {
    text: Signal<String>,
    placeholder: String,
    on_change: Option<TextChangeHandler>,
    width: Dimension,
    flex_grow: f32,
}

impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Input")
            .field("text", &self.text)
            .field("placeholder", &self.placeholder)
            .field("width", &self.width)
            .field("flex_grow", &self.flex_grow)
            .finish_non_exhaustive()
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
            width: Dimension::Points(240.0),
            flex_grow: 0.0,
        }
    }

    /// Set placeholder text.
    #[must_use]
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set explicit width constraint.
    #[must_use]
    pub const fn width(mut self, width: Dimension) -> Self {
        self.width = width;
        self
    }

    /// Set flex grow factor (e.g. 1.0 to expand and fill remaining space).
    #[must_use]
    pub const fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.flex_grow = flex_grow;
        self
    }

    /// Register text change listener.
    #[must_use]
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
            width: self.width,
            height: Dimension::Points(36.0),
            padding: tej_ui_layout::Insets::symmetric(6.0, 12.0),
            flex_grow: self.flex_grow,
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
        let radius = ctx.theme.radii.sm;
        let is_focused = ctx.is_focused(state.node);

        let border_color = if is_focused {
            ctx.theme.colors.ring
        } else {
            ctx.theme.colors.input_border
        };

        // 1px border rect + inset input_bg fill
        ctx.fill_rounded_rect(bounds, radius, border_color);
        let inner = tej_ui_layout::Rect::new(
            bounds.origin.x + 1.0,
            bounds.origin.y + 1.0,
            (bounds.size.width - 2.0).max(0.0),
            (bounds.size.height - 2.0).max(0.0),
        );
        ctx.fill_rounded_rect(inner, (radius - 1.0).max(0.0), ctx.theme.colors.input_bg);

        let pad_x = 12.0;
        let clip_rect = tej_ui_layout::Rect::new(
            bounds.origin.x + pad_x,
            bounds.origin.y + 1.0,
            (bounds.size.width - pad_x * 2.0).max(0.0),
            (bounds.size.height - 2.0).max(0.0),
        );
        ctx.push_clip(clip_rect, [0.0; 4]);

        let font_size = ctx.theme.typography.body_small.font_size;
        let text_y = bounds.origin.y + (bounds.size.height - font_size) * 0.5;
        let origin = tej_ui_layout::Point::new(bounds.origin.x + pad_x, text_y);
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

        if is_focused {
            let text_w = if state.current_text.is_empty() {
                0.0
            } else {
                tej_ui_text::FontRegistry::global()
                    .measure_text(
                        &state.current_text,
                        font_size,
                        tej_ui_text::FontFamilyKind::SansSerif,
                        400,
                    )
                    .width
            };
            let caret_h = (font_size * 1.25).min((bounds.size.height - 8.0).max(1.0));
            let caret_y = bounds.origin.y + (bounds.size.height - caret_h) * 0.5;
            let caret = tej_ui_layout::Rect::new(
                bounds.origin.x + pad_x + text_w + 1.0,
                caret_y,
                1.5,
                caret_h,
            );
            ctx.fill_rect(caret, ctx.theme.colors.foreground);
        }

        ctx.pop_clip();
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
                    // Backspace: delete a full UAX #29 extended grapheme cluster
                    let mut current = sig.get();
                    if tej_ui_text::pop_grapheme(&mut current).is_some() {
                        sig.set(current.clone());
                        if let Some(h) = &on_change_del {
                            h(&current);
                        }
                    }
                }
            }),
        );
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
        let mut node =
            tej_ui_a11y::A11yNode::new(id, tej_ui_a11y::A11yRole::TextInput).with_bounds(bounds);
        if !state.placeholder.is_empty() {
            node.label = Some(state.placeholder.clone());
        }
        node.value = Some(state.current_text.clone());
        tree.insert_node(node);
        if let Some(parent_id) = parent {
            if let Some(p) = tree.get_node_mut(parent_id) {
                p.children.push(id);
            }
            tree.mark_dirty(parent_id);
        }
    }
}

/// Specialized SearchBar alias.
pub type SearchBar = Input;
