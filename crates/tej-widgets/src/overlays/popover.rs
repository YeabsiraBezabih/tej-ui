use std::fmt::Debug;
use tej_core::Signal;
use tej_view::View;

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
}
