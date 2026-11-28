//! Data pagination navigation (shadcn `Pagination`).

use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_layout::{Dimension, FlexDirection, LayoutArena, LayoutStyle, Point, Rect, Size};
use tej_ui_view::{EventDispatcher, PaintContext, View};

/// Retained state for a mounted Pagination.
#[derive(Debug, Default)]
pub struct PaginationState {
    pub bounds: Rect,
    pub current_page: usize,
    pub total_pages: usize,
    pub node: u32,
    pub prev_node: u32,
    pub next_node: u32,
    pub page_nodes: Vec<(usize, u32)>,
}

/// Paginated data set navigation with prev/next buttons.
#[derive(Debug, Clone)]
pub struct Pagination {
    current_page: Signal<usize>,
    total_pages: usize,
}

impl Pagination {
    /// Creates a pagination control bound to a current page signal.
    #[must_use]
    pub const fn new(current_page: Signal<usize>, total_pages: usize) -> Self {
        Self {
            current_page,
            total_pages: if total_pages > 0 { total_pages } else { 1 },
        }
    }

    /// Moves to the next page if available.
    pub fn next_page(&self) {
        let cur = self.current_page.get();
        if cur < self.total_pages {
            self.current_page.set(cur + 1);
        }
    }

    /// Moves to the previous page if available.
    pub fn prev_page(&self) {
        let cur = self.current_page.get();
        if cur > 1 {
            self.current_page.set(cur - 1);
        }
    }

    /// Sets explicit page number.
    pub fn set_page(&self, page: usize) {
        let clamped = page.clamp(1, self.total_pages);
        self.current_page.set(clamped);
    }

    fn visible_pages(&self, current: usize) -> Vec<usize> {
        const MAX_BUTTONS: usize = 7;
        if self.total_pages <= MAX_BUTTONS {
            return (1..=self.total_pages).collect();
        }
        let half = MAX_BUTTONS / 2;
        let start = current
            .saturating_sub(half)
            .max(1)
            .min(self.total_pages - MAX_BUTTONS + 1);
        let end = (start + MAX_BUTTONS - 1).min(self.total_pages);
        (start..=end).collect()
    }
}

#[derive(Clone, Copy)]
enum NavButtonState {
    Disabled,
    Enabled,
}

#[derive(Clone, Copy)]
enum PageButtonState {
    Inactive,
    Active,
}

impl Pagination {
    fn paint_nav_button(
        ctx: &mut PaintContext<'_>,
        node: u32,
        bounds: Rect,
        label: &str,
        btn_state: NavButtonState,
        font_size: f32,
    ) {
        let enabled = matches!(btn_state, NavButtonState::Enabled);
        let bg = if !enabled {
            ctx.theme.colors.muted
        } else if ctx.is_hovered(node) {
            ctx.theme.colors.accent
        } else {
            ctx.theme.colors.secondary
        };
        if ctx.is_focused(node) && enabled {
            let ring = Rect::new(
                bounds.origin.x - 1.0,
                bounds.origin.y - 1.0,
                bounds.size.width + 2.0,
                bounds.size.height + 2.0,
            );
            ctx.fill_rounded_rect(ring, ctx.theme.radii.sm, ctx.theme.colors.ring);
        }
        ctx.fill_rounded_rect(bounds, ctx.theme.radii.sm, bg);
        let fg = if enabled {
            ctx.theme.colors.foreground
        } else {
            ctx.theme.colors.muted_foreground
        };
        ctx.draw_text(
            label,
            Point::new(
                bounds.origin.x + 14.0,
                bounds.origin.y + (bounds.size.height - font_size) * 0.5,
            ),
            font_size,
            fg,
        );
    }

    fn paint_page_button(
        ctx: &mut PaintContext<'_>,
        node: u32,
        bounds: Rect,
        page_num: usize,
        page_state: PageButtonState,
        font_size: f32,
    ) {
        let is_active = matches!(page_state, PageButtonState::Active);
        if ctx.is_focused(node) {
            let ring = Rect::new(
                bounds.origin.x - 1.0,
                bounds.origin.y - 1.0,
                bounds.size.width + 2.0,
                bounds.size.height + 2.0,
            );
            ctx.fill_rounded_rect(ring, ctx.theme.radii.sm, ctx.theme.colors.ring);
        }
        let bg = if is_active {
            ctx.theme.colors.primary
        } else if ctx.is_hovered(node) {
            ctx.theme.colors.accent
        } else {
            ctx.theme.colors.secondary
        };
        let fg = if is_active {
            ctx.theme.colors.primary_foreground
        } else {
            ctx.theme.colors.foreground
        };
        ctx.fill_rounded_rect(bounds, ctx.theme.radii.sm, bg);
        let label = page_num.to_string();
        let approx_w = crate::count_to_f32(label.len()) * font_size * 0.6;
        ctx.draw_text(
            &label,
            Point::new(
                bounds.origin.x + (bounds.size.width - approx_w) * 0.5,
                bounds.origin.y + (bounds.size.height - font_size) * 0.5,
            ),
            font_size,
            fg,
        );
    }
}

impl View for Pagination {
    type State = PaginationState;

    fn build(&self) -> Self::State {
        PaginationState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(320.0, 36.0)),
            current_page: self.current_page.get(),
            total_pages: self.total_pages,
            node: 0,
            prev_node: 0,
            next_node: 0,
            page_nodes: Vec::new(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.current_page = self.current_page.get();
        state.total_pages = self.total_pages;
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        let root_style = LayoutStyle {
            direction: FlexDirection::Row,
            height: Dimension::Points(36.0),
            spacing: 4.0,
            ..Default::default()
        };
        let root = arena.allocate_node(root_style);
        state.node = root;

        let prev_style = LayoutStyle {
            width: Dimension::Points(64.0),
            height: Dimension::Points(32.0),
            ..Default::default()
        };
        let prev_node = arena.allocate_node(prev_style);
        state.prev_node = prev_node;
        arena.add_child(root, prev_node);

        state.page_nodes.clear();
        for page_num in self.visible_pages(state.current_page) {
            let page_style = LayoutStyle {
                width: Dimension::Points(32.0),
                height: Dimension::Points(32.0),
                ..Default::default()
            };
            let page_node = arena.allocate_node(page_style);
            state.page_nodes.push((page_num, page_node));
            arena.add_child(root, page_node);
        }

        let next_style = LayoutStyle {
            width: Dimension::Points(64.0),
            height: Dimension::Points(32.0),
            ..Default::default()
        };
        let next_node = arena.allocate_node(next_style);
        state.next_node = next_node;
        arena.add_child(root, next_node);

        root
    }

    fn paint(&self, state: &Self::State, ctx: &mut PaintContext<'_>) {
        let font_size = ctx.theme.typography.body_small.font_size;

        if let Some(prev_bounds) = ctx
            .arena
            .computed_bounds
            .get(state.prev_node as usize)
            .copied()
        {
            let prev_state = if state.current_page > 1 {
                NavButtonState::Enabled
            } else {
                NavButtonState::Disabled
            };
            Self::paint_nav_button(
                ctx,
                state.prev_node,
                prev_bounds,
                "Prev",
                prev_state,
                font_size,
            );
        }

        for &(page_num, page_node) in &state.page_nodes {
            if let Some(bounds) = ctx.arena.computed_bounds.get(page_node as usize).copied() {
                let page_state = if page_num == state.current_page {
                    PageButtonState::Active
                } else {
                    PageButtonState::Inactive
                };
                Self::paint_page_button(ctx, page_node, bounds, page_num, page_state, font_size);
            }
        }

        if let Some(next_bounds) = ctx
            .arena
            .computed_bounds
            .get(state.next_node as usize)
            .copied()
        {
            let next_state = if state.current_page < state.total_pages {
                NavButtonState::Enabled
            } else {
                NavButtonState::Disabled
            };
            Self::paint_nav_button(
                ctx,
                state.next_node,
                next_bounds,
                "Next",
                next_state,
                font_size,
            );
        }
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut EventDispatcher) {
        let total = self.total_pages;

        // Prev click
        let page_prev = self.current_page;
        dispatcher.register_click(
            state.prev_node,
            Arc::new(move || {
                let cur = page_prev.get();
                if cur > 1 {
                    page_prev.set(cur - 1);
                }
            }),
        );

        // Next click
        let page_next = self.current_page;
        dispatcher.register_click(
            state.next_node,
            Arc::new(move || {
                let cur = page_next.get();
                if cur < total {
                    page_next.set(cur + 1);
                }
            }),
        );

        // Direct page selection clicks
        for &(page_num, page_node) in &state.page_nodes {
            let page_sig = self.current_page;
            dispatcher.register_click(
                page_node,
                Arc::new(move || {
                    page_sig.set(page_num.clamp(1, total));
                }),
            );
        }

        // Arrow key navigation across root, prev, next, and page buttons
        let mut key_nodes = vec![state.node, state.prev_node, state.next_node];
        key_nodes.extend(state.page_nodes.iter().map(|&(_, n)| n));
        for target in key_nodes {
            let page_sig = self.current_page;
            dispatcher.register_key_down(
                target,
                Arc::new(move |key_code| match key_code {
                    37 => {
                        // ArrowLeft -> prev page
                        let cur = page_sig.get();
                        if cur > 1 {
                            page_sig.set(cur - 1);
                        }
                    }
                    39 => {
                        // ArrowRight -> next page
                        let cur = page_sig.get();
                        if cur < total {
                            page_sig.set(cur + 1);
                        }
                    }
                    36 => {
                        // Home -> first page
                        page_sig.set(1);
                    }
                    35 => {
                        // End -> last page
                        page_sig.set(total);
                    }
                    _ => {}
                }),
            );
        }
    }
}
