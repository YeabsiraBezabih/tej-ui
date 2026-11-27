use crate::id::NodeId;
use crate::runtime::ReactiveRuntime;
use crate::signal::{ReadSignal, Signal};
use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// A derived reactive value (Memo) that recomputes only when its tracked dependencies change.
pub struct Memo<T: 'static> {
    read_signal: ReadSignal<T>,
    node_id: NodeId,
}

impl<T: Clone + PartialEq + Send + Sync + Any + 'static> Memo<T> {
    /// Create a new memoized computation.
    pub fn new<F>(compute: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let compute = Arc::new(compute);
        let node_id = ReactiveRuntime::allocate_node_id();

        // Single initial evaluation under observer tracking to compute initial value and subscribe dependencies
        let initial_val = ReactiveRuntime::track_node(node_id, || compute());
        let signal = Signal::new(initial_val);
        let (read_signal, write_signal) = signal.split();

        ReactiveRuntime::register_node_output(node_id, signal.id());

        let compute_clone = Arc::clone(&compute);
        ReactiveRuntime::register_node_with_id(node_id, move || {
            let new_val = compute_clone();
            if let Ok(old_val) = ReactiveRuntime::untracked_read::<T>(write_signal.id()) {
                if new_val != old_val {
                    write_signal.set(new_val);
                }
            }
        });

        Self {
            read_signal,
            node_id,
        }
    }

    /// Read the memo's current value and track dependency.
    pub fn get(&self) -> T {
        self.read_signal.get()
    }

    /// Read without tracking.
    pub fn untracked_get(&self) -> T {
        self.read_signal.untracked_get()
    }

    /// Dispose of this memo, removing its computation node and underlying signal.
    pub fn dispose(self) {
        ReactiveRuntime::dispose_node(self.node_id);
        ReactiveRuntime::dispose_signal(self.read_signal.id());
    }

    /// Underlying read signal handle.
    #[must_use]
    pub const fn signal(&self) -> ReadSignal<T> {
        self.read_signal
    }

    /// Node ID in the computation graph.
    #[must_use]
    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }
}

impl<T: 'static> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for Memo<T> {}

impl<T: 'static> fmt::Debug for Memo<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memo")
            .field("signal", &self.read_signal)
            .field("node_id", &self.node_id)
            .finish()
    }
}
