//! `tej-db-sync`: Zero-copy database-driven UI state integration.
//!
//! Connects the `mydb` ACID storage engine directly to the reactive core via WAL/commit
//! hooks, memory-mapped page-slice zero-copy rows, prefix-bounded range scans,
//! secondary index acceleration, and optimistic transaction rollbacks.

pub mod cursor;
pub mod hook;
pub mod index_query;
pub mod optimistic;
pub mod query;
pub mod virtual_list;

pub use cursor::ZeroCopyRow;
pub use hook::DbSyncHook;
pub use index_query::ReactiveSecondaryQuery;
pub use optimistic::optimistic_transaction;
pub use query::{ReactiveDbQuery, prefix_to_range};
pub use virtual_list::{VirtualList, VirtualListState};
