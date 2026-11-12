use mydb::Database;
use mydb::index::SecondaryIndex;
use std::sync::Arc;
use tej_core::Signal;
use tej_db_sync::{
    DbSyncHook, ReactiveDbQuery, ReactiveSecondaryQuery, VirtualList, ZeroCopyRow,
    optimistic_transaction, prefix_to_range,
};
use tej_view::{Text, View};

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
    use mydb_core::Hook;
    hook.after_commit(&changes);

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
