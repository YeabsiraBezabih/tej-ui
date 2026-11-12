use std::fmt::Debug;
use tej_core::Signal;
use tej_view::View;

/// A node within the hierarchical TreeView.
#[derive(Clone, Debug)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub is_expanded: bool,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    /// Create a leaf tree node.
    pub fn leaf(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            is_expanded: false,
            children: Vec::new(),
        }
    }

    /// Create an expandable parent branch node.
    pub fn branch(
        id: impl Into<String>,
        label: impl Into<String>,
        children: Vec<TreeNode>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            is_expanded: true,
            children,
        }
    }
}

/// Collapsible hierarchical tree view widget.
#[derive(Clone, Debug)]
pub struct TreeView {
    pub roots: Vec<TreeNode>,
    pub selected_id: Signal<Option<String>>,
}

impl TreeView {
    /// Create a new tree view.
    pub fn new(roots: Vec<TreeNode>, selected_id: Signal<Option<String>>) -> Self {
        Self { roots, selected_id }
    }

    /// Flatten visible nodes accounting for expansion state.
    #[must_use]
    pub fn visible_flattened(&self) -> Vec<(usize, &TreeNode)> {
        let mut result = Vec::new();
        for root in &self.roots {
            Self::collect_visible(root, 0, &mut result);
        }
        result
    }

    fn collect_visible<'a>(node: &'a TreeNode, depth: usize, out: &mut Vec<(usize, &'a TreeNode)>) {
        out.push((depth, node));
        if node.is_expanded {
            for child in &node.children {
                Self::collect_visible(child, depth + 1, out);
            }
        }
    }
}

/// State of a mounted TreeView.
#[derive(Debug)]
pub struct TreeViewState {
    pub selected_id: Option<String>,
    pub visible_count: usize,
}

impl View for TreeView {
    type State = TreeViewState;

    fn build(&self) -> Self::State {
        TreeViewState {
            selected_id: self.selected_id.get(),
            visible_count: self.visible_flattened().len(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.selected_id = self.selected_id.get();
        state.visible_count = self.visible_flattened().len();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
