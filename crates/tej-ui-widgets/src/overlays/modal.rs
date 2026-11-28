use std::fmt::Debug;
use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_view::View;

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
    pub node: u32,
    pub content_node: Option<u32>,
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
            node: 0,
            content_node: None,
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

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        if state.is_open {
            let overlay_style = tej_ui_layout::LayoutStyle {
                width: tej_ui_layout::Dimension::Percent(1.0),
                height: tej_ui_layout::Dimension::Percent(1.0),
                align_items: tej_ui_layout::AlignItems::Center,
                justify_content: tej_ui_layout::JustifyContent::Center,
                ..Default::default()
            };
            let root = arena.allocate_node(overlay_style);
            state.node = root;

            if let Some(cs) = state.content_state.as_mut() {
                let cn = self.content.layout(cs, arena);
                state.content_node = Some(cn);
                arena.add_child(root, cn);
            }
            root
        } else {
            let empty_style = tej_ui_layout::LayoutStyle {
                width: tej_ui_layout::Dimension::Points(0.0),
                height: tej_ui_layout::Dimension::Points(0.0),
                ..Default::default()
            };
            let root = arena.allocate_node(empty_style);
            state.node = root;
            state.content_node = None;
            root
        }
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        if state.is_open {
            if let Some(bounds) = ctx.arena.computed_bounds.get(state.node as usize).copied() {
                // Dimmed backdrop
                ctx.fill_rect(bounds, ctx.theme.colors.scrim);
            }
            if let Some(cs) = state.content_state.as_ref() {
                self.content.paint(cs, ctx);
            }
        }
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut tej_ui_view::EventDispatcher) {
        if state.is_open {
            let is_open = self.is_open;
            dispatcher.register_key_down(
                state.node,
                Arc::new(move |key_code| {
                    if key_code == 27 {
                        is_open.set(false);
                    }
                }),
            );
            if let Some(cs) = state.content_state.as_ref() {
                self.content.register_events(cs, dispatcher);
            }
        }
    }
}
