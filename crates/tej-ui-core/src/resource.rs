//! Reactive async data fetching resource primitive.
//!
//! Provides `Resource<T, E>` tracking loading states, data cache, error recovery,
//! and automated re-fetching when dependent source signals mutate.

use crate::computed::Memo;
use crate::effect::create_effect;
use crate::signal::Signal;
use std::fmt::Debug;
use std::sync::Arc;

/// State lifecycle of an asynchronous resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState<T, E> {
    /// Resource has not initiated fetching.
    Uninitialized,
    /// Asynchronous operation is currently pending.
    Loading,
    /// Asynchronous operation succeeded with data.
    Ready(T),
    /// Asynchronous operation failed with an error.
    Error(E),
}

impl<T, E> ResourceState<T, E> {
    /// Returns `true` if the resource is currently loading.
    #[must_use]
    pub const fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Returns `true` if the resource is resolved and ready.
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    /// Returns `true` if the resource is in an error state.
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Returns a reference to the ready data, if available.
    #[must_use]
    pub const fn data(&self) -> Option<&T> {
        match self {
            Self::Ready(data) => Some(data),
            _ => None,
        }
    }

    /// Returns a reference to the error, if available.
    #[must_use]
    pub const fn error(&self) -> Option<&E> {
        match self {
            Self::Error(err) => Some(err),
            _ => None,
        }
    }
}

/// Reactive wrapper around asynchronous data queries.
#[derive(Clone)]
pub struct Resource<T: Clone + Send + Sync + 'static, E: Clone + Send + Sync + 'static> {
    state: Signal<ResourceState<T, E>>,
    fetcher: Arc<dyn Fn() -> std::result::Result<T, E> + Send + Sync>,
}

impl<T: Clone + Debug + Send + Sync + 'static, E: Clone + Debug + Send + Sync + 'static> Debug
    for Resource<T, E>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resource")
            .field("state", &self.state.get())
            .finish()
    }
}

impl<T: Clone + Send + Sync + 'static, E: Clone + Send + Sync + 'static> Resource<T, E> {
    /// Creates and executes a new asynchronous resource.
    pub fn new<F>(fetcher: F) -> Self
    where
        F: Fn() -> std::result::Result<T, E> + Send + Sync + 'static,
    {
        let fetcher_arc = Arc::new(fetcher);
        let state = Signal::new(ResourceState::Loading);
        let res = Self {
            state,
            fetcher: fetcher_arc,
        };
        res.refetch();
        res
    }

    /// Creates a resource that automatically refetches whenever `source` signal changes.
    pub fn new_with_source<S, F>(source: Signal<S>, fetcher: F) -> Self
    where
        S: Clone + Send + Sync + 'static,
        F: Fn(S) -> std::result::Result<T, E> + Send + Sync + 'static,
    {
        let state = Signal::new(ResourceState::Loading);
        let fetcher_arc = Arc::new(fetcher);

        let s_clone = state;
        let f_clone = Arc::clone(&fetcher_arc);
        create_effect(move || {
            let val = source.get();
            s_clone.set(ResourceState::Loading);
            match f_clone(val) {
                Ok(data) => s_clone.set(ResourceState::Ready(data)),
                Err(err) => s_clone.set(ResourceState::Error(err)),
            }
        });

        Self {
            state,
            fetcher: Arc::new(move || {
                let val = source.get();
                fetcher_arc(val)
            }),
        }
    }

    /// Read the current resource state while tracking dependency in reactive contexts.
    #[must_use]
    pub fn get(&self) -> ResourceState<T, E> {
        self.state.get()
    }

    /// Returns the resolved data if ready, or `None` if loading or in error.
    #[must_use]
    pub fn data(&self) -> Option<T> {
        match self.state.get() {
            ResourceState::Ready(d) => Some(d),
            _ => None,
        }
    }

    /// Returns `true` if the resource is currently in a loading state.
    #[must_use]
    pub fn is_loading(&self) -> bool {
        self.state.get().is_loading()
    }

    /// Triggers an immediate re-fetch of the resource.
    pub fn refetch(&self) {
        self.state.set(ResourceState::Loading);
        let res = (self.fetcher)();
        match res {
            Ok(data) => self.state.set(ResourceState::Ready(data)),
            Err(err) => self.state.set(ResourceState::Error(err)),
        }
    }

    /// Creates a derived reactive memo representing whether data is available.
    #[must_use]
    pub fn has_data(&self) -> Memo<bool> {
        let state = self.state;
        Memo::new(move || state.get().is_ready())
    }
}
