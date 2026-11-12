//! Main view layout and UI composition for the Obsidian clone desktop app.

use crate::models::NoteItem;
use tej::prelude::*;
use tej_animation::{AnimatedSpring, SpringConfig};
use tej_icons::LucideIcon;
use tej_widgets::{
    Badge, BadgeVariant, Card, RichTextEditor, Sidebar, SidebarItem, TabItem, Tabs, TreeNode,
    TreeView,
};

/// Master application state for the Obsidian clone.
#[derive(Clone, Debug)]
pub struct ObsidianState {
    pub active_tab: Signal<usize>,
    pub selected_note_id: Signal<Option<String>>,
    pub sidebar_collapsed: Signal<bool>,
    #[allow(dead_code)]
    pub palette_open: Signal<bool>,
    #[allow(dead_code)]
    pub palette_query: Signal<String>,
    #[allow(dead_code)]
    pub palette_index: Signal<usize>,
    pub sidebar_width_spring: AnimatedSpring,
}

impl Default for ObsidianState {
    fn default() -> Self {
        Self::new()
    }
}

impl ObsidianState {
    /// Create initial Obsidian desktop state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_tab: Signal::new(0),
            selected_note_id: Signal::new(Some("1".to_string())),
            sidebar_collapsed: Signal::new(false),
            palette_open: Signal::new(false),
            palette_query: Signal::new(String::new()),
            palette_index: Signal::new(0),
            sidebar_width_spring: AnimatedSpring::new(260.0, SpringConfig::snappy()),
        }
    }

    /// Toggle sidebar with smooth spring physics transition.
    pub fn toggle_sidebar(&mut self) {
        let is_collapsed = !self.sidebar_collapsed.get();
        self.sidebar_collapsed.set(is_collapsed);
        let target_width = if is_collapsed { 0.0 } else { 260.0 };
        self.sidebar_width_spring.set_target(target_width);
    }
}

/// Construct the hierarchical vault explorer tree view.
#[must_use]
pub fn build_vault_tree(selected_id: Signal<Option<String>>) -> TreeView {
    let roots = vec![
        TreeNode::branch(
            "folder:arch",
            "Architecture",
            vec![TreeNode::leaf("1", "Architecture of tej-ui.md")],
        ),
        TreeNode::branch(
            "folder:db",
            "Database",
            vec![TreeNode::leaf("2", "Zero-Copy DBMS Integration.md")],
        ),
        TreeNode::branch(
            "folder:graphics",
            "Graphics",
            vec![TreeNode::leaf("3", "3D Knowledge Graph Rendering.md")],
        ),
        TreeNode::branch(
            "folder:anim",
            "Animation",
            vec![TreeNode::leaf("4", "Spring Physics and VSync.md")],
        ),
        TreeNode::branch(
            "folder:platforms",
            "Platforms",
            vec![TreeNode::leaf("5", "Native Platform Compositing.md")],
        ),
        TreeNode::branch(
            "folder:daily",
            "Daily",
            vec![TreeNode::leaf("6", "Daily Note - 2026-11-13.md")],
        ),
    ];

    TreeView::new(roots, selected_id)
}

/// Construct the Obsidian desktop view tree.
pub fn build_obsidian_view(state: &ObsidianState, notes: &[NoteItem]) -> impl View {
    // 1. Sidebar items and badges
    let sidebar_items = vec![
        SidebarItem::new("nav_files", "Files")
            .icon("Folder")
            .active(true),
        SidebarItem::new("nav_search", "Search").icon("Search"),
        SidebarItem::new("nav_bookmarks", "Bookmarks").icon("Bookmark"),
        SidebarItem::new("nav_tags", "Tags").badge(format!("{} tags", 8)),
    ];

    let sidebar_widget = Sidebar::new(state.sidebar_collapsed, sidebar_items);
    let tree_widget = build_vault_tree(state.selected_note_id);

    // 2. Active document editor
    let active_content = notes.first().map_or("# Empty Note", |n| n.content.as_str());

    let editor_widget = RichTextEditor::with_text(active_content)
        .font_size(15.0)
        .line_height(24.0)
        .show_line_numbers(true);

    // 3. Tab headers
    let tab_items = vec![
        TabItem::new("tab_editor", "Architecture of tej-ui.md").badge("Markdown"),
        TabItem::new("tab_graph", "3D Knowledge Graph").badge("Three.js/tej-3d"),
        TabItem::new("tab_grid", "Vault Records").badge("100k Rows"),
    ];

    let tabs_widget = Tabs::new(tab_items, state.active_tab, move |_idx| {
        // Return active view for selected tab
        VStack::new((
            HStack::new((
                Badge::new("Workspace").variant(BadgeVariant::Secondary),
                Badge::new("Memory Paged").variant(BadgeVariant::Outline),
            ))
            .spacing(8),
            Text::new("Obsidian Workspace Active"),
        ))
    });

    // 4. Top App Header Bar
    let header_bar = HStack::new((
        Icon::new(LucideIcon::Menu)
            .size(18.0)
            .color(Color::rgb(156, 163, 175)),
        Text::new("Obsidian Vault — tej-ui (Native Fast)"),
        HStack::new((
            Icon::new(LucideIcon::Search).size(16.0),
            Icon::new(LucideIcon::Plus).size(16.0),
            Icon::new(LucideIcon::Settings).size(16.0),
        ))
        .spacing(12),
    ))
    .spacing(16);

    // 5. Compose full horizontal layout (Sidebar | Main Document Area)
    let left_panel = VStack::new((
        Card::new("Vault Explorer").description("Local files & tags"),
        header_bar,
        sidebar_widget,
        tree_widget,
    ));

    let main_panel = VStack::new((tabs_widget, editor_widget));

    HStack::new((left_panel, main_panel)).spacing(12)
}
