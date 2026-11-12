use tej_core::Signal;
use tej_view::{Text, View};
use tej_widgets::{
    CommandItem, CommandPalette, DataGrid, GridColumn, Modal, PinState, Popover, TabItem, Tabs,
    TreeNode, TreeView, fuzzy_match,
};

#[test]
fn test_datagrid_2d_virtualization() {
    let columns = vec![
        GridColumn::new("id", "ID").width(60.0).pin(PinState::Left),
        GridColumn::new("title", "Title").width(200.0),
        GridColumn::new("status", "Status").width(120.0),
        GridColumn::new("actions", "Actions")
            .width(80.0)
            .pin(PinState::Right),
    ];

    let scroll_x = Signal::new(0.0f32);
    let scroll_y = Signal::new(100.0f32); // Scrolled down

    let grid = DataGrid::new(
        columns,
        tej_widgets::DataGridDimensions {
            total_rows: 100_000,
            row_height: 30.0,
            viewport_width: 800.0,
            viewport_height: 600.0,
        },
        scroll_x,
        scroll_y,
        |coords| Text::new(format!("Cell ({}, {})", coords.row, coords.col)),
    );

    let (start_row, end_row) = grid.visible_rows();
    assert!(start_row >= 3);
    assert!(end_row <= 25);

    let visible_cols = grid.visible_columns();
    // Must include pinned left (0) and pinned right (3)
    assert!(visible_cols.contains(&0));
    assert!(visible_cols.contains(&3));

    let state = grid.build();
    assert!(!state.cell_states.is_empty());
}

#[test]
fn test_fuzzy_matching_and_command_palette() {
    // Test fuzzy algorithm directly
    let m1 = fuzzy_match("cmk", "Command Palette");
    assert!(m1.is_none());

    let m2 = fuzzy_match("pal", "Command Palette");
    assert!(m2.is_some());
    assert!(m2.unwrap().score > 0);

    let m_word_boundary = fuzzy_match("cp", "Command-Palette");
    assert!(m_word_boundary.is_some());

    // Test CommandPalette filtering
    let query = Signal::new(String::from("set"));
    let selected = Signal::new(0usize);
    let items = vec![
        CommandItem::new("1", "Open Settings", "Preferences").shortcut("Cmd+,"),
        CommandItem::new("2", "Toggle Terminal", "View").shortcut("Ctrl+`"),
        CommandItem::new("3", "Reset Cache", "Developer"),
    ];

    let palette = CommandPalette::new(query, selected, items);
    let filtered = palette.filtered_items();
    assert_eq!(filtered.len(), 2); // "Open Settings", "Reset Cache"
    assert_eq!(filtered[0].title, "Open Settings");

    let state = palette.build();
    assert_eq!(state.visible_items.len(), 2);
}

#[test]
fn test_modal_and_popover() {
    let is_open = Signal::new(false);
    let modal = Modal::new(is_open, Text::new("Dialog Body"));

    let mut state = modal.build();
    assert!(!state.is_open);
    assert!(state.content_state.is_none());

    is_open.set(true);
    modal.patch(&mut state, 1);
    assert!(state.is_open);
    assert!(state.content_state.is_some());

    let popover = Popover::new(is_open, Text::new("Anchor"), Text::new("Tooltip"));
    let p_state = popover.build();
    assert!(p_state.is_open);
    assert!(p_state.content_state.is_some());
}

#[test]
fn test_tabs_container() {
    let tabs = vec![
        TabItem::new("overview", "Overview"),
        TabItem::new("specs", "Specifications").badge("3"),
    ];

    let active_tab = Signal::new(0usize);
    let tabs_view = Tabs::new(tabs, active_tab, |idx| {
        Text::new(format!("Tab content #{idx}"))
    });

    let mut state = tabs_view.build();
    assert_eq!(state.active_tab, 0);
    assert_eq!(state.content_state.current_text, "Tab content #0");

    active_tab.set(1);
    tabs_view.patch(&mut state, 1);
    assert_eq!(state.active_tab, 1);
    assert_eq!(state.content_state.current_text, "Tab content #1");
}

#[test]
fn test_tree_view_hierarchy() {
    let roots = vec![
        TreeNode::branch(
            "src",
            "src",
            vec![
                TreeNode::leaf("main.rs", "main.rs"),
                TreeNode::leaf("lib.rs", "lib.rs"),
            ],
        ),
        TreeNode::leaf("Cargo.toml", "Cargo.toml"),
    ];

    let selected = Signal::new(None);
    let tree = TreeView::new(roots, selected);

    let visible = tree.visible_flattened();
    assert_eq!(visible.len(), 4);
    assert_eq!(visible[0].0, 0); // depth 0 (src)
    assert_eq!(visible[1].0, 1); // depth 1 (main.rs)
    assert_eq!(visible[2].0, 1); // depth 1 (lib.rs)
    assert_eq!(visible[3].0, 0); // depth 0 (Cargo.toml)

    let state = tree.build();
    assert_eq!(state.visible_count, 4);
}
