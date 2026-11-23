use std::collections::HashSet;
use tej_ui_core::Signal;
use tej_ui_view::{Text, View};
use tej_ui_widgets::{
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
        tej_ui_widgets::DataGridDimensions {
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
        tej_ui_layout::Size::new(500.0, 2000.0),
        tej_ui_layout::Size::new(500.0, 800.0),
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

#[test]
fn test_popover_collision_aware_auto_flip() {
    use tej_ui_layout::{Rect, Size};
    use tej_ui_widgets::overlays::{
        CollisionPlacementOptions, PopoverAlignment, PopoverPlacement, compute_collision_placement,
    };

    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    let overlay_size = Size::new(200.0, 150.0);

    // Case 1: Centered anchor with preferred Bottom placement fits comfortably without flip
    let center_anchor = Rect::new(500.0, 400.0, 100.0, 40.0);
    let opts = CollisionPlacementOptions {
        preferred_placement: PopoverPlacement::Bottom,
        alignment: PopoverAlignment::Center,
        offset: 8.0,
        viewport_padding: 10.0,
        auto_flip: true,
    };
    let res = compute_collision_placement(center_anchor, overlay_size, viewport, opts);
    assert_eq!(res.actual_placement, PopoverPlacement::Bottom);
    assert!(!res.is_flipped);
    assert_eq!(res.rect.origin.y, 448.0); // 400 + 40 + 8
    assert_eq!(res.rect.origin.x, 450.0); // 500 + (100 - 200)/2
    assert_eq!(res.arrow_offset, 100.0); // Center of 200px overlay

    // Case 2: Near bottom edge with preferred Bottom placement -> auto-flips to Top!
    let bottom_anchor = Rect::new(500.0, 980.0, 100.0, 40.0);
    let res_flip = compute_collision_placement(bottom_anchor, overlay_size, viewport, opts);
    assert_eq!(res_flip.actual_placement, PopoverPlacement::Top);
    assert!(res_flip.is_flipped);
    assert_eq!(res_flip.rect.origin.y, 980.0 - 150.0 - 8.0); // 822.0

    // Case 3: Near right edge with preferred Right placement -> auto-flips to Left!
    let right_anchor = Rect::new(1800.0, 300.0, 80.0, 30.0);
    let right_opts = CollisionPlacementOptions {
        preferred_placement: PopoverPlacement::Right,
        alignment: PopoverAlignment::Start,
        offset: 5.0,
        viewport_padding: 10.0,
        auto_flip: true,
    };
    let res_right_flip =
        compute_collision_placement(right_anchor, overlay_size, viewport, right_opts);
    assert_eq!(res_right_flip.actual_placement, PopoverPlacement::Left);
    assert!(res_right_flip.is_flipped);
    assert_eq!(res_right_flip.rect.origin.x, 1800.0 - 200.0 - 5.0); // 1595.0

    // Case 4: Clamping near left boundary
    let left_edge_anchor = Rect::new(5.0, 200.0, 30.0, 30.0);
    let res_clamped = compute_collision_placement(left_edge_anchor, overlay_size, viewport, opts);
    assert!(res_clamped.rect.origin.x >= 10.0); // Clamped to viewport_padding
}

#[test]
fn test_rich_text_multi_cursor_and_undo_coalescing() {
    use tej_ui_widgets::{CursorPos, EditAction, MultiCursorSelection, TextBuffer, UndoStack};

    // 1. MultiCursorSelection management
    let mut multi = MultiCursorSelection::new(CursorPos::new(0, 0));
    assert_eq!(multi.count(), 1);
    multi.add_cursor(CursorPos::new(1, 5));
    multi.add_cursor(CursorPos::new(2, 10));
    assert_eq!(multi.count(), 3);
    assert_eq!(multi.primary().head, CursorPos::new(0, 0));

    // Overlapping additions auto-merge
    multi.add_cursor(CursorPos::new(1, 5));
    assert_eq!(multi.count(), 3);

    multi.clear_secondary();
    assert_eq!(multi.count(), 1);
    assert_eq!(multi.primary().head, CursorPos::new(0, 0));

    // 2. Typing burst coalescing
    let mut buf = TextBuffer::new();
    let mut undo_stack = UndoStack::new(50);

    // Simulate typing "Hello" char-by-char with push_coalesced
    for (i, ch) in "Hello".chars().enumerate() {
        let pos = CursorPos::new(0, i);
        let text = ch.to_string();
        buf.insert_str(pos, &text);
        undo_stack.push_coalesced(EditAction::Insert { pos, text });
    }
    assert_eq!(buf.text(), "Hello");

    // Single undo step should undo the whole continuous "Hello" word burst!
    let restored = undo_stack.undo(&mut buf);
    assert_eq!(restored, Some(CursorPos::new(0, 0)));
    assert_eq!(buf.text(), "");

    // Redo brings back "Hello"
    undo_stack.redo(&mut buf);
    assert_eq!(buf.text(), "Hello");

    // 3. Multi-cursor atomic transaction
    undo_stack.begin_transaction();
    // Simulate 2 cursors deleting at two positions in one transaction
    buf.insert_str(CursorPos::new(0, 5), " World");
    undo_stack.push(EditAction::Insert {
        pos: CursorPos::new(0, 5),
        text: " World".to_string(),
    });
    undo_stack.push(EditAction::Insert {
        pos: CursorPos::new(0, 11),
        text: "!".to_string(),
    });
    buf.insert_str(CursorPos::new(0, 11), "!");
    undo_stack.commit_transaction();

    assert_eq!(buf.text(), "Hello World!");

    // Single undo reverses both edits of the transaction atomically!
    undo_stack.undo(&mut buf);
    assert_eq!(buf.text(), "Hello");

    undo_stack.redo(&mut buf);
    assert_eq!(buf.text(), "Hello World!");
}

#[test]
fn test_datagrid_multi_column_sorting_and_resizing() {
    use std::cmp::Ordering;
    use tej_ui_widgets::{ColumnResizeSession, InlineCellEditor, MultiColumnSorter, SortDirection};

    // 1. Multi-column sorting
    let mut sorter = MultiColumnSorter::new();
    // Normal click sorts by 'department'
    sorter.toggle_sort("department", false);
    assert_eq!(sorter.direction_for("department"), SortDirection::Ascending);
    assert_eq!(sorter.criteria().len(), 1);

    // Shift-click adds 'salary' as secondary sort criterion
    sorter.toggle_sort("salary", true);
    assert_eq!(sorter.criteria().len(), 2);
    assert_eq!(sorter.direction_for("salary"), SortDirection::Ascending);

    // Shift-click 'salary' again to cycle to Descending
    sorter.toggle_sort("salary", true);
    assert_eq!(sorter.direction_for("salary"), SortDirection::Descending);

    // Test row comparison
    #[derive(Debug)]
    struct Employee {
        dept: &'static str,
        salary: u64,
    }
    let emp1 = Employee {
        dept: "Engineering",
        salary: 120_000,
    };
    let emp2 = Employee {
        dept: "Engineering",
        salary: 150_000,
    };

    let ord = sorter.compare_records(&emp1, &emp2, |col, a, b| match col {
        "department" => a.dept.cmp(b.dept),
        "salary" => a.salary.cmp(&b.salary),
        _ => Ordering::Equal,
    });
    // Since department is equal and salary is descending: 120k vs 150k inverted -> Greater
    assert_eq!(ord, Ordering::Greater);

    // 2. Column resizing session
    let session = ColumnResizeSession {
        column_index: 0,
        start_pointer_x: 200.0,
        initial_width: 150.0,
    };
    // Dragged right +40px
    let new_width = session.compute_new_width(240.0, 60.0);
    assert_eq!(new_width, 190.0);

    // Dragged left past min_width (60.0) -> clamped
    let clamped_width = session.compute_new_width(50.0, 60.0);
    assert_eq!(clamped_width, 60.0);

    // 3. Inline cell editor lifecycle
    let mut editor = InlineCellEditor::new();
    assert!(!editor.is_active);

    editor.begin_edit(5, 2, "Original Text");
    assert!(editor.is_active);
    assert_eq!(editor.draft_value, "Original Text");

    editor.draft_value = "Updated Text".to_string();
    let committed = editor.commit();
    assert_eq!(committed, Some((5, 2, "Updated Text".to_string())));
    assert!(!editor.is_active);
}

#[test]
fn test_tree_virtualization_and_split_pane() {
    use tej_ui_core::Signal;
    use tej_ui_widgets::{
        SplitDividerDrag, SplitOrientation, SplitPaneController, TreeMultiSelection, TreeNode,
        VirtualizedTreeIndex,
    };

    // 1. Virtualized Tree Index
    let tree = vec![
        TreeNode::branch(
            "src",
            "src",
            vec![
                TreeNode::leaf("main_rs", "main.rs"),
                TreeNode::branch(
                    "components",
                    "components",
                    vec![TreeNode::leaf("btn_rs", "button.rs")],
                ),
            ],
        ),
        TreeNode::leaf("cargo_toml", "Cargo.toml"),
    ];

    let v_index = VirtualizedTreeIndex::from_roots(&tree);
    assert_eq!(v_index.total_visible_rows(), 5);

    let row1 = v_index.get(1).unwrap();
    assert_eq!(row1.id, "main_rs");
    assert_eq!(row1.depth, 1);
    assert!(!row1.has_children);

    let row2 = v_index.get(2).unwrap();
    assert_eq!(row2.id, "components");
    assert_eq!(row2.depth, 1);
    assert!(row2.has_children);

    // Visible range window calculation: row_height 20px, scroll_y 20px, viewport 40px
    let range = v_index.compute_visible_range(20.0, 20.0, 40.0, 0);
    assert_eq!(range, 1..3);

    // 2. Tree Multi-selection
    let mut multi = TreeMultiSelection::new();
    assert_eq!(multi.count(), 0);

    // Single click selects node
    multi.select("src", false);
    assert!(multi.is_selected("src"));
    assert_eq!(multi.count(), 1);

    // Single click replaces selection
    multi.select("cargo_toml", false);
    assert!(!multi.is_selected("src"));
    assert!(multi.is_selected("cargo_toml"));
    assert_eq!(multi.count(), 1);

    // Toggle click adds to selection
    multi.select("src", true);
    assert!(multi.is_selected("cargo_toml"));
    assert!(multi.is_selected("src"));
    assert_eq!(multi.count(), 2);

    // 3. SplitPane controller & divider drag
    let ratio = Signal::new(0.3f32);
    let split = SplitPaneController::new(ratio)
        .orientation(SplitOrientation::Horizontal)
        .limits(0.2, 0.8);

    assert_eq!(split.split_ratio.get(), 0.3);

    // Simulate divider drag
    let drag = SplitDividerDrag {
        start_coord: 300.0,
        start_ratio: 0.3,
        total_length: 1000.0,
    };
    // Dragged +200px -> ratio becomes 0.3 + 0.2 = 0.5
    let new_ratio = drag.compute_ratio(500.0, split.min_ratio, split.max_ratio);
    assert!((new_ratio - 0.5).abs() < 1e-4);
    split.set_ratio(new_ratio);
    assert_eq!(split.split_ratio.get(), 0.5);

    // Panel collapse
    assert!(!split.is_collapsed.get());
    split.toggle_collapsed();
    assert!(split.is_collapsed.get());
}

#[test]
fn test_color_and_calendar_pickers() {
    use tej_ui_widgets::{CalendarDate, CalendarPickerState, ColorHsv, DateRange};

    // 1. Color HSV <-> RGB <-> HEX
    let red_hsv = ColorHsv::new(0.0, 1.0, 1.0, 1.0);
    assert_eq!(red_hsv.to_rgba8(), (255, 0, 0, 255));
    assert_eq!(red_hsv.to_hex_string(), "#FF0000");

    let green_hsv = ColorHsv::new(120.0, 1.0, 1.0, 1.0);
    assert_eq!(green_hsv.to_rgba8(), (0, 255, 0, 255));
    assert_eq!(green_hsv.to_hex_string(), "#00FF00");

    let parsed_blue = ColorHsv::from_hex_str("#0000FF").unwrap();
    assert_eq!(parsed_blue.to_rgba8(), (0, 0, 255, 255));

    // Semi-transparent hex
    let parsed_alpha = ColorHsv::from_hex_str("#FF000080").unwrap();
    assert_eq!(parsed_alpha.a, 128.0 / 255.0);

    // 2. Calendar Date and Leap Year
    assert!(CalendarDate::is_leap_year(2024));
    assert!(!CalendarDate::is_leap_year(2023));
    assert_eq!(CalendarDate::days_in_month(2024, 2), 29);
    assert_eq!(CalendarDate::days_in_month(2023, 2), 28);

    let d = CalendarDate::new(2026, 11, 15);
    assert_eq!(d.to_iso_string(), "2026-11-15");

    // 3. Calendar Range Picker
    let mut picker = CalendarPickerState::new(2026, 11);
    assert_eq!(picker.view_year, 2026);
    assert_eq!(picker.view_month, 11);

    picker.next_month();
    assert_eq!(picker.view_month, 12);
    picker.next_month();
    assert_eq!(picker.view_year, 2027);
    assert_eq!(picker.view_month, 1);

    picker.prev_month();
    assert_eq!(picker.view_year, 2026);
    assert_eq!(picker.view_month, 12);

    // Select date range
    let start = CalendarDate::new(2026, 12, 10);
    let end = CalendarDate::new(2026, 12, 20);
    picker.select_date(start);
    picker.select_date(end);

    let expected_range = DateRange::new(start, end);
    assert_eq!(picker.range, Some(expected_range));

    assert!(expected_range.contains(CalendarDate::new(2026, 12, 15)));
    assert!(!expected_range.contains(CalendarDate::new(2026, 12, 5)));
}

#[test]
fn test_ime_pipeline_and_dead_keys() {
    use tej_ui_layout::{Point, Rect, Size};
    use tej_ui_widgets::editor::{
        DeadKeyResolver, PreeditUnderline, compute_candidate_window_position,
    };

    // 1. Candidate window position computation
    let cursor = Rect::new(100.0, 50.0, 2.0, 16.0);
    let candidate_size = Size::new(200.0, 150.0);
    let screen = Rect::new(0.0, 0.0, 1920.0, 1080.0);

    let pos = compute_candidate_window_position(cursor, candidate_size, screen);
    assert_eq!(pos, Point::new(100.0, 70.0)); // 50 + 16 + 4

    // Screen bottom overflow flips window above cursor
    let bottom_cursor = Rect::new(100.0, 1000.0, 2.0, 16.0);
    let flipped_pos = compute_candidate_window_position(bottom_cursor, candidate_size, screen);
    assert!(flipped_pos.y < 1000.0);

    // 2. Dead key resolver for accents
    let mut resolver = DeadKeyResolver::new();
    assert!(!resolver.has_pending_dead_key());

    // Press '~' (tilde dead key) -> returns None and sets pending
    assert_eq!(resolver.feed_char('~'), None);
    assert!(resolver.has_pending_dead_key());

    // Press 'a' -> resolves to 'ã'
    assert_eq!(resolver.feed_char('a'), Some('ã'));
    assert!(!resolver.has_pending_dead_key());

    // Press '^' followed by 'e' -> 'ê'
    assert_eq!(resolver.feed_char('^'), None);
    assert_eq!(resolver.feed_char('e'), Some('ê'));

    // Press '\'' followed by 'c' -> 'ç'
    assert_eq!(resolver.feed_char('\''), None);
    assert_eq!(resolver.feed_char('c'), Some('ç'));

    // Normal characters pass straight through
    assert_eq!(resolver.feed_char('x'), Some('x'));

    // Underline styles
    assert_ne!(PreeditUnderline::Thick, PreeditUnderline::Squiggly);
}
