//! `tej-db-sync`: Zero-copy database-driven UI state integration.
//!
//! Connects the `mydb` ACID storage engine directly to the reactive core via WAL/commit
//! hooks, memory-mapped page-slice zero-copy rows, prefix-bounded range scans,
//! secondary index acceleration, optimistic transaction rollbacks, direct RLE filtering,
//! RaBitQ semantic palettes, and copy-on-write snapshot time-travel.

pub mod columnar_filter;
pub mod cursor;
pub mod hook;
pub mod index_query;
pub mod mmap_lens;
pub mod optimistic;
pub mod query;
pub mod semantic_palette;
pub mod sync_queue;
pub mod time_travel;
pub mod virtual_list;

pub use columnar_filter::{DirectRleFilter, FilterBitset, RleColumn, RleRun};
pub use cursor::ZeroCopyRow;
pub use hook::DbSyncHook;
pub use index_query::ReactiveSecondaryQuery;
pub use mmap_lens::{DbRowLens, DbVirtualWindow};
pub use optimistic::optimistic_transaction;
pub use query::{ReactiveDbQuery, prefix_to_range};
pub use semantic_palette::{PaletteItem, PaletteSearchResult, SemanticPalette};
pub use sync_queue::{ConflictStrategy, MergeFn, OfflineSyncQueue, PendingMutation, SyncStatus};
pub use time_travel::CowTimeTravelStack;
pub use virtual_list::{VirtualList, VirtualListState};
