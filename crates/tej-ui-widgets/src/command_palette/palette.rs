use crate::command_palette::fuzzy::fuzzy_match;
use crate::command_palette::item::CommandItem;
use std::fmt::Debug;
use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_hal::UiEvent;
use tej_ui_layout::{Dimension, FlexDirection, Insets, LayoutArena, LayoutStyle, Point, Rect};
use tej_ui_view::{EventDispatcher, EventPropagation, PaintContext, View};

/// Enterprise Command Palette (`Cmd + K`) overlay widget.
#[derive(Clone, Debug)]
pub struct CommandPalette {
    query: Signal<String>,
    selected_index: Signal<usize>,
    is_open: Option<Signal<bool>>,
    items: Vec<CommandItem>,
    max_visible: usize,
}

impl CommandPalette {
    /// Create a new command palette with reactive query and selection signals.
    pub fn new(
        query: Signal<String>,
        selected_index: Signal<usize>,
        items: Vec<CommandItem>,
    ) -> Self {
        Self {
            query,
            selected_index,
            is_open: None,
            items,
            max_visible: 10,
        }
    }

    /// Attach an optional open-state signal dismissed when Escape is pressed.
    #[must_use]
    pub const fn open(mut self, is_open: Signal<bool>) -> Self {
        self.is_open = Some(is_open);
        self
    }

    /// Set maximum visible suggestions.
    #[must_use]
    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    /// Filter and rank items based on the active query string.
    #[must_use]
    pub fn filtered_items(&self) -> Vec<CommandItem> {
        filter_items(&self.items, &self.query.get(), self.max_visible)
    }
}

fn filter_items(items: &[CommandItem], q: &str, max_visible: usize) -> Vec<CommandItem> {
    if q.is_empty() {
        return items.iter().take(max_visible).cloned().collect();
    }

    let mut scored: Vec<(i32, CommandItem)> = items
        .iter()
        .filter_map(|item| fuzzy_match(q, &item.title).map(|m| (m.score, item.clone())))
        .collect();

    // Sort descending by score
    scored.sort_by_key(|a| std::cmp::Reverse(a.0));
    scored
        .into_iter()
        .take(max_visible)
        .map(|(_, item)| item)
        .collect()
}

/// State of a mounted CommandPalette.
#[derive(Debug)]
pub struct CommandPaletteState {
    pub current_query: String,
    pub current_selection: usize,
    pub visible_items: Vec<CommandItem>,
    pub node: u32,
    pub search_node: u32,
    pub item_nodes: Vec<u32>,
}

impl CommandPalette {
    fn paint_search_box(
        state: &CommandPaletteState,
        ctx: &mut PaintContext<'_>,
        bounds: Rect,
    ) -> Rect {
        let search_bounds = ctx
            .arena
            .computed_bounds
            .get(state.search_node as usize)
            .copied()
            .unwrap_or(Rect::new(
                bounds.origin.x + 8.0,
                bounds.origin.y + 8.0,
                (bounds.size.width - 16.0).max(0.0),
                40.0,
            ));

        if ctx.is_focused(state.node) || ctx.is_focused(state.search_node) {
            let ring = Rect::new(
                search_bounds.origin.x - 1.0,
                search_bounds.origin.y - 1.0,
                search_bounds.size.width + 2.0,
                search_bounds.size.height + 2.0,
            );
            ctx.fill_rounded_rect(ring, ctx.theme.radii.md, ctx.theme.colors.ring);
        }
        ctx.fill_rounded_rect(search_bounds, ctx.theme.radii.sm, ctx.theme.colors.input);

        let search_text_pt = Point::new(
            search_bounds.origin.x + 12.0,
            search_bounds.origin.y + (search_bounds.size.height - 14.0) * 0.5,
        );
        if state.current_query.is_empty() {
            ctx.draw_text(
                "Type a command or search...",
                search_text_pt,
                ctx.theme.typography.body_small.font_size,
                ctx.theme.colors.muted_foreground,
            );
        } else {
            ctx.draw_text(
                &state.current_query,
                search_text_pt,
                ctx.theme.typography.body_small.font_size,
                ctx.theme.colors.foreground,
            );
        }
        search_bounds
    }

    fn paint_result_rows(
        state: &CommandPaletteState,
        ctx: &mut PaintContext<'_>,
        bounds: Rect,
        div_y: f32,
    ) {
        if state.visible_items.is_empty() {
            let empty_pt = Point::new(bounds.origin.x + 16.0, div_y + 12.0);
            ctx.draw_text(
                "No results found.",
                empty_pt,
                ctx.theme.typography.body_small.font_size,
                ctx.theme.colors.muted_foreground,
            );
            return;
        }

        for (i, item) in state.visible_items.iter().enumerate() {
            let row_node = state.item_nodes.get(i).copied().unwrap_or(0);
            let row_bounds = ctx
                .arena
                .computed_bounds
                .get(row_node as usize)
                .copied()
                .unwrap_or(Rect::new(
                    bounds.origin.x + 8.0,
                    div_y + 6.0 + crate::count_to_f32(i) * 38.0,
                    (bounds.size.width - 16.0).max(0.0),
                    34.0,
                ));

            let is_selected = i == state.current_selection;
            let is_hovered = ctx.is_hovered(row_node);

            if is_selected {
                ctx.fill_rounded_rect(row_bounds, ctx.theme.radii.sm, ctx.theme.colors.accent);
            } else if is_hovered {
                ctx.fill_rounded_rect(
                    row_bounds,
                    ctx.theme.radii.sm,
                    ctx.theme.colors.accent.with_alpha(90),
                );
            }

            let title_color = if is_selected {
                ctx.theme.colors.accent_foreground
            } else {
                ctx.theme.colors.popover_foreground
            };
            let title_pt = Point::new(
                row_bounds.origin.x + 10.0,
                row_bounds.origin.y + (row_bounds.size.height - 14.0) * 0.5,
            );
            ctx.draw_text(
                &item.title,
                title_pt,
                ctx.theme.typography.body_small.font_size,
                title_color,
            );

            if let Some(shortcut) = &item.shortcut {
                let badge_w = crate::count_to_f32(shortcut.chars().count()) * 7.0 + 12.0;
                let badge_rect = Rect::new(
                    row_bounds.origin.x + row_bounds.size.width - badge_w - 8.0,
                    row_bounds.origin.y + 6.0,
                    badge_w,
                    row_bounds.size.height - 12.0,
                );
                ctx.fill_rounded_rect(badge_rect, ctx.theme.radii.sm, ctx.theme.colors.muted);
                ctx.draw_text(
                    shortcut,
                    Point::new(badge_rect.origin.x + 6.0, badge_rect.origin.y + 4.0),
                    ctx.theme.typography.caption.font_size,
                    ctx.theme.colors.muted_foreground,
                );
            }
        }
    }

    fn register_input_and_key_handlers(
        &self,
        state: &CommandPaletteState,
        dispatcher: &mut EventDispatcher,
    ) {
        for target_node in [state.node, state.search_node] {
            let query_sig = self.query;
            let sel_sig = self.selected_index;
            dispatcher.register_text_input(
                target_node,
                Arc::new(move |text| {
                    let mut q = query_sig.get();
                    q.push_str(text);
                    query_sig.set(q);
                    sel_sig.set(0);
                }),
            );

            let items = self.items.clone();
            let max_visible = self.max_visible;
            let query_sig = self.query;
            let sel_sig = self.selected_index;
            let is_open = self.is_open;
            dispatcher.register_key_down(
                target_node,
                Arc::new(move |key_code| match key_code {
                    38 => {
                        let visible = filter_items(&items, &query_sig.get(), max_visible);
                        if !visible.is_empty() {
                            let cur = sel_sig.get();
                            let next = if cur == 0 { visible.len() - 1 } else { cur - 1 };
                            sel_sig.set(next);
                        }
                    }
                    40 => {
                        let visible = filter_items(&items, &query_sig.get(), max_visible);
                        if !visible.is_empty() {
                            let cur = sel_sig.get();
                            let next = (cur + 1) % visible.len();
                            sel_sig.set(next);
                        }
                    }
                    27 => {
                        query_sig.set(String::new());
                        sel_sig.set(0);
                        if let Some(open) = is_open {
                            open.set(false);
                        }
                    }
                    8 => {
                        let mut q = query_sig.get();
                        if !q.is_empty() {
                            q.pop();
                            query_sig.set(q);
                            sel_sig.set(0);
                        }
                    }
                    _ => {}
                }),
            );
        }
    }

    fn register_row_click_handlers(
        &self,
        state: &CommandPaletteState,
        dispatcher: &mut EventDispatcher,
    ) {
        let items_enter = self.items.clone();
        let max_visible_enter = self.max_visible;
        let query_enter = self.query;
        let sel_enter = self.selected_index;
        let is_open_enter = self.is_open;
        dispatcher.register_bubble(
            state.node,
            Arc::new(move |event, _phase, _node| {
                if let UiEvent::KeyDown { key_code: 13 } = event {
                    let visible = filter_items(&items_enter, &query_enter.get(), max_visible_enter);
                    let idx = sel_enter.get();
                    if let Some(item) = visible.get(idx) {
                        if let Some(action) = &item.action {
                            action();
                        }
                        if let Some(open) = is_open_enter {
                            open.set(false);
                        }
                        return EventPropagation::Stop;
                    }
                }
                EventPropagation::Continue
            }),
        );

        for (i, item) in state.visible_items.iter().enumerate() {
            if let Some(&row_node) = state.item_nodes.get(i) {
                let sel_sig = self.selected_index;
                let action = item.action.clone();
                let is_open_click = self.is_open;
                dispatcher.register_click(
                    row_node,
                    Arc::new(move || {
                        sel_sig.set(i);
                        if let Some(cb) = &action {
                            cb();
                        }
                        if let Some(open) = is_open_click {
                            open.set(false);
                        }
                    }),
                );
            }
        }
    }
}

impl View for CommandPalette {
    type State = CommandPaletteState;

    fn build(&self) -> Self::State {
        let visible_items = self.filtered_items();
        let current_selection = if visible_items.is_empty() {
            0
        } else {
            self.selected_index.get().min(visible_items.len() - 1)
        };
        CommandPaletteState {
            current_query: self.query.get(),
            current_selection,
            visible_items,
            node: 0,
            search_node: 0,
            item_nodes: Vec::new(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.current_query = self.query.get();
        state.visible_items = self.filtered_items();
        state.current_selection = if state.visible_items.is_empty() {
            0
        } else {
            self.selected_index.get().min(state.visible_items.len() - 1)
        };
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        if let Some(open_sig) = self.is_open
            && !open_sig.get()
        {
            let empty = arena.allocate_node(LayoutStyle {
                width: Dimension::Points(0.0),
                height: Dimension::Points(0.0),
                ..Default::default()
            });
            state.node = empty;
            state.search_node = empty;
            state.item_nodes.clear();
            return empty;
        }

        let row_count = state.visible_items.len().max(1);
        let total_height = 44.0 + 16.0 + crate::count_to_f32(row_count) * 38.0;

        let root_style = LayoutStyle {
            direction: FlexDirection::Column,
            width: Dimension::Points(520.0),
            height: Dimension::Points(total_height),
            padding: Insets::all(8.0),
            spacing: 4.0,
            ..Default::default()
        };
        let root = arena.allocate_node(root_style);
        state.node = root;

        let search_style = LayoutStyle {
            width: Dimension::Percent(1.0),
            height: Dimension::Points(40.0),
            padding: Insets::symmetric(8.0, 12.0),
            ..Default::default()
        };
        let search_node = arena.allocate_node(search_style);
        state.search_node = search_node;
        arena.add_child(root, search_node);

        state.item_nodes.clear();
        for _ in 0..state.visible_items.len() {
            let row_style = LayoutStyle {
                width: Dimension::Percent(1.0),
                height: Dimension::Points(34.0),
                padding: Insets::symmetric(6.0, 10.0),
                ..Default::default()
            };
            let row_node = arena.allocate_node(row_style);
            state.item_nodes.push(row_node);
            arena.add_child(root, row_node);
        }

        root
    }

    fn paint(&self, state: &Self::State, ctx: &mut PaintContext<'_>) {
        if let Some(open_sig) = self.is_open
            && !open_sig.get()
        {
            return;
        }

        let Some(bounds) = ctx.arena.computed_bounds.get(state.node as usize).copied() else {
            return;
        };

        ctx.fill_rounded_rect(bounds, ctx.theme.radii.lg, ctx.theme.colors.border);
        let inner = Rect::new(
            bounds.origin.x + 1.0,
            bounds.origin.y + 1.0,
            (bounds.size.width - 2.0).max(0.0),
            (bounds.size.height - 2.0).max(0.0),
        );
        ctx.fill_rounded_rect(inner, ctx.theme.radii.lg, ctx.theme.colors.popover);

        let search_bounds = Self::paint_search_box(state, ctx, bounds);
        let div_y = search_bounds.origin.y + search_bounds.size.height + 2.0;
        ctx.draw_line(
            Point::new(bounds.origin.x + 8.0, div_y),
            Point::new(bounds.origin.x + bounds.size.width - 8.0, div_y),
            1.0,
            ctx.theme.colors.border,
        );

        Self::paint_result_rows(state, ctx, bounds, div_y);
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut EventDispatcher) {
        if let Some(open_sig) = self.is_open
            && !open_sig.get()
        {
            return;
        }

        dispatcher.register_click(state.search_node, Arc::new(|| {}));
        self.register_input_and_key_handlers(state, dispatcher);
        self.register_row_click_handlers(state, dispatcher);
    }
}
