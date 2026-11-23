use crate::id::NodeId;
use crate::runtime::ReactiveRuntime;
use std::fmt;
use std::sync::Arc;

/// Handle to a registered reactive side-effect.
pub struct Effect {
    node_id: NodeId,
}

impl Effect {
    /// Create and run a new reactive effect.
    ///
    /// The closure is executed immediately to capture dependencies. Whenever any
    /// captured signal changes, the effect re-runs automatically.
    pub fn new<F>(effect_fn: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        let effect_fn = Arc::new(effect_fn);
        let fn_clone = Arc::clone(&effect_fn);

        let node_id = ReactiveRuntime::register_node(move || {
            fn_clone();
        });

        // Run immediately to record initial dependencies
        ReactiveRuntime::run_node_callback(node_id);

        Self { node_id }
    }

    /// Explicitly dispose of this effect, removing it from all dependency lists.
    pub fn dispose(self) {
        ReactiveRuntime::dispose_node(self.node_id);
    }

    /// Get the node identifier of this effect.
    #[must_use]
    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }
}

impl fmt::Debug for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Effect")
            .field("node_id", &self.node_id)
            .finish()
    }
}

/// Helper function to create a reactive effect.
pub fn create_effect<F>(f: F) -> Effect
where
    F: Fn() + Send + Sync + 'static,
{
    Effect::new(f)
}
