//! Ambient context and dependency injection system for `tej-core`.
//!
//! Provides thread-safe, type-indexed ambient state propagation down
//! the reactive view tree without manual parameter passing.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

thread_local! {
    static CONTEXT_STACK: RefCell<HashMap<TypeId, Vec<Arc<dyn Any + Send + Sync>>>> = RefCell::new(HashMap::new());
}

/// RAII guard that pops a scoped context value when leaving scope.
#[derive(Debug)]
struct ContextGuard(TypeId);

impl Drop for ContextGuard {
    fn drop(&mut self) {
        CONTEXT_STACK.with(|stack| {
            if let Some(list) = stack.borrow_mut().get_mut(&self.0) {
                list.pop();
            }
        });
    }
}

/// Provides a context value of type `T` into the current ambient scope.
///
/// If a context value of type `T` already exists at the current scope level,
/// it is overwritten. Use [`with_context`] to safely scope overrides to a closure.
pub fn provide_context<T: Any + Send + Sync + Clone>(value: T) {
    let type_id = TypeId::of::<T>();
    CONTEXT_STACK.with(|stack| {
        let mut map = stack.borrow_mut();
        let list = map.entry(type_id).or_default();
        if let Some(last) = list.last_mut() {
            *last = Arc::new(value);
        } else {
            list.push(Arc::new(value));
        }
    });
}

/// Retrieves the nearest ambient context value of type `T`.
///
/// Returns `None` if no context of type `T` has been provided in the current scope.
#[must_use]
pub fn use_context<T: Any + Send + Sync + Clone>() -> Option<T> {
    let type_id = TypeId::of::<T>();
    CONTEXT_STACK.with(|stack| {
        stack.borrow().get(&type_id).and_then(|list| {
            list.last()
                .and_then(|boxed| boxed.downcast_ref::<T>().cloned())
        })
    })
}

/// Executes a closure with a scoped context value pushed onto the ambient stack.
///
/// The context value is automatically popped when `f` concludes, ensuring
/// strict stack discipline even in the presence of early returns or panics.
pub fn with_context<T: Any + Send + Sync + Clone, R>(value: T, f: impl FnOnce() -> R) -> R {
    let type_id = TypeId::of::<T>();
    CONTEXT_STACK.with(|stack| {
        stack
            .borrow_mut()
            .entry(type_id)
            .or_default()
            .push(Arc::new(value));
    });

    let _guard = ContextGuard(type_id);
    f()
}

/// Clears all ambient context entries for the current thread.
///
/// Primary utility for isolated test suites.
pub fn clear_context_stack() {
    CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().clear();
    });
}
