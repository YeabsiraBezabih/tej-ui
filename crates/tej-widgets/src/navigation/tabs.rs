use std::fmt::Debug;
use std::sync::Arc;
use tej_core::Signal;
use tej_view::View;

/// An individual tab header entry.
#[derive(Clone, Debug)]
pub struct TabItem {
    pub id: String,
    pub title: String,
    pub badge: Option<String>,
}

impl TabItem {
    /// Create a tab item with ID and title.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            badge: None,
        }
    }

    /// Add badge counter or status label.
    pub fn badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }
}

/// Accessible tab strip container with reactive active tab selection.
pub struct Tabs<V: View, ContentBuilder>
where
    ContentBuilder: Fn(usize) -> V + 'static,
{
    tabs: Vec<TabItem>,
    active_tab: Signal<usize>,
    content_builder: Arc<ContentBuilder>,
}

impl<V: View, ContentBuilder> Debug for Tabs<V, ContentBuilder>
where
    ContentBuilder: Fn(usize) -> V + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tabs")
            .field("tabs_count", &self.tabs.len())
            .field("active_tab", &self.active_tab)
            .finish()
    }
}

impl<V: View, ContentBuilder> Tabs<V, ContentBuilder>
where
    ContentBuilder: Fn(usize) -> V + 'static,
{
    /// Create tabs container with list of tab items, active tab signal, and content builder.
    pub fn new(
        tabs: Vec<TabItem>,
        active_tab: Signal<usize>,
        content_builder: ContentBuilder,
    ) -> Self {
        Self {
            tabs,
            active_tab,
            content_builder: Arc::new(content_builder),
        }
    }
}

/// State of a mounted Tabs container.
#[derive(Debug)]
pub struct TabsState<S: Debug> {
    pub active_tab: usize,
    pub content_state: S,
}

impl<V: View, ContentBuilder> View for Tabs<V, ContentBuilder>
where
    ContentBuilder: Fn(usize) -> V + 'static,
{
    type State = TabsState<V::State>;

    fn build(&self) -> Self::State {
        let active = self.active_tab.get();
        let content_view = (self.content_builder)(active);
        TabsState {
            active_tab: active,
            content_state: content_view.build(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let active = self.active_tab.get();
        let content_view = (self.content_builder)(active);
        if active != state.active_tab {
            state.active_tab = active;
            state.content_state = content_view.build();
        } else {
            content_view.rebuild(&mut state.content_state);
        }
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        let active = self.active_tab.get();
        if active != state.active_tab {
            self.rebuild(state);
        } else {
            let content_view = (self.content_builder)(active);
            content_view.patch(&mut state.content_state, dirty_bits);
        }
    }
}
