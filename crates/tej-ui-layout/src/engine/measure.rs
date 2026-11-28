//! Intrinsic content measurement pass for `Dimension::Auto` layout nodes.

use super::{LayoutEngine, MeasureAxis, count_to_f32};
use crate::geometry::Size;
use crate::storage::LayoutArena;
use crate::style::{Dimension, DisplayMode, FlexDirection, LayoutStyle};

/// Recursively calculates intrinsic content extents for a node without fixed dimensions.
pub(crate) fn measure_intrinsic_extent(
    arena: &LayoutArena,
    node: u32,
    axis: MeasureAxis,
    parent_size: Size,
) -> Option<f32> {
    let style = arena.styles.get(node as usize)?;
    if style.display == DisplayMode::None {
        return Some(0.0);
    }

    let (dim, min_dim, max_dim, parent_basis) = match axis {
        MeasureAxis::Width => (
            style.width,
            style.min_width,
            style.max_width,
            parent_size.width,
        ),
        MeasureAxis::Height => (
            style.height,
            style.min_height,
            style.max_height,
            parent_size.height,
        ),
    };

    match dim {
        Dimension::Points(p) => Some(LayoutEngine::clamp_dimension(
            p,
            min_dim,
            max_dim,
            parent_basis,
        )),
        Dimension::Percent(pct) => Some(LayoutEngine::clamp_dimension(
            parent_basis * pct,
            min_dim,
            max_dim,
            parent_basis,
        )),
        Dimension::Fill => None,
        Dimension::Auto => {
            measure_auto_extent(arena, node, style, axis, min_dim, max_dim, parent_size)
        }
    }
}

fn measure_auto_extent(
    arena: &LayoutArena,
    node: u32,
    style: &LayoutStyle,
    axis: MeasureAxis,
    min_dim: Dimension,
    max_dim: Dimension,
    parent_size: Size,
) -> Option<f32> {
    let parent_basis = match axis {
        MeasureAxis::Width => parent_size.width,
        MeasureAxis::Height => parent_size.height,
    };
    let raw_children = arena.children.get(node as usize)?;
    let mut visible_children: smallvec::SmallVec<[u32; 8]> =
        smallvec::SmallVec::with_capacity(raw_children.len());
    for &child in raw_children {
        if let Some(cs) = arena.styles.get(child as usize)
            && cs.display != DisplayMode::None
        {
            visible_children.push(child);
        }
    }

    if visible_children.is_empty() {
        if min_dim != Dimension::Auto {
            let min_val = LayoutEngine::resolve_min(min_dim, parent_basis);
            return Some(LayoutEngine::clamp_dimension(
                min_val,
                min_dim,
                max_dim,
                parent_basis,
            ));
        }
        return None;
    }

    let (inset_top, inset_right, inset_bottom, inset_left) = LayoutEngine::content_insets(style);
    let inset_h = inset_left + inset_right;
    let inset_v = inset_top + inset_bottom;
    let node_avail_w = resolve_avail(
        style.width,
        style.min_width,
        style.max_width,
        parent_size.width,
    );
    let node_avail_h = resolve_avail(
        style.height,
        style.min_height,
        style.max_height,
        parent_size.height,
    );
    let child_inner = Size::new(
        (node_avail_w - inset_h).max(0.0),
        (node_avail_h - inset_v).max(0.0),
    );

    let is_row = style.direction == FlexDirection::Row;
    let axis_role =
        if (axis == MeasureAxis::Width && is_row) || (axis == MeasureAxis::Height && !is_row) {
            AxisRole::Main
        } else {
            AxisRole::Cross
        };
    let inset_axis = if axis == MeasureAxis::Width {
        inset_h
    } else {
        inset_v
    };
    let total_spacing = style.spacing * count_to_f32(visible_children.len().saturating_sub(1));

    let measured = aggregate_children_extent(
        arena,
        &visible_children,
        axis,
        axis_role,
        child_inner,
        inset_axis,
        total_spacing,
    );

    measured.map(|v| LayoutEngine::clamp_dimension(v, min_dim, max_dim, parent_basis))
}

fn resolve_avail(dim: Dimension, min_d: Dimension, max_d: Dimension, parent_extent: f32) -> f32 {
    match dim {
        Dimension::Points(p) => LayoutEngine::clamp_dimension(p, min_d, max_d, parent_extent),
        Dimension::Percent(pct) => {
            LayoutEngine::clamp_dimension(parent_extent * pct, min_d, max_d, parent_extent)
        }
        Dimension::Auto | Dimension::Fill => parent_extent,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AxisRole {
    Main,
    Cross,
}

fn aggregate_children_extent(
    arena: &LayoutArena,
    visible_children: &[u32],
    axis: MeasureAxis,
    axis_role: AxisRole,
    child_inner: Size,
    inset_axis: f32,
    total_spacing: f32,
) -> Option<f32> {
    let is_main_axis = axis_role == AxisRole::Main;
    let mut acc = 0.0f32;
    let mut has_any = false;
    for &child in visible_children {
        let cm = arena.styles[child as usize].margin;
        let margin_axis = match axis {
            MeasureAxis::Width => cm.left + cm.right,
            MeasureAxis::Height => cm.top + cm.bottom,
        };
        if let Some(extent) = measure_intrinsic_extent(arena, child, axis, child_inner) {
            let outer = extent + margin_axis;
            if is_main_axis {
                acc += outer;
            } else {
                acc = acc.max(outer);
            }
            has_any = true;
        }
    }

    if has_any {
        let gap = if is_main_axis { total_spacing } else { 0.0 };
        Some(acc + gap + inset_axis)
    } else if inset_axis > 0.0 {
        Some(inset_axis)
    } else {
        None
    }
}
