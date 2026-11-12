//! Optimistic UI transaction with rollback support on ACID commit failure.

use mydb_core::DbError;
use tej_core::Signal;

/// Execute an optimistic UI update immediately on a reactive signal,
/// then execute the database transaction closure.
///
/// If the transaction succeeds, the signal retains its updated value.
/// If the transaction fails (e.g. OCC conflict, constraint violation, or I/O error),
/// the signal is automatically restored to its previous value and the error is returned.
pub fn optimistic_transaction<T, F>(
    signal: Signal<T>,
    mutation: impl FnOnce(&mut T),
    db_tx: F,
) -> Result<(), DbError>
where
    T: Clone + Send + Sync + 'static,
    F: FnOnce() -> Result<(), DbError>,
{
    let previous_state = signal.get();
    signal.update(mutation);

    if let Err(err) = db_tx() {
        // Rollback signal state
        signal.set(previous_state);
        return Err(err);
    }

    Ok(())
}
