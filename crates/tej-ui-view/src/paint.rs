//! Visual rendering context and paint dispatch.

use crate::theme::Theme;
use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{LayoutArena, Point, Rect};

/// Deferred overlay draw closure executed after base content painting.
pub type OverlayCallback = Box<dyn FnOnce(&mut PaintContext<'_>)>;

/// Execution context for recursive view rasterization.
pub struct PaintContext<'a> {
    pub compositor: &'a mut dyn SurfaceCompositor,
    pub arena: &'a LayoutArena,
    pub theme: &'a Theme,
    pub dirty_rects: &'a [Rect],
    pub focused_node: Option<u32>,
    pub hovered_node: Option<u32>,
    pub pressed_node: Option<u32>,
    pub scroll_offset_y: f32,
    pub min_scroll_x: f32,
    pub min_scroll_y: f32,
    deferred_overlays: Vec<OverlayCallback>,
}

impl std::fmt::Debug for PaintContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PaintContext")
            .field("theme", &self.theme)
            .field("dirty_rects", &self.dirty_rects)
            .field("focused_node", &self.focused_node)
            .field("hovered_node", &self.hovered_node)
            .field("pressed_node", &self.pressed_node)
            .field("scroll_offset_y", &self.scroll_offset_y)
            .field("min_scroll_x", &self.min_scroll_x)
            .field("min_scroll_y", &self.min_scroll_y)
            .field("deferred_overlays_count", &self.deferred_overlays.len())
            .finish()
    }
}

impl<'a> PaintContext<'a> {
    /// Constructs a new paint context.
    pub fn new(
        compositor: &'a mut dyn SurfaceCompositor,
        arena: &'a LayoutArena,
        theme: &'a Theme,
        dirty_rects: &'a [Rect],
    ) -> Self {
        Self {
            compositor,
            arena,
            theme,
            dirty_rects,
            focused_node: None,
            hovered_node: None,
            pressed_node: None,
            scroll_offset_y: 0.0,
            min_scroll_x: 0.0,
            min_scroll_y: 0.0,
            deferred_overlays: Vec::new(),
        }
    }

    /// Attaches vertical scroll offset and horizontal scroll region start from the application runner.
    pub fn with_scroll(mut self, scroll_y: f32, min_x: f32) -> Self {
        self.scroll_offset_y = scroll_y;
        self.min_scroll_x = min_x;
        self
    }

    /// Attaches vertical scroll offset and configurable scroll viewport origin `(min_x, min_y)`.
    pub fn with_scroll_viewport(mut self, scroll_y: f32, min_x: f32, min_y: f32) -> Self {
        self.scroll_offset_y = scroll_y;
        self.min_scroll_x = min_x;
        self.min_scroll_y = min_y;
        self
    }

    /// Attaches interaction state from the event dispatcher.
    pub fn with_interaction(
        mut self,
        focused: Option<u32>,
        hovered: Option<u32>,
        pressed: Option<u32>,
    ) -> Self {
        self.focused_node = focused;
        self.hovered_node = hovered;
        self.pressed_node = pressed;
        self
    }

    /// Queues an overlay paint closure (modal, popover, tooltip) to be rendered
    /// after all base content during [`PaintContext::flush_overlays`].
    pub fn defer_overlay(&mut self, callback: OverlayCallback) {
        self.deferred_overlays.push(callback);
    }

    /// Returns the number of queued overlay callbacks awaiting flush.
    #[must_use]
    pub fn pending_overlay_count(&self) -> usize {
        self.deferred_overlays.len()
    }

    /// Executes all queued overlay paint callbacks in FIFO order above base content.
    ///
    /// Nested overlays scheduled while flushing are also drained in order.
    pub fn flush_overlays(&mut self) {
        while !self.deferred_overlays.is_empty() {
            let pending = std::mem::take(&mut self.deferred_overlays);
            for draw_overlay in pending {
                draw_overlay(self);
            }
        }
    }

    #[must_use]
    fn adjust_rect(&self, rect: Rect) -> Rect {
        if self.scroll_offset_y != 0.0
            && rect.origin.x >= self.min_scroll_x
            && rect.origin.y >= self.min_scroll_y
        {
            Rect::new(
                rect.origin.x,
                rect.origin.y - self.scroll_offset_y,
                rect.size.width,
                rect.size.height,
            )
        } else {
            rect
        }
    }

    #[must_use]
    fn adjust_point(&self, pt: Point) -> Point {
        if self.scroll_offset_y != 0.0 && pt.x >= self.min_scroll_x && pt.y >= self.min_scroll_y {
            Point::new(pt.x, pt.y - self.scroll_offset_y)
        } else {
            pt
        }
    }

    #[must_use]
    fn should_clip_scroll(&self, x: f32, y: f32) -> bool {
        self.scroll_offset_y > 0.0 && x >= self.min_scroll_x && y <= self.min_scroll_y
    }

    /// Returns `true` if `node` has keyboard focus.
    #[must_use]
    pub fn is_focused(&self, node: u32) -> bool {
        self.focused_node == Some(node)
    }

    /// Returns `true` if `node` (or any of its descendants) is hovered by the mouse.
    #[must_use]
    pub fn is_hovered(&self, node: u32) -> bool {
        if let Some(mut curr) = self.hovered_node {
            loop {
                if curr == node {
                    return true;
                }
                if let Some(Some(parent)) = self.arena.parents.get(curr as usize) {
                    curr = *parent;
                } else {
                    break;
                }
            }
        }
        false
    }

    /// Returns `true` if `node` (or any of its descendants) is actively pressed.
    #[must_use]
    pub fn is_pressed(&self, node: u32) -> bool {
        if let Some(mut curr) = self.pressed_node {
            loop {
                if curr == node {
                    return true;
                }
                if let Some(Some(parent)) = self.arena.parents.get(curr as usize) {
                    curr = *parent;
                } else {
                    break;
                }
            }
        }
        false
    }

    /// Returns `true` if `bounds` overlaps any active dirty rectangle.
    #[must_use]
    pub fn is_dirty(&self, bounds: Rect) -> bool {
        if self.dirty_rects.is_empty() {
            return true;
        }
        self.dirty_rects.iter().any(|d| d.intersects(&bounds))
    }

    /// Fills a solid rectangle if within invalidated bounds.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let r = self.adjust_rect(rect);
        if rect.origin.y >= self.min_scroll_y
            && self.should_clip_scroll(r.origin.x, r.origin.y + r.size.height)
        {
            return;
        }
        if self.is_dirty(r) {
            self.compositor.fill_rect(r, color);
        }
    }

    /// Fills a rounded rectangle if within invalidated bounds.
    pub fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: Color) {
        let r = self.adjust_rect(rect);
        if rect.origin.y >= self.min_scroll_y
            && self.should_clip_scroll(r.origin.x, r.origin.y + r.size.height)
        {
            return;
        }
        if self.is_dirty(r) {
            self.compositor.fill_rounded_rect(r, radius, color);
        }
    }

    /// Draws a text string if within invalidated bounds.
    pub fn draw_text(&mut self, text: &str, origin: Point, font_size: f32, color: Color) {
        let pt = self.adjust_point(origin);
        if origin.y >= self.min_scroll_y && self.should_clip_scroll(pt.x, pt.y) {
            return;
        }
        let approx_w = (text.chars().count() as f32) * font_size * 0.6;
        let bounds = Rect::new(pt.x, pt.y, approx_w, font_size * 1.2);
        if self.is_dirty(bounds) {
            self.compositor.draw_text(text, pt, font_size, color);
        }
    }

    /// Draws a stroke line if within invalidated bounds.
    pub fn draw_line(&mut self, p1: Point, p2: Point, width: f32, color: Color) {
        let pt1 = self.adjust_point(p1);
        let pt2 = self.adjust_point(p2);
        if p1.y.min(p2.y) >= self.min_scroll_y
            && self.should_clip_scroll(pt1.x.min(pt2.x), pt1.y.max(pt2.y))
        {
            return;
        }
        let min_x = pt1.x.min(pt2.x);
        let min_y = pt1.y.min(pt2.y);
        let max_x = pt1.x.max(pt2.x);
        let max_y = pt1.y.max(pt2.y);
        let bounds = Rect::new(
            min_x,
            min_y,
            (max_x - min_x).max(width),
            (max_y - min_y).max(width),
        );
        if self.is_dirty(bounds) {
            self.compositor.draw_line(pt1, pt2, width, color);
        }
    }
}
