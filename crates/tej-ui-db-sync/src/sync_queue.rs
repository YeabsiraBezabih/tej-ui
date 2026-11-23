//! Offline sync mutation queue with retry policies and conflict resolution strategies.

use parking_lot::RwLock;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::Arc;
use tej_ui_core::Signal;

/// Status of an enqueued offline mutation.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SyncStatus {
    #[default]
    Pending,
    Syncing,
    Synced,
    Conflict,
    Failed,
}

pub type MergeFn<T> = Arc<dyn Fn(&T, &T) -> T + Send + Sync>;

/// Conflict resolution strategy when remote database state conflicts with local offline mutation.
#[derive(Clone)]
pub enum ConflictStrategy<T> {
    /// Client mutation overrides remote server state.
    ClientWins,
    /// Remote server state overrides client mutation (rolls back client signal).
    ServerWins,
    /// Custom merge function reconciling client and remote versions.
    Merge(MergeFn<T>),
}

impl<T> Debug for ConflictStrategy<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClientWins => write!(f, "ConflictStrategy::ClientWins"),
            Self::ServerWins => write!(f, "ConflictStrategy::ServerWins"),
            Self::Merge(_) => write!(f, "ConflictStrategy::Merge(fn)"),
        }
    }
}

/// Individual recorded offline mutation awaiting synchronization.
#[derive(Clone, Debug)]
pub struct PendingMutation<T: Clone + Send + Sync + 'static> {
    pub id: u64,
    pub entity_key: String,
    pub client_value: T,
    pub original_value: T,
    pub target_signal: Signal<T>,
    pub attempts: usize,
    pub status: SyncStatus,
}

/// Thread-safe queue of offline mutations with optimistic UI application and conflict reconciliation.
#[derive(Clone, Debug)]
pub struct OfflineSyncQueue<T: Clone + Send + Sync + 'static> {
    queue: Arc<RwLock<VecDeque<PendingMutation<T>>>>,
    next_id: Arc<RwLock<u64>>,
    default_strategy: Arc<RwLock<ConflictStrategy<T>>>,
}

impl<T: Clone + Send + Sync + 'static> Default for OfflineSyncQueue<T> {
    fn default() -> Self {
        Self::new(ConflictStrategy::ClientWins)
    }
}

impl<T: Clone + Send + Sync + 'static> OfflineSyncQueue<T> {
    #[must_use]
    pub fn new(strategy: ConflictStrategy<T>) -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            next_id: Arc::new(RwLock::new(1)),
            default_strategy: Arc::new(RwLock::new(strategy)),
        }
    }

    /// Enqueue an optimistic mutation: immediately updates `signal` to `new_val` and buffers mutation.
    pub fn mutate_optimistic(
        &self,
        entity_key: impl Into<String>,
        new_val: T,
        signal: Signal<T>,
    ) -> u64 {
        let original = signal.get();
        signal.set(new_val.clone());

        let mut id_lock = self.next_id.write();
        let id = *id_lock;
        *id_lock += 1;

        let mutation = PendingMutation {
            id,
            entity_key: entity_key.into(),
            client_value: new_val,
            original_value: original,
            target_signal: signal,
            attempts: 0,
            status: SyncStatus::Pending,
        };

        self.queue.write().push_back(mutation);
        id
    }

    /// Number of mutations currently pending sync.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.queue.read().len()
    }

    /// Process the next pending mutation with a remote synchronization worker.
    ///
    /// The `sync_fn` receives `(entity_key, client_val)` and returns:
    /// - `Ok(remote_accepted_val)` on success.
    /// - `Err(Some(conflicting_remote_val))` if a version conflict occurred.
    /// - `Err(None)` on network/transient failure.
    pub fn process_next<F>(&self, sync_fn: F) -> Option<SyncStatus>
    where
        F: FnOnce(&str, &T) -> Result<T, Option<T>>,
    {
        let mut mutation = self.queue.write().pop_front()?;
        mutation.attempts += 1;
        mutation.status = SyncStatus::Syncing;

        match sync_fn(&mutation.entity_key, &mutation.client_value) {
            Ok(server_val) => {
                mutation.status = SyncStatus::Synced;
                mutation.target_signal.set(server_val);
                Some(SyncStatus::Synced)
            }
            Err(Some(remote_val)) => {
                // Conflict resolution
                mutation.status = SyncStatus::Conflict;
                let strategy = self.default_strategy.read().clone();
                match strategy {
                    ConflictStrategy::ClientWins => {
                        // Re-queue to push local client value
                        mutation.status = SyncStatus::Pending;
                        self.queue.write().push_front(mutation);
                    }
                    ConflictStrategy::ServerWins => {
                        // Revert signal to remote server state
                        mutation.target_signal.set(remote_val);
                    }
                    ConflictStrategy::Merge(merge_fn) => {
                        let merged = merge_fn(&mutation.client_value, &remote_val);
                        mutation.target_signal.set(merged);
                    }
                }
                Some(SyncStatus::Conflict)
            }
            Err(None) => {
                // Transient network failure: keep in queue for retry
                mutation.status = SyncStatus::Failed;
                self.queue.write().push_back(mutation);
                Some(SyncStatus::Failed)
            }
        }
    }
}
