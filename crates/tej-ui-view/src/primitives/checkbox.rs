use crate::view::View;
use std::fmt;
use std::sync::Arc;
use tej_ui_core::Signal;

pub type ToggleHandler = Arc<dyn Fn(bool) + Send + Sync>;

/// Two-state checkbox widget bound to a reactive `Signal<bool>`.
#[derive(Clone)]
pub struct Checkbox {
    checked: Signal<bool>,
    on_toggle: Option<ToggleHandler>,
}

impl fmt::Debug for Checkbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Checkbox")
            .field("checked", &self.checked)
            .field("has_toggle_handler", &self.on_toggle.is_some())
            .finish()
    }
}

/// State retained by a mounted Checkbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckboxState {
    pub is_checked: bool,
    pub node: u32,
}

impl Checkbox {
    /// Create a checkbox bound to `checked` signal.
    pub fn new(checked: Signal<bool>) -> Self {
        Self {
            checked,
            on_toggle: None,
        }
    }

    /// Register a toggle event callback.
    pub fn on_toggle<F: Fn(bool) + Send + Sync + 'static>(mut self, handler: F) -> Self {
        self.on_toggle = Some(Arc::new(handler));
        self
    }
}

impl View for Checkbox {
    type State = CheckboxState;

    fn build(&self) -> Self::State {
        CheckboxState {
            is_checked: self.checked.get(),
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let current = self.checked.get();
        if state.is_checked != current {
            state.is_checked = current;
        }
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        let current = self.checked.get();
        if state.is_checked != current {
            state.is_checked = current;
        }
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let style = tej_ui_layout::LayoutStyle {
            width: tej_ui_layout::Dimension::Points(18.0),
            height: tej_ui_layout::Dimension::Points(18.0),
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
        let bg = if state.is_checked {
            ctx.theme.colors.primary
        } else {
            ctx.theme.colors.input
        };
        let radius = ctx.theme.radii.sm;
        ctx.fill_rounded_rect(bounds, radius, bg);
        if state.is_checked {
            let inner = tej_ui_layout::Rect::new(
                bounds.origin.x + 4.0,
                bounds.origin.y + 4.0,
                (bounds.size.width - 8.0).max(1.0),
                (bounds.size.height - 8.0).max(1.0),
            );
            ctx.fill_rounded_rect(inner, 2.0, ctx.theme.colors.primary_foreground);
        }
    }

    fn register_events(
        &self,
        state: &Self::State,
        dispatcher: &mut crate::events::EventDispatcher,
    ) {
        let sig = self.checked;
        let on_toggle = self.on_toggle.clone();
        dispatcher.register_click(
            state.node,
            Arc::new(move || {
                let next = !sig.get();
                sig.set(next);
                if let Some(h) = &on_toggle {
                    h(next);
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
            tej_ui_a11y::A11yNode::new(id, tej_ui_a11y::A11yRole::Checkbox).with_bounds(bounds);
        node.is_checked = Some(state.is_checked);
        tree.insert_node(node);
        if let Some(parent_id) = parent {
            if let Some(p) = tree.get_node_mut(parent_id) {
                p.children.push(id);
            }
            tree.mark_dirty(parent_id);
        }
    }
}
