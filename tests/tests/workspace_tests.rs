use mydb::Database;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tej::prelude::*;

#[test]
fn test_end_to_end_reactivity_and_view_lifecycle() {
    let count = Signal::new(0);
    let double_count = Memo::new(move || count.get() * 2);

    let effect_runs = Arc::new(AtomicUsize::new(0));
    let effect_clone = Arc::clone(&effect_runs);
    let _effect = create_effect(move || {
        let _ = double_count.get();
        effect_clone.fetch_add(1, Ordering::SeqCst);
    });

    assert_eq!(double_count.get(), 0);
    assert_eq!(effect_runs.load(Ordering::SeqCst), 1);

    // Build static view tree
    let view = VStack::new((
        Text::new("Counter App"),
        HStack::new((
            Button::new(Text::new("Add")).on_click(move || count.update(|c| *c += 1)),
            Checkbox::new(Signal::new(true)),
        ))
        .spacing(8),
    ))
    .padding(16);

    let mut state = view.build();
    assert_eq!(state.padding, 16);

    // Update signal
    count.set(5);
    assert_eq!(double_count.get(), 10);
    assert_eq!(effect_runs.load(Ordering::SeqCst), 2);

    // Patch view with dirty signal
    view.patch(&mut state, 1);
}

#[test]
fn test_database_sync_and_virtual_list_integration() {
    let db = Arc::new(Database::open_memory().unwrap());
    let hook = Arc::new(DbSyncHook::new());

    let mut tx = db.begin_write().unwrap();
    tx.put(b"item:1", b"First Item").unwrap();
    tx.put(b"item:2", b"Second Item").unwrap();
    tx.commit().unwrap();

    let query = ReactiveDbQuery::new(Arc::clone(&db), &hook, b"item:".to_vec());
    let mut collected = Vec::new();
    query
        .scan_zero_copy(|row| {
            collected.push((
                row.key_str().unwrap().to_string(),
                row.value_str().unwrap().to_string(),
            ));
            true
        })
        .unwrap();

    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0].1, "First Item");
    assert_eq!(collected[1].1, "Second Item");

    // Virtual list over 500,000 items
    let scroll_pos = Signal::new(0.0);
    let vlist = VirtualList::new(500_000, 40.0, 800.0, scroll_pos, |idx| {
        Text::new(format!("Virtual Row #{idx}"))
    });

    let state = vlist.build();
    assert!(state.visible_states.len() <= 25);
}

#[test]
fn test_flat_soa_layout_and_dirty_clip_pipeline() {
    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(1024.0),
        height: Dimension::Points(768.0),
        direction: FlexDirection::Column,
        padding: Insets::all(24.0),
        spacing: 12.0,
        ..Default::default()
    });

    let top_bar = arena.allocate_node(LayoutStyle {
        height: Dimension::Points(60.0),
        ..Default::default()
    });
    let content_area = arena.allocate_node(LayoutStyle {
        height: Dimension::Points(600.0),
        ..Default::default()
    });

    arena.add_child(root, top_bar);
    arena.add_child(root, content_area);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(1024.0, 768.0));

    let top_bounds = arena.computed_bounds[top_bar as usize];
    assert_eq!(top_bounds.origin.x, 24.0);
    assert_eq!(top_bounds.origin.y, 24.0);
    assert_eq!(top_bounds.size.width, 976.0); // 1024 - 48
    assert_eq!(top_bounds.size.height, 60.0);

    // Verify dirty rect tracker accumulates bounds
    let mut tracker = DirtyRectTracker::new();
    tracker.add_rect(top_bounds);
    assert!(tracker.has_dirty());
    assert_eq!(tracker.rects().len(), 1);
}

#[test]
fn test_enterprise_datagrid_and_command_palette() {
    // 1. DataGrid test with 100,000 cells
    let columns = vec![
        GridColumn::new("id", "Task ID")
            .width(80.0)
            .pin(PinState::Left),
        GridColumn::new("title", "Title").width(300.0),
        GridColumn::new("assignee", "Assignee").width(150.0),
    ];

    let grid = DataGrid::new(
        columns,
        DataGridDimensions {
            total_rows: 100_000,
            row_height: 32.0,
            viewport_width: 1000.0,
            viewport_height: 640.0,
        },
        Signal::new(0.0),
        Signal::new(320.0),
        |coords| Text::new(format!("Cell ({}, {})", coords.row, coords.col)),
    );

    let state = grid.build();
    assert_eq!(state.start_row, 10);
    assert!(state.end_row <= 35);
    assert!(state.visible_cols.contains(&0)); // Pinned left column

    // 2. Command Palette fuzzy match
    let query = Signal::new(String::from("deploy"));
    let palette = CommandPalette::new(
        query,
        Signal::new(0),
        vec![
            CommandItem::new("d1", "Deploy Production", "Release").shortcut("Cmd+Shift+D"),
            CommandItem::new("d2", "Develop Locally", "Environment"),
        ],
    );
    let p_state = palette.build();
    assert_eq!(p_state.visible_items.len(), 2);
    assert_eq!(p_state.visible_items[0].title, "Deploy Production");
}

#[test]
fn test_enterprise_devtools_and_i18n_pipeline() {
    // 1. i18n BiDi classification and font fallback
    let mixed_title = "Project مشروع Alpha";
    let classifier = BidiClassifier::new();
    let runs = classifier.segment(mixed_title);
    assert_eq!(runs.len(), 3);
    assert_eq!(runs[1].direction, TextDirection::RightToLeft);

    let fallback = FontFallbackChain::default();
    assert_eq!(
        FontFallbackChain::categorize_char('م'),
        FontCategory::ComplexScript
    );
    assert_eq!(fallback.font_for_char('م'), &fallback.rtl_fallback);

    // 2. DevTools HotReloadHost with preserved Signal state
    let active_tab = Signal::new(1);
    let view = Text::new("Initial View");
    let mut host = HotReloadHost::new(view);
    assert_eq!(host.generation(), 1);

    active_tab.set(2);
    assert_eq!(active_tab.get(), 2);

    let reload_duration = host.swap_view(Text::new("Hot-Reloaded View"));
    assert!(reload_duration >= 0.0);
    assert_eq!(host.generation(), 2);
    assert_eq!(active_tab.get(), 2); // Reactive state intact

    // 3. Multi-monitor scale factor handling
    let mut manager = MonitorManager::default();
    manager.add_monitor(MonitorInfo::new(
        MonitorId(2),
        "4K External Display",
        Rect::new(1920.0, 0.0, 3840.0, 2160.0),
        2.0,
        false,
    ));
    let external = manager.monitor_at_point(Point::new(2000.0, 100.0)).unwrap();
    assert_eq!(external.scale_factor, 2.0);
}
