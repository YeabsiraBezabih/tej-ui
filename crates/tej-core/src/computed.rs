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
        // Initial computation to seed signal
        let initial_val = compute();
        let signal = Signal::new(initial_val);
        let write_signal = signal.split().1;

        let compute_clone = Arc::clone(&compute);
        let node_id = ReactiveRuntime::register_node(move || {
            let new_val = compute_clone();
            write_signal.set(new_val);
        });

        // Run once with observer to establish initial dependency subscriptions
        ReactiveRuntime::run_node_callback(node_id);

        Self {
            read_signal: signal.split().0,
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
