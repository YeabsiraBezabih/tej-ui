use crate::id::SignalId;
use crate::runtime::ReactiveRuntime;
use std::any::Any;
use std::fmt;
use std::marker::PhantomData;

/// An 8-byte `Copy` handle to reactive state managed inside the generational arena.
///
/// Because it is `Copy`, it can be captured by value in closures, event handlers,
/// and view trees without `.clone()` or explicit lifetime annotations.
#[repr(transparent)]
pub struct Signal<T: 'static> {
    pub(crate) id: SignalId,
    pub(crate) _marker: PhantomData<fn() -> T>,
}

// Manual Copy/Clone implementation so T does not need to be Clone to copy the handle.
impl<T: 'static> Clone for Signal<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for Signal<T> {}

impl<T: 'static> PartialEq for Signal<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: 'static> Eq for Signal<T> {}

impl<T: 'static> std::hash::Hash for Signal<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: Any + Send + Sync + 'static> Signal<T> {
    /// Create a new reactive signal initialized with `value`.
    pub fn new(value: T) -> Self {
        let id = ReactiveRuntime::create_signal(value);
        Self {
            id,
            _marker: PhantomData,
        }
    }

    /// Read the current value of the signal, automatically tracking dependency if within an observer.
    ///
    /// # Panics
    ///
    /// Panics if the underlying signal slot is missing or its generation is stale.
    pub fn get(self) -> T
    where
        T: Clone,
    {
        ReactiveRuntime::read_signal::<T>(self.id)
            .expect("Signal read failed: slot missing or stale generation")
    }

    /// Read the current value without recording a reactive dependency.
    ///
    /// # Panics
    ///
    /// Panics if the underlying signal slot is missing or its generation is stale.
    pub fn untracked_get(self) -> T
    where
        T: Clone,
    {
        ReactiveRuntime::untracked_read::<T>(self.id).expect("Signal untracked read failed")
    }

    /// Overwrite the signal's value and trigger downstream notifications.
    ///
    /// # Panics
    ///
    /// Panics if the underlying signal slot is missing or its generation is stale.
    pub fn set(self, value: T) {
        ReactiveRuntime::write_signal(self.id, value)
            .expect("Signal write failed: slot missing or stale generation");
    }

    /// Mutate the signal value in place via closure and trigger notifications.
    pub fn update<F: FnOnce(&mut T)>(self, f: F)
    where
        T: Clone,
    {
        let mut val = self.untracked_get();
        f(&mut val);
        self.set(val);
    }

    /// Dispose of this signal, freeing its arena slot and clearing its dependency graph edges.
    pub fn dispose(self) {
        ReactiveRuntime::dispose_signal(self.id);
    }

    /// Get underlying generational `SignalId`.
    #[inline]
    #[must_use]
    pub const fn id(self) -> SignalId {
        self.id
    }

    /// Split this signal into a read-only and a write-only handle.
    #[must_use]
    pub const fn split(self) -> (ReadSignal<T>, WriteSignal<T>) {
        (
            ReadSignal {
                id: self.id,
                _marker: PhantomData,
            },
            WriteSignal {
                id: self.id,
                _marker: PhantomData,
            },
        )
    }
}

impl<T: 'static> fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Signal")
            .field("id", &self.id)
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}

/// Read-only 8-byte `Copy` handle to a reactive signal.
#[repr(transparent)]
pub struct ReadSignal<T: 'static> {
    pub(crate) id: SignalId,
    pub(crate) _marker: PhantomData<fn() -> T>,
}

impl<T: 'static> Clone for ReadSignal<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for ReadSignal<T> {}

impl<T: Any + Send + Sync + 'static> ReadSignal<T> {
    /// Read the current value and register dependency.
    ///
    /// # Panics
    ///
    /// Panics if the underlying signal slot is missing or its generation is stale.
    pub fn get(self) -> T
    where
        T: Clone,
    {
        ReactiveRuntime::read_signal::<T>(self.id).expect("ReadSignal get failed")
    }

    /// Read without dependency tracking.
    ///
    /// # Panics
    ///
    /// Panics if the underlying signal slot is missing or its generation is stale.
    pub fn untracked_get(self) -> T
    where
        T: Clone,
    {
        ReactiveRuntime::untracked_read::<T>(self.id).expect("ReadSignal untracked_get failed")
    }

    /// Get underlying `SignalId`.
    #[inline]
    #[must_use]
    pub const fn id(self) -> SignalId {
        self.id
    }
}

impl<T: 'static> fmt::Debug for ReadSignal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadSignal").field("id", &self.id).finish()
    }
}

/// Write-only 8-byte `Copy` handle to a reactive signal.
#[repr(transparent)]
pub struct WriteSignal<T: 'static> {
    pub(crate) id: SignalId,
    pub(crate) _marker: PhantomData<fn() -> T>,
}

impl<T: 'static> Clone for WriteSignal<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for WriteSignal<T> {}

impl<T: Any + Send + Sync + 'static> WriteSignal<T> {
    /// Set a new value and trigger downstream notifications.
    ///
    /// # Panics
    ///
    /// Panics if the underlying signal slot is missing or its generation is stale.
    pub fn set(self, value: T) {
        ReactiveRuntime::write_signal(self.id, value).expect("WriteSignal set failed");
    }

    /// Mutate the value via closure.
    ///
    /// # Panics
    ///
    /// Panics if the underlying signal slot is missing or its generation is stale.
    pub fn update<F: FnOnce(&mut T)>(self, f: F)
    where
        T: Clone,
    {
        let mut val =
            ReactiveRuntime::untracked_read::<T>(self.id).expect("WriteSignal update read failed");
        f(&mut val);
        self.set(val);
    }

    /// Get underlying `SignalId`.
    #[inline]
    #[must_use]
    pub const fn id(self) -> SignalId {
        self.id
    }
}

impl<T: 'static> fmt::Debug for WriteSignal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriteSignal").field("id", &self.id).finish()
    }
}
