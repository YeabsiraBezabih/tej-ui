//! Virtualized tree indexer and multi-selection manager for 100k+ hierarchical tree nodes.

use std::collections::HashSet;
use std::fmt::Debug;
use std::ops::Range;

use crate::navigation::tree_view::TreeNode;

/// Flat projection of an expanded tree node for high-performance virtualization.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FlatTreeItem {
    pub id: String,
    pub label: String,
    pub depth: usize,
    pub has_children: bool,
    pub is_expanded: bool,
}

/// Virtualized tree row indexer calculating visible row windows without allocating nested views.
#[derive(Clone, Debug, Default)]
pub struct VirtualizedTreeIndex {
    items: Vec<FlatTreeItem>,
}

impl VirtualizedTreeIndex {
    /// Build a flat virtual index from tree roots.
    #[must_use]
    pub fn from_roots(roots: &[TreeNode]) -> Self {
        let mut items = Vec::new();
        for root in roots {
            Self::flatten_recursive(root, 0, &mut items);
        }
        Self { items }
    }

    fn flatten_recursive(node: &TreeNode, depth: usize, out: &mut Vec<FlatTreeItem>) {
        out.push(FlatTreeItem {
            id: node.id.clone(),
            label: node.label.clone(),
            depth,
            has_children: !node.children.is_empty(),
            is_expanded: node.is_expanded,
        });

        if node.is_expanded {
            for child in &node.children {
                Self::flatten_recursive(child, depth + 1, out);
            }
        }
    }

    /// Total number of currently visible/expanded rows.
    #[must_use]
    pub fn total_visible_rows(&self) -> usize {
        self.items.len()
    }

    /// Read flat tree item at row index.
    #[must_use]
    pub fn get(&self, row_index: usize) -> Option<&FlatTreeItem> {
        self.items.get(row_index)
    }

    /// Calculate the visible row slice range for a virtual viewport.
    #[must_use]
    pub fn compute_visible_range(
        &self,
        scroll_y: f32,
        row_height: f32,
        viewport_height: f32,
        overscan: usize,
    ) -> Range<usize> {
        if self.items.is_empty() || row_height <= 0.0 {
            return 0..0;
        }

        let first_row = ((scroll_y / row_height).floor() as usize).saturating_sub(overscan);
        let visible_count = ((viewport_height / row_height).ceil() as usize) + (overscan * 2);
        let end_row = (first_row + visible_count).min(self.items.len());

        first_row..end_row
    }
}

/// Multi-selection state for tree views supporting Shift-range and Cmd/Ctrl toggle selections.
#[derive(Clone, Debug, Default)]
pub struct TreeMultiSelection {
    selected_ids: HashSet<String>,
    last_focused_id: Option<String>,
}

impl TreeMultiSelection {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn is_selected(&self, id: &str) -> bool {
        self.selected_ids.contains(id)
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.selected_ids.len()
    }

    #[must_use]
    pub fn selected_ids(&self) -> &HashSet<String> {
        &self.selected_ids
    }

    /// Handle click selection:
    /// - `is_toggle` (Cmd/Ctrl): toggles the clicked node.
    /// - Single click: selects only this node.
    pub fn select(&mut self, id: impl Into<String>, is_toggle: bool) {
        let id_str = id.into();
        if is_toggle {
            if self.selected_ids.contains(&id_str) {
                self.selected_ids.remove(&id_str);
            } else {
                self.selected_ids.insert(id_str.clone());
            }
        } else {
            self.selected_ids.clear();
            self.selected_ids.insert(id_str.clone());
        }
        self.last_focused_id = Some(id_str);
    }

    /// Clear all selections.
    pub fn clear(&mut self) {
        self.selected_ids.clear();
        self.last_focused_id = None;
    }
}
