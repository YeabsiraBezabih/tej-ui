//! Styled wrapper view applying layout constraints and visual paint decorations.

use super::props::StyleProps;
use crate::view::View;
use std::fmt::Debug;
use tej_ui_hal::Color;
use tej_ui_layout::{Dimension, Insets, Overflow};

/// A wrapper view that decorates an inner view with layout and visual style properties.
#[derive(Clone, Debug)]
pub struct Styled<V: View> {
    pub inner: V,
    pub style: StyleProps,
}

impl<V: View> Styled<V> {
    /// Add padding around all four edges.
    #[must_use]
    pub fn padding(mut self, value: f32) -> Self {
        self.style.padding = Insets::all(value);
        self
    }

    /// Add custom edge padding insets.
    #[must_use]
    pub fn padding_insets(mut self, insets: Insets) -> Self {
        self.style.padding = insets;
        self
    }

    /// Add uniform margin on all sides.
    #[must_use]
    pub fn margin(mut self, value: f32) -> Self {
        self.style.margin = Insets::all(value);
        self
    }

    /// Add custom edge margin insets.
    #[must_use]
    pub fn margin_insets(mut self, insets: Insets) -> Self {
        self.style.margin = insets;
        self
    }

    /// Set solid background color.
    #[must_use]
    pub fn background(mut self, color: Color) -> Self {
        self.style.background = Some(color);
        self
    }

    /// Set rounded corner radius.
    #[must_use]
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.style.corner_radius = radius;
        self
    }

    /// Set stroke border.
    #[must_use]
    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.style.border_width = width;
        self.style.border_color = Some(color);
        self
    }

    /// Set width constraint.
    #[must_use]
    pub fn width(mut self, width: Dimension) -> Self {
        self.style.width = width;
        self
    }

    /// Set height constraint.
    #[must_use]
    pub fn height(mut self, height: Dimension) -> Self {
        self.style.height = height;
        self
    }

    /// Set minimum width constraint.
    #[must_use]
    pub fn min_width(mut self, min_width: Dimension) -> Self {
        self.style.min_width = min_width;
        self
    }

    /// Set maximum width constraint.
    #[must_use]
    pub fn max_width(mut self, max_width: Dimension) -> Self {
        self.style.max_width = max_width;
        self
    }

    /// Set minimum height constraint.
    #[must_use]
    pub fn min_height(mut self, min_height: Dimension) -> Self {
        self.style.min_height = min_height;
        self
    }

    /// Set maximum height constraint.
    #[must_use]
    pub fn max_height(mut self, max_height: Dimension) -> Self {
        self.style.max_height = max_height;
        self
    }

    /// Set flex wrap behavior.
    #[must_use]
    pub fn flex_wrap(mut self, wrap: tej_ui_layout::FlexWrap) -> Self {
        self.style.flex_wrap = Some(wrap);
        self
    }

    /// Set flex grow factor (e.g. 1.0 to expand and fill remaining space).
    #[must_use]
    pub fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.style.flex_grow = flex_grow;
        self
    }

    /// Set flex shrink factor (default 1.0; set to 0.0 to prevent shrinking).
    #[must_use]
    pub fn flex_shrink(mut self, flex_shrink: f32) -> Self {
        self.style.flex_shrink = Some(flex_shrink);
        self
    }

    /// Set overflow clipping behavior on both X and Y axes.
    #[must_use]
    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.style.overflow_x = overflow;
        self.style.overflow_y = overflow;
        self
    }

    /// Set column span in CSS Grid layout (e.g. 2, 4, 6 for col-span-*).
    #[must_use]
    pub fn col_span(mut self, span: u16) -> Self {
        self.style.col_span = span;
        self
    }

    /// Set row span in CSS Grid layout.
    #[must_use]
    pub fn row_span(mut self, span: u16) -> Self {
        self.style.row_span = span;
        self
    }

    /// Set a responsive container query for dynamic layout adaptation.
    #[must_use]
    pub fn container_query(mut self, query: tej_ui_layout::ContainerQuery) -> Self {
        self.style.container_query = Some(query);
        self
    }

    /// Set layout display mode (Flex or Grid).
    #[must_use]
    pub fn display(mut self, display: tej_ui_layout::DisplayMode) -> Self {
        self.style.display = Some(display);
        self
    }

    /// Set main axis flex layout direction.
    #[must_use]
    pub fn flex_direction(mut self, direction: tej_ui_layout::FlexDirection) -> Self {
        self.style.flex_direction = Some(direction);
        self
    }

    /// Set cross-axis child item alignment.
    #[must_use]
    pub fn align_items(mut self, align: tej_ui_layout::AlignItems) -> Self {
        self.style.align_items = Some(align);
        self
    }

    /// Set main-axis item distribution.
    #[must_use]
    pub fn justify_content(mut self, justify: tej_ui_layout::JustifyContent) -> Self {
        self.style.justify_content = Some(justify);
        self
    }

    /// Set gap spacing between items.
    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap = gap;
        self
    }

    /// Set layer opacity (0.0 to 1.0).
    #[must_use]
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.style.opacity = Some(opacity);
        self
    }

    /// Set elevation drop shadow.
    #[must_use]
    pub fn shadow(mut self, shadow: crate::theme::Shadow) -> Self {
        self.style.shadow = Some(shadow);
        self
    }

    /// Set interactive pointer cursor icon.
    #[must_use]
    pub fn cursor(mut self, cursor: crate::events::CursorIcon) -> Self {
        self.style.cursor = Some(cursor);
        self
    }

    /// Apply styles from a typed CssStyleSheet.
    #[must_use]
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
        let border_insets = if self.style.border_edges.iter().any(|e| e.width > 0.0) {
            Insets::new(
                self.style.border_edges[0].width,
                self.style.border_edges[1].width,
                self.style.border_edges[2].width,
                self.style.border_edges[3].width,
            )
        } else {
            Insets::all(self.style.border_width)
        };
        let layout_style = tej_ui_layout::LayoutStyle {
            display: self
                .style
                .display
                .unwrap_or(tej_ui_layout::DisplayMode::Flex),
            direction: self
                .style
                .flex_direction
                .unwrap_or(tej_ui_layout::FlexDirection::Column),
            layout_direction: self.style.layout_direction,
            align_items: self.style.align_items.unwrap_or_default(),
            align_self: self.style.align_self,
            align_content: self.style.align_content,
            justify_content: self.style.justify_content.unwrap_or_default(),
            justify_items: self.style.justify_items,
            justify_self: self.style.justify_self,
            place_content: self.style.place_content,
            order: self.style.order,
            z_index: self.style.z_index.unwrap_or(0),
            spacing: self.style.gap,
            width: self.style.width,
            height: self.style.height,
            min_width: self.style.min_width,
            max_width: self.style.max_width,
            min_height: self.style.min_height,
            max_height: self.style.max_height,
            padding: self.style.padding,
            border: border_insets,
            margin: self.style.margin,
            flex_wrap: self.style.flex_wrap.unwrap_or_default(),
            flex_grow: self.style.flex_grow,
            flex_shrink: self.style.flex_shrink.unwrap_or(1.0),
            flex_basis: self.style.flex_basis,
            overflow_x: self.style.overflow_x,
            overflow_y: self.style.overflow_y,
            grid_columns: self.style.grid_columns,
            grid_rows: self.style.grid_rows,
            grid_auto_flow: self.style.grid_auto_flow,
            grid_column_gap: self.style.grid_column_gap,
            grid_row_gap: self.style.grid_row_gap,
            col_span: self.style.col_span,
            row_span: self.style.row_span,
            container_query: self.style.container_query,
        };
        let node = arena.allocate_node(layout_style);
        state.node = node;
        let child_node = self.inner.layout(&mut state.inner_state, arena);
        arena.add_child(node, child_node);
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut crate::paint::PaintContext<'_>) {
        super::paint_helpers::paint_styled_node(
            &self.inner,
            &state.inner_state,
            &self.style,
            state.node,
            ctx,
        );
    }

    fn register_events(
        &self,
        state: &Self::State,
        dispatcher: &mut crate::events::EventDispatcher,
    ) {
        if self.style.pointer_events == crate::css::PointerEvents::None {
            return;
        }
        self.inner.register_events(&state.inner_state, dispatcher);
    }

    fn accessibility(
        &self,
        state: &Self::State,
        arena: &tej_ui_layout::LayoutArena,
        tree: &mut tej_ui_a11y::A11yTree,
        parent: Option<tej_ui_a11y::A11yId>,
    ) {
        self.inner
            .accessibility(&state.inner_state, arena, tree, parent);
    }
}
