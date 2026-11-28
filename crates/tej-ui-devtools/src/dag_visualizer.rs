//! Reactive dependency Directed Acyclic Graph (DAG) inspector and visualizer.

use std::collections::HashMap;
use tej_ui_core::{Epoch, NodeId, SignalId};

/// Metadata description of a reactive node in the runtime graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DagNodeInfo {
    /// Node identifier in the arena.
    pub id: NodeId,
    /// Human-readable label or debug name.
    pub label: String,
    /// Last epoch when this node was evaluated.
    pub last_updated_epoch: Epoch,
    /// Whether this node is currently marked dirty.
    pub is_dirty: bool,
    /// Number of upstream dependencies.
    pub dependency_count: usize,
    /// Number of downstream subscribers.
    pub subscriber_count: usize,
}

/// Snapshot of the complete reactive dependency DAG for debugging.
#[derive(Debug, Clone, Default)]
pub struct DagSnapshot {
    /// Active nodes indexed by NodeId.
    pub nodes: HashMap<NodeId, DagNodeInfo>,
    /// Directed edges: from dependency (upstream) -> to subscriber (downstream).
    pub edges: Vec<(NodeId, NodeId)>,
}

impl DagSnapshot {
    /// Creates a new empty DAG snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node with metadata to the snapshot.
    pub fn add_node(&mut self, node: DagNodeInfo) {
        self.nodes.insert(node.id, node);
    }

    /// Adds a directed dependency edge from source -> target.
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        self.edges.push((from, to));
    }

    /// Exports the dependency graph as Graphviz DOT format for rendering in DevTools.
    #[must_use]
    pub fn to_dot(&self) -> String {
        use std::fmt::Write;
        let mut out = String::from("digraph ReactiveDAG {\n  rankdir=LR;\n  node [shape=box];\n");
        for (id, node) in &self.nodes {
            let color = if node.is_dirty { "red" } else { "black" };
            let _ = writeln!(
                out,
                "  \"{id:?}\" [label=\"{} (Epoch: {})\" color=\"{color}\"];",
                node.label, node.last_updated_epoch.0
            );
        }
        for (from, to) in &self.edges {
            let _ = writeln!(out, "  \"{from:?}\" -> \"{to:?}\";");
        }
        out.push_str("}\n");
        out
    }

    /// Counts how many nodes are currently marked dirty.
    #[must_use]
    pub fn dirty_count(&self) -> usize {
        self.nodes.values().filter(|n| n.is_dirty).count()
    }
}

/// Helper to track and snapshot reactive dependencies.
#[derive(Debug, Default)]
pub struct ReactiveDagInspector {
    node_names: HashMap<NodeId, String>,
}

impl ReactiveDagInspector {
    /// Creates a new inspector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Names a signal or node for human-readable debugging.
    pub fn name_node(&mut self, node_id: NodeId, name: impl Into<String>) {
        self.node_names.insert(node_id, name.into());
    }

    /// Returns recorded name or default label.
    #[must_use]
    pub fn node_label(&self, node_id: NodeId) -> String {
        self.node_names
            .get(&node_id)
            .cloned()
            .unwrap_or_else(|| format!("Node_{node_id:?}"))
    }

    /// Names a signal by converting its SignalId to NodeId.
    pub fn name_signal(&mut self, signal_id: SignalId, name: impl Into<String>) {
        self.name_node(NodeId::new(signal_id.index, signal_id.generation), name);
    }
}
