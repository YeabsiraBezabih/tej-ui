use thiserror::Error;

/// Core reactivity errors.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// Signal was accessed with an invalid or expired generation.
    #[error("Signal slot {id} was accessed with stale generation {expected}, current is {actual}")]
    StaleSignal { id: u32, expected: u32, actual: u32 },

    /// Signal slot does not exist in the arena.
    #[error("Signal slot {0} does not exist")]
    SlotNotFound(u32),

    /// Type mismatch when downcasting value stored in dynamic signal slot.
    #[error("Downcast failed for signal slot {0}: type mismatch")]
    TypeMismatch(u32),

    /// Cyclic dependency detected in reactive computation graph.
    #[error("Cycle detected in reactive dependency graph involving node {0}")]
    CyclicDependency(u32),

    /// Reactive runtime is not initialized or poisoned.
    #[error("Reactive runtime error: {0}")]
    RuntimeError(String),
}

/// Specialized result type for `tej-core`.
pub type Result<T> = std::result::Result<T, CoreError>;
