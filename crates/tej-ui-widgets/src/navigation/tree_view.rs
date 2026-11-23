use std::fmt::Debug;
use tej_ui_core::Signal;
use tej_ui_view::View;

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
    pub node: u32,
    pub item_nodes: Vec<u32>,
}

impl View for TreeView {
    type State = TreeViewState;

    fn build(&self) -> Self::State {
        TreeViewState {
            selected_id: self.selected_id.get(),
            visible_count: self.visible_flattened().len(),
            node: 0,
            item_nodes: Vec::new(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.selected_id = self.selected_id.get();
        state.visible_count = self.visible_flattened().len();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let visible = self.visible_flattened();
        let root_style = tej_ui_layout::LayoutStyle {
            direction: tej_ui_layout::FlexDirection::Column,
            spacing: 2.0,
            ..Default::default()
        };
        let root = arena.allocate_node(root_style);
        state.node = root;
        state.item_nodes.clear();

        for _ in 0..visible.len() {
            let row_style = tej_ui_layout::LayoutStyle {
                height: tej_ui_layout::Dimension::Points(26.0),
                ..Default::default()
            };
            let row_node = arena.allocate_node(row_style);
            state.item_nodes.push(row_node);
            arena.add_child(root, row_node);
        }

        root
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        let visible = self.visible_flattened();
        for (i, (depth, node)) in visible.iter().enumerate() {
            if let Some(&node_id) = state.item_nodes.get(i) {
                if let Some(bounds) = ctx.arena.computed_bounds.get(node_id as usize).copied() {
                    let is_selected = state.selected_id.as_deref() == Some(&node.id);
                    let is_hovered = ctx.is_hovered(node_id);

                    if is_selected {
                        ctx.fill_rounded_rect(bounds, ctx.theme.radii.sm, ctx.theme.colors.accent);
                    } else if is_hovered {
                        ctx.fill_rounded_rect(
                            bounds,
                            ctx.theme.radii.sm,
                            ctx.theme.colors.accent.with_alpha(80),
                        );
                    }

                    let indent = (*depth as f32) * 16.0 + 8.0;
                    let prefix = if node.children.is_empty() {
                        "• "
                    } else if node.is_expanded {
                        "▾ "
                    } else {
                        "▸ "
                    };
                    let label = format!("{prefix}{}", node.label);

                    let text_color = if is_selected {
                        ctx.theme.colors.accent_foreground
                    } else {
                        ctx.theme.colors.foreground
                    };
                    let pt = tej_ui_layout::Point::new(
                        bounds.origin.x + indent,
                        bounds.origin.y + (bounds.size.height - 14.0) * 0.5,
                    );
                    ctx.draw_text(
                        &label,
                        pt,
                        ctx.theme.typography.body_small.font_size,
                        text_color,
                    );
                }
            }
        }
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut tej_ui_view::EventDispatcher) {
        let visible = self.visible_flattened();
        for (i, (_, node)) in visible.iter().enumerate() {
            if let Some(&node_id) = state.item_nodes.get(i) {
                let id = node.id.clone();
                let sig = self.selected_id;
                dispatcher.register_click(
                    node_id,
                    std::sync::Arc::new(move || {
                        sig.set(Some(id.clone()));
                    }),
                );
            }
        }
    }
}
