use std::fmt::Debug;

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
}

/// Unit type `()` acts as an empty view.
impl View for () {
    type State = ();

    fn build(&self) -> Self::State {}
    fn rebuild(&self, _state: &mut Self::State) {}
    fn patch(&self, _state: &mut Self::State, _dirty_bits: u64) {}
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
}
