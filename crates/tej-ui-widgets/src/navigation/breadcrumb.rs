//! Navigational breadcrumb trail (shadcn `Breadcrumb`).

use std::sync::Arc;
use tej_ui_layout::{Dimension, FlexDirection, LayoutArena, LayoutStyle, Point, Rect, Size};
use tej_ui_view::{EventDispatcher, PaintContext, View};

/// Callback invoked when a breadcrumb segment is activated.
pub type BreadcrumbNavigateCallback = Arc<dyn Fn(&str) + Send + Sync>;

/// A single segment in a breadcrumb trail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreadcrumbItem {
    pub label: String,
    pub href: Option<String>,
    pub is_current: bool,
}

impl BreadcrumbItem {
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: None,
            is_current: false,
        }
    }

    #[must_use]
    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    #[must_use]
    pub const fn mark_current(mut self) -> Self {
        self.is_current = true;
        self
    }

    #[must_use]
    pub const fn inactive(mut self) -> Self {
        self.is_current = false;
        self
    }
}

/// Retained state for a mounted Breadcrumb.
#[derive(Debug, Default)]
pub struct BreadcrumbState {
    pub bounds: Rect,
    pub count: usize,
    pub node: u32,
    pub item_nodes: Vec<u32>,
}

/// Horizontal navigation trail revealing the current page hierarchy.
#[derive(Clone, Default)]
pub struct Breadcrumb {
    pub items: Vec<BreadcrumbItem>,
    pub separator: String,
    on_navigate: Option<BreadcrumbNavigateCallback>,
}

impl std::fmt::Debug for Breadcrumb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Breadcrumb")
            .field("items", &self.items)
            .field("separator", &self.separator)
            .finish_non_exhaustive()
    }
}

impl Breadcrumb {
    /// Creates a breadcrumb trail from a list of items.
    #[must_use]
    pub fn new(items: Vec<BreadcrumbItem>) -> Self {
        Self {
            items,
            separator: "/".to_string(),
            on_navigate: None,
        }
    }

    /// Sets custom separator character or icon.
    #[must_use]
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// Registers a navigation callback invoked with the item's `href` (or label if absent).
    #[must_use]
    pub fn on_navigate<F>(mut self, cb: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_navigate = Some(Arc::new(cb));
        self
    }

    fn item_width(label: &str) -> f32 {
        crate::count_to_f32(label.chars().count()) * 8.0 + 16.0
    }
}

impl View for Breadcrumb {
    type State = BreadcrumbState;

    fn build(&self) -> Self::State {
        let width = crate::count_to_f32(self.items.len()) * 80.0;
        BreadcrumbState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(width, 24.0)),
            count: self.items.len(),
            node: 0,
            item_nodes: Vec::new(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.count = self.items.len();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        let root_style = LayoutStyle {
            direction: FlexDirection::Row,
            height: Dimension::Points(28.0),
            spacing: 14.0,
            ..Default::default()
        };
        let root = arena.allocate_node(root_style);
        state.node = root;
        state.item_nodes.clear();

        for item in &self.items {
            let item_style = LayoutStyle {
                width: Dimension::Points(Self::item_width(&item.label)),
                height: Dimension::Points(24.0),
                ..Default::default()
            };
            let item_node = arena.allocate_node(item_style);
            state.item_nodes.push(item_node);
            arena.add_child(root, item_node);
        }

        root
    }

    fn paint(&self, state: &Self::State, ctx: &mut PaintContext<'_>) {
        let last_idx = self.items.len().saturating_sub(1);
        for (i, item) in self.items.iter().enumerate() {
            let Some(&item_node) = state.item_nodes.get(i) else {
                continue;
            };
            let item_bounds = ctx
                .arena
                .computed_bounds
                .get(item_node as usize)
                .copied()
                .unwrap_or(Rect::new(crate::count_to_f32(i) * 94.0, 0.0, 80.0, 24.0));

            let is_hovered = ctx.is_hovered(item_node);
            let is_focused = ctx.is_focused(item_node);

            if is_focused && !item.is_current {
                ctx.fill_rounded_rect(
                    item_bounds,
                    ctx.theme.radii.sm,
                    ctx.theme.colors.ring.with_alpha(60),
                );
            } else if is_hovered && !item.is_current {
                ctx.fill_rounded_rect(item_bounds, ctx.theme.radii.sm, ctx.theme.colors.accent);
            }

            let text_color = if item.is_current {
                ctx.theme.colors.foreground
            } else {
                ctx.theme.colors.muted_foreground
            };

            let font_size = ctx.theme.typography.body_small.font_size;
            let text_pt = Point::new(
                item_bounds.origin.x + 6.0,
                item_bounds.origin.y + (item_bounds.size.height - font_size) * 0.5,
            );
            ctx.draw_text(&item.label, text_pt, font_size, text_color);

            // Draw separator between segments
            if i < last_idx {
                let sep_pt = Point::new(
                    item_bounds.origin.x + item_bounds.size.width + 4.0,
                    text_pt.y,
                );
                ctx.draw_text(
                    &self.separator,
                    sep_pt,
                    font_size,
                    ctx.theme.colors.muted_foreground,
                );
            }
        }
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut EventDispatcher) {
        for (i, item) in self.items.iter().enumerate() {
            if item.is_current {
                continue;
            }
            if let Some(&item_node) = state.item_nodes.get(i) {
                let target = item.href.clone().unwrap_or_else(|| item.label.clone());
                let on_nav = self.on_navigate.clone();
                dispatcher.register_click(
                    item_node,
                    Arc::new(move || {
                        if let Some(cb) = &on_nav {
                            cb(&target);
                        }
                    }),
                );
            }
        }
    }
}
