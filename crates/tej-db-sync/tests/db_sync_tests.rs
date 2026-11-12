use mydb::Database;
use std::sync::Arc;
use tej_core::Signal;
use tej_db_sync::{DbSyncHook, VirtualList, ZeroCopyRow};
use tej_view::{Text, View};

#[test]
fn test_zero_copy_row_slicing() {
    let key = b"task:00000001";
    let val = b"Implement tej-ui reactive core";

    let row = ZeroCopyRow::new(key, val);
    assert_eq!(row.key_str().unwrap(), "task:00000001");
    assert_eq!(row.value_str().unwrap(), "Implement tej-ui reactive core");
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
