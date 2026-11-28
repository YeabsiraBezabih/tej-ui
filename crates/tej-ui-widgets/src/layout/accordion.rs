//! Vertically stacked collapsible panels (shadcn `Accordion`).

use std::collections::HashSet;
use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_layout::{Dimension, FlexDirection, LayoutArena, LayoutStyle, Point, Rect, Size};
use tej_ui_view::{EventDispatcher, PaintContext, View};

/// An individual collapsible section within an Accordion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccordionItem {
    pub id: String,
    pub title: String,
    pub content: String,
}

impl AccordionItem {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
        }
    }
}

/// Retained state for a mounted Accordion.
#[derive(Debug, Default)]
pub struct AccordionState {
    pub bounds: Rect,
    pub open_items: HashSet<String>,
    pub node: u32,
    pub header_nodes: Vec<u32>,
    pub content_nodes: Vec<Option<u32>>,
}

/// Interactive accordion revealing and collapsing content sections.
#[derive(Debug, Clone)]
pub struct Accordion {
    open_items: Signal<HashSet<String>>,
    items: Vec<AccordionItem>,
    allow_multiple: bool,
}

impl Accordion {
    /// Creates a new accordion with open items tracked by signal.
    #[must_use]
    pub fn new(open_items: Signal<HashSet<String>>, items: Vec<AccordionItem>) -> Self {
        Self {
            open_items,
            items,
            allow_multiple: false,
        }
    }

    /// Allows multiple sections to be open simultaneously.
    #[must_use]
    pub const fn allow_multiple(mut self) -> Self {
        self.allow_multiple = true;
        self
    }

    /// Restricts the accordion so at most one section is open at a time.
    #[must_use]
    pub const fn single_open(mut self) -> Self {
        self.allow_multiple = false;
        self
    }

    /// Toggles open state for an item.
    pub fn toggle_item(&self, item_id: &str) {
        let policy = if self.allow_multiple {
            OpenPolicy::Multiple
        } else {
            OpenPolicy::Single
        };
        toggle_item_signal(self.open_items, item_id, policy);
    }
}

#[derive(Clone, Copy)]
enum OpenPolicy {
    Single,
    Multiple,
}

fn toggle_item_signal(open_items: Signal<HashSet<String>>, item_id: &str, policy: OpenPolicy) {
    let mut current = open_items.get();
    if current.contains(item_id) {
        current.remove(item_id);
    } else {
        if matches!(policy, OpenPolicy::Single) {
            current.clear();
        }
        current.insert(item_id.to_string());
    }
    open_items.set(current);
}

impl View for Accordion {
    type State = AccordionState;

    fn build(&self) -> Self::State {
        let open_items = self.open_items.get();
        let total_height = self
            .items
            .iter()
            .map(|it| {
                if open_items.contains(&it.id) {
                    120.0
                } else {
                    48.0
                }
            })
            .sum();

        AccordionState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(400.0, total_height)),
            open_items,
            node: 0,
            header_nodes: Vec::new(),
            content_nodes: Vec::new(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.open_items = self.open_items.get();
        let total_height: f32 = self
            .items
            .iter()
            .map(|it| {
                if state.open_items.contains(&it.id) {
                    120.0
                } else {
                    48.0
                }
            })
            .sum();
        state.bounds.size.height = total_height;
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        let root_style = LayoutStyle {
            direction: FlexDirection::Column,
            width: Dimension::Percent(1.0),
            spacing: 2.0,
            ..Default::default()
        };
        let root = arena.allocate_node(root_style);
        state.node = root;
        state.header_nodes.clear();
        state.content_nodes.clear();

        for item in &self.items {
            let header_style = LayoutStyle {
                width: Dimension::Percent(1.0),
                height: Dimension::Points(48.0),
                ..Default::default()
            };
            let header_node = arena.allocate_node(header_style);
            state.header_nodes.push(header_node);
            arena.add_child(root, header_node);

            if state.open_items.contains(&item.id) {
                let content_style = LayoutStyle {
                    width: Dimension::Percent(1.0),
                    height: Dimension::Points(72.0),
                    ..Default::default()
                };
                let content_node = arena.allocate_node(content_style);
                state.content_nodes.push(Some(content_node));
                arena.add_child(root, content_node);
            } else {
                state.content_nodes.push(None);
            }
        }

        root
    }

    fn paint(&self, state: &Self::State, ctx: &mut PaintContext<'_>) {
        for (i, item) in self.items.iter().enumerate() {
            let Some(&header_node) = state.header_nodes.get(i) else {
                continue;
            };
            let header_bounds = ctx
                .arena
                .computed_bounds
                .get(header_node as usize)
                .copied()
                .unwrap_or(Rect::new(0.0, crate::count_to_f32(i) * 48.0, 400.0, 48.0));

            let is_open = state.open_items.contains(&item.id);
            let is_hovered = ctx.is_hovered(header_node);
            let is_focused = ctx.is_focused(header_node);

            if is_focused {
                let ring = Rect::new(
                    header_bounds.origin.x - 1.0,
                    header_bounds.origin.y - 1.0,
                    header_bounds.size.width + 2.0,
                    header_bounds.size.height + 2.0,
                );
                ctx.fill_rounded_rect(ring, ctx.theme.radii.sm, ctx.theme.colors.ring);
            }

            let header_bg = if is_hovered {
                ctx.theme.colors.accent
            } else {
                ctx.theme.colors.card
            };
            ctx.fill_rounded_rect(header_bounds, ctx.theme.radii.sm, header_bg);

            // Title and expand/collapse indicator
            let title_pt = Point::new(
                header_bounds.origin.x + 12.0,
                header_bounds.origin.y + (header_bounds.size.height - 14.0) * 0.5,
            );
            ctx.draw_text(
                &item.title,
                title_pt,
                ctx.theme.typography.body.font_size,
                ctx.theme.colors.card_foreground,
            );

            let chevron = if is_open { "▾" } else { "▸" };
            let chevron_pt = Point::new(
                header_bounds.origin.x + header_bounds.size.width - 24.0,
                header_bounds.origin.y + (header_bounds.size.height - 14.0) * 0.5,
            );
            ctx.draw_text(
                chevron,
                chevron_pt,
                ctx.theme.typography.body.font_size,
                ctx.theme.colors.muted_foreground,
            );

            // Separator line under header
            ctx.draw_line(
                Point::new(
                    header_bounds.origin.x,
                    header_bounds.origin.y + header_bounds.size.height,
                ),
                Point::new(
                    header_bounds.origin.x + header_bounds.size.width,
                    header_bounds.origin.y + header_bounds.size.height,
                ),
                1.0,
                ctx.theme.colors.border,
            );

            // Expanded content panel
            if is_open
                && let Some(Some(content_node)) = state.content_nodes.get(i)
                && let Some(content_bounds) = ctx
                    .arena
                    .computed_bounds
                    .get(*content_node as usize)
                    .copied()
            {
                ctx.fill_rect(content_bounds, ctx.theme.colors.background);
                let content_pt = Point::new(
                    content_bounds.origin.x + 12.0,
                    content_bounds.origin.y + 12.0,
                );
                ctx.draw_text(
                    &item.content,
                    content_pt,
                    ctx.theme.typography.body_small.font_size,
                    ctx.theme.colors.muted_foreground,
                );
            }
        }
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut EventDispatcher) {
        for (i, item) in self.items.iter().enumerate() {
            let Some(&header_node) = state.header_nodes.get(i) else {
                continue;
            };
            let open_sig = self.open_items;
            let item_id = item.id.clone();
            let policy = if self.allow_multiple {
                OpenPolicy::Multiple
            } else {
                OpenPolicy::Single
            };

            // Click (and Enter 13 / Space 32 via EventDispatcher) toggles expand/collapse
            dispatcher.register_click(
                header_node,
                Arc::new(move || {
                    toggle_item_signal(open_sig, &item_id, policy);
                }),
            );

            // Explicit key handler for Space (32) and Enter (13)
            let open_sig_key = self.open_items;
            let item_id_key = item.id.clone();
            dispatcher.register_key_down(
                header_node,
                Arc::new(move |key_code| {
                    if key_code == 13 || key_code == 32 {
                        toggle_item_signal(open_sig_key, &item_id_key, policy);
                    }
                }),
            );
        }
    }
}
