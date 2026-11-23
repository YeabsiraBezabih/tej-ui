//! Context provider view wrapper for ambient dependency injection.
//!
//! Scopes a typed context value over an entire view subtree during
//! mount (`build`), update (`rebuild`), and patch cycles.

use crate::view::View;
use std::any::Any;
use std::fmt::Debug;
use tej_ui_core::with_context;

/// View wrapper providing an ambient typed context value to its descendant subtree.
#[derive(Clone, Debug)]
pub struct ContextProvider<T: Any + Send + Sync + Clone, V: View> {
    value: T,
    child: V,
}

impl<T: Any + Send + Sync + Clone, V: View> ContextProvider<T, V> {
    /// Constructs a new context provider wrapping `child`.
    pub fn new(value: T, child: V) -> Self {
        Self { value, child }
    }
}

impl<T: Any + Send + Sync + Clone + Debug, V: View> View for ContextProvider<T, V> {
    type State = V::State;

    fn build(&self) -> Self::State {
        with_context(self.value.clone(), || self.child.build())
    }

    fn rebuild(&self, state: &mut Self::State) {
        with_context(self.value.clone(), || self.child.rebuild(state));
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        with_context(self.value.clone(), || {
            self.child.patch(state, dirty_bits);
        });
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        with_context(self.value.clone(), || self.child.layout(state, arena))
    }

    fn paint(&self, state: &Self::State, ctx: &mut crate::paint::PaintContext<'_>) {
        with_context(self.value.clone(), || self.child.paint(state, ctx));
    }

    fn register_events(
        &self,
        state: &Self::State,
        dispatcher: &mut crate::events::EventDispatcher,
    ) {
        with_context(self.value.clone(), || {
            self.child.register_events(state, dispatcher);
        });
    }
}
