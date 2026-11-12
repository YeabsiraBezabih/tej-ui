use crate::node::{A11yId, A11yNode};
use crate::role::A11yRole;
use std::collections::HashMap;
use tej_layout::Rect;

/// Semantic accessibility tree bridge for platform screen readers.
#[derive(Debug, Default)]
pub struct A11yTree {
    root_id: Option<A11yId>,
    nodes: HashMap<A11yId, A11yNode>,
    next_id: u32,
}

impl A11yTree {
    /// Create a new accessibility tree.
    #[must_use]
    pub fn new() -> Self {
        Self {
            root_id: None,
            nodes: HashMap::new(),
            next_id: 1,
        }
    }

    /// Allocate a new node ID.
    pub fn allocate_id(&mut self) -> A11yId {
        let id = A11yId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Insert or update a node in the tree.
    pub fn insert_node(&mut self, node: A11yNode) {
        if self.root_id.is_none() && node.role == A11yRole::Window {
            self.root_id = Some(node.id);
        }
        self.nodes.insert(node.id, node);
    }

    /// Update node bounds from layout calculation.
    pub fn update_bounds(&mut self, id: A11yId, bounds: Rect) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.bounds = bounds;
        }
    }

    /// Look up node by ID.
    #[must_use]
    pub fn get_node(&self, id: A11yId) -> Option<&A11yNode> {
        self.nodes.get(&id)
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
}
