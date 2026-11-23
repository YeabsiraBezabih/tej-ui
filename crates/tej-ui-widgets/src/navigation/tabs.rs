use std::fmt::Debug;
use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_view::View;

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
    pub node: u32,
    pub header_row_node: u32,
    pub tab_nodes: Vec<u32>,
    pub content_node: u32,
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
            node: 0,
            header_row_node: 0,
            tab_nodes: Vec::new(),
            content_node: 0,
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

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let root_style = tej_ui_layout::LayoutStyle {
            direction: tej_ui_layout::FlexDirection::Column,
            width: tej_ui_layout::Dimension::Percent(1.0),
            spacing: 8.0,
            ..Default::default()
        };
        let root = arena.allocate_node(root_style);
        state.node = root;

        let header_style = tej_ui_layout::LayoutStyle {
            direction: tej_ui_layout::FlexDirection::Row,
            height: tej_ui_layout::Dimension::Points(36.0),
            spacing: 4.0,
            padding: tej_ui_layout::Insets::all(2.0),
            ..Default::default()
        };
        let header_node = arena.allocate_node(header_style);
        state.header_row_node = header_node;
        arena.add_child(root, header_node);

        state.tab_nodes.clear();
        for _ in 0..self.tabs.len() {
            let tab_style = tej_ui_layout::LayoutStyle {
                height: tej_ui_layout::Dimension::Points(32.0),
                padding: tej_ui_layout::Insets::symmetric(8.0, 12.0),
                ..Default::default()
            };
            let tab_n = arena.allocate_node(tab_style);
            state.tab_nodes.push(tab_n);
            arena.add_child(header_node, tab_n);
        }

        let content_view = (self.content_builder)(state.active_tab);
        let content_n = content_view.layout(&mut state.content_state, arena);
        state.content_node = content_n;
        arena.add_child(root, content_n);

        root
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        if let Some(header_bounds) = ctx
            .arena
            .computed_bounds
            .get(state.header_row_node as usize)
            .copied()
        {
            ctx.fill_rounded_rect(header_bounds, ctx.theme.radii.md, ctx.theme.colors.muted);

            for (i, tab) in self.tabs.iter().enumerate() {
                if let Some(&tab_n) = state.tab_nodes.get(i) {
                    let tab_bounds = ctx
                        .arena
                        .computed_bounds
                        .get(tab_n as usize)
                        .copied()
                        .unwrap_or(header_bounds);
                    let is_active = i == state.active_tab;
                    let is_hovered = ctx.is_hovered(tab_n);

                    if is_active {
                        ctx.fill_rounded_rect(
                            tab_bounds,
                            ctx.theme.radii.sm,
                            ctx.theme.colors.background,
                        );
                    } else if is_hovered {
                        ctx.fill_rounded_rect(
                            tab_bounds,
                            ctx.theme.radii.sm,
                            ctx.theme.colors.accent.with_alpha(80),
                        );
                    }

                    let text_color = if is_active {
                        ctx.theme.colors.foreground
                    } else {
                        ctx.theme.colors.muted_foreground
                    };
                    let text_pt = tej_ui_layout::Point::new(
                        tab_bounds.origin.x + 8.0,
                        tab_bounds.origin.y + (tab_bounds.size.height - 14.0) * 0.5,
                    );
                    ctx.draw_text(
                        &tab.title,
                        text_pt,
                        ctx.theme.typography.body_small.font_size,
                        text_color,
                    );
                }
            }
        }

        let content_view = (self.content_builder)(state.active_tab);
        content_view.paint(&state.content_state, ctx);
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut tej_ui_view::EventDispatcher) {
        for (i, _) in self.tabs.iter().enumerate() {
            if let Some(&tab_n) = state.tab_nodes.get(i) {
                let sig = self.active_tab;
                dispatcher.register_click(
                    tab_n,
                    Arc::new(move || {
                        sig.set(i);
                    }),
                );
            }
        }

        let content_view = (self.content_builder)(state.active_tab);
        content_view.register_events(&state.content_state, dispatcher);
    }
}
