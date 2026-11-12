//! `tej-db-sync`: Zero-copy database-driven UI state integration.
//!
//! Connects the `mydb` ACID storage engine directly to the reactive core via WAL/commit
//! hooks, memory-mapped page-slice zero-copy rows, and virtualized list viewport cursors.

pub mod cursor;
pub mod hook;
pub mod query;
pub mod virtual_list;

pub use cursor::ZeroCopyRow;
pub use hook::DbSyncHook;
pub use query::ReactiveDbQuery;
pub use virtual_list::{VirtualList, VirtualListState};
