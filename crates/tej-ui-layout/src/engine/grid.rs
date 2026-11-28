//! CSS Grid 2D placement and alignment pass.

use super::measure::measure_intrinsic_extent;
use super::{LayoutEngine, LayoutScope, MeasureAxis, count_to_f32};
use crate::geometry::{Point, Size};
use crate::grid_extended::{JustifyItems, JustifySelf};
use crate::storage::LayoutArena;
use crate::style::{Dimension, LayoutStyle};

struct PlacedItem {
    child: u32,
    row: usize,
    col: usize,
    col_span: usize,
    row_span: usize,
    measured_h: f32,
}

pub(crate) fn compute_grid_layout(
    arena: &mut LayoutArena,
    children: &[u32],
    style: LayoutStyle,
    inner_origin: Point,
    inner_width: f32,
    inner_height: f32,
) {
    let num_cols = style.grid_columns.max(1) as usize;
    let col_gap = style.grid_column_gap.max(0.0);
    let row_gap = style.grid_row_gap.max(0.0);
    let total_col_gaps = col_gap * count_to_f32(num_cols.saturating_sub(1));
    let col_width = ((inner_width - total_col_gaps) / count_to_f32(num_cols)).max(0.0);

    let (placements, mut row_heights) =
        place_grid_items(arena, children, num_cols, col_width, col_gap, inner_height);
    resolve_row_spans(arena, &placements, &mut row_heights, row_gap);
    layout_placed_items(
        arena,
        &placements,
        &row_heights,
        style,
        inner_origin,
        col_width,
        col_gap,
        row_gap,
    );
}

fn find_first_fit(
    occupied: &mut Vec<Vec<bool>>,
    row_heights: &mut Vec<f32>,
    num_cols: usize,
    c_span: usize,
    r_span: usize,
) -> (usize, usize) {
    let mut placed_row = 0usize;
    loop {
        while occupied.len() < placed_row + r_span {
            occupied.push(vec![false; num_cols]);
            row_heights.push(0.0);
        }
        for col in 0..=(num_cols - c_span) {
            let fits = occupied[placed_row..(placed_row + r_span)]
                .iter()
                .all(|row_occ| row_occ[col..(col + c_span)].iter().all(|&cell| !cell));
            if fits {
                return (placed_row, col);
            }
        }
        placed_row += 1;
    }
}

fn place_grid_items(
    arena: &LayoutArena,
    children: &[u32],
    num_cols: usize,
    col_width: f32,
    col_gap: f32,
    inner_height: f32,
) -> (Vec<PlacedItem>, Vec<f32>) {
    let mut occupied: Vec<Vec<bool>> = Vec::new();
    let mut row_heights: Vec<f32> = Vec::new();
    let mut placements: Vec<PlacedItem> = Vec::with_capacity(children.len());

    for &child in children {
        let cs = arena.styles[child as usize];
        let c_span = (cs.col_span.max(1) as usize).min(num_cols);
        let r_span = cs.row_span.max(1) as usize;
        let (placed_row, placed_col) =
            find_first_fit(&mut occupied, &mut row_heights, num_cols, c_span, r_span);

        for row_occ in &mut occupied[placed_row..(placed_row + r_span)] {
            row_occ[placed_col..(placed_col + c_span)].fill(true);
        }

        let cell_w = count_to_f32(c_span) * col_width + count_to_f32(c_span - 1) * col_gap;
        let content_w = (cell_w - cs.margin.left - cs.margin.right).max(0.0);
        let raw_h = match cs.height {
            Dimension::Points(p) => p,
            Dimension::Percent(pct) => inner_height * pct,
            Dimension::Fill => (inner_height - cs.margin.top - cs.margin.bottom).max(0.0),
            Dimension::Auto => measure_intrinsic_extent(
                arena,
                child,
                MeasureAxis::Height,
                Size::new(content_w, inner_height),
            )
            .unwrap_or(24.0),
        };
        let measured_h =
            LayoutEngine::clamp_dimension(raw_h, cs.min_height, cs.max_height, inner_height);
        let outer_h = measured_h + cs.margin.top + cs.margin.bottom;

        if r_span == 1 {
            row_heights[placed_row] = row_heights[placed_row].max(outer_h);
        }

        placements.push(PlacedItem {
            child,
            row: placed_row,
            col: placed_col,
            col_span: c_span,
            row_span: r_span,
            measured_h,
        });
    }

    (placements, row_heights)
}

fn resolve_row_spans(
    arena: &LayoutArena,
    placements: &[PlacedItem],
    row_heights: &mut [f32],
    row_gap: f32,
) {
    for item in placements {
        if item.row_span > 1 {
            let cs = arena.styles[item.child as usize];
            let outer_h = item.measured_h + cs.margin.top + cs.margin.bottom;
            let span_gaps = row_gap * count_to_f32(item.row_span - 1);
            let current_span_h: f32 = row_heights[item.row..(item.row + item.row_span)]
                .iter()
                .sum::<f32>()
                + span_gaps;
            if outer_h > current_span_h {
                let deficit_per_row = (outer_h - current_span_h) / count_to_f32(item.row_span);
                for h in &mut row_heights[item.row..(item.row + item.row_span)] {
                    *h += deficit_per_row;
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn layout_placed_items(
    arena: &mut LayoutArena,
    placements: &[PlacedItem],
    row_heights: &[f32],
    style: LayoutStyle,
    inner_origin: Point,
    col_width: f32,
    col_gap: f32,
    row_gap: f32,
) {
    let mut row_y_offsets = Vec::with_capacity(row_heights.len());
    let mut acc_y = 0.0f32;
    for &rh in row_heights {
        row_y_offsets.push(acc_y);
        acc_y += rh + row_gap;
    }

    for item in placements {
        let cs = arena.styles[item.child as usize];
        let cell_x = inner_origin.x + count_to_f32(item.col) * (col_width + col_gap);
        let cell_w =
            count_to_f32(item.col_span) * col_width + count_to_f32(item.col_span - 1) * col_gap;
        let cell_y = inner_origin.y + row_y_offsets[item.row];
        let cell_h = if item.row_span > 1 {
            row_heights[item.row..(item.row + item.row_span)]
                .iter()
                .sum::<f32>()
                + row_gap * count_to_f32(item.row_span - 1)
        } else {
            item.measured_h + cs.margin.top + cs.margin.bottom
        };

        let raw_w = (cell_w - cs.margin.left - cs.margin.right).max(0.0);
        let raw_h = (cell_h - cs.margin.top - cs.margin.bottom).max(0.0);
        let effective_justify = match cs.justify_self {
            JustifySelf::Auto => match style.justify_items {
                JustifyItems::Stretch => JustifySelf::Stretch,
                JustifyItems::Start => JustifySelf::Start,
                JustifyItems::Center => JustifySelf::Center,
                JustifyItems::End => JustifySelf::End,
            },
            other => other,
        };
        let target_w = if effective_justify == JustifySelf::Stretch {
            raw_w
        } else {
            measure_intrinsic_extent(
                arena,
                item.child,
                MeasureAxis::Width,
                Size::new(raw_w, raw_h),
            )
            .map_or(raw_w, |iw| iw.min(raw_w))
        };
        let child_w = LayoutEngine::clamp_dimension(target_w, cs.min_width, cs.max_width, cell_w);
        let align_dx = match effective_justify {
            JustifySelf::Center => ((raw_w - child_w) * 0.5).max(0.0),
            JustifySelf::End => (raw_w - child_w).max(0.0),
            _ => 0.0,
        };
        let child_x = cell_x + cs.margin.left + align_dx;
        let child_y = cell_y + cs.margin.top;
        let child_h = LayoutEngine::clamp_dimension(raw_h, cs.min_height, cs.max_height, cell_h);

        LayoutEngine::compute_node_layout(
            arena,
            item.child,
            Point::new(child_x, child_y),
            Size::new(child_w, child_h),
            LayoutScope::Child,
        );
    }
}
