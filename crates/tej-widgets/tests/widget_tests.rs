use std::collections::HashSet;
use tej_core::Signal;
use tej_view::{Text, View};
use tej_widgets::{
    Accordion, AccordionItem, Alert, AlertSeverity, Avatar, Badge, BadgeVariant, Breadcrumb,
    BreadcrumbItem, Card, CommandItem, CommandPalette, CursorPos, DataGrid, GridColumn, Modal,
    Pagination, PinState, Popover, Progress, RadioGroup, RadioItem, RichTextEditor, ScrollArea,
    Select, SelectOption, Separator, Sheet, SheetSide, Sidebar, SidebarItem, Skeleton, Slider,
    Switch, TabItem, Tabs, TextBuffer, Tooltip, TreeNode, TreeView, fuzzy_match,
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
    let scroll_y = Signal::new(100.0f32);

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
    assert!(visible_cols.contains(&0));
    assert!(visible_cols.contains(&3));

    let state = grid.build();
    assert!(!state.cell_states.is_empty());
}

#[test]
fn test_fuzzy_matching_and_command_palette() {
    let m1 = fuzzy_match("cmk", "Command Palette");
    assert!(m1.is_none());

    let m2 = fuzzy_match("pal", "Command Palette");
    assert!(m2.is_some());
    assert!(m2.unwrap().score > 0);

    let query = Signal::new(String::from("set"));
    let selected = Signal::new(0usize);
    let items = vec![
        CommandItem::new("1", "Open Settings", "Preferences").shortcut("Cmd+,"),
        CommandItem::new("2", "Toggle Terminal", "View").shortcut("Ctrl+`"),
        CommandItem::new("3", "Reset Cache", "Developer"),
    ];

    let palette = CommandPalette::new(query, selected, items);
    let filtered = palette.filtered_items();
    assert_eq!(filtered.len(), 2);

    let state = palette.build();
    assert_eq!(state.visible_items.len(), 2);
}

#[test]
fn test_modal_and_popover() {
    let is_open = Signal::new(false);
    let modal = Modal::new(is_open, Text::new("Dialog Body"));

    let mut state = modal.build();
    assert!(!state.is_open);

    is_open.set(true);
    modal.patch(&mut state, 1);
    assert!(state.is_open);

    let popover = Popover::new(is_open, Text::new("Anchor"), Text::new("Tooltip"));
    let p_state = popover.build();
    assert!(p_state.is_open);
}

#[test]
fn test_tabs_and_tree_view() {
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

    active_tab.set(1);
    tabs_view.patch(&mut state, 1);
    assert_eq!(state.active_tab, 1);

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
}

#[test]
fn test_form_controls_switch_slider_radio_select() {
    // Switch
    let switch_on = Signal::new(false);
    let switch = Switch::new(switch_on);
    let mut sw_state = switch.build();
    assert!(!sw_state.is_checked);
    switch.toggle();
    assert!(switch_on.get());
    switch.patch(&mut sw_state, 1);
    assert!(sw_state.is_checked);

    // Slider
    let slider_val = Signal::new(25.0f32);
    let slider = Slider::new(slider_val, 0.0, 100.0).step(5.0);
    let mut sl_state = slider.build();
    assert_eq!(sl_state.current_value, 25.0);
    assert_eq!(sl_state.normalized_ratio, 0.25);
    slider.set_normalized(0.8);
    assert_eq!(slider_val.get(), 80.0);
    slider.patch(&mut sl_state, 1);
    assert_eq!(sl_state.normalized_ratio, 0.8);

    // RadioGroup
    let radio_val = Signal::new("standard".to_string());
    let radio = RadioGroup::new(
        radio_val,
        vec![
            RadioItem::new("standard".to_string(), "Standard"),
            RadioItem::new("pro".to_string(), "Pro"),
        ],
    );
    let mut rg_state = radio.build();
    assert_eq!(rg_state.selected_index, Some(0));
    radio.select_index(1);
    assert_eq!(radio_val.get(), "pro");
    radio.patch(&mut rg_state, 1);
    assert_eq!(rg_state.selected_index, Some(1));

    // Select
    let sel_val = Signal::new("usd".to_string());
    let sel_open = Signal::new(false);
    let select = Select::new(
        sel_val,
        sel_open,
        vec![
            SelectOption::new("usd".to_string(), "US Dollar ($)"),
            SelectOption::new("eur".to_string(), "Euro (€)"),
        ],
        "Select currency",
    );
    let mut sel_state = select.build();
    assert_eq!(sel_state.selected_label, "US Dollar ($)");
    select.choose_index(1);
    assert_eq!(sel_val.get(), "eur");
    select.patch(&mut sel_state, 1);
    assert_eq!(sel_state.selected_label, "Euro (€)");
}

#[test]
fn test_layout_sidebar_card_separator_accordion_scroll() {
    // Sidebar
    let collapsed = Signal::new(false);
    let sidebar = Sidebar::new(
        collapsed,
        vec![
            SidebarItem::new("home", "Home").active(true),
            SidebarItem::new("inbox", "Inbox").badge("5"),
        ],
    );
    let mut sb_state = sidebar.build();
    assert_eq!(sb_state.active_width, 240.0);
    sidebar.toggle_collapse();
    sidebar.patch(&mut sb_state, 1);
    assert_eq!(sb_state.active_width, 64.0);

    // Card
    let card = Card::new("Project Velocity")
        .description("Weekly sprint burndown")
        .footer("View full report");
    let c_state = card.build();
    assert_eq!(c_state.title, "Project Velocity");

    // Separator
    let sep = Separator::horizontal();
    let sep_state = sep.build();
    assert_eq!(sep_state.bounds.size.height, 1.0);

    // Accordion
    let mut initial_open = HashSet::new();
    initial_open.insert("item1".to_string());
    let open_sig = Signal::new(initial_open);
    let accordion = Accordion::new(
        open_sig,
        vec![
            AccordionItem::new("item1", "Billing FAQ", "Billing details..."),
            AccordionItem::new("item2", "Security", "Security compliance..."),
        ],
    );
    let mut acc_state = accordion.build();
    assert!(acc_state.open_items.contains("item1"));
    accordion.toggle_item("item1");
    accordion.patch(&mut acc_state, 1);
    assert!(!acc_state.open_items.contains("item1"));

    // ScrollArea
    let sx = Signal::new(0.0);
    let sy = Signal::new(0.0);
    let scroll = ScrollArea::new(
        sx,
        sy,
        tej_layout::Size::new(500.0, 2000.0),
        tej_layout::Size::new(500.0, 800.0),
    );
    scroll.scroll_by(0.0, 300.0);
    assert_eq!(sy.get(), 300.0);
}

#[test]
fn test_feedback_badge_avatar_progress_skeleton_alert_tooltip() {
    // Badge
    let badge = Badge::new("Active").variant(BadgeVariant::Destructive);
    let b_state = badge.build();
    assert_eq!(b_state.text, "Active");

    // Avatar
    let avatar = Avatar::new("Jane Doe");
    let av_state = avatar.build();
    assert_eq!(av_state.initials, "JD");

    // Progress
    let pct = Signal::new(65.0f32);
    let progress = Progress::new(pct);
    let p_state = progress.build();
    assert_eq!(p_state.percentage, 65.0);

    // Skeleton
    let skeleton = Skeleton::circle(48.0);
    let sk_state = skeleton.build();
    assert_eq!(sk_state.bounds.size.width, 48.0);

    // Alert
    let alert = Alert::new("Security Update")
        .description("Please rotate your API keys")
        .severity(AlertSeverity::Warning);
    let al_state = alert.build();
    assert_eq!(al_state.title, "Security Update");

    // Tooltip
    let visible = Signal::new(false);
    let tooltip = Tooltip::new("Help text", visible);
    let mut t_state = tooltip.build();
    assert!(!t_state.is_visible);
    tooltip.set_visible(true);
    tooltip.patch(&mut t_state, 1);
    assert!(t_state.is_visible);
}

#[test]
fn test_navigation_breadcrumb_pagination_and_sheet() {
    // Breadcrumb
    let breadcrumb = Breadcrumb::new(vec![
        BreadcrumbItem::new("Workspace").href("/workspace"),
        BreadcrumbItem::new("Settings").current(true),
    ]);
    let bc_state = breadcrumb.build();
    assert_eq!(bc_state.count, 2);

    // Pagination
    let page = Signal::new(1usize);
    let pagination = Pagination::new(page, 10);
    let mut pg_state = pagination.build();
    assert_eq!(pg_state.current_page, 1);
    pagination.next_page();
    assert_eq!(page.get(), 2);
    pagination.patch(&mut pg_state, 1);
    assert_eq!(pg_state.current_page, 2);

    // Sheet
    let sheet_open = Signal::new(true);
    let sheet = Sheet::new(sheet_open, "Edit Profile").side(SheetSide::Right);
    let mut sh_state = sheet.build();
    assert!(sh_state.is_open);
    sheet.close();
    assert!(!sheet_open.get());
    sheet.patch(&mut sh_state, 1);
    assert!(!sh_state.is_open);
}

#[test]
fn test_rich_text_editor_operations_and_ime() {
    // 1. TextBuffer line creation and multi-line insertion
    let mut buf = TextBuffer::new();
    assert_eq!(buf.line_count(), 1);
    let p = buf.insert_str(CursorPos::zero(), "Hello\nWorld");
    assert_eq!(buf.line_count(), 2);
    assert_eq!(p.line, 1);
    assert_eq!(p.col, 5);
    assert_eq!(buf.text(), "Hello\nWorld");

    // 2. Editor typing and backspace
    let mut editor = RichTextEditor::new();
    editor.insert_text("Rust");
    assert_eq!(editor.text(), "Rust");
    assert_eq!(editor.selection().head.col, 4);

    editor.backspace();
    assert_eq!(editor.text(), "Rus");

    // 3. Multi-line selection and replacement
    editor.insert_text("t 2024\nFast UI");
    editor.select_all();
    assert!(!editor.selection().is_collapsed());
    editor.insert_text("Replaced");
    assert_eq!(editor.text(), "Replaced");

    // 4. Undo / Redo
    assert!(editor.undo());
    assert_ne!(editor.text(), "Replaced");
    assert!(editor.redo());
    assert_eq!(editor.text(), "Replaced");

    // 5. Native IME composition (e.g. Japanese or Amharic input)
    editor.update_ime("にほんご".to_string(), 4);
    assert!(editor.ime().is_composing);
    assert_eq!(editor.ime().preedit, "にほんご");

    editor.commit_ime();
    assert!(!editor.ime().is_composing);
    assert!(editor.text().ends_with("にほんご"));

    // 6. View build
    let state = editor.build();
    assert!(format!("{state:?}").contains("RichTextEditorState"));
}
