use crate::id::{Epoch, NodeId, SignalId};
use smallvec::SmallVec;
use std::collections::{HashMap, HashSet, VecDeque};

/// Dependency graph tracking subscribers and dependencies for glitch-free reactivity.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// SignalId -> list of NodeIds observing this signal.
    signal_subscribers: HashMap<SignalId, SmallVec<[NodeId; 4]>>,
    /// NodeId -> list of SignalIds this node depends upon.
    node_dependencies: HashMap<NodeId, SmallVec<[SignalId; 4]>>,
    /// Depth/rank of each node for topological sorting.
    node_depth: HashMap<NodeId, u32>,
    /// Global reactive epoch.
    current_epoch: Epoch,
}

impl DependencyGraph {
    /// Create a new dependency graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance the global reactive epoch.
    pub fn advance_epoch(&mut self) -> Epoch {
        self.current_epoch.advance()
    }

    /// Get current epoch.
    #[must_use]
    pub const fn epoch(&self) -> Epoch {
        self.current_epoch
    }

    /// Register a dependency edge: `node` depends on `signal`.
    pub fn add_dependency(&mut self, node: NodeId, signal: SignalId) {
        let subs = self.signal_subscribers.entry(signal).or_default();
        if !subs.contains(&node) {
            subs.push(node);
        }

        let deps = self.node_dependencies.entry(node).or_default();
        if !deps.contains(&signal) {
            deps.push(signal);
        }

        // Initialize node depth if not present
        self.node_depth.entry(node).or_insert(0);
    }

    /// Clear all dependencies for a node before re-evaluating its computation.
    pub fn clear_dependencies_for_node(&mut self, node: NodeId) {
        if let Some(deps) = self.node_dependencies.remove(&node) {
            for signal in deps {
                if let Some(subs) = self.signal_subscribers.get_mut(&signal) {
                    subs.retain(|n| *n != node);
                }
            }
        }
    }

    /// Remove a node entirely from the dependency graph.
    pub fn remove_node(&mut self, node: NodeId) {
        self.clear_dependencies_for_node(node);
        self.node_depth.remove(&node);
    }

    /// Remove a signal entirely from the graph.
    pub fn remove_signal(&mut self, signal: SignalId) {
        if let Some(subs) = self.signal_subscribers.remove(&signal) {
            for node in subs {
                if let Some(deps) = self.node_dependencies.get_mut(&node) {
                    deps.retain(|s| *s != signal);
                }
            }
        }
    }

    /// Collect all subscribers for a signal, topologically sorted by graph depth.
    #[must_use]
    pub fn collect_subscribers_topological(&self, signal: SignalId) -> Vec<NodeId> {
        let mut dirty_nodes = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(subs) = self.signal_subscribers.get(&signal) {
            for &node in subs {
                if dirty_nodes.insert(node) {
                    queue.push_back(node);
                }
            }
        }

        // Sort by topological depth to guarantee parents evaluate before children
        let mut sorted: Vec<NodeId> = dirty_nodes.into_iter().collect();
        sorted.sort_by_key(|node| self.node_depth.get(node).copied().unwrap_or(0));
        sorted
    }

    /// Collect all subscribers for a set of signals, topologically sorted.
    #[must_use]
    pub fn collect_batch_subscribers_topological(&self, signals: &[SignalId]) -> Vec<NodeId> {
        let mut dirty_nodes = HashSet::new();
        for &signal in signals {
            if let Some(subs) = self.signal_subscribers.get(&signal) {
                for &node in subs {
                    dirty_nodes.insert(node);
                }
            }
        }

        let mut sorted: Vec<NodeId> = dirty_nodes.into_iter().collect();
        sorted.sort_by_key(|node| self.node_depth.get(node).copied().unwrap_or(0));
        sorted
    }

    /// Set topological depth for a node.
    pub fn set_node_depth(&mut self, node: NodeId, depth: u32) {
        self.node_depth.insert(node, depth);
    }
}
