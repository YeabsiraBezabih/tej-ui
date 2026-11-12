use crate::runtime::ReactiveRuntime;

/// Execute a batch of state mutations, deferring subscriber notifications until the batch finishes.
///
/// Prevents intermediate glitches and redundant evaluations when updating multiple signals simultaneously.
pub fn batch<R, F: FnOnce() -> R>(f: F) -> R {
    ReactiveRuntime::batch(f)
}
