use crate::arena::SignalArena;
use crate::error::Result;
use crate::graph::DependencyGraph;
use crate::id::{Epoch, NodeId, SignalId};
use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub type NodeCallback = Arc<dyn Fn() + Send + Sync>;

#[derive(Default, Debug)]
struct BatchState {
    depth: usize,
    pending_signals: Vec<SignalId>,
}

#[derive(Default, Debug)]
struct PropagationState {
    active: bool,
    changed_signals: HashSet<SignalId>,
    extra_signals: Vec<SignalId>,
}

thread_local! {
    static ARENA: RefCell<SignalArena> = const { RefCell::new(SignalArena::new()) };
    static GRAPH: RefCell<DependencyGraph> = RefCell::new(DependencyGraph::new());
    static OBSERVER_STACK: RefCell<Vec<NodeId>> = const { RefCell::new(Vec::new()) };
    static CALLBACKS: RefCell<HashMap<NodeId, NodeCallback>> = RefCell::new(HashMap::new());
    static BATCH: RefCell<BatchState> = RefCell::new(BatchState::default());
    static PROPAGATION: RefCell<PropagationState> = RefCell::new(PropagationState::default());
    static NEXT_NODE_INDEX: RefCell<u32> = const { RefCell::new(1) };
}

/// RAII guard ensuring `OBSERVER_STACK` is always popped even on panic unwind.
struct ObserverGuard;

impl ObserverGuard {
    fn push(node: NodeId) -> Self {
        OBSERVER_STACK.with(|s| s.borrow_mut().push(node));
        Self
    }
}

impl Drop for ObserverGuard {
    fn drop(&mut self) {
        OBSERVER_STACK.with(|s| {
            s.borrow_mut().pop();
        });
    }
}

/// RAII guard ensuring `BATCH.depth` is always decremented even on panic unwind.
struct BatchGuard {
    active: bool,
}

impl BatchGuard {
    fn enter() -> Self {
        BATCH.with(|b| b.borrow_mut().depth += 1);
        Self { active: true }
    }

    fn disarm(&mut self) {
        self.active = false;
    }
}

impl Drop for BatchGuard {
    fn drop(&mut self) {
        if self.active {
            BATCH.with(|b| {
                let mut state = b.borrow_mut();
                state.depth = state.depth.saturating_sub(1);
                if state.depth == 0 {
                    state.pending_signals.clear();
                }
            });
        }
    }
}

/// RAII guard ensuring `PROPAGATION` state is always reset even on panic unwind.
struct PropagationGuard;

impl Drop for PropagationGuard {
    fn drop(&mut self) {
        PROPAGATION.with(|p| {
            let mut state = p.borrow_mut();
            state.active = false;
            state.changed_signals.clear();
            state.extra_signals.clear();
        });
    }
}

/// Central reactive engine coordinating signal arenas, dependency graphs, and observer contexts.
#[derive(Debug, Default)]
pub struct ReactiveRuntime;

impl ReactiveRuntime {
    /// Get the current global reactive epoch (monotonically incremented on every signal write).
    #[must_use]
    pub fn epoch() -> Epoch {
        GRAPH.with(|graph| graph.borrow().epoch())
    }

    /// Allocate a new signal in the thread-local arena.
    pub fn create_signal<T: Any + Send + Sync + 'static>(value: T) -> SignalId {
        ARENA.with(|arena| arena.borrow_mut().insert(value))
    }

    /// Dispose of a signal from the thread-local arena and remove its edges from the dependency graph.
    pub fn dispose_signal(id: SignalId) {
        ARENA.with(|arena| {
            arena.borrow_mut().remove_untyped(id);
        });
        GRAPH.with(|graph| {
            graph.borrow_mut().remove_signal(id);
        });
    }

    /// Check whether a signal handle is still valid in the thread-local arena.
    #[must_use]
    pub fn is_signal_valid(id: SignalId) -> bool {
        ARENA.with(|arena| arena.borrow().is_valid(id))
    }

    /// Pre-allocate capacity in the thread-local signal arena.
    pub fn reserve_signals(additional: usize) {
        ARENA.with(|arena| arena.borrow_mut().reserve(additional));
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

        if in_batch {
            return Ok(());
        }

        let handled_by_active_wave = PROPAGATION.with(|p| {
            let mut state = p.borrow_mut();
            if state.active {
                state.changed_signals.insert(id);
                // Check if this signal is the output of the currently executing derived node.
                // If so, its transitive subscribers are already scheduled in topological order.
                let active_obs = OBSERVER_STACK.with(|s| s.borrow().last().copied());
                let is_current_node_output = active_obs
                    .is_some_and(|obs| GRAPH.with(|g| g.borrow().output_signal(obs) == Some(id)));
                if !is_current_node_output && !state.extra_signals.contains(&id) {
                    state.extra_signals.push(id);
                }
                true
            } else {
                false
            }
        });

        if !handled_by_active_wave {
            Self::propagate_signals(&[id]);
        }
        Ok(())
    }

    /// Allocate a fresh `NodeId` without yet attaching a callback.
    pub fn allocate_node_id() -> NodeId {
        NEXT_NODE_INDEX.with(|next| {
            let mut val = next.borrow_mut();
            let id = NodeId::new(*val, 1);
            *val += 1;
            id
        })
    }

    /// Register a callback for a previously allocated `NodeId`.
    pub fn register_node_with_id<F>(id: NodeId, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        CALLBACKS.with(|cbs| {
            cbs.borrow_mut().insert(id, Arc::new(callback));
        });
    }

    /// Associate a derived node (e.g. a `Memo`) with the output signal it writes.
    pub fn register_node_output(node: NodeId, signal: SignalId) {
        GRAPH.with(|graph| {
            graph.borrow_mut().register_node_output(node, signal);
        });
    }

    /// Register a new computation node.
    pub fn register_node<F>(callback: F) -> NodeId
    where
        F: Fn() + Send + Sync + 'static,
    {
        let id = Self::allocate_node_id();
        Self::register_node_with_id(id, callback);
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

    /// Execute a closure while tracking signal reads as dependencies of `node`.
    pub fn track_node<R, F: FnOnce() -> R>(node: NodeId, f: F) -> R {
        GRAPH.with(|g| g.borrow_mut().clear_dependencies_for_node(node));
        let _guard = ObserverGuard::push(node);
        f()
    }

    /// Run a closure inside a batched transaction.
    pub fn batch<R, F: FnOnce() -> R>(f: F) -> R {
        let mut guard = BatchGuard::enter();
        let result = f();
        guard.disarm();

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
            Self::propagate_signals(&pending);
        }
        result
    }

    /// Propagate changes for one or more signals to all transitive subscribers in topological order.
    fn propagate_signals(initial_signals: &[SignalId]) {
        const MAX_WAVES: usize = 64;
        let _guard = PropagationGuard;

        let mut wave_signals: Vec<SignalId> = initial_signals.to_vec();
        for _ in 0..MAX_WAVES {
            if wave_signals.is_empty() {
                break;
            }

            let affected = GRAPH.with(|g| {
                g.borrow()
                    .collect_batch_subscribers_topological(&wave_signals)
            });

            PROPAGATION.with(|p| {
                let mut state = p.borrow_mut();
                state.active = true;
                state.changed_signals.clear();
                for &sig in &wave_signals {
                    state.changed_signals.insert(sig);
                }
                state.extra_signals.clear();
            });

            for node in affected {
                let should_run = PROPAGATION.with(|p| {
                    let changed = &p.borrow().changed_signals;
                    GRAPH.with(|g| g.borrow().has_dependency_in(node, changed))
                });
                if should_run {
                    Self::run_node_callback(node);
                }
            }

            wave_signals = PROPAGATION.with(|p| std::mem::take(&mut p.borrow_mut().extra_signals));
        }
    }

    /// Execute the registered callback for a node with dependency tracking.
    pub fn run_node_callback(node: NodeId) {
        let callback = CALLBACKS.with(|cbs| cbs.borrow().get(&node).cloned());
        if let Some(cb) = callback {
            GRAPH.with(|g| g.borrow_mut().clear_dependencies_for_node(node));
            let _guard = ObserverGuard::push(node);
            cb();
        }
    }
}
