use crate::arena::SignalArena;
use crate::error::Result;
use crate::graph::DependencyGraph;
use crate::id::{NodeId, SignalId};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

pub type NodeCallback = Arc<dyn Fn() + Send + Sync>;

#[derive(Default, Debug)]
struct BatchState {
    depth: usize,
    pending_signals: Vec<SignalId>,
}

thread_local! {
    static ARENA: RefCell<SignalArena> = const { RefCell::new(SignalArena::new()) };
    static GRAPH: RefCell<DependencyGraph> = RefCell::new(DependencyGraph::new());
    static OBSERVER_STACK: RefCell<Vec<NodeId>> = const { RefCell::new(Vec::new()) };
    static CALLBACKS: RefCell<HashMap<NodeId, NodeCallback>> = RefCell::new(HashMap::new());
    static BATCH: RefCell<BatchState> = RefCell::new(BatchState::default());
    static NEXT_NODE_INDEX: RefCell<u32> = const { RefCell::new(1) };
}

/// Central reactive engine coordinating signal arenas, dependency graphs, and observer contexts.
#[derive(Debug, Default)]
pub struct ReactiveRuntime;

impl ReactiveRuntime {
    /// Allocate a new signal in the thread-local arena.
    pub fn create_signal<T: Any + Send + Sync + 'static>(value: T) -> SignalId {
        ARENA.with(|arena| arena.borrow_mut().insert(value))
    }

    /// Read signal value while recording dependency if an observer is active.
    pub fn read_signal<T: Clone + 'static>(id: SignalId) -> Result<T> {
        let active_observer = OBSERVER_STACK.with(|stack| stack.borrow().last().copied());
        if let Some(obs) = active_observer {
            GRAPH.with(|graph| graph.borrow_mut().add_dependency(obs, id));
        }
        ARENA.with(|arena| arena.borrow().get::<T>(id).cloned())
    }

    /// Read signal without tracking dependency.
    pub fn untracked_read<T: Clone + 'static>(id: SignalId) -> Result<T> {
        ARENA.with(|arena| arena.borrow().get::<T>(id).cloned())
    }

    /// Update signal value and propagate changes to downstream observers.
    pub fn write_signal<T: Any + Send + Sync + 'static>(id: SignalId, value: T) -> Result<()> {
        ARENA.with(|arena| arena.borrow_mut().set(id, value))?;
        GRAPH.with(|graph| graph.borrow_mut().advance_epoch());

        let in_batch = BATCH.with(|b| {
            let mut state = b.borrow_mut();
            if state.depth > 0 {
                if !state.pending_signals.contains(&id) {
                    state.pending_signals.push(id);
                }
                true
            } else {
                false
            }
        });

        if !in_batch {
            Self::propagate_signal_change(id);
        }
        Ok(())
    }

    /// Register a new computation node.
    pub fn register_node<F>(callback: F) -> NodeId
    where
        F: Fn() + Send + Sync + 'static,
    {
        let id = NEXT_NODE_INDEX.with(|next| {
            let mut val = next.borrow_mut();
            let id = NodeId::new(*val, 1);
            *val += 1;
            id
        });

        CALLBACKS.with(|cbs| {
            cbs.borrow_mut().insert(id, Arc::new(callback));
        });
        id
    }

    /// Remove a reactive node and clean up its dependencies.
    pub fn dispose_node(node: NodeId) {
        CALLBACKS.with(|cbs| {
            cbs.borrow_mut().remove(&node);
        });
        GRAPH.with(|graph| {
            graph.borrow_mut().remove_node(node);
        });
    }

    /// Run a closure inside a batched transaction.
    pub fn batch<R, F: FnOnce() -> R>(f: F) -> R {
        BATCH.with(|b| b.borrow_mut().depth += 1);
        let result = f();

        let pending = BATCH.with(|b| {
            let mut state = b.borrow_mut();
            state.depth = state.depth.saturating_sub(1);
            if state.depth == 0 {
                std::mem::take(&mut state.pending_signals)
            } else {
                Vec::new()
            }
        });

        if !pending.is_empty() {
            let affected =
                GRAPH.with(|g| g.borrow().collect_batch_subscribers_topological(&pending));
            for node in affected {
                Self::run_node_callback(node);
            }
        }
        result
    }

    /// Propagate changes for a single signal to its dependent nodes in topological order.
    fn propagate_signal_change(signal: SignalId) {
        let subscribers = GRAPH.with(|g| g.borrow().collect_subscribers_topological(signal));
        for node in subscribers {
            Self::run_node_callback(node);
        }
    }

    /// Execute the registered callback for a node with dependency tracking.
    pub fn run_node_callback(node: NodeId) {
        let callback = CALLBACKS.with(|cbs| cbs.borrow().get(&node).cloned());
        if let Some(cb) = callback {
            GRAPH.with(|g| g.borrow_mut().clear_dependencies_for_node(node));
            OBSERVER_STACK.with(|s| s.borrow_mut().push(node));

            cb();

            OBSERVER_STACK.with(|s| s.borrow_mut().pop());
        }
    }
}
