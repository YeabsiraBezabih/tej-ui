//! Slide-over drawer panel (shadcn `Sheet`).

use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_hal::Color;
use tej_ui_layout::{
    Dimension, FlexDirection, Insets, JustifyContent, LayoutArena, LayoutStyle, Point, Rect, Size,
};
use tej_ui_view::{EventDispatcher, PaintContext, View};

/// Edge from which the sheet drawer slides out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetSide {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}

/// Retained state for a mounted Sheet.
#[derive(Debug, Default)]
pub struct SheetState {
    pub bounds: Rect,
    pub is_open: bool,
    pub node: u32,
    pub panel_node: u32,
    pub close_node: u32,
}

/// Slide-over drawer panel anchored to a screen edge.
#[derive(Debug, Clone)]
pub struct Sheet {
    is_open: Signal<bool>,
    side: SheetSide,
    size_points: f32,
    title: String,
}

impl Sheet {
    /// Creates a new sheet drawer.
    #[must_use]
    pub fn new(is_open: Signal<bool>, title: impl Into<String>) -> Self {
        Self {
            is_open,
            side: SheetSide::Right,
            size_points: 380.0,
            title: title.into(),
        }
    }

    /// Sets edge side from which the sheet slides.
    #[must_use]
    pub const fn side(mut self, side: SheetSide) -> Self {
        self.side = side;
        self
    }

    /// Sets drawer width or height in points.
    #[must_use]
    pub const fn size(mut self, points: f32) -> Self {
        self.size_points = points;
        self
    }

    /// Title of the sheet drawer.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Closes the sheet.
    pub fn close(&self) {
        self.is_open.set(false);
    }
}

impl View for Sheet {
    type State = SheetState;

    fn build(&self) -> Self::State {
        let is_open = self.is_open.get();
        let size = match self.side {
            SheetSide::Left | SheetSide::Right => Size::new(self.size_points, 768.0),
            SheetSide::Top | SheetSide::Bottom => Size::new(1024.0, self.size_points),
        };

        SheetState {
            bounds: Rect::from_origin_size(Point::ZERO, size),
            is_open,
            node: 0,
            panel_node: 0,
            close_node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.is_open = self.is_open.get();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        if !state.is_open {
            let empty = arena.allocate_node(LayoutStyle {
                width: Dimension::Points(0.0),
                height: Dimension::Points(0.0),
                ..Default::default()
            });
            state.node = empty;
            state.panel_node = empty;
            state.close_node = empty;
            return empty;
        }

        let (direction, justify_content) = match self.side {
            SheetSide::Left => (FlexDirection::Row, JustifyContent::FlexStart),
            SheetSide::Right => (FlexDirection::Row, JustifyContent::FlexEnd),
            SheetSide::Top => (FlexDirection::Column, JustifyContent::FlexStart),
            SheetSide::Bottom => (FlexDirection::Column, JustifyContent::FlexEnd),
        };

        let backdrop_style = LayoutStyle {
            direction,
            justify_content,
            width: Dimension::Percent(1.0),
            height: Dimension::Percent(1.0),
            ..Default::default()
        };
        let root = arena.allocate_node(backdrop_style);
        state.node = root;

        let (panel_w, panel_h) = match self.side {
            SheetSide::Left | SheetSide::Right => {
                (Dimension::Points(self.size_points), Dimension::Percent(1.0))
            }
            SheetSide::Top | SheetSide::Bottom => {
                (Dimension::Percent(1.0), Dimension::Points(self.size_points))
            }
        };

        let panel_style = LayoutStyle {
            direction: FlexDirection::Column,
            width: panel_w,
            height: panel_h,
            padding: Insets::all(16.0),
            ..Default::default()
        };
        let panel_node = arena.allocate_node(panel_style);
        state.panel_node = panel_node;
        arena.add_child(root, panel_node);

        let close_style = LayoutStyle {
            width: Dimension::Points(28.0),
            height: Dimension::Points(28.0),
            ..Default::default()
        };
        let close_node = arena.allocate_node(close_style);
        state.close_node = close_node;
        arena.add_child(panel_node, close_node);

        root
    }

    fn paint(&self, state: &Self::State, ctx: &mut PaintContext<'_>) {
        if !state.is_open {
            return;
        }

        // Dimmed backdrop covering viewport
        if let Some(backdrop_bounds) = ctx.arena.computed_bounds.get(state.node as usize).copied() {
            ctx.fill_rect(backdrop_bounds, Color::rgba(0, 0, 0, 140));
        }

        // Drawer panel surface
        let panel_bounds = ctx
            .arena
            .computed_bounds
            .get(state.panel_node as usize)
            .copied()
            .unwrap_or(state.bounds);

        ctx.fill_rect(panel_bounds, ctx.theme.colors.border);
        let inner = Rect::new(
            panel_bounds.origin.x + 1.0,
            panel_bounds.origin.y + 1.0,
            (panel_bounds.size.width - 2.0).max(0.0),
            (panel_bounds.size.height - 2.0).max(0.0),
        );
        ctx.fill_rect(inner, ctx.theme.colors.card);

        // Header title
        let title_pt = Point::new(panel_bounds.origin.x + 16.0, panel_bounds.origin.y + 18.0);
        ctx.draw_text(
            &self.title,
            title_pt,
            ctx.theme.typography.h3.font_size,
            ctx.theme.colors.card_foreground,
        );

        // Close button
        if let Some(close_bounds) = ctx
            .arena
            .computed_bounds
            .get(state.close_node as usize)
            .copied()
        {
            let close_bg = if ctx.is_hovered(state.close_node) {
                ctx.theme.colors.accent
            } else {
                ctx.theme.colors.secondary
            };
            ctx.fill_rounded_rect(close_bounds, ctx.theme.radii.sm, close_bg);
            ctx.draw_text(
                "X",
                Point::new(close_bounds.origin.x + 9.0, close_bounds.origin.y + 7.0),
                ctx.theme.typography.body_small.font_size,
                ctx.theme.colors.foreground,
            );
        }

        // Header divider line
        let div_y = panel_bounds.origin.y + 52.0;
        ctx.draw_line(
            Point::new(panel_bounds.origin.x, div_y),
            Point::new(panel_bounds.origin.x + panel_bounds.size.width, div_y),
            1.0,
            ctx.theme.colors.border,
        );
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut EventDispatcher) {
        if !state.is_open {
            return;
        }

        // Backdrop click dismisses sheet
        let is_open_backdrop = self.is_open;
        dispatcher.register_click(
            state.node,
            Arc::new(move || {
                is_open_backdrop.set(false);
            }),
        );

        // Panel absorbs clicks so clicking inside drawer does not dismiss
        dispatcher.register_click(state.panel_node, Arc::new(|| {}));

        // Close button dismisses sheet
        let is_open_close = self.is_open;
        dispatcher.register_click(
            state.close_node,
            Arc::new(move || {
                is_open_close.set(false);
            }),
        );

        // Escape key (27) dismisses sheet from backdrop, panel, or close button
        for target in [state.node, state.panel_node, state.close_node] {
            let is_open_esc = self.is_open;
            dispatcher.register_key_down(
                target,
                Arc::new(move |key_code| {
                    if key_code == 27 {
                        is_open_esc.set(false);
                    }
                }),
            );
        }
    }
}
