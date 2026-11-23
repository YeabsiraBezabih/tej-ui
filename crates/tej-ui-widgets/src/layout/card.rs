//! Container card component (shadcn `Card`).

use std::fmt::Debug;
use tej_ui_hal::Color;
use tej_ui_layout::{Insets, Point, Rect, Size};
use tej_ui_view::View;

/// Retained state for a mounted Card.
#[derive(Debug, Default)]
pub struct CardState<S: Debug> {
    pub bounds: Rect,
    pub title: String,
    pub node: u32,
    pub child_state: S,
}

/// Structural container card with header, title, description, content, and footer.
#[derive(Debug, Clone)]
pub struct Card<V: View = ()> {
    pub title: String,
    pub description: Option<String>,
    pub footer: Option<String>,
    pub width: tej_ui_layout::Dimension,
    pub height: tej_ui_layout::Dimension,
    pub flex_grow: f32,
    pub padding: Insets,
    pub radius: Option<f32>,
    pub border_color: Option<Color>,
    pub child: V,
}

impl Card<()> {
    /// Creates a new card with a title.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            footer: None,
            width: tej_ui_layout::Dimension::Auto,
            height: tej_ui_layout::Dimension::Auto,
            flex_grow: 0.0,
            padding: Insets::all(16.0),
            radius: None,
            border_color: None,
            child: (),
        }
    }

    /// Creates a container card wrapping arbitrary child content.
    #[must_use]
    pub fn wrap<V: View>(child: V) -> Card<V> {
        Card {
            title: String::new(),
            description: None,
            footer: None,
            width: tej_ui_layout::Dimension::Auto,
            height: tej_ui_layout::Dimension::Auto,
            flex_grow: 0.0,
            padding: Insets::all(16.0),
            radius: None,
            border_color: None,
            child,
        }
    }
}

impl<V: View> Card<V> {
    /// Wrap child content inside this card.
    pub fn content<C: View>(self, content: C) -> Card<C> {
        Card {
            title: self.title,
            description: self.description,
            footer: self.footer,
            width: self.width,
            height: self.height,
            flex_grow: self.flex_grow,
            padding: self.padding,
            radius: self.radius,
            border_color: self.border_color,
            child: content,
        }
    }

    /// Sets explicit card flex-grow factor in flex containers.
    #[must_use]
    pub const fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.flex_grow = flex_grow;
        self
    }

    /// Sets explicit card width constraint.
    #[must_use]
    pub const fn width(mut self, width: tej_ui_layout::Dimension) -> Self {
        self.width = width;
        self
    }

    /// Sets explicit card height constraint.
    #[must_use]
    pub const fn height(mut self, height: tej_ui_layout::Dimension) -> Self {
        self.height = height;
        self
    }

    /// Sets inner padding.
    #[must_use]
    pub const fn padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }

    /// Sets corner radius override.
    #[must_use]
    pub const fn radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

    /// Sets border color override.
    #[must_use]
    pub const fn border(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Sets secondary description text.
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets footer text or action label.
    #[must_use]
    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }
}

impl<V: View> View for Card<V> {
    type State = CardState<V::State>;

    fn build(&self) -> Self::State {
        CardState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(320.0, 180.0)),
            title: self.title.clone(),
            node: 0,
            child_state: self.child.build(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.title = self.title.clone();
        self.child.rebuild(&mut state.child_state);
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        state.title = self.title.clone();
        self.child.patch(&mut state.child_state, dirty_bits);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let child_node = self.child.layout(&mut state.child_state, arena);
        let mut header_offset = 0.0f32;
        if !self.title.is_empty() {
            header_offset += 24.0;
        }
        if self.description.is_some() {
            header_offset += 20.0;
        }
        let top_padding = if header_offset > 0.0 {
            self.padding.top + header_offset + 8.0
        } else {
            self.padding.top
        };

        let width = self.width;

        let line_height = 16.0f32;
        let mut est_lines = 1;
        if let Some(desc) = &self.description {
            let max_w = 400.0f32;
            let char_w = 7.0f32;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let max_chars = ((max_w / char_w) as usize).max(15);
            let mut count = 0;
            for paragraph in desc.lines() {
                let p = paragraph.trim();
                if p.is_empty() {
                    count += 1;
                } else {
                    count += (p.len() / max_chars).max(1);
                }
            }
            est_lines = count.max(1);
        }
        let total_desc_h = (est_lines as f32) * line_height;
        let min_h = top_padding + total_desc_h + self.padding.bottom + 12.0;

        let has_child_content = !arena.children[child_node as usize].is_empty()
            || arena.styles[child_node as usize].height != tej_ui_layout::Dimension::Auto;

        let height = match self.height {
            tej_ui_layout::Dimension::Auto if has_child_content => tej_ui_layout::Dimension::Auto,
            tej_ui_layout::Dimension::Auto => tej_ui_layout::Dimension::Points(min_h),
            other => other,
        };

        let style = tej_ui_layout::LayoutStyle {
            direction: tej_ui_layout::FlexDirection::Column,
            width,
            height,
            flex_grow: self.flex_grow,
            padding: Insets::new(
                top_padding,
                self.padding.right,
                self.padding.bottom,
                self.padding.left,
            ),
            ..Default::default()
        };
        let node = arena.allocate_node(style);
        arena.add_child(node, child_node);
        state.node = node;
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(state.bounds);

        let radius = self.radius.unwrap_or(ctx.theme.radii.lg);
        let border = self.border_color.unwrap_or(ctx.theme.colors.border);

        // Border & card surface
        ctx.fill_rounded_rect(bounds, radius, border);
        let inner = Rect::new(
            bounds.origin.x + 1.0,
            bounds.origin.y + 1.0,
            (bounds.size.width - 2.0).max(0.0),
            (bounds.size.height - 2.0).max(0.0),
        );
        ctx.fill_rounded_rect(inner, (radius - 1.0).max(0.0), ctx.theme.colors.card);

        // Title
        if !state.title.is_empty() {
            let title_pt = Point::new(
                bounds.origin.x + self.padding.left,
                bounds.origin.y + self.padding.top,
            );
            ctx.draw_text(
                &state.title,
                title_pt,
                ctx.theme.typography.h3.font_size,
                ctx.theme.colors.card_foreground,
            );
        }

        // Description with paragraph & word-wrapping
        if let Some(desc) = &self.description {
            let max_w = (bounds.size.width - self.padding.left - self.padding.right).max(60.0);
            let char_w = ctx.theme.typography.body_small.font_size * 0.55;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let max_chars_per_line = ((max_w / char_w).floor() as usize).max(10);
            let desc_offset_y = if !state.title.is_empty() { 24.0 } else { 0.0 };
            let mut y = bounds.origin.y + self.padding.top + desc_offset_y;
            let line_height = ctx.theme.typography.body_small.font_size * 1.35;

            for paragraph in desc.lines() {
                let trimmed = paragraph.trim();
                if trimmed.is_empty() {
                    y += line_height * 0.5;
                    continue;
                }
                let mut current_line = String::new();
                for word in trimmed.split_whitespace() {
                    if !current_line.is_empty()
                        && current_line.len() + 1 + word.len() > max_chars_per_line
                    {
                        ctx.draw_text(
                            &current_line,
                            Point::new(bounds.origin.x + self.padding.left, y),
                            ctx.theme.typography.body_small.font_size,
                            ctx.theme.colors.muted_foreground,
                        );
                        y += line_height;
                        current_line.clear();
                    }
                    if !current_line.is_empty() {
                        current_line.push(' ');
                    }
                    current_line.push_str(word);
                }
                if !current_line.is_empty() {
                    ctx.draw_text(
                        &current_line,
                        Point::new(bounds.origin.x + self.padding.left, y),
                        ctx.theme.typography.body_small.font_size,
                        ctx.theme.colors.muted_foreground,
                    );
                    y += line_height;
                }
            }
        }

        // Paint child contents
        self.child.paint(&state.child_state, ctx);

        // Footer
        if let Some(footer) = &self.footer {
            let footer_pt = Point::new(
                bounds.origin.x + self.padding.left,
                bounds.origin.y + bounds.size.height - 28.0,
            );
            ctx.draw_text(
                footer,
                footer_pt,
                ctx.theme.typography.caption.font_size,
                ctx.theme.colors.muted_foreground,
            );
        }
    }

    fn register_events(
        &self,
        state: &Self::State,
        dispatcher: &mut tej_ui_view::events::EventDispatcher,
    ) {
        self.child.register_events(&state.child_state, dispatcher);
    }
}
