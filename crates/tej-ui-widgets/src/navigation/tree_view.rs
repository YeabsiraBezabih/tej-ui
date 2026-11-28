use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_view::View;

/// Callback invoked when a branch node's expansion state changes: `(node_id, is_expanded)`.
pub type TreeToggleCallback = Arc<dyn Fn(&str, bool) + Send + Sync>;

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

fn collect_initial_expanded(nodes: &[TreeNode], out: &mut HashSet<String>) {
    for node in nodes {
        if node.is_expanded && !node.children.is_empty() {
            out.insert(node.id.clone());
        }
        collect_initial_expanded(&node.children, out);
    }
}

/// Collapsible hierarchical tree view widget.
#[derive(Clone)]
pub struct TreeView {
    pub roots: Vec<TreeNode>,
    pub selected_id: Signal<Option<String>>,
    pub expanded_ids: Signal<HashSet<String>>,
    on_toggle: Option<TreeToggleCallback>,
}

impl Debug for TreeView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeView")
            .field("roots", &self.roots)
            .field("selected_id", &self.selected_id)
            .field("expanded_ids", &self.expanded_ids)
            .finish_non_exhaustive()
    }
}

impl TreeView {
    /// Create a new tree view with expansion state initialized from `TreeNode::is_expanded`.
    pub fn new(roots: Vec<TreeNode>, selected_id: Signal<Option<String>>) -> Self {
        let mut expanded = HashSet::new();
        collect_initial_expanded(&roots, &mut expanded);
        Self {
            roots,
            selected_id,
            expanded_ids: Signal::new(expanded),
            on_toggle: None,
        }
    }

    /// Bind an external reactive `Signal<HashSet<String>>` controlling expanded node IDs.
    #[must_use]
    pub fn expanded_ids(mut self, expanded_ids: Signal<HashSet<String>>) -> Self {
        self.expanded_ids = expanded_ids;
        self
    }

    /// Register a callback invoked whenever a branch node is expanded or collapsed.
    #[must_use]
    pub fn on_toggle<F>(mut self, cb: F) -> Self
    where
        F: Fn(&str, bool) + Send + Sync + 'static,
    {
        self.on_toggle = Some(Arc::new(cb));
        self
    }

    /// Returns whether the node with `id` is currently expanded.
    #[must_use]
    pub fn is_expanded(&self, id: &str) -> bool {
        self.expanded_ids.get().contains(id)
    }

    /// Toggles expansion state for a branch node ID.
    pub fn toggle_expand(&self, id: &str) {
        toggle_expanded_signal(self.expanded_ids, id, self.on_toggle.as_ref());
    }

    /// Expands a branch node ID.
    pub fn expand(&self, id: &str) {
        set_expanded_signal(
            self.expanded_ids,
            id,
            ExpandState::Expanded,
            self.on_toggle.as_ref(),
        );
    }

    /// Collapses a branch node ID.
    pub fn collapse(&self, id: &str) {
        set_expanded_signal(
            self.expanded_ids,
            id,
            ExpandState::Collapsed,
            self.on_toggle.as_ref(),
        );
    }

    /// Flatten visible nodes accounting for reactive expansion state.
    #[must_use]
    pub fn visible_flattened(&self) -> Vec<(usize, &TreeNode)> {
        let expanded = self.expanded_ids.get();
        let mut result = Vec::new();
        for root in &self.roots {
            Self::collect_visible(root, 0, &expanded, &mut result);
        }
        result
    }

    fn collect_visible<'a>(
        node: &'a TreeNode,
        depth: usize,
        expanded: &HashSet<String>,
        out: &mut Vec<(usize, &'a TreeNode)>,
    ) {
        out.push((depth, node));
        if !node.children.is_empty() && expanded.contains(&node.id) {
            for child in &node.children {
                Self::collect_visible(child, depth + 1, expanded, out);
            }
        }
    }
}

#[derive(Clone, Copy)]
enum ExpandState {
    Expanded,
    Collapsed,
}

fn toggle_expanded_signal(
    expanded_ids: Signal<HashSet<String>>,
    id: &str,
    on_toggle: Option<&TreeToggleCallback>,
) {
    let mut set = expanded_ids.get();
    let now_expanded = if set.contains(id) {
        set.remove(id);
        false
    } else {
        set.insert(id.to_string());
        true
    };
    expanded_ids.set(set);
    if let Some(cb) = on_toggle {
        cb(id, now_expanded);
    }
}

fn set_expanded_signal(
    expanded_ids: Signal<HashSet<String>>,
    id: &str,
    state: ExpandState,
    on_toggle: Option<&TreeToggleCallback>,
) {
    let expanded = matches!(state, ExpandState::Expanded);
    let mut set = expanded_ids.get();
    let changed = if expanded {
        set.insert(id.to_string())
    } else {
        set.remove(id)
    };
    if changed {
        expanded_ids.set(set);
        if let Some(cb) = on_toggle {
            cb(id, expanded);
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
        let expanded = self.expanded_ids.get();
        for (i, (depth, node)) in visible.iter().enumerate() {
            if let Some(&node_id) = state.item_nodes.get(i)
                && let Some(bounds) = ctx.arena.computed_bounds.get(node_id as usize).copied()
            {
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

                let indent = crate::count_to_f32(*depth) * 16.0 + 8.0;
                let prefix = if node.children.is_empty() {
                    "• "
                } else if expanded.contains(&node.id) {
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

    fn register_events(&self, state: &Self::State, dispatcher: &mut tej_ui_view::EventDispatcher) {
        let visible = self.visible_flattened();
        let visible_ids: Vec<String> = visible.iter().map(|(_, n)| n.id.clone()).collect();

        for (i, (_, node)) in visible.iter().enumerate() {
            if let Some(&node_id) = state.item_nodes.get(i) {
                let id = node.id.clone();
                let has_children = !node.children.is_empty();
                let sig = self.selected_id;
                let exp_sig = self.expanded_ids;
                let on_toggle = self.on_toggle.clone();

                // Click selects the node and toggles expand/collapse for branches
                dispatcher.register_click(
                    node_id,
                    Arc::new(move || {
                        sig.set(Some(id.clone()));
                        if has_children {
                            toggle_expanded_signal(exp_sig, &id, on_toggle.as_ref());
                        }
                    }),
                );

                // Keyboard navigation: Left collapses, Right expands, Up/Down navigates, Enter/Space toggles
                let id_key = node.id.clone();
                let sig_key = self.selected_id;
                let exp_key = self.expanded_ids;
                let on_toggle_key = self.on_toggle.clone();
                let ids_for_nav = visible_ids.clone();
                dispatcher.register_key_down(
                    node_id,
                    Arc::new(move |key_code| match key_code {
                        37 => {
                            // ArrowLeft -> collapse branch
                            if has_children {
                                set_expanded_signal(
                                    exp_key,
                                    &id_key,
                                    ExpandState::Collapsed,
                                    on_toggle_key.as_ref(),
                                );
                            }
                        }
                        39 => {
                            // ArrowRight -> expand branch
                            if has_children {
                                set_expanded_signal(
                                    exp_key,
                                    &id_key,
                                    ExpandState::Expanded,
                                    on_toggle_key.as_ref(),
                                );
                            }
                        }
                        38 => {
                            // ArrowUp -> previous visible row
                            if i > 0
                                && let Some(prev_id) = ids_for_nav.get(i - 1)
                            {
                                sig_key.set(Some(prev_id.clone()));
                            }
                        }
                        40 => {
                            // ArrowDown -> next visible row
                            if let Some(next_id) = ids_for_nav.get(i + 1) {
                                sig_key.set(Some(next_id.clone()));
                            }
                        }
                        13 | 32 => {
                            sig_key.set(Some(id_key.clone()));
                            if has_children {
                                toggle_expanded_signal(exp_key, &id_key, on_toggle_key.as_ref());
                            }
                        }
                        _ => {}
                    }),
                );
            }
        }
    }
}
