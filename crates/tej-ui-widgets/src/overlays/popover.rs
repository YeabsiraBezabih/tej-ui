use std::fmt::Debug;
use tej_ui_core::Signal;
use tej_ui_view::View;

/// Positioning alignment for floating popovers.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PopoverPlacement {
    #[default]
    Bottom,
    Top,
    Left,
    Right,
}

/// Floating popover overlay anchored to an interactive target element.
#[derive(Clone, Debug)]
pub struct Popover<Target: View, Content: View> {
    is_open: Signal<bool>,
    placement: PopoverPlacement,
    target: Target,
    content: Content,
}

impl<Target: View, Content: View> Popover<Target, Content> {
    /// Create a new popover anchored to `target`.
    pub fn new(is_open: Signal<bool>, target: Target, content: Content) -> Self {
        Self {
            is_open,
            placement: PopoverPlacement::Bottom,
            target,
            content,
        }
    }

    /// Set preferred popover placement relative to anchor.
    pub fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }
}

/// State of a mounted Popover.
#[derive(Debug)]
pub struct PopoverState<T: Debug, C: Debug> {
    pub is_open: bool,
    pub target_state: T,
    pub content_state: Option<C>,
    pub node: u32,
    pub target_node: u32,
    pub content_node: Option<u32>,
}

impl<Target: View, Content: View> View for Popover<Target, Content> {
    type State = PopoverState<Target::State, Content::State>;

    fn build(&self) -> Self::State {
        let open = self.is_open.get();
        PopoverState {
            is_open: open,
            target_state: self.target.build(),
            content_state: if open {
                Some(self.content.build())
            } else {
                None
            },
            node: 0,
            target_node: 0,
            content_node: None,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        self.target.rebuild(&mut state.target_state);
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
        self.target.patch(&mut state.target_state, dirty_bits);
        let open = self.is_open.get();
        if open != state.is_open {
            self.rebuild(state);
        } else if let Some(cs) = state.content_state.as_mut() {
            self.content.patch(cs, dirty_bits);
        }
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let direction = match self.placement {
            PopoverPlacement::Bottom | PopoverPlacement::Top => {
                tej_ui_layout::FlexDirection::Column
            }
            PopoverPlacement::Left | PopoverPlacement::Right => tej_ui_layout::FlexDirection::Row,
        };
        let root_style = tej_ui_layout::LayoutStyle {
            direction,
            ..Default::default()
        };
        let root = arena.allocate_node(root_style);
        state.node = root;

        let tn = self.target.layout(&mut state.target_state, arena);
        state.target_node = tn;

        if state.is_open {
            if let Some(cs) = state.content_state.as_mut() {
                let cn = self.content.layout(cs, arena);
                state.content_node = Some(cn);
                match self.placement {
                    PopoverPlacement::Top | PopoverPlacement::Left => {
                        arena.add_child(root, cn);
                        arena.add_child(root, tn);
                    }
                    PopoverPlacement::Bottom | PopoverPlacement::Right => {
                        arena.add_child(root, tn);
                        arena.add_child(root, cn);
                    }
                }
            } else {
                arena.add_child(root, tn);
            }
        } else {
            arena.add_child(root, tn);
            state.content_node = None;
        }

        root
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        self.target.paint(&state.target_state, ctx);
        if state.is_open {
            if let Some(cs) = state.content_state.as_ref() {
                self.content.paint(cs, ctx);
            }
        }
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut tej_ui_view::EventDispatcher) {
        self.target.register_events(&state.target_state, dispatcher);
        if state.is_open {
            if let Some(cs) = state.content_state.as_ref() {
                self.content.register_events(cs, dispatcher);
            }
        }
    }
}
