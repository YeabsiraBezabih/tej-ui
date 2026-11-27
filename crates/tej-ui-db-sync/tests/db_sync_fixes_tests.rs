//! Comprehensive unit tests for `tej-ui-db-sync` overflow, concurrency, snapshot, and NaN fixes.

use mydb::Database;
use mydb::index::SecondaryIndex;
use mydb_core::{ChangeSet, Hook};
use std::sync::Arc;
use tej_ui_core::{Signal, create_effect};
use tej_ui_db_sync::{DbRowLens, DbSyncHook, ReactiveSecondaryQuery, RleColumn, VirtualList};
use tej_ui_view::Text;

#[test]
fn test_mmap_lens_checked_add_prevents_overflow() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&42u64.to_le_bytes());
    payload.extend_from_slice(&3.5f32.to_le_bytes());
    payload.extend_from_slice(b"hello");

    let lens = DbRowLens::new(b"k1", &payload);

    // Normal reads succeed
    assert_eq!(lens.read_u64_le(0), Some(42));
    assert_eq!(lens.read_f32_le(8), Some(3.5));
    assert_eq!(lens.slice_str(12, 5).unwrap(), "hello");

    // Extreme offsets near usize::MAX must return None / empty slice rather than wrap or panic
    assert_eq!(lens.read_u64_le(usize::MAX), None);
    assert_eq!(lens.read_u64_le(usize::MAX - 3), None);
    assert_eq!(lens.read_f32_le(usize::MAX), None);
    assert_eq!(lens.read_f32_le(usize::MAX - 1), None);
    assert_eq!(lens.slice_str(usize::MAX, 10).unwrap(), "");
    assert_eq!(lens.slice_str(12, usize::MAX).unwrap(), "hello");
}

#[test]
fn test_rle_column_push_run_splits_on_u32_max_overflow() {
    let mut col = RleColumn::new();
    col.push_run(true, u32::MAX - 2);
    col.push_run(true, 10);

    // Must split into two runs: u32::MAX and 8, with exact total_elements
    assert_eq!(col.runs_count(), 2);
    assert_eq!(col.runs()[0].count, u32::MAX);
    assert_eq!(col.runs()[1].count, 8);
    assert_eq!(col.total_elements(), (u32::MAX as usize) + 8);
}

#[test]
fn test_db_sync_hook_no_deadlock_on_reentrant_subscribe_and_unsubscribe() {
    let hook = Arc::new(DbSyncHook::new());
    let sig = Signal::new(0u64);
    let secondary_sig = Signal::new(0u64);

    hook.subscribe_prefix(b"users:".to_vec(), sig);

    // Register a reactive effect that mutates hook subscriptions while handling after_commit.
    // Previously this would deadlock because after_commit held the RwLock read guard.
    let hook_clone = Arc::clone(&hook);
    let _effect = create_effect(move || {
        let ver = sig.get();
        if ver == 1 {
            hook_clone.subscribe_prefix(b"audit:".to_vec(), secondary_sig);
        }
    });

    let changes = ChangeSet {
        writes: vec![(b"users:42".to_vec(), b"Alice".to_vec())],
        deletes: vec![],
    };
    hook.after_commit(&changes);
    assert_eq!(sig.get(), 1);

    // Unsubscribe stops further notifications
    assert!(hook.unsubscribe_prefix(b"users:", sig));
    assert!(!hook.unsubscribe_prefix(b"users:", sig));

    hook.after_commit(&changes);
    assert_eq!(sig.get(), 1);
}

#[test]
fn test_reactive_secondary_query_single_transaction_scan() {
    let db = Arc::new(Database::open_memory().unwrap());
    let hook = Arc::new(DbSyncHook::new());
    let sec_idx = SecondaryIndex::new("role_idx");

    let mut tx = db.begin_write().unwrap();
    tx.put(b"user:1", b"Admin").unwrap();
    tx.put(b"user:2", b"Editor").unwrap();
    sec_idx.insert_entry(&mut tx, b"admin", b"user:1").unwrap();
    tx.commit().unwrap();

    let query = ReactiveSecondaryQuery::new(Arc::clone(&db), &hook, sec_idx);
    let mut rows = Vec::new();
    query
        .scan_matching_records(b"admin", None, |row| {
            rows.push((
                row.key_str().unwrap().to_string(),
                row.value_str().unwrap().to_string(),
            ));
            true
        })
        .unwrap();

    assert_eq!(rows, vec![("user:1".to_string(), "Admin".to_string())]);
}

#[test]
fn test_virtual_list_guards_invalid_item_height() {
    let scroll = Signal::new(100.0f32);

    let zero_list = VirtualList::new(100, 0.0, 500.0, scroll, |i| Text::new(format!("{i}")));
    assert_eq!(zero_list.visible_range(), (0, 0));

    let neg_list = VirtualList::new(100, -15.0, 500.0, scroll, |i| Text::new(format!("{i}")));
    assert_eq!(neg_list.visible_range(), (0, 0));

    let nan_list = VirtualList::new(100, f32::NAN, 500.0, scroll, |i| Text::new(format!("{i}")));
    assert_eq!(nan_list.visible_range(), (0, 0));
}
