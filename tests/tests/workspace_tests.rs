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
