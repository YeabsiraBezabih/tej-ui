use mydb_core::{ChangeSet, Hook};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tej_core::Signal;

type SubscriptionMap = HashMap<Vec<u8>, Vec<Signal<u64>>>;
type PrefixSubscriptions = Arc<RwLock<SubscriptionMap>>;

/// Database commit listener hook that maps storage mutations to reactive UI signals.
pub struct DbSyncHook {
    /// Maps key prefixes to reactive version signals.
    prefix_subscriptions: PrefixSubscriptions,
}

impl Default for DbSyncHook {
    fn default() -> Self {
        Self::new()
    }
}

impl DbSyncHook {
    /// Create a new commit hook.
    #[must_use]
    pub fn new() -> Self {
        Self {
            prefix_subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a reactive signal to be notified whenever keys matching `prefix` mutate.
    pub fn subscribe_prefix(&self, prefix: Vec<u8>, signal: Signal<u64>) {
        let mut subs = self.prefix_subscriptions.write();
        subs.entry(prefix).or_default().push(signal);
    }
}

impl std::fmt::Debug for DbSyncHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DbSyncHook")
            .field(
                "active_subscriptions",
                &self.prefix_subscriptions.read().len(),
            )
            .finish()
    }
}

impl Hook for DbSyncHook {
    fn after_commit(&self, changes: &ChangeSet) {
        let subs = self.prefix_subscriptions.read();
        if subs.is_empty() {
            return;
        }

        let mut notified_signals = Vec::new();

        for (key, _) in &changes.writes {
            for (prefix, signals) in subs.iter() {
                if key.starts_with(prefix) {
                    for &sig in signals {
                        if !notified_signals.contains(&sig) {
                            notified_signals.push(sig);
                        }
                    }
                }
            }
        }

        for key in &changes.deletes {
            for (prefix, signals) in subs.iter() {
                if key.starts_with(prefix) {
                    for &sig in signals {
                        if !notified_signals.contains(&sig) {
                            notified_signals.push(sig);
                        }
                    }
                }
            }
        }

        // Bump version signals to notify reactive UI observers
        for sig in notified_signals {
            sig.update(|v| *v = v.wrapping_add(1));
        }
    }
}
