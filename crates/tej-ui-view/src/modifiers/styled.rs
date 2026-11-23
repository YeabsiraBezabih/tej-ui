//! Styled wrapper view applying layout constraints and visual paint decorations.

use super::props::StyleProps;
use crate::view::View;
use std::fmt::Debug;
use tej_ui_hal::Color;
use tej_ui_layout::{Dimension, Insets};

/// A wrapper view that decorates an inner view with layout and visual style properties.
#[derive(Clone, Debug)]
pub struct Styled<V: View> {
    pub inner: V,
    pub style: StyleProps,
}

impl<V: View> Styled<V> {
    /// Add padding around all four edges.
    pub fn padding(mut self, value: f32) -> Self {
        self.style.padding = Insets::all(value);
        self
    }

    /// Add custom edge padding insets.
    pub fn padding_insets(mut self, insets: Insets) -> Self {
        self.style.padding = insets;
        self
    }

    /// Add uniform margin on all sides.
    pub fn margin(mut self, value: f32) -> Self {
        self.style.margin = Insets::all(value);
        self
    }

    /// Add custom edge margin insets.
    pub fn margin_insets(mut self, insets: Insets) -> Self {
        self.style.margin = insets;
        self
    }

    /// Set solid background color.
    pub fn background(mut self, color: Color) -> Self {
        self.style.background = Some(color);
        self
    }

    /// Set rounded corner radius.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.style.corner_radius = radius;
        self
    }

    /// Set stroke border.
    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.style.border_width = width;
        self.style.border_color = Some(color);
        self
    }

    /// Set width constraint.
    pub fn width(mut self, width: Dimension) -> Self {
        self.style.width = width;
        self
    }

    /// Set height constraint.
    pub fn height(mut self, height: Dimension) -> Self {
        self.style.height = height;
        self
    }

    /// Set flex wrap behavior.
    pub fn flex_wrap(mut self, wrap: tej_ui_layout::FlexWrap) -> Self {
        self.style.flex_wrap = Some(wrap);
        self
    }

    /// Set flex grow factor (e.g. 1.0 to expand and fill remaining space).
    pub fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.style.flex_grow = flex_grow;
        self
    }

    /// Set column span in CSS Grid layout (e.g. 2, 4, 6 for col-span-*).
    pub fn col_span(mut self, span: u16) -> Self {
        self.style.col_span = span;
        self
    }

    /// Set row span in CSS Grid layout.
    pub fn row_span(mut self, span: u16) -> Self {
        self.style.row_span = span;
        self
    }

    /// Set a responsive container query for dynamic layout adaptation.
    pub fn container_query(mut self, query: tej_ui_layout::ContainerQuery) -> Self {
        self.style.container_query = Some(query);
        self
    }

    /// Set layout display mode (Flex or Grid).
    pub fn display(mut self, display: tej_ui_layout::DisplayMode) -> Self {
        self.style.display = Some(display);
        self
    }

    /// Set main axis flex layout direction.
    pub fn flex_direction(mut self, direction: tej_ui_layout::FlexDirection) -> Self {
        self.style.flex_direction = Some(direction);
        self
    }

    /// Set cross-axis child item alignment.
    pub fn align_items(mut self, align: tej_ui_layout::AlignItems) -> Self {
        self.style.align_items = Some(align);
        self
    }

    /// Set main-axis item distribution.
    pub fn justify_content(mut self, justify: tej_ui_layout::JustifyContent) -> Self {
        self.style.justify_content = Some(justify);
        self
    }

    /// Set gap spacing between items.
    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap = gap;
        self
    }

    /// Set layer opacity (0.0 to 1.0).
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.style.opacity = Some(opacity);
        self
    }

    /// Set elevation drop shadow.
    pub fn shadow(mut self, shadow: crate::theme::Shadow) -> Self {
        self.style.shadow = Some(shadow);
        self
    }

    /// Set interactive pointer cursor icon.
    pub fn cursor(mut self, cursor: crate::events::CursorIcon) -> Self {
        self.style.cursor = Some(cursor);
        self
    }

    /// Apply styles from a typed CssStyleSheet.
    pub fn css(mut self, sheet: &crate::css::CssStyleSheet) -> Self {
        self.style.apply_css(sheet);
        self
    }
}

/// State of a mounted Styled view.
#[derive(Debug)]
pub struct StyledState<S: Debug> {
    pub inner_state: S,
    pub style: StyleProps,
    pub node: u32,
}

impl<V: View> View for Styled<V> {
    type State = StyledState<V::State>;

    fn build(&self) -> Self::State {
        StyledState {
            inner_state: self.inner.build(),
            style: self.style,
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        self.inner.rebuild(&mut state.inner_state);
        state.style = self.style;
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        self.inner.patch(&mut state.inner_state, dirty_bits);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let layout_style = tej_ui_layout::LayoutStyle {
            display: self
                .style
                .display
                .unwrap_or(tej_ui_layout::DisplayMode::Flex),
            direction: self
                .style
                .flex_direction
                .unwrap_or(tej_ui_layout::FlexDirection::Row),
            align_items: self.style.align_items.unwrap_or_default(),
            justify_content: self.style.justify_content.unwrap_or_default(),
            spacing: self.style.gap,
            width: self.style.width,
            height: self.style.height,
            padding: self.style.padding,
            margin: self.style.margin,
            flex_wrap: self.style.flex_wrap.unwrap_or_default(),
            flex_grow: self.style.flex_grow,
            col_span: self.style.col_span,
            row_span: self.style.row_span,
            container_query: self.style.container_query,
            ..Default::default()
        };
        let node = arena.allocate_node(layout_style);
        state.node = node;
        let child_node = self.inner.layout(&mut state.inner_state, arena);
        if arena.styles[child_node as usize].flex_grow == 0.0
            && arena.styles[child_node as usize].width == tej_ui_layout::Dimension::Auto
        {
            arena.styles[child_node as usize].flex_grow = 1.0;
        }
        arena.add_child(node, child_node);
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut crate::paint::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(tej_ui_layout::Rect::ZERO);

        if let Some(shadow) = self.style.shadow {
            if shadow.blur > 0.0 || shadow.offset_x != 0.0 || shadow.offset_y != 0.0 {
                let shadow_bounds = tej_ui_layout::Rect::new(
                    bounds.origin.x + shadow.offset_x - shadow.spread,
                    bounds.origin.y + shadow.offset_y - shadow.spread,
                    bounds.size.width + shadow.spread * 2.0,
                    bounds.size.height + shadow.spread * 2.0,
                );
                ctx.fill_rounded_rect(
                    shadow_bounds,
                    self.style.corner_radius + shadow.blur * 0.5,
                    shadow.color,
                );
            }
        }

        if let Some(bg) = self.style.background {
            let col = if let Some(op) = self.style.opacity {
                Color::rgba(bg.r, bg.g, bg.b, ((bg.a as f32) * op) as u8)
            } else {
                bg
            };
            if self.style.corner_radius > 0.0 {
                ctx.fill_rounded_rect(bounds, self.style.corner_radius, col);
            } else {
                ctx.fill_rect(bounds, col);
            }
        }
        if self.style.border_width > 0.0 {
            if let Some(border_col) = self.style.border_color {
                let col = if let Some(op) = self.style.opacity {
                    Color::rgba(
                        border_col.r,
                        border_col.g,
                        border_col.b,
                        ((border_col.a as f32) * op) as u8,
                    )
                } else {
                    border_col
                };
                ctx.fill_rounded_rect(bounds, self.style.corner_radius, col);
            }
        }
        self.inner.paint(&state.inner_state, ctx);
    }

    fn register_events(
        &self,
        state: &Self::State,
        dispatcher: &mut crate::events::EventDispatcher,
    ) {
        self.inner.register_events(&state.inner_state, dispatcher);
    }
}
