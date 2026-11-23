//! Deterministic state store and reducer architecture for enterprise apps.
//!
//! Provides unidirectional data flow, multi-level undo/redo history,
//! and fine-grained derived state selection via reactive memos.

use crate::computed::Memo;
use crate::signal::Signal;
use parking_lot::Mutex;
use std::fmt::Debug;
use std::sync::Arc;

/// Linear undo/redo history container.
#[derive(Debug, Default)]
struct StoreHistory<S> {
    undo_stack: Vec<S>,
    redo_stack: Vec<S>,
    max_history: usize,
}

impl<S: Clone> StoreHistory<S> {
    fn with_capacity(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    fn push_undo(&mut self, state: S) {
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(state);
        self.redo_stack.clear();
    }
}

/// Reducer function type transitioning state given an action.
pub type ReducerFn<S, A> = Arc<dyn Fn(&S, A) -> S + Send + Sync>;

/// Central state store coordinating actions, reducers, and time-travel history.
#[derive(Clone)]
pub struct Store<S: Clone + Send + Sync + 'static, A: Send + Sync + 'static> {
    state: Signal<S>,
    reducer: ReducerFn<S, A>,
    history: Arc<Mutex<StoreHistory<S>>>,
}

impl<S: Clone + Debug + Send + Sync + 'static, A: Send + Sync + 'static> Debug for Store<S, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store")
            .field("state", &self.state.get())
            .finish()
    }
}

impl<S: Clone + Send + Sync + 'static, A: Send + Sync + 'static> Store<S, A> {
    /// Constructs a new store with an initial state and pure reducer function.
    pub fn new<R>(initial_state: S, reducer: R) -> Self
    where
        R: Fn(&S, A) -> S + Send + Sync + 'static,
    {
        Self::with_history_limit(initial_state, reducer, 50)
    }

    /// Constructs a new store with a configured maximum undo history capacity.
    pub fn with_history_limit<R>(initial_state: S, reducer: R, max_history: usize) -> Self
    where
        R: Fn(&S, A) -> S + Send + Sync + 'static,
    {
        Self {
            state: Signal::new(initial_state),
            reducer: Arc::new(reducer),
            history: Arc::new(Mutex::new(StoreHistory::with_capacity(max_history))),
        }
    }

    /// Dispatches an action through the reducer, transitioning state and recording history.
    pub fn dispatch(&self, action: A) {
        let current = self.state.get();
        self.history.lock().push_undo(current.clone());
        let next = (self.reducer)(&current, action);
        self.state.set(next);
    }

    /// Reads the current state value while recording dependencies in reactive computations.
    #[must_use]
    pub fn get(&self) -> S {
        self.state.get()
    }

    /// Returns a direct read handle to the internal state signal.
    #[must_use]
    pub const fn signal(&self) -> Signal<S> {
        self.state
    }

    /// Creates a fine-grained derived memo computing a sub-slice of store state.
    pub fn select<U, F>(&self, selector: F) -> Memo<U>
    where
        U: PartialEq + Clone + Send + Sync + 'static,
        F: Fn(&S) -> U + Send + Sync + 'static,
    {
        let state_sig = self.state;
        Memo::new(move || {
            let s = state_sig.get();
            selector(&s)
        })
    }

    /// Undoes the last dispatched action, restoring previous state.
    ///
    /// Returns `true` if an undo was executed, or `false` if history is empty.
    pub fn undo(&self) -> bool {
        let mut hist = self.history.lock();
        if let Some(prev) = hist.undo_stack.pop() {
            let current = self.state.get();
            hist.redo_stack.push(current);
            self.state.set(prev);
            true
        } else {
            false
        }
    }

    /// Redoes the previously undone action.
    ///
    /// Returns `true` if a redo was executed, or `false` if redo history is empty.
    pub fn redo(&self) -> bool {
        let mut hist = self.history.lock();
        if let Some(next) = hist.redo_stack.pop() {
            let current = self.state.get();
            hist.undo_stack.push(current);
            self.state.set(next);
            true
        } else {
            false
        }
    }

    /// Returns `true` if previous states exist in the undo stack.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.history.lock().undo_stack.is_empty()
    }

    /// Returns `true` if states exist in the redo stack.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.history.lock().redo_stack.is_empty()
    }
}
