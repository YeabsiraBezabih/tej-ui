use crate::datagrid::cell::GridCellCoords;
use crate::datagrid::column::{GridColumn, PinState};
use std::fmt::Debug;
use std::sync::Arc;
use tej_ui_core::Signal;
use tej_ui_layout::{Dimension, LayoutStyle, Point, Rect};
use tej_ui_view::View;

/// Viewport and layout sizing dimensions for the DataGrid.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DataGridDimensions {
    pub total_rows: usize,
    pub row_height: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

/// High-performance 2D virtualized enterprise DataGrid.
///
/// Virtualizes both rows and columns simultaneously, enabling fluid 120 FPS
/// navigation across 100,000+ cells while maintaining a strictly flat memory footprint.
pub struct DataGrid<V: View, CellBuilder>
where
    CellBuilder: Fn(GridCellCoords) -> V + 'static,
{
    columns: Vec<GridColumn>,
    dimensions: DataGridDimensions,
    scroll_x: Signal<f32>,
    scroll_y: Signal<f32>,
    cell_builder: Arc<CellBuilder>,
}

impl<V: View, CellBuilder> Debug for DataGrid<V, CellBuilder>
where
    CellBuilder: Fn(GridCellCoords) -> V + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataGrid")
            .field("columns_count", &self.columns.len())
            .field("dimensions", &self.dimensions)
            .finish()
    }
}

impl<V: View, CellBuilder> DataGrid<V, CellBuilder>
where
    CellBuilder: Fn(GridCellCoords) -> V + 'static,
{
    /// Create a new 2D virtualized DataGrid.
    pub fn new(
        columns: Vec<GridColumn>,
        dimensions: DataGridDimensions,
        scroll_x: Signal<f32>,
        scroll_y: Signal<f32>,
        cell_builder: CellBuilder,
    ) -> Self {
        Self {
            columns,
            dimensions,
            scroll_x,
            scroll_y,
            cell_builder: Arc::new(cell_builder),
        }
    }

    /// Compute visible row range `(start_row, end_row)`.
    #[must_use]
    pub fn visible_rows(&self) -> (usize, usize) {
        if self.dimensions.total_rows == 0
            || self.dimensions.row_height <= 0.0
            || !self.dimensions.row_height.is_finite()
            || !self.dimensions.viewport_height.is_finite()
        {
            return (0, 0);
        }
        let raw_offset = self.scroll_y.get();
        let offset = if raw_offset.is_finite() {
            raw_offset.max(0.0)
        } else {
            0.0
        };
        let viewport = self.dimensions.viewport_height.max(0.0);
        let start = (offset / self.dimensions.row_height).floor() as usize;
        let visible_count =
            ((viewport / self.dimensions.row_height).ceil() as usize).saturating_add(2);
        let end = start
            .saturating_add(visible_count)
            .min(self.dimensions.total_rows);
        (start.min(self.dimensions.total_rows), end)
    }

    /// Compute visible column indices including pinned columns, accounting for
    /// left-pinned (and right-pinned) column widths when sizing the scrollable center viewport.
    #[must_use]
    pub fn visible_columns(&self) -> Vec<usize> {
        let mut visible = Vec::new();
        let mut pinned_left_width = 0.0_f32;
        let mut pinned_right_width = 0.0_f32;

        // 1. Always include pinned left columns and accumulate their width
        for (idx, col) in self.columns.iter().enumerate() {
            match col.pin {
                PinState::Left => {
                    visible.push(idx);
                    pinned_left_width += col.width.max(0.0);
                }
                PinState::Right => {
                    pinned_right_width += col.width.max(0.0);
                }
                PinState::None => {}
            }
        }

        // 2. Add scrollable center columns intersecting the remaining center viewport
        let raw_offset_x = self.scroll_x.get();
        let offset_x = if raw_offset_x.is_finite() {
            raw_offset_x.max(0.0)
        } else {
            0.0
        };
        let center_viewport_width =
            (self.dimensions.viewport_width - pinned_left_width - pinned_right_width).max(0.0);
        let mut current_x = 0.0_f32;
        for (idx, col) in self.columns.iter().enumerate() {
            if col.pin == PinState::None {
                let col_right = current_x + col.width.max(0.0);
                if center_viewport_width > 0.0
                    && col_right > offset_x
                    && current_x < offset_x + center_viewport_width
                    && !visible.contains(&idx)
                {
                    visible.push(idx);
                }
                current_x = col_right;
            }
        }

        // 3. Always include pinned right columns
        for (idx, col) in self.columns.iter().enumerate() {
            if col.pin == PinState::Right && !visible.contains(&idx) {
                visible.push(idx);
            }
        }

        visible
    }
}

/// State retained across renders by a mounted DataGrid.
#[derive(Debug)]
pub struct DataGridState<S: Debug> {
    pub start_row: usize,
    pub end_row: usize,
    pub visible_cols: Vec<usize>,
    pub cell_states: Vec<(GridCellCoords, S)>,
    pub node: u32,
    pub cell_nodes: Vec<(GridCellCoords, u32)>,
}

impl<V: View, CellBuilder> View for DataGrid<V, CellBuilder>
where
    CellBuilder: Fn(GridCellCoords) -> V + 'static,
{
    type State = DataGridState<V::State>;

    fn build(&self) -> Self::State {
        let (start_row, end_row) = self.visible_rows();
        let visible_cols = self.visible_columns();

        let mut cell_states =
            Vec::with_capacity(end_row.saturating_sub(start_row) * visible_cols.len());
        for row in start_row..end_row {
            for &col in &visible_cols {
                let coords = GridCellCoords::new(row, col);
                let cell_view = (self.cell_builder)(coords);
                cell_states.push((coords, cell_view.build()));
            }
        }

        DataGridState {
            start_row,
            end_row,
            visible_cols,
            cell_states,
            node: 0,
            cell_nodes: Vec::new(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        let (start_row, end_row) = self.visible_rows();
        let visible_cols = self.visible_columns();

        let mut new_states =
            Vec::with_capacity(end_row.saturating_sub(start_row) * visible_cols.len());
        for row in start_row..end_row {
            for &col in &visible_cols {
                let coords = GridCellCoords::new(row, col);
                let cell_view = (self.cell_builder)(coords);

                if let Some(pos) = state.cell_states.iter().position(|(c, _)| *c == coords) {
                    let mut existing = state.cell_states.remove(pos);
                    cell_view.rebuild(&mut existing.1);
                    new_states.push(existing);
                } else {
                    new_states.push((coords, cell_view.build()));
                }
            }
        }

        state.start_row = start_row;
        state.end_row = end_row;
        state.visible_cols = visible_cols;
        state.cell_states = new_states;
    }

    fn patch(&self, state: &mut Self::State, dirty_bits: u64) {
        let (start_row, end_row) = self.visible_rows();
        let visible_cols = self.visible_columns();

        if start_row != state.start_row
            || end_row != state.end_row
            || visible_cols != state.visible_cols
        {
            self.rebuild(state);
        } else {
            for (coords, child_state) in &mut state.cell_states {
                let cell_view = (self.cell_builder)(*coords);
                cell_view.patch(child_state, dirty_bits);
            }
        }
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let root_style = LayoutStyle {
            width: Dimension::Points(self.dimensions.viewport_width.max(0.0)),
            height: Dimension::Points(self.dimensions.viewport_height.max(0.0)),
            ..Default::default()
        };
        let root = arena.allocate_node(root_style);
        state.node = root;
        state.cell_nodes.clear();

        for (coords, child_state) in &mut state.cell_states {
            let cell_view = (self.cell_builder)(*coords);
            let child_node = cell_view.layout(child_state, arena);
            arena.add_child(root, child_node);
            state.cell_nodes.push((*coords, child_node));
        }

        root
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or_else(|| {
                Rect::new(
                    0.0,
                    0.0,
                    self.dimensions.viewport_width.max(0.0),
                    self.dimensions.viewport_height.max(0.0),
                )
            });

        // Grid surface background
        ctx.fill_rect(bounds, ctx.theme.colors.background);

        let row_h = if self.dimensions.row_height > 0.0 && self.dimensions.row_height.is_finite() {
            self.dimensions.row_height
        } else {
            28.0
        };
        let header_h = row_h.max(28.0);

        // Header bar background and column titles
        let header_rect = Rect::new(
            bounds.origin.x,
            bounds.origin.y,
            bounds.size.width,
            header_h,
        );
        ctx.fill_rect(header_rect, ctx.theme.colors.muted);

        let mut col_x = bounds.origin.x;
        for &col_idx in &state.visible_cols {
            if let Some(col) = self.columns.get(col_idx) {
                let title_pt = Point::new(col_x + 8.0, bounds.origin.y + 6.0);
                ctx.draw_text(
                    &col.title,
                    title_pt,
                    ctx.theme.typography.body.font_size,
                    ctx.theme.colors.foreground,
                );
                col_x += col.width.max(0.0);
                ctx.draw_line(
                    Point::new(col_x, bounds.origin.y),
                    Point::new(col_x, bounds.origin.y + header_h),
                    1.0,
                    ctx.theme.colors.border,
                );
            }
        }

        // Bottom border of header bar
        ctx.draw_line(
            Point::new(bounds.origin.x, bounds.origin.y + header_h),
            Point::new(
                bounds.origin.x + bounds.size.width,
                bounds.origin.y + header_h,
            ),
            1.0,
            ctx.theme.colors.border,
        );

        // Visible row backgrounds and horizontal dividers
        for (rel_idx, _row) in (state.start_row..state.end_row).enumerate() {
            let row_y = bounds.origin.y + header_h + (rel_idx as f32) * row_h;
            if row_y >= bounds.origin.y + bounds.size.height {
                break;
            }
            if rel_idx % 2 == 1 {
                ctx.fill_rect(
                    Rect::new(bounds.origin.x, row_y, bounds.size.width, row_h),
                    ctx.theme.colors.card,
                );
            }
            ctx.draw_line(
                Point::new(bounds.origin.x, row_y + row_h),
                Point::new(bounds.origin.x + bounds.size.width, row_y + row_h),
                1.0,
                ctx.theme.colors.border,
            );
        }

        // Paint visible cell views
        for (coords, child_state) in &state.cell_states {
            let cell_view = (self.cell_builder)(*coords);
            cell_view.paint(child_state, ctx);
        }
    }
}
