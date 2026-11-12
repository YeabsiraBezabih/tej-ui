use crate::view::View;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

/// Keyed iteration view managing a dynamic collection of items.
pub struct For<T, K, KeyFn, ViewFn, V>
where
    KeyFn: Fn(&T) -> K + 'static,
    ViewFn: Fn(&T) -> V + 'static,
    V: View,
{
    items: Vec<T>,
    key_fn: Arc<KeyFn>,
    view_fn: Arc<ViewFn>,
}

impl<T: Debug, K, KeyFn, ViewFn, V: View> Debug for For<T, K, KeyFn, ViewFn, V>
where
    KeyFn: Fn(&T) -> K + 'static,
    ViewFn: Fn(&T) -> V + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("For")
            .field("items_len", &self.items.len())
            .finish()
    }
}

impl<T, K, KeyFn, ViewFn, V> For<T, K, KeyFn, ViewFn, V>
where
    K: Eq + Hash + Clone + Debug + 'static,
    KeyFn: Fn(&T) -> K + 'static,
    ViewFn: Fn(&T) -> V + 'static,
    V: View,
{
    /// Create a new keyed `For` view.
    pub fn new(items: Vec<T>, key_fn: KeyFn, view_fn: ViewFn) -> Self {
        Self {
            items,
            key_fn: Arc::new(key_fn),
            view_fn: Arc::new(view_fn),
        }
    }
}

/// State of a mounted `For` view.
#[derive(Debug)]
pub struct ForState<K: Debug, S: Debug> {
    pub items: Vec<(K, S)>,
}

impl<T, K, KeyFn, ViewFn, V> View for For<T, K, KeyFn, ViewFn, V>
where
    T: Debug,
    K: Eq + Hash + Clone + Debug + 'static,
    KeyFn: Fn(&T) -> K + 'static,
    ViewFn: Fn(&T) -> V + 'static,
    V: View,
{
    type State = ForState<K, V::State>;

    fn build(&self) -> Self::State {
        let mut items = Vec::with_capacity(self.items.len());
        for item in &self.items {
            let key = (self.key_fn)(item);
            let view = (self.view_fn)(item);
            items.push((key, view.build()));
        }
        ForState { items }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let mut new_items = Vec::with_capacity(self.items.len());
        for item in &self.items {
            let key = (self.key_fn)(item);
            let view = (self.view_fn)(item);
            if let Some(pos) = state.items.iter().position(|(k, _)| *k == key) {
                let mut existing = state.items.remove(pos);
                view.rebuild(&mut existing.1);
                new_items.push(existing);
            } else {
                new_items.push((key, view.build()));
            }
        }
        state.items = new_items;
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
