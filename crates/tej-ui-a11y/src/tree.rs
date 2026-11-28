//! Semantic accessibility tree bridge for platform screen readers.

use crate::node::{A11yId, A11yNode};
use crate::role::A11yRole;
use std::collections::{HashMap, HashSet};
use tej_ui_layout::Rect;

/// Semantic accessibility tree bridge for platform screen readers.
#[derive(Debug)]
pub struct A11yTree {
    root_id: Option<A11yId>,
    nodes: HashMap<A11yId, A11yNode>,
    dirty_nodes: HashSet<A11yId>,
    next_id: u32,
}

impl Default for A11yTree {
    fn default() -> Self {
        Self::new()
    }
}

impl A11yTree {
    /// Create a new accessibility tree.
    #[must_use]
    pub fn new() -> Self {
        Self {
            root_id: None,
            nodes: HashMap::new(),
            dirty_nodes: HashSet::new(),
            next_id: 1,
        }
    }

    /// Allocate a new node ID.
    pub fn allocate_id(&mut self) -> A11yId {
        let id = A11yId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Insert or update a node in the tree, marking it dirty for incremental updates.
    pub fn insert_node(&mut self, node: A11yNode) {
        if self.root_id.is_none() && node.role == A11yRole::Window {
            self.root_id = Some(node.id);
        }
        let id = node.id;
        self.nodes.insert(id, node);
        self.dirty_nodes.insert(id);
    }

    /// Remove a node from the tree, detaching it from any parent's children list
    /// and clearing its dirty flag to prevent memory leaks.
    pub fn remove_node(&mut self, id: A11yId) -> Option<A11yNode> {
        let removed = self.nodes.remove(&id)?;
        self.dirty_nodes.remove(&id);
        if self.root_id == Some(id) {
            self.root_id = None;
        }
        let mut modified_parents = Vec::new();
        for (&parent_id, parent) in &mut self.nodes {
            let before = parent.children.len();
            parent.children.retain(|child| *child != id);
            if parent.children.len() != before {
                modified_parents.push(parent_id);
            }
        }
        for parent_id in modified_parents {
            self.dirty_nodes.insert(parent_id);
        }
        Some(removed)
    }

    /// Explicitly mark a node as dirty.
    pub fn mark_dirty(&mut self, id: A11yId) {
        if self.nodes.contains_key(&id) {
            self.dirty_nodes.insert(id);
        }
    }

    /// Update node bounds from layout calculation, marking it dirty if changed.
    pub fn update_bounds(&mut self, id: A11yId, bounds: Rect) {
        if let Some(node) = self.nodes.get_mut(&id)
            && node.bounds != bounds
        {
            node.bounds = bounds;
            self.dirty_nodes.insert(id);
        }
    }

    /// Update node value, marking it dirty if changed.
    pub fn update_value(&mut self, id: A11yId, value: impl Into<String>) {
        if let Some(node) = self.nodes.get_mut(&id) {
            let val = value.into();
            if node.value.as_ref() != Some(&val) {
                node.value = Some(val);
                self.dirty_nodes.insert(id);
            }
        }
    }

    /// Look up node by ID.
    #[must_use]
    pub fn get_node(&self, id: A11yId) -> Option<&A11yNode> {
        self.nodes.get(&id)
    }

    /// Look up mutable node by ID.
    pub fn get_node_mut(&mut self, id: A11yId) -> Option<&mut A11yNode> {
        self.nodes.get_mut(&id)
    }

    /// Get root node ID.
    #[must_use]
    pub const fn root(&self) -> Option<A11yId> {
        self.root_id
    }

    /// Total number of accessibility nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if tree is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the number of currently dirty nodes.
    #[must_use]
    pub fn dirty_count(&self) -> usize {
        self.dirty_nodes.len()
    }

    /// Check if there are any dirty nodes.
    #[must_use]
    pub fn has_dirty(&self) -> bool {
        !self.dirty_nodes.is_empty()
    }

    /// Drain all dirty node IDs, clearing the dirty set.
    pub fn drain_dirty(&mut self) -> Vec<A11yId> {
        self.dirty_nodes.drain().collect()
    }

    /// Iterate over all nodes in the tree.
    pub fn iter(&self) -> impl Iterator<Item = (&A11yId, &A11yNode)> {
        self.nodes.iter()
    }
}
