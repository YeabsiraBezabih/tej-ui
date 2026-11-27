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
    /// SignalId -> NodeId that produces/writes this derived signal (e.g. a `Memo`).
    signal_producer: HashMap<SignalId, NodeId>,
    /// NodeId -> SignalId produced by this derived node (if any).
    node_output_signal: HashMap<NodeId, SignalId>,
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

    /// Associate a derived computation `node` with the output `signal` it updates.
    ///
    /// Enables transitive BFS traversal and topological depth propagation across chains and diamonds.
    pub fn register_node_output(&mut self, node: NodeId, signal: SignalId) {
        self.signal_producer.insert(signal, node);
        self.node_output_signal.insert(node, signal);
        let producer_depth = self.node_depth.get(&node).copied().unwrap_or(0);
        self.propagate_depth_downstream(node, producer_depth);
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

        // Compute true topological depth: depth[sub] = max(depth[sub], depth[dep] + 1)
        let dep_depth = self
            .signal_producer
            .get(&signal)
            .and_then(|producer| self.node_depth.get(producer).copied())
            .unwrap_or(0);
        let candidate_depth = dep_depth.saturating_add(1);
        let entry = self.node_depth.entry(node).or_insert(0);
        if candidate_depth > *entry {
            *entry = candidate_depth;
            self.propagate_depth_downstream(node, candidate_depth);
        }
    }

    /// Propagate increased topological depth to all downstream subscribers with a cycle guard.
    fn propagate_depth_downstream(&mut self, start_node: NodeId, start_depth: u32) {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        visited.insert(start_node);
        queue.push_back((start_node, start_depth));

        while let Some((curr_node, curr_depth)) = queue.pop_front() {
            let Some(&out_sig) = self.node_output_signal.get(&curr_node) else {
                continue;
            };
            let Some(subs) = self.signal_subscribers.get(&out_sig) else {
                continue;
            };
            let subs_snapshot: SmallVec<[NodeId; 4]> = subs.clone();
            let next_depth = curr_depth.saturating_add(1);
            for sub in subs_snapshot {
                if !visited.insert(sub) {
                    continue;
                }
                let sub_entry = self.node_depth.entry(sub).or_insert(0);
                if next_depth > *sub_entry {
                    *sub_entry = next_depth;
                    queue.push_back((sub, next_depth));
                }
            }
        }
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
        if let Some(sig) = self.node_output_signal.remove(&node) {
            self.signal_producer.remove(&sig);
        }
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
        if let Some(node) = self.signal_producer.remove(&signal) {
            self.node_output_signal.remove(&node);
        }
    }

    /// Collect all transitive subscribers for a signal, topologically sorted by ascending depth.
    #[must_use]
    pub fn collect_subscribers_topological(&self, signal: SignalId) -> Vec<NodeId> {
        self.collect_batch_subscribers_topological(&[signal])
    }

    /// Collect all transitive subscribers for a set of signals, topologically sorted by ascending depth.
    #[must_use]
    pub fn collect_batch_subscribers_topological(&self, signals: &[SignalId]) -> Vec<NodeId> {
        let mut dirty_nodes = HashSet::new();
        let mut queue = VecDeque::new();

        for &signal in signals {
            if let Some(subs) = self.signal_subscribers.get(&signal) {
                for &node in subs {
                    if dirty_nodes.insert(node) {
                        queue.push_back(node);
                    }
                }
            }
        }

        // Drain the BFS queue to discover all transitive subscribers across derived signals
        while let Some(node) = queue.pop_front() {
            if let Some(&out_sig) = self.node_output_signal.get(&node) {
                if let Some(subs) = self.signal_subscribers.get(&out_sig) {
                    for &sub in subs {
                        if dirty_nodes.insert(sub) {
                            queue.push_back(sub);
                        }
                    }
                }
            }
        }

        // Sort by ascending topological depth (with deterministic NodeId tie-break)
        let mut sorted: Vec<NodeId> = dirty_nodes.into_iter().collect();
        sorted.sort_by_key(|node| (self.node_depth.get(node).copied().unwrap_or(0), node.index));
        sorted
    }

    /// Set topological depth for a node.
    pub fn set_node_depth(&mut self, node: NodeId, depth: u32) {
        self.node_depth.insert(node, depth);
    }

    /// Get the current topological depth of a node.
    #[must_use]
    pub fn node_depth(&self, node: NodeId) -> u32 {
        self.node_depth.get(&node).copied().unwrap_or(0)
    }

    /// Returns `true` if `node` depends on at least one signal in `changed`.
    #[must_use]
    pub fn has_dependency_in(&self, node: NodeId, changed: &HashSet<SignalId>) -> bool {
        self.node_dependencies
            .get(&node)
            .is_some_and(|deps| deps.iter().any(|s| changed.contains(s)))
    }

    /// Returns the output signal produced by `node`, if it is a derived node.
    #[must_use]
    pub fn output_signal(&self, node: NodeId) -> Option<SignalId> {
        self.node_output_signal.get(&node).copied()
    }
}
