//! Visual rendering context and paint dispatch.

use crate::css::{
    TextAlign, TextDecorationLine, TextOverflow, TextTransform, WhiteSpace, WordBreak, WritingMode,
};
use crate::theme::Theme;
use crate::typography_extended::{CaretStyle, FontVariantNumeric, TextUnderlineStyle, TextWrap};
use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{LayoutArena, Point, Rect};

/// Deferred overlay draw closure executed after base content painting.
pub type OverlayCallback = Box<dyn FnOnce(&mut PaintContext<'_>)>;

/// Inherited CSS typography properties propagated down the view paint tree.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct InheritedTextStyle {
    pub color: Option<Color>,
    pub font_size: Option<f32>,
    pub font_weight: Option<u16>,
    pub italic: Option<bool>,
    pub monospace: Option<bool>,
    pub line_height: Option<f32>,
    pub letter_spacing: Option<f32>,
    pub word_spacing: Option<f32>,
    pub text_align: Option<TextAlign>,
    pub text_decoration: Option<TextDecorationLine>,
    pub text_underline: Option<TextUnderlineStyle>,
    pub text_transform: Option<TextTransform>,
    pub white_space: Option<WhiteSpace>,
    pub text_overflow: Option<TextOverflow>,
    pub word_break: Option<WordBreak>,
    pub overflow_wrap_anywhere: Option<bool>,
    pub writing_mode: Option<WritingMode>,
    pub text_wrap: Option<TextWrap>,
    pub text_indent: Option<f32>,
    pub tab_size: Option<u8>,
    pub caret: Option<CaretStyle>,
    pub font_variant_numeric: Option<FontVariantNumeric>,
    pub font_optical_sizing_auto: Option<bool>,
}

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
    foreground_stack: Vec<Color>,
    text_style_stack: Vec<InheritedTextStyle>,
    transform_stack: Vec<(f32, f32)>,
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
            .field("foreground_stack_len", &self.foreground_stack.len())
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
            foreground_stack: Vec::new(),
            text_style_stack: Vec::new(),
            transform_stack: Vec::new(),
            deferred_overlays: Vec::new(),
        }
    }

    /// Pushes an inherited foreground color onto the foreground stack.
    pub fn push_foreground(&mut self, color: Color) {
        self.foreground_stack.push(color);
    }

    /// Pops the most recently pushed foreground color from the stack.
    pub fn pop_foreground(&mut self) {
        self.foreground_stack.pop();
    }

    /// Returns the active inherited foreground color, or `theme.colors.foreground` if empty.
    #[must_use]
    pub fn current_foreground(&self) -> Color {
        self.foreground_stack
            .last()
            .copied()
            .unwrap_or(self.theme.colors.foreground)
    }

    /// Pushes inherited CSS typography properties onto the text style stack, merging with parent values.
    pub fn push_text_style(&mut self, style: InheritedTextStyle) {
        let merged = if let Some(&parent) = self.text_style_stack.last() {
            InheritedTextStyle {
                color: style.color.or(parent.color),
                font_size: style.font_size.or(parent.font_size),
                font_weight: style.font_weight.or(parent.font_weight),
                italic: style.italic.or(parent.italic),
                monospace: style.monospace.or(parent.monospace),
                line_height: style.line_height.or(parent.line_height),
                letter_spacing: style.letter_spacing.or(parent.letter_spacing),
                word_spacing: style.word_spacing.or(parent.word_spacing),
                text_align: style.text_align.or(parent.text_align),
                text_decoration: style.text_decoration.or(parent.text_decoration),
                text_underline: style.text_underline.or(parent.text_underline),
                text_transform: style.text_transform.or(parent.text_transform),
                white_space: style.white_space.or(parent.white_space),
                text_overflow: style.text_overflow.or(parent.text_overflow),
                word_break: style.word_break.or(parent.word_break),
                overflow_wrap_anywhere: style
                    .overflow_wrap_anywhere
                    .or(parent.overflow_wrap_anywhere),
                writing_mode: style.writing_mode.or(parent.writing_mode),
                text_wrap: style.text_wrap.or(parent.text_wrap),
                text_indent: style.text_indent.or(parent.text_indent),
                tab_size: style.tab_size.or(parent.tab_size),
                caret: style.caret.or(parent.caret),
                font_variant_numeric: style.font_variant_numeric.or(parent.font_variant_numeric),
                font_optical_sizing_auto: style
                    .font_optical_sizing_auto
                    .or(parent.font_optical_sizing_auto),
            }
        } else {
            style
        };
        self.text_style_stack.push(merged);
    }

    /// Pops the most recently pushed inherited text style from the stack.
    pub fn pop_text_style(&mut self) {
        self.text_style_stack.pop();
    }

    /// Returns the active inherited CSS text style, if any.
    #[must_use]
    pub fn current_text_style(&self) -> Option<InheritedTextStyle> {
        self.text_style_stack.last().copied()
    }

    /// Pushes an accumulated 2D translation offset onto the transform stack.
    pub fn push_transform(&mut self, dx: f32, dy: f32) {
        let (px, py) = self.current_transform();
        self.transform_stack.push((px + dx, py + dy));
    }

    /// Pops the most recently pushed 2D translation offset.
    pub fn pop_transform(&mut self) {
        self.transform_stack.pop();
    }

    /// Returns the current accumulated 2D translation offset `(dx, dy)`.
    #[must_use]
    pub fn current_transform(&self) -> (f32, f32) {
        self.transform_stack.last().copied().unwrap_or((0.0, 0.0))
    }

    /// Pushes a scroll-adjusted clipping rectangle onto the compositor clip stack.
    pub fn push_clip(&mut self, rect: Rect, corner_radii: [f32; 4]) {
        let adjusted = self.adjust_rect(rect);
        self.compositor.push_clip(adjusted, corner_radii);
    }

    /// Pops the most recently pushed clipping rectangle from the compositor clip stack.
    pub fn pop_clip(&mut self) {
        self.compositor.pop_clip();
    }

    /// Attaches vertical scroll offset and horizontal scroll region start from the application runner.
    #[must_use]
    pub fn with_scroll(mut self, scroll_y: f32, min_x: f32) -> Self {
        self.scroll_offset_y = scroll_y;
        self.min_scroll_x = min_x;
        self
    }

    /// Attaches vertical scroll offset and configurable scroll viewport origin `(min_x, min_y)`.
    #[must_use]
    pub fn with_scroll_viewport(mut self, scroll_y: f32, min_x: f32, min_y: f32) -> Self {
        self.scroll_offset_y = scroll_y;
        self.min_scroll_x = min_x;
        self.min_scroll_y = min_y;
        self
    }

    /// Attaches interaction state from the event dispatcher.
    #[must_use]
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
        let (tx, ty) = self.current_transform();
        let scroll_dy = if self.scroll_offset_y != 0.0
            && rect.origin.x >= self.min_scroll_x
            && rect.origin.y >= self.min_scroll_y
        {
            self.scroll_offset_y
        } else {
            0.0
        };
        if tx == 0.0 && ty == 0.0 && scroll_dy == 0.0 {
            rect
        } else {
            Rect::new(
                rect.origin.x + tx,
                rect.origin.y + ty - scroll_dy,
                rect.size.width,
                rect.size.height,
            )
        }
    }

    #[must_use]
    fn adjust_point(&self, pt: Point) -> Point {
        let (tx, ty) = self.current_transform();
        let scroll_dy = if self.scroll_offset_y != 0.0
            && pt.x >= self.min_scroll_x
            && pt.y >= self.min_scroll_y
        {
            self.scroll_offset_y
        } else {
            0.0
        };
        if tx == 0.0 && ty == 0.0 && scroll_dy == 0.0 {
            pt
        } else {
            Point::new(pt.x + tx, pt.y + ty - scroll_dy)
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
        let approx_w =
            f32::from(u16::try_from(text.chars().count()).unwrap_or(u16::MAX)) * font_size * 0.6;
        let bounds = Rect::new(pt.x, pt.y, approx_w, font_size * 1.2);
        if self.is_dirty(bounds) {
            self.compositor.draw_text(text, pt, font_size, color);
        }
    }

    /// Draws a styled text string (with explicit font weight and family) if within
    /// invalidated bounds.
    pub fn draw_text_styled(
        &mut self,
        text: &str,
        origin: Point,
        font_size: f32,
        weight: u16,
        family: tej_ui_hal::TextFamily,
        color: Color,
    ) {
        let pt = self.adjust_point(origin);
        if origin.y >= self.min_scroll_y && self.should_clip_scroll(pt.x, pt.y) {
            return;
        }
        let approx_w =
            f32::from(u16::try_from(text.chars().count()).unwrap_or(u16::MAX)) * font_size * 0.6;
        let bounds = Rect::new(pt.x, pt.y, approx_w, font_size * 1.2);
        if self.is_dirty(bounds) {
            self.compositor
                .draw_text_styled(text, pt, font_size, weight, family, color);
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
