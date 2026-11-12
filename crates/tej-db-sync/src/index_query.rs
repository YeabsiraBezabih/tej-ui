//! Reactive secondary index queries integrating with `mydb::index::SecondaryIndex`.

use crate::cursor::ZeroCopyRow;
use crate::hook::DbSyncHook;
use mydb::Database;
use mydb::index::SecondaryIndex;
use mydb_core::{DbError, Storage};
use std::sync::Arc;
use tej_core::Signal;

/// Reactive query handle powered by a `mydb` secondary B+ Tree index.
#[derive(Clone, Debug)]
pub struct ReactiveSecondaryQuery<S: Storage> {
    db: Arc<Database<S>>,
    index: SecondaryIndex,
    version_signal: Signal<u64>,
}

impl<S: Storage> ReactiveSecondaryQuery<S> {
    /// Create a new reactive secondary index query observer.
    pub fn new(db: Arc<Database<S>>, hook: &DbSyncHook, index: SecondaryIndex) -> Self {
        let version_signal = Signal::new(0);
        // Subscribe to changes under the secondary index prefix table
        let prefix = format!("__idx:{}", index.name()).into_bytes();
        hook.subscribe_prefix(prefix, version_signal);

        Self {
            db,
            index,
            version_signal,
        }
    }

    /// Reactive version signal that ticks whenever indexed entries mutate.
    #[must_use]
    pub const fn version_signal(&self) -> Signal<u64> {
        self.version_signal
    }

    /// Secondary index reference.
    #[must_use]
    pub const fn index(&self) -> &SecondaryIndex {
        &self.index
    }

    /// Lookup primary keys matching `secondary_key`.
    pub fn lookup_primary_keys(&self, secondary_key: &[u8]) -> Result<Vec<Vec<u8>>, DbError> {
        let _ = self.version_signal.get();
        let rx = self.db.begin_read()?;
        self.index.lookup_primary_keys(&rx, secondary_key)
    }

    /// Stream resolved primary records matching `secondary_key` directly to visitor.
    pub fn scan_matching_records<F>(
        &self,
        secondary_key: &[u8],
        table_name: Option<&str>,
        mut visitor: F,
    ) -> Result<(), DbError>
    where
        F: FnMut(ZeroCopyRow<'_>) -> bool,
    {
        let primary_keys = self.lookup_primary_keys(secondary_key)?;
        let rx = self.db.begin_read()?;

        for pk in &primary_keys {
            let val = if let Some(tbl) = table_name {
                let table = rx.open_table(tbl)?;
                table.get(pk)?
            } else {
                rx.get(pk)?
            };

            if let Some(v) = val {
                let row = ZeroCopyRow::new(pk, &v);
                if !visitor(row) {
                    break;
                }
            }
        }

        Ok(())
    }
}
