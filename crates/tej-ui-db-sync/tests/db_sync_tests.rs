use mydb::Database;
use mydb::index::SecondaryIndex;
use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_db_sync::{
    DbSyncHook, ReactiveDbQuery, ReactiveSecondaryQuery, VirtualList, ZeroCopyRow,
    optimistic_transaction, prefix_to_range,
};
use tej_ui_view::{Text, View};

#[test]
fn test_zero_copy_row_slicing() {
    let key = b"task:00000001";
    let val = b"Implement tej-ui reactive core";

    let row = ZeroCopyRow::new(key, val);
    assert_eq!(row.key_str().unwrap(), "task:00000001");
    assert_eq!(row.value_str().unwrap(), "Implement tej-ui reactive core");

    let num_key = 42u64.to_be_bytes();
    let num_row = ZeroCopyRow::new(&num_key, val);
    assert_eq!(num_row.id_u64(), Some(42));

    let short_row = ZeroCopyRow::new(b"short", val);
    assert_eq!(short_row.id_u64(), None);
}

#[test]
fn test_db_commit_hook_notification() {
    let db = Arc::new(Database::open_memory().unwrap());
    let hook = Arc::new(DbSyncHook::new());
    db.add_hook(Box::new(DbSyncHook::new())); // test trait registration

    let signal = Signal::new(0u64);
    hook.subscribe_prefix(b"tasks:".to_vec(), signal);

    assert_eq!(signal.get(), 0);

    // Simulate committing mutations matching prefix
    let changes = mydb_core::ChangeSet {
        writes: vec![(b"tasks:1".to_vec(), b"New Task".to_vec())],
        deletes: vec![],
    };
    mydb_core::Hook::after_commit(&*hook, &changes);

    assert_eq!(signal.get(), 1);
}

#[test]
fn test_virtual_list_viewport_bounds() {
    // 1,000,000 items, each 30px high, 600px viewport
    let scroll_pos = Signal::new(300.0); // Scrolled down 10 items
    let list = VirtualList::new(1_000_000, 30.0, 600.0, scroll_pos, |idx| {
        Text::new(format!("Task #{idx}"))
    });

    let (start, end) = list.visible_range();
    assert_eq!(start, 10);
    assert_eq!(end, 32); // 10 + 20 visible + 2 buffer = 32

    let state = list.build();
    // Only 22 visible rows materialized instead of 1,000,000!
    assert_eq!(state.visible_states.len(), 22);
}

#[test]
fn test_prefix_to_range_bounds() {
    use std::ops::Bound;

    let range_all = prefix_to_range(b"");
    assert_eq!(range_all.start, Bound::Unbounded);
    assert_eq!(range_all.end, Bound::Unbounded);

    let prefix = b"user:";
    let range = prefix_to_range(prefix);
    assert_eq!(range.start, Bound::Included(b"user:".to_vec()));
    let mut expected_end = b"user:".to_vec();
    *expected_end.last_mut().unwrap() += 1;
    assert_eq!(range.end, Bound::Excluded(expected_end));
}

#[test]
fn test_optimistic_transaction_and_rollback() {
    let signal = Signal::new(10);

    // 1. Successful transaction preserves optimistic update
    let res = optimistic_transaction(
        signal,
        |val| *val = 20,
        || Ok(()), // successful DB commit
    );
    assert!(res.is_ok());
    assert_eq!(signal.get(), 20);

    // 2. Failed transaction rolls back to previous state
    let res_err = optimistic_transaction(
        signal,
        |val| *val = 99,
        || Err(mydb_core::DbError::Concurrency("OCC conflict".into())),
    );
    assert!(res_err.is_err());
    assert_eq!(signal.get(), 20); // restored back to 20!
}

#[test]
fn test_reactive_secondary_query_and_prefix_scan() {
    let db = Arc::new(Database::open_memory().unwrap());
    let hook = Arc::new(DbSyncHook::new());

    // Populate data
    let mut tx = db.begin_write().unwrap();
    tx.put(b"item:1", b"apple").unwrap();
    tx.put(b"item:2", b"banana").unwrap();
    tx.put(b"item:3", b"apricot").unwrap();
    tx.put(b"other:1", b"ignore").unwrap();

    let sec_idx = SecondaryIndex::new("fruit_type");
    sec_idx.insert_entry(&mut tx, b"citrus", b"item:1").unwrap();
    tx.commit().unwrap();

    // 1. Test prefix-bounded scan via ReactiveDbQuery
    let query = ReactiveDbQuery::new(Arc::clone(&db), &hook, b"item:".to_vec());
    let mut items = Vec::new();
    query
        .scan_zero_copy(|row| {
            items.push((
                row.key_str().unwrap().to_string(),
                row.value_str().unwrap().to_string(),
            ));
            true
        })
        .unwrap();

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].0, "item:1");
    assert_eq!(items[1].0, "item:2");
    assert_eq!(items[2].0, "item:3");

    // 2. Test secondary index query
    let sec_query = ReactiveSecondaryQuery::new(Arc::clone(&db), &hook, sec_idx);
    let pks = sec_query.lookup_primary_keys(b"citrus").unwrap();
    assert_eq!(pks.len(), 1);
    assert_eq!(pks[0], b"item:1");
}

#[test]
fn test_offline_sync_queue_and_conflict_resolution() {
    use tej_ui_db_sync::{ConflictStrategy, OfflineSyncQueue, SyncStatus};

    // 1. ClientWins strategy
    let queue = OfflineSyncQueue::new(ConflictStrategy::ClientWins);
    let signal = Signal::new("Original".to_string());

    let _id = queue.mutate_optimistic("doc:1", "Client Edit".to_string(), signal);
    assert_eq!(signal.get(), "Client Edit");
    assert_eq!(queue.pending_count(), 1);

    // Successful sync
    let status = queue.process_next(|key, val| {
        assert_eq!(key, "doc:1");
        assert_eq!(val, "Client Edit");
        Ok("Server Accepted".to_string())
    });
    assert_eq!(status, Some(SyncStatus::Synced));
    assert_eq!(signal.get(), "Server Accepted");
    assert_eq!(queue.pending_count(), 0);

    // 2. ServerWins strategy on conflict
    let queue_sw = OfflineSyncQueue::new(ConflictStrategy::ServerWins);
    let sig2 = Signal::new("Initial".to_string());
    queue_sw.mutate_optimistic("doc:2", "My Edit".to_string(), sig2);
    assert_eq!(sig2.get(), "My Edit");

    // Simulate conflict: server sends back conflicting state
    let status_conflict =
        queue_sw.process_next(|_key, _val| Err(Some("Server Authority Version".to_string())));
    assert_eq!(status_conflict, Some(SyncStatus::Conflict));
    // Reverted to server authoritative value!
    assert_eq!(sig2.get(), "Server Authority Version");
}

#[test]
fn test_db_mmap_row_lens() {
    use tej_ui_db_sync::{DbRowLens, DbVirtualWindow};

    let key = b"order:00000042";
    let mut payload = Vec::new();
    payload.extend_from_slice(b"ITEM:Widget-Pro\0");
    payload.extend_from_slice(&1_234_567_890_u64.to_le_bytes());
    payload.extend_from_slice(&99.95f32.to_le_bytes());

    let lens = DbRowLens::new(key, &payload);
    assert_eq!(lens.key_str().unwrap(), "order:00000042");
    assert_eq!(lens.slice_str(0, 15).unwrap(), "ITEM:Widget-Pro");
    assert_eq!(lens.read_u64_le(16).unwrap(), 1_234_567_890);
    assert!((lens.read_f32_le(24).unwrap() - 99.95).abs() < 1e-4);

    // Test virtual window
    let mut window = DbVirtualWindow::new(50, 20.0, 500.0);
    window.update_scroll(100.0, 25.0, 500.0);
    assert_eq!(window.start_index(), 4);
    assert!(window.end_index() > 20);
    assert!(window.is_visible(10));
    assert!(!window.is_visible(0));
}

#[test]
fn test_direct_rle_filtering() {
    use tej_ui_db_sync::{DirectRleFilter, RleColumn};

    let mut col = RleColumn::new();
    // 50,000 active, 50,000 archived, 20,000 pending
    col.push_run(1, 50_000);
    col.push_run(2, 50_000);
    col.push_run(3, 20_000);
    assert_eq!(col.total_rows(), 120_000);
    assert_eq!(col.runs().len(), 3);

    // Filter directly on RLE runs without decompressing 120,000 rows
    let bitset = DirectRleFilter::filter_equals(&col, 1);
    assert_eq!(bitset.total_set(), 50_000);
    assert!(bitset.get(0));
    assert!(bitset.get(49_999));
    assert!(!bitset.get(50_000));
    assert!(!bitset.get(100_000));

    // Filter range: status in [1..=2]
    let bitset_range = DirectRleFilter::filter_range(&col, 1, 2);
    assert_eq!(bitset_range.total_set(), 100_000);
    assert!(bitset_range.get(0));
    assert!(bitset_range.get(99_999));
    assert!(!bitset_range.get(100_000));
}

#[test]
fn test_rabitq_semantic_palette() {
    use tej_ui_db_sync::SemanticPalette;

    let mut palette = SemanticPalette::new();
    palette.register(
        1,
        "Create New File",
        0b1111_0000_1111_0000_1111_0000_1111_0000_u64,
    );
    palette.register(
        2,
        "Close Active Tab",
        0b1111_0000_1111_0000_0000_0000_0000_0000_u64,
    );
    palette.register(
        3,
        "Toggle Terminal",
        0b0000_1111_0000_1111_0000_1111_0000_1111_u64,
    );

    // Query closest to "Create New File"
    let query_sig = 0b1111_0000_1111_0000_1111_0000_1111_1111_u64;
    let results = palette.search(query_sig, 2);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].item.id, 1); // Distance is 4 bits
    assert_eq!(results[0].distance, 4);
    assert_eq!(results[0].item.title, "Create New File");
}

#[test]
fn test_cow_time_travel_undo_stack() {
    use tej_ui_db_sync::CowTimeTravelStack;

    let mut stack = CowTimeTravelStack::new(100);
    assert_eq!(stack.current_root(), 100);
    assert!(!stack.can_undo());
    assert!(!stack.can_redo());

    stack.record_commit(101);
    stack.record_commit(102);
    assert_eq!(stack.current_root(), 102);
    assert_eq!(stack.counts(), (2, 0));
    assert!(stack.can_undo());

    // Undo to 101
    assert_eq!(stack.undo(), Some(101));
    assert_eq!(stack.current_root(), 101);
    assert!(stack.can_redo());

    // Undo to 100
    assert_eq!(stack.undo(), Some(100));
    assert_eq!(stack.current_root(), 100);
    assert!(!stack.can_undo());

    // Redo to 101
    assert_eq!(stack.redo(), Some(101));
    assert_eq!(stack.current_root(), 101);

    // New commit clears future
    stack.record_commit(105);
    assert!(!stack.can_redo());
    assert_eq!(stack.current_root(), 105);

    // Memory footprint is tiny: under 1 KB!
    assert!(stack.heap_memory_bytes() < 1024);
}
