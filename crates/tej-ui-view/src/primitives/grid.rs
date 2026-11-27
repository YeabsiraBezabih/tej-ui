//! CSS Grid container widget with multi-column distribution, column span, and responsive gap spacing.

use crate::view::View;
use std::fmt::Debug;
use tej_ui_layout::{Dimension, DisplayMode};

/// State of a mounted CSS Grid view.
#[derive(Debug)]
pub struct GridState<S: Debug> {
    pub child_state: S,
    pub columns: u16,
    pub column_gap: f32,
    pub row_gap: f32,
    pub node: u32,
}

/// A CSS Grid container view implementing 2D multi-column layouts with track gaps and spans.
#[derive(Debug, Clone)]
pub struct Grid<V: View> {
    pub children: V,
    pub columns: u16,
    pub column_gap: f32,
    pub row_gap: f32,
    pub padding: u32,
    pub width: Dimension,
    pub height: Dimension,
    pub flex_grow: f32,
}

impl<V: View> Grid<V> {
    /// Create a new CSS Grid container with the specified child views.
    pub fn new(children: V) -> Self {
        Self {
            children,
            columns: 12,
            column_gap: 0.0,
            row_gap: 0.0,
            padding: 0,
            width: Dimension::Auto,
            height: Dimension::Auto,
            flex_grow: 0.0,
        }
    }

    /// Set number of columns (e.g. 12 for grid-cols-12, 6, 4, etc.).
    pub fn columns(mut self, cols: u16) -> Self {
        self.columns = cols;
        self
    }

    /// Set uniform gap between grid items in pixels.
    pub fn gap(mut self, gap: f32) -> Self {
        self.column_gap = gap;
        self.row_gap = gap;
        self
    }

    /// Set horizontal column gap in pixels.
    pub fn column_gap(mut self, gap: f32) -> Self {
        self.column_gap = gap;
        self
    }

    /// Set vertical row gap in pixels.
    pub fn row_gap(mut self, gap: f32) -> Self {
        self.row_gap = gap;
        self
    }

    /// Set padding around grid container.
    pub fn padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Set explicit width constraint.
    pub fn width(mut self, width: Dimension) -> Self {
        self.width = width;
        self
    }

    /// Set explicit height constraint.
    pub fn height(mut self, height: Dimension) -> Self {
        self.height = height;
        self
    }

    /// Set flex grow factor when placed inside a flex parent.
    pub fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.flex_grow = flex_grow;
        self
    }
}

impl<V: View> View for Grid<V> {
    type State = GridState<V::State>;

    fn build(&self) -> Self::State {
        GridState {
            child_state: self.children.build(),
            columns: self.columns,
            column_gap: self.column_gap,
            row_gap: self.row_gap,
            node: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        self.children.rebuild(&mut state.child_state);
        state.columns = self.columns;
        state.column_gap = self.column_gap;
        state.row_gap = self.row_gap;
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        self.children.patch(&mut state.child_state, dirty_bits);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let style = tej_ui_layout::LayoutStyle {
            display: DisplayMode::Grid,
            grid_columns: self.columns,
            grid_column_gap: self.column_gap,
            grid_row_gap: self.row_gap,
            padding: tej_ui_layout::Insets::all(self.padding as f32),
            width: self.width,
            height: self.height,
            flex_grow: self.flex_grow,
            ..Default::default()
        };
        let container = arena.allocate_node(style);
        self.children
            .layout_children(&mut state.child_state, arena, container);
        state.node = container;
        container
    }

    fn paint(&self, state: &Self::State, ctx: &mut crate::paint::PaintContext<'_>) {
        self.children.paint(&state.child_state, ctx);
    }

    fn register_events(
        &self,
        state: &Self::State,
        dispatcher: &mut crate::events::EventDispatcher,
    ) {
        self.children
            .register_events(&state.child_state, dispatcher);
    }

    fn accessibility(
        &self,
        state: &Self::State,
        arena: &tej_ui_layout::LayoutArena,
        tree: &mut tej_ui_a11y::A11yTree,
        parent: Option<tej_ui_a11y::A11yId>,
    ) {
        let id = tree.allocate_id();
        let bounds = arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(tej_ui_layout::Rect::ZERO);
        let node = tej_ui_a11y::A11yNode::new(id, tej_ui_a11y::A11yRole::Grid).with_bounds(bounds);
        tree.insert_node(node);
        if let Some(parent_id) = parent {
            if let Some(p) = tree.get_node_mut(parent_id) {
                p.children.push(id);
            }
            tree.mark_dirty(parent_id);
        }
        self.children
            .accessibility(&state.child_state, arena, tree, Some(id));
    }
}
