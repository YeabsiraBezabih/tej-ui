use crate::paint::PaintContext;
use std::fmt::Debug;
use tej_ui_layout::{LayoutArena, LayoutStyle};

/// Statically typed, zero-vtable view tree trait.
///
/// Unlike Virtual DOM frameworks (which allocate heap trees every frame) or immediate-mode
/// frameworks (which re-execute layout every frame), `tej-view` builds an initial state token
/// and mutates leaf widgets in place ($O(1)$) when fine-grained signal bitsets change.
pub trait View: Debug {
    /// Associated persistent state retained across render epochs.
    type State: Debug;

    /// Mount the view, creating its initial widget state.
    fn build(&self) -> Self::State;

    /// Full rebuild/update when structural properties change.
    fn rebuild(&self, state: &mut Self::State);

    /// Fine-grained $O(1)$ in-place patch based on dirty signal bits.
    fn patch(&self, state: &mut Self::State, dirty_bits: u64);

    /// Allocate or update layout nodes in the layout arena.
    fn layout(&self, _state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        arena.allocate_node(LayoutStyle::default())
    }

    /// Layout this view's node(s) and attach them directly as children of `parent` in `arena`.
    ///
    /// Container views (`VStack`, `HStack`, `Grid`) allocate their own dedicated container node
    /// and call `layout_children` so tuple views attach each item directly without overwriting
    /// child styles in place.
    fn layout_children(&self, state: &mut Self::State, arena: &mut LayoutArena, parent: u32) {
        let child_node = self.layout(state, arena);
        arena.add_child(parent, child_node);
    }

    /// Render visual elements onto the platform compositor surface.
    fn paint(&self, _state: &Self::State, _ctx: &mut PaintContext<'_>) {}

    /// Register interactive event handlers with the event dispatcher.
    fn register_events(
        &self,
        _state: &Self::State,
        _dispatcher: &mut crate::events::EventDispatcher,
    ) {
    }

    /// Emit semantic accessibility nodes for this view subtree into `tree`.
    fn accessibility(
        &self,
        _state: &Self::State,
        _arena: &LayoutArena,
        _tree: &mut tej_ui_a11y::A11yTree,
        _parent: Option<tej_ui_a11y::A11yId>,
    ) {
    }
}

/// Unit type `()` acts as an empty view.
impl View for () {
    type State = ();

    fn build(&self) -> Self::State {}
    fn rebuild(&self, _state: &mut Self::State) {}
    fn patch(&self, _state: &mut Self::State, _dirty_bits: u64) {}
    fn layout(&self, _state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        arena.allocate_node(LayoutStyle::default())
    }
    fn paint(&self, _state: &Self::State, _ctx: &mut PaintContext<'_>) {}
    fn register_events(
        &self,
        _state: &Self::State,
        _dispatcher: &mut crate::events::EventDispatcher,
    ) {
    }
}

/// `Option<V>` allows optional view mounting.
impl<V: View> View for Option<V> {
    type State = Option<V::State>;

    fn build(&self) -> Self::State {
        self.as_ref().map(View::build)
    }

    fn rebuild(&self, state: &mut Self::State) {
        match (self, state.as_mut()) {
            (Some(view), Some(s)) => view.rebuild(s),
            (Some(view), None) => *state = Some(view.build()),
            (None, Some(_)) => *state = None,
            (None, None) => {}
        }
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        if let (Some(view), Some(s)) = (self, state.as_mut()) {
            view.patch(s, dirty_bits);
        }
    }

    fn layout(&self, state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        if let (Some(view), Some(s)) = (self, state.as_mut()) {
            view.layout(s, arena)
        } else {
            arena.allocate_node(LayoutStyle::default())
        }
    }

    fn paint(&self, state: &Self::State, ctx: &mut PaintContext<'_>) {
        if let (Some(view), Some(s)) = (self, state.as_ref()) {
            view.paint(s, ctx);
        }
    }

    fn register_events(
        &self,
        state: &Self::State,
        dispatcher: &mut crate::events::EventDispatcher,
    ) {
        if let (Some(view), Some(s)) = (self, state.as_ref()) {
            view.register_events(s, dispatcher);
        }
    }

    fn accessibility(
        &self,
        state: &Self::State,
        arena: &LayoutArena,
        tree: &mut tej_ui_a11y::A11yTree,
        parent: Option<tej_ui_a11y::A11yId>,
    ) {
        if let (Some(view), Some(s)) = (self, state.as_ref()) {
            view.accessibility(s, arena, tree, parent);
        }
    }
}
