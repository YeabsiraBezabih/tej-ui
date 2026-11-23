//! Visual rendering context and paint dispatch.

use crate::theme::Theme;
use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{LayoutArena, Point, Rect};

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
}

impl<'a> std::fmt::Debug for PaintContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PaintContext")
            .field("theme", &self.theme)
            .field("dirty_rects", &self.dirty_rects)
            .field("focused_node", &self.focused_node)
            .field("hovered_node", &self.hovered_node)
            .field("pressed_node", &self.pressed_node)
            .field("scroll_offset_y", &self.scroll_offset_y)
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
            min_scroll_x: f32::MAX,
        }
    }

    /// Attaches interaction state and scroll offset from the application runner.
    pub fn with_scroll(mut self, scroll_y: f32, min_x: f32) -> Self {
        self.scroll_offset_y = scroll_y;
        self.min_scroll_x = min_x;
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

    #[must_use]
    fn adjust_rect(&self, rect: Rect) -> Rect {
        if self.scroll_offset_y != 0.0
            && rect.origin.x >= self.min_scroll_x
            && rect.origin.y >= 50.0
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
        if self.scroll_offset_y != 0.0 && pt.x >= self.min_scroll_x && pt.y >= 50.0 {
            Point::new(pt.x, pt.y - self.scroll_offset_y)
        } else {
            pt
        }
    }

    #[must_use]
    fn should_clip_scroll(&self, x: f32, y: f32) -> bool {
        self.scroll_offset_y > 0.0 && x >= self.min_scroll_x && y <= 48.0
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
        if self.should_clip_scroll(r.origin.x, r.origin.y + r.size.height) {
            return;
        }
        if self.is_dirty(r) {
            self.compositor.fill_rect(r, color);
        }
    }

    /// Fills a rounded rectangle if within invalidated bounds.
    pub fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: Color) {
        let r = self.adjust_rect(rect);
        if self.should_clip_scroll(r.origin.x, r.origin.y + r.size.height) {
            return;
        }
        if self.is_dirty(r) {
            self.compositor.fill_rounded_rect(r, radius, color);
        }
    }

    /// Draws a text string if within invalidated bounds.
    pub fn draw_text(&mut self, text: &str, origin: Point, font_size: f32, color: Color) {
        let pt = self.adjust_point(origin);
        if self.should_clip_scroll(pt.x, pt.y) {
            return;
        }
        let approx_w = (text.len() as f32) * font_size * 0.6;
        let bounds = Rect::new(pt.x, pt.y, approx_w, font_size * 1.2);
        if self.is_dirty(bounds) {
            self.compositor.draw_text(text, pt, font_size, color);
        }
    }

    /// Draws a stroke line if within invalidated bounds.
    pub fn draw_line(&mut self, p1: Point, p2: Point, width: f32, color: Color) {
        let pt1 = self.adjust_point(p1);
        let pt2 = self.adjust_point(p2);
        if self.should_clip_scroll(pt1.x.min(pt2.x), pt1.y.max(pt2.y)) {
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
