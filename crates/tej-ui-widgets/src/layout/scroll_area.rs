//! Custom scrollable container viewport (shadcn `ScrollArea`).

use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_hal::UiEvent;
use tej_ui_layout::{Dimension, LayoutArena, LayoutStyle, Point, Rect, Size};
use tej_ui_view::{EventDispatcher, EventPropagation, PaintContext, View};

/// Retained state for a mounted ScrollArea.
#[derive(Debug, Default)]
pub struct ScrollAreaState {
    pub viewport_bounds: Rect,
    pub scroll_offset_y: f32,
    pub scroll_offset_x: f32,
    pub node: u32,
    pub v_thumb_rect: Option<Rect>,
    pub h_thumb_rect: Option<Rect>,
}

/// Custom scrollable viewport with kinetic inertia and scrollbar track bounds.
#[derive(Debug, Clone)]
pub struct ScrollArea {
    scroll_x: Signal<f32>,
    scroll_y: Signal<f32>,
    content_size: Size,
    viewport_size: Size,
}

impl ScrollArea {
    /// Creates a new scroll area.
    #[must_use]
    pub const fn new(
        scroll_x: Signal<f32>,
        scroll_y: Signal<f32>,
        content_size: Size,
        viewport_size: Size,
    ) -> Self {
        Self {
            scroll_x,
            scroll_y,
            content_size,
            viewport_size,
        }
    }

    /// Scrolls vertically by delta points.
    pub fn scroll_by(&self, delta_x: f32, delta_y: f32) {
        apply_scroll(
            self.scroll_x,
            self.scroll_y,
            self.content_size,
            self.viewport_size,
            delta_x,
            delta_y,
        );
    }

    /// Computes vertical scrollbar thumb rect clipped within `viewport`.
    #[must_use]
    pub fn vertical_thumb_rect(&self, viewport: Rect, scroll_y: f32) -> Option<Rect> {
        if self.content_size.height <= viewport.size.height || viewport.size.height <= 4.0 {
            return None;
        }
        let track_w = 8.0;
        let track_x =
            (viewport.origin.x + viewport.size.width - track_w - 2.0).max(viewport.origin.x);
        let track_y = viewport.origin.y + 2.0;
        let track_h = (viewport.size.height - 4.0).max(1.0);

        let ratio = (viewport.size.height / self.content_size.height).clamp(0.05, 1.0);
        let thumb_h = (track_h * ratio).max(20.0).min(track_h);
        let max_scroll = (self.content_size.height - viewport.size.height).max(1.0);
        let progress = (scroll_y / max_scroll).clamp(0.0, 1.0);
        let thumb_y = track_y + progress * (track_h - thumb_h);

        clip_rect_to_viewport(Rect::new(track_x, thumb_y, track_w, thumb_h), viewport)
    }

    /// Computes horizontal scrollbar thumb rect clipped within `viewport`.
    #[must_use]
    pub fn horizontal_thumb_rect(&self, viewport: Rect, scroll_x: f32) -> Option<Rect> {
        if self.content_size.width <= viewport.size.width || viewport.size.width <= 4.0 {
            return None;
        }
        let track_h = 8.0;
        let track_x = viewport.origin.x + 2.0;
        let track_y =
            (viewport.origin.y + viewport.size.height - track_h - 2.0).max(viewport.origin.y);
        let track_w = (viewport.size.width - 4.0).max(1.0);

        let ratio = (viewport.size.width / self.content_size.width).clamp(0.05, 1.0);
        let thumb_w = (track_w * ratio).max(20.0).min(track_w);
        let max_scroll = (self.content_size.width - viewport.size.width).max(1.0);
        let progress = (scroll_x / max_scroll).clamp(0.0, 1.0);
        let thumb_x = track_x + progress * (track_w - thumb_w);

        clip_rect_to_viewport(Rect::new(thumb_x, track_y, thumb_w, track_h), viewport)
    }
}

fn apply_scroll(
    scroll_x: Signal<f32>,
    scroll_y: Signal<f32>,
    content_size: Size,
    viewport_size: Size,
    delta_x: f32,
    delta_y: f32,
) {
    let max_x = (content_size.width - viewport_size.width).max(0.0);
    let max_y = (content_size.height - viewport_size.height).max(0.0);

    let next_x = (scroll_x.get() + delta_x).clamp(0.0, max_x);
    let next_y = (scroll_y.get() + delta_y).clamp(0.0, max_y);

    scroll_x.set(next_x);
    scroll_y.set(next_y);
}

fn clip_rect_to_viewport(rect: Rect, viewport: Rect) -> Option<Rect> {
    let x1 = rect.origin.x.max(viewport.origin.x);
    let y1 = rect.origin.y.max(viewport.origin.y);
    let x2 = (rect.origin.x + rect.size.width).min(viewport.origin.x + viewport.size.width);
    let y2 = (rect.origin.y + rect.size.height).min(viewport.origin.y + viewport.size.height);
    if x2 > x1 && y2 > y1 {
        Some(Rect::new(x1, y1, x2 - x1, y2 - y1))
    } else {
        None
    }
}

impl View for ScrollArea {
    type State = ScrollAreaState;

    fn build(&self) -> Self::State {
        let viewport_bounds = Rect::from_origin_size(Point::ZERO, self.viewport_size);
        let sx = self.scroll_x.get();
        let sy = self.scroll_y.get();
        ScrollAreaState {
            viewport_bounds,
            scroll_offset_x: sx,
            scroll_offset_y: sy,
            node: 0,
            v_thumb_rect: self.vertical_thumb_rect(viewport_bounds, sy),
            h_thumb_rect: self.horizontal_thumb_rect(viewport_bounds, sx),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.scroll_offset_x = self.scroll_x.get();
        state.scroll_offset_y = self.scroll_y.get();
        state.v_thumb_rect = self.vertical_thumb_rect(state.viewport_bounds, state.scroll_offset_y);
        state.h_thumb_rect =
            self.horizontal_thumb_rect(state.viewport_bounds, state.scroll_offset_x);
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut LayoutArena) -> u32 {
        let style = LayoutStyle {
            width: Dimension::Points(self.viewport_size.width),
            height: Dimension::Points(self.viewport_size.height),
            ..Default::default()
        };
        let node = arena.allocate_node(style);
        state.node = node;
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(state.viewport_bounds);

        // Viewport container surface clipped strictly to bounds
        ctx.fill_rounded_rect(bounds, ctx.theme.radii.sm, ctx.theme.colors.border);
        if let Some(inner) = clip_rect_to_viewport(
            Rect::new(
                bounds.origin.x + 1.0,
                bounds.origin.y + 1.0,
                (bounds.size.width - 2.0).max(0.0),
                (bounds.size.height - 2.0).max(0.0),
            ),
            bounds,
        ) {
            ctx.fill_rect(inner, ctx.theme.colors.background);
        }

        // Vertical scrollbar track + thumb
        if let Some(thumb) = self.vertical_thumb_rect(bounds, state.scroll_offset_y) {
            let track = Rect::new(
                thumb.origin.x,
                bounds.origin.y + 2.0,
                thumb.size.width,
                (bounds.size.height - 4.0).max(0.0),
            );
            if let Some(clipped_track) = clip_rect_to_viewport(track, bounds) {
                ctx.fill_rounded_rect(clipped_track, 4.0, ctx.theme.colors.muted);
            }
            ctx.fill_rounded_rect(thumb, 4.0, ctx.theme.colors.muted_foreground);
        }

        // Horizontal scrollbar track + thumb
        if let Some(thumb) = self.horizontal_thumb_rect(bounds, state.scroll_offset_x) {
            let track = Rect::new(
                bounds.origin.x + 2.0,
                thumb.origin.y,
                (bounds.size.width - 4.0).max(0.0),
                thumb.size.height,
            );
            if let Some(clipped_track) = clip_rect_to_viewport(track, bounds) {
                ctx.fill_rounded_rect(clipped_track, 4.0, ctx.theme.colors.muted);
            }
            ctx.fill_rounded_rect(thumb, 4.0, ctx.theme.colors.muted_foreground);
        }
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut EventDispatcher) {
        dispatcher.register_click(state.node, Arc::new(|| {}));

        let sx = self.scroll_x;
        let sy = self.scroll_y;
        let content_size = self.content_size;
        let viewport_size = self.viewport_size;
        dispatcher.register_bubble(
            state.node,
            Arc::new(move |event, _phase, _node| {
                if let UiEvent::Scroll {
                    delta_x, delta_y, ..
                } = event
                {
                    apply_scroll(sx, sy, content_size, viewport_size, *delta_x, *delta_y);
                    return EventPropagation::Repaint;
                }
                EventPropagation::Continue
            }),
        );

        let sx_key = self.scroll_x;
        let sy_key = self.scroll_y;
        dispatcher.register_key_down(
            state.node,
            Arc::new(move |key_code| match key_code {
                37 => apply_scroll(sx_key, sy_key, content_size, viewport_size, -40.0, 0.0),
                38 => apply_scroll(sx_key, sy_key, content_size, viewport_size, 0.0, -40.0),
                39 => apply_scroll(sx_key, sy_key, content_size, viewport_size, 40.0, 0.0),
                40 => apply_scroll(sx_key, sy_key, content_size, viewport_size, 0.0, 40.0),
                _ => {}
            }),
        );
    }
}
