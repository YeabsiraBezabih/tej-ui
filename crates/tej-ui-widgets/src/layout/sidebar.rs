//! Collapsible application sidebar (shadcn `Sidebar`).

use tej_ui_core::Signal;
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::View;

/// An item in the application sidebar navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub is_active: bool,
}

impl SidebarItem {
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            badge: None,
            is_active: false,
        }
    }

    #[must_use]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    #[must_use]
    pub fn badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    #[must_use]
    pub const fn mark_active(mut self) -> Self {
        self.is_active = true;
        self
    }

    #[must_use]
    pub const fn inactive(mut self) -> Self {
        self.is_active = false;
        self
    }
}

/// Retained state for a mounted Sidebar.
#[derive(Debug, Default)]
pub struct SidebarState {
    pub bounds: Rect,
    pub is_collapsed: bool,
    pub active_width: f32,
    pub node: u32,
    pub item_nodes: Vec<u32>,
}

/// Collapsible desktop application sidebar layout.
#[derive(Debug, Clone)]
pub struct Sidebar {
    is_collapsed: Signal<bool>,
    expanded_width: f32,
    collapsed_width: f32,
    items: Vec<SidebarItem>,
}

impl Sidebar {
    /// Creates a new sidebar bound to a collapse signal.
    #[must_use]
    pub fn new(is_collapsed: Signal<bool>, items: Vec<SidebarItem>) -> Self {
        Self {
            is_collapsed,
            expanded_width: 240.0,
            collapsed_width: 64.0,
            items,
        }
    }

    /// Sets custom expanded and collapsed widths.
    #[must_use]
    pub const fn widths(mut self, expanded: f32, collapsed: f32) -> Self {
        self.expanded_width = expanded;
        self.collapsed_width = collapsed;
        self
    }

    /// Returns sidebar navigation items.
    #[must_use]
    pub fn items(&self) -> &[SidebarItem] {
        &self.items
    }

    /// Toggles the collapsed state.
    pub fn toggle_collapse(&self) {
        self.is_collapsed.set(!self.is_collapsed.get());
    }
}

impl View for Sidebar {
    type State = SidebarState;

    fn build(&self) -> Self::State {
        let is_collapsed = self.is_collapsed.get();
        let width = if is_collapsed {
            self.collapsed_width
        } else {
            self.expanded_width
        };

        SidebarState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(width, 768.0)),
            is_collapsed,
            active_width: width,
            node: 0,
            item_nodes: Vec::new(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let is_collapsed = self.is_collapsed.get();
        let width = if is_collapsed {
            self.collapsed_width
        } else {
            self.expanded_width
        };
        state.is_collapsed = is_collapsed;
        state.active_width = width;
        state.bounds.size.width = width;
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let width = if self.is_collapsed.get() {
            self.collapsed_width
        } else {
            self.expanded_width
        };
        let sidebar_style = tej_ui_layout::LayoutStyle {
            width: tej_ui_layout::Dimension::Points(width),
            height: tej_ui_layout::Dimension::Percent(1.0),
            direction: tej_ui_layout::FlexDirection::Column,
            padding: tej_ui_layout::Insets::all(8.0),
            ..Default::default()
        };
        let root = arena.allocate_node(sidebar_style);
        state.node = root;
        state.item_nodes.clear();

        for _ in 0..self.items.len() {
            let item_style = tej_ui_layout::LayoutStyle {
                width: tej_ui_layout::Dimension::Percent(1.0),
                height: tej_ui_layout::Dimension::Points(36.0),
                ..Default::default()
            };
            let child = arena.allocate_node(item_style);
            state.item_nodes.push(child);
            arena.add_child(root, child);
        }

        root
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(state.bounds);

        // Sidebar background & border
        ctx.fill_rect(bounds, ctx.theme.colors.card);
        let border_r = Rect::new(
            bounds.origin.x + bounds.size.width - 1.0,
            bounds.origin.y,
            1.0,
            bounds.size.height,
        );
        ctx.fill_rect(border_r, ctx.theme.colors.border);

        // Render items
        for (i, item) in self.items.iter().enumerate() {
            if let Some(&item_node) = state.item_nodes.get(i) {
                let item_bounds = ctx
                    .arena
                    .computed_bounds
                    .get(item_node as usize)
                    .copied()
                    .unwrap_or(Rect::new(
                        bounds.origin.x + 8.0,
                        bounds.origin.y + 8.0 + crate::count_to_f32(i) * 40.0,
                        bounds.size.width - 16.0,
                        36.0,
                    ));

                let is_hovered = ctx.is_hovered(item_node);
                if item.is_active {
                    ctx.fill_rounded_rect(item_bounds, 6.0, ctx.theme.colors.accent);
                } else if is_hovered {
                    ctx.fill_rounded_rect(
                        item_bounds,
                        6.0,
                        ctx.theme.colors.accent.with_alpha(100),
                    );
                }

                // Item label clipped to item bounds
                if !state.is_collapsed {
                    let text_color = if item.is_active {
                        ctx.theme.colors.accent_foreground
                    } else {
                        ctx.theme.colors.foreground
                    };
                    let text_pt = Point::new(
                        item_bounds.origin.x + 12.0,
                        item_bounds.origin.y + (item_bounds.size.height - 14.0) * 0.5,
                    );
                    ctx.push_clip(item_bounds, [6.0; 4]);
                    ctx.draw_text(
                        &item.label,
                        text_pt,
                        ctx.theme.typography.body_small.font_size,
                        text_color,
                    );
                    ctx.pop_clip();
                }
            }
        }
    }
}
