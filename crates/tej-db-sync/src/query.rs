use crate::cursor::ZeroCopyRow;
use crate::hook::DbSyncHook;
use mydb::Database;
use mydb_core::{KeyRange, Storage};
use std::sync::Arc;
use tej_core::Signal;

/// Reactive database query handle bound to a table or key prefix.
#[derive(Clone, Debug)]
pub struct ReactiveDbQuery<S: Storage> {
    db: Arc<Database<S>>,
    prefix: Vec<u8>,
    version_signal: Signal<u64>,
}

impl<S: Storage> ReactiveDbQuery<S> {
    /// Create a new reactive query watching key `prefix`.
    pub fn new(db: Arc<Database<S>>, hook: &DbSyncHook, prefix: Vec<u8>) -> Self {
        let version_signal = Signal::new(0);
        hook.subscribe_prefix(prefix.clone(), version_signal);
        Self {
            db,
            prefix,
            version_signal,
        }
    }

    /// Reactive version signal that ticks whenever matching keys mutate.
    #[must_use]
    pub const fn version_signal(&self) -> Signal<u64> {
        self.version_signal
    }

    /// Execute a zero-copy read scan passing rows to visitor closure.
    pub fn scan_zero_copy<F>(&self, mut visitor: F) -> Result<(), mydb_core::DbError>
    where
        F: FnMut(ZeroCopyRow<'_>) -> bool,
    {
        // Touch reactive signal to register dependency with caller's observer
        let _ = self.version_signal.get();

        let rx = self.db.begin_read()?;
        let range = KeyRange::all();

        rx.scan_range_visitor(&range, |k, v| {
            if k.starts_with(&self.prefix) {
                let row = ZeroCopyRow::new(k, v);
                let cont = visitor(row);
                Ok(cont)
            } else {
                Ok(true)
            }
        })?;

        Ok(())
    }
}
