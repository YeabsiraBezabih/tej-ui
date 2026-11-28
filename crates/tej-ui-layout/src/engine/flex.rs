//! Two-pass CSS Flexbox layout solver (grow, shrink, wrap, and alignment).

use super::measure::measure_intrinsic_extent;
use super::{LayoutEngine, LayoutScope, MeasureAxis, count_to_f32};
use crate::direction::LayoutDirection;
use crate::geometry::{Point, Size};
use crate::storage::LayoutArena;
use crate::style::{
    AlignItems, AlignSelf, Dimension, FlexDirection, FlexWrap, JustifyContent, LayoutStyle,
};

struct FlexPassData {
    base_mains: smallvec::SmallVec<[f32; 8]>,
    min_mains: smallvec::SmallVec<[f32; 8]>,
    max_mains: smallvec::SmallVec<[f32; 8]>,
    eff_grows: smallvec::SmallVec<[f32; 8]>,
    eff_shrinks: smallvec::SmallVec<[f32; 8]>,
    unsized_autos: smallvec::SmallVec<[bool; 8]>,
    free_space: f32,
    total_flex_grow: f32,
    total_flex_weight: f32,
    auto_extent: f32,
}

pub(crate) fn compute_flex_layout(
    arena: &mut LayoutArena,
    children: &[u32],
    style: LayoutStyle,
    inner_origin: Point,
    inner_size: Size,
) {
    let is_row = style.direction == FlexDirection::Row;
    let spacing = style.spacing;
    let total_spacing = spacing * count_to_f32(children.len().saturating_sub(1));
    let avail_main = if is_row {
        inner_size.width
    } else {
        inner_size.height
    };

    let pass = collect_flex_pass(
        arena,
        children,
        style.direction,
        inner_size,
        avail_main,
        total_spacing,
    );

    if style.flex_wrap == FlexWrap::Wrap {
        layout_wrapped(
            arena,
            children,
            style.direction,
            style.layout_direction,
            spacing,
            inner_origin,
            inner_size,
            pass.auto_extent,
        );
    } else {
        layout_nowrap(
            arena,
            children,
            style,
            inner_origin,
            inner_size,
            total_spacing,
            &pass,
        );
    }
}

fn collect_flex_pass(
    arena: &LayoutArena,
    children: &[u32],
    direction: FlexDirection,
    inner_size: Size,
    avail_main: f32,
    total_spacing: f32,
) -> FlexPassData {
    let is_row = direction == FlexDirection::Row;
    let n = children.len();
    let main_axis = if is_row {
        MeasureAxis::Width
    } else {
        MeasureAxis::Height
    };
    let mut base_mains = smallvec::SmallVec::with_capacity(n);
    let mut min_mains = smallvec::SmallVec::with_capacity(n);
    let mut max_mains = smallvec::SmallVec::with_capacity(n);
    let mut eff_grows = smallvec::SmallVec::with_capacity(n);
    let mut eff_shrinks = smallvec::SmallVec::with_capacity(n);
    let mut unsized_autos = smallvec::SmallVec::with_capacity(n);

    let mut fixed_main_extent = 0.0f32;
    let mut auto_count = 0usize;
    let mut total_flex_grow = 0.0f32;

    for &child in children {
        let cs = arena.styles[child as usize];
        let main_margin = if is_row {
            cs.margin.left + cs.margin.right
        } else {
            cs.margin.top + cs.margin.bottom
        };
        let (main_dim, min_main_dim, max_main_dim) = if is_row {
            (cs.width, cs.min_width, cs.max_width)
        } else {
            (cs.height, cs.min_height, cs.max_height)
        };
        let min_m = LayoutEngine::resolve_min(min_main_dim, avail_main);
        let max_m = LayoutEngine::resolve_max(max_main_dim, avail_main);
        let is_fill = main_dim == Dimension::Fill || cs.flex_basis == Dimension::Fill;
        let eff_grow = if is_fill {
            cs.flex_grow.max(1.0)
        } else {
            cs.flex_grow.max(0.0)
        };

        let (raw_base, is_unsized_auto) = match cs.flex_basis {
            Dimension::Points(p) => (p, false),
            Dimension::Percent(pct) => (avail_main * pct, false),
            Dimension::Fill => (0.0, false),
            Dimension::Auto => match main_dim {
                Dimension::Points(p) => (p, false),
                Dimension::Percent(pct) => (avail_main * pct, false),
                Dimension::Fill => (0.0, false),
                Dimension::Auto => {
                    if let Some(m) = measure_intrinsic_extent(arena, child, main_axis, inner_size) {
                        (m, false)
                    } else {
                        (0.0, eff_grow == 0.0)
                    }
                }
            },
        };

        let clamped_base = if is_unsized_auto && min_main_dim == Dimension::Auto {
            0.0
        } else {
            raw_base.min(max_m).max(min_m).max(0.0)
        };

        fixed_main_extent += main_margin + clamped_base;
        if eff_grow > 0.0 {
            total_flex_grow += eff_grow;
        } else if is_unsized_auto {
            auto_count += 1;
        }

        base_mains.push(clamped_base);
        min_mains.push(min_m);
        max_mains.push(max_m);
        eff_grows.push(eff_grow);
        eff_shrinks.push(cs.flex_shrink.max(0.0));
        unsized_autos.push(is_unsized_auto);
    }

    let free_space = avail_main - fixed_main_extent - total_spacing;
    let total_flex_weight = total_flex_grow + count_to_f32(auto_count);
    let auto_extent = if auto_count > 0 && total_flex_weight > 0.0 {
        free_space.max(0.0) / total_flex_weight
    } else {
        0.0
    };

    FlexPassData {
        base_mains,
        min_mains,
        max_mains,
        eff_grows,
        eff_shrinks,
        unsized_autos,
        free_space,
        total_flex_grow,
        total_flex_weight,
        auto_extent,
    }
}

#[allow(clippy::too_many_arguments)]
fn layout_wrapped(
    arena: &mut LayoutArena,
    children: &[u32],
    direction: FlexDirection,
    layout_direction: LayoutDirection,
    spacing: f32,
    inner_origin: Point,
    inner_size: Size,
    auto_extent: f32,
) {
    let is_row = direction == FlexDirection::Row;
    let is_rtl = layout_direction.is_rtl();
    let mut line_x = 0.0f32;
    let mut line_y = 0.0f32;
    let mut line_cross = 0.0f32;

    for &child in children {
        let cs = arena.styles[child as usize];
        let margin_h = cs.margin.left + cs.margin.right;
        let margin_v = cs.margin.top + cs.margin.bottom;
        let fallback_w = if is_row { auto_extent.max(30.0) } else { 30.0 };
        let fallback_h = if is_row { 24.0 } else { auto_extent.max(24.0) };

        let raw_w = match cs.width {
            Dimension::Points(p) => p,
            Dimension::Percent(pct) => inner_size.width * pct,
            Dimension::Fill => (inner_size.width - margin_h).max(0.0),
            Dimension::Auto => {
                measure_intrinsic_extent(arena, child, MeasureAxis::Width, inner_size)
                    .unwrap_or(fallback_w)
            }
        };
        let child_w =
            LayoutEngine::clamp_dimension(raw_w, cs.min_width, cs.max_width, inner_size.width);

        let raw_h = match cs.height {
            Dimension::Points(p) => p,
            Dimension::Percent(pct) => inner_size.height * pct,
            Dimension::Fill => (inner_size.height - margin_v).max(0.0),
            Dimension::Auto => {
                measure_intrinsic_extent(arena, child, MeasureAxis::Height, inner_size)
                    .unwrap_or(fallback_h)
            }
        };
        let child_h =
            LayoutEngine::clamp_dimension(raw_h, cs.min_height, cs.max_height, inner_size.height);

        let outer_w = child_w + margin_h;
        let outer_h = child_h + margin_v;

        if is_row {
            if line_x > 0.0 && line_x + outer_w > inner_size.width {
                line_x = 0.0;
                line_y += line_cross + spacing;
                line_cross = 0.0;
            }
        } else if line_y > 0.0 && line_y + outer_h > inner_size.height {
            line_y = 0.0;
            line_x += line_cross + spacing;
            line_cross = 0.0;
        }

        let origin_x = if is_rtl {
            inner_origin.x + inner_size.width - line_x - cs.margin.right - child_w
        } else {
            inner_origin.x + line_x + cs.margin.left
        };
        let child_origin = Point::new(origin_x, inner_origin.y + line_y + cs.margin.top);
        LayoutEngine::compute_node_layout(
            arena,
            child,
            child_origin,
            Size::new(child_w, child_h),
            LayoutScope::Child,
        );

        if is_row {
            line_cross = line_cross.max(outer_h);
            line_x += outer_w + spacing;
        } else {
            line_cross = line_cross.max(outer_w);
            line_y += outer_h + spacing;
        }
    }
}

fn resolve_main_extents(num_children: usize, pass: &FlexPassData) -> smallvec::SmallVec<[f32; 8]> {
    let mut resolved = pass.base_mains.clone();
    if pass.free_space > 0.0 && pass.total_flex_weight > 0.0 {
        for i in 0..num_children {
            if pass.eff_grows[i] > 0.0 {
                let grown = pass.base_mains[i]
                    + pass.free_space * (pass.eff_grows[i] / pass.total_flex_weight);
                resolved[i] = grown.min(pass.max_mains[i]).max(pass.min_mains[i]).max(0.0);
            } else if pass.unsized_autos[i] {
                let grown = pass.base_mains[i] + pass.free_space * (1.0 / pass.total_flex_weight);
                resolved[i] = grown.min(pass.max_mains[i]).max(pass.min_mains[i]).max(0.0);
            }
        }
    } else if pass.free_space < 0.0 {
        let mut remaining_overflow = -pass.free_space;
        let mut frozen: smallvec::SmallVec<[bool; 8]> = (0..num_children)
            .map(|i| pass.eff_shrinks[i] <= 0.0 || resolved[i] <= pass.min_mains[i])
            .collect();

        while remaining_overflow > 1e-4 {
            let total_scaled: f32 = (0..num_children)
                .filter(|&i| !frozen[i])
                .map(|i| pass.eff_shrinks[i] * resolved[i])
                .sum();
            if total_scaled <= 1e-6 {
                break;
            }
            let mut any_clamped = false;
            let mut shrunk = 0.0f32;
            for i in 0..num_children {
                if frozen[i] {
                    continue;
                }
                let delta =
                    remaining_overflow * ((pass.eff_shrinks[i] * resolved[i]) / total_scaled);
                let target = resolved[i] - delta;
                if target <= pass.min_mains[i] {
                    shrunk += resolved[i] - pass.min_mains[i];
                    resolved[i] = pass.min_mains[i];
                    frozen[i] = true;
                    any_clamped = true;
                } else {
                    shrunk += delta;
                    resolved[i] = target;
                }
            }
            remaining_overflow = (remaining_overflow - shrunk).max(0.0);
            if !any_clamped {
                break;
            }
        }
    }
    let _ = pass.total_flex_grow;
    resolved
}

fn layout_nowrap(
    arena: &mut LayoutArena,
    children: &[u32],
    style: LayoutStyle,
    inner_origin: Point,
    inner_size: Size,
    total_spacing: f32,
    pass: &FlexPassData,
) {
    let is_row = style.direction == FlexDirection::Row;
    let num_children = children.len();
    let resolved_mains = resolve_main_extents(num_children, pass);
    let avail_main = if is_row {
        inner_size.width
    } else {
        inner_size.height
    };
    let avail_cross = if is_row {
        inner_size.height
    } else {
        inner_size.width
    };
    let cross_axis = if is_row {
        MeasureAxis::Height
    } else {
        MeasureAxis::Width
    };

    let mut child_sizes: smallvec::SmallVec<[(f32, f32); 8]> =
        smallvec::SmallVec::with_capacity(num_children);
    let mut total_children_main = 0.0f32;

    for (i, &child) in children.iter().enumerate() {
        let cs = arena.styles[child as usize];
        let margin_h = cs.margin.left + cs.margin.right;
        let margin_v = cs.margin.top + cs.margin.bottom;
        let avail_c = (avail_cross - if is_row { margin_v } else { margin_h }).max(0.0);
        let eff_align = match cs.align_self {
            AlignSelf::Auto => style.align_items,
            AlignSelf::FlexStart => AlignItems::FlexStart,
            AlignSelf::Center => AlignItems::Center,
            AlignSelf::FlexEnd => AlignItems::FlexEnd,
            AlignSelf::Stretch => AlignItems::Stretch,
        };
        let (cross_dim, min_c, max_c) = if is_row {
            (cs.height, cs.min_height, cs.max_height)
        } else {
            (cs.width, cs.min_width, cs.max_width)
        };
        let raw_cross = match cross_dim {
            Dimension::Points(p) => p,
            Dimension::Percent(pct) => avail_cross * pct,
            Dimension::Fill => avail_c,
            Dimension::Auto => match eff_align {
                AlignItems::Center | AlignItems::FlexEnd => {
                    measure_intrinsic_extent(arena, child, cross_axis, inner_size)
                        .map_or(avail_c, |m| m.min(avail_c))
                }
                AlignItems::FlexStart | AlignItems::Stretch => avail_c,
            },
        };
        let resolved_cross = LayoutEngine::clamp_dimension(raw_cross, min_c, max_c, avail_cross);
        let (cw, ch) = if is_row {
            (resolved_mains[i], resolved_cross)
        } else {
            (resolved_cross, resolved_mains[i])
        };
        total_children_main += if is_row { cw + margin_h } else { ch + margin_v };
        child_sizes.push((cw, ch));
    }

    let free_main = (avail_main - total_children_main - total_spacing).max(0.0);
    let (start_offset, effective_spacing) = match style.justify_content {
        JustifyContent::Center => (free_main * 0.5, style.spacing),
        JustifyContent::FlexEnd => (free_main, style.spacing),
        JustifyContent::SpaceBetween if num_children > 1 => (
            0.0,
            style.spacing + free_main / count_to_f32(num_children - 1),
        ),
        JustifyContent::SpaceAround if num_children > 0 => {
            let per_item = free_main / count_to_f32(num_children);
            (per_item * 0.5, style.spacing + per_item)
        }
        _ => (0.0, style.spacing),
    };

    place_nowrap_children(
        arena,
        children,
        &child_sizes,
        style,
        inner_origin,
        inner_size,
        start_offset,
        effective_spacing,
    );
}

#[allow(clippy::too_many_arguments)]
fn place_nowrap_children(
    arena: &mut LayoutArena,
    children: &[u32],
    child_sizes: &[(f32, f32)],
    style: LayoutStyle,
    inner_origin: Point,
    inner_size: Size,
    start_offset: f32,
    effective_spacing: f32,
) {
    let is_row = style.direction == FlexDirection::Row;
    let is_rtl = style.layout_direction.is_rtl();
    let num_children = children.len();
    let mut current_offset = start_offset;
    for (idx, (&child, &(cw, ch))) in children.iter().zip(child_sizes.iter()).enumerate() {
        let cs = arena.styles[child as usize];
        let eff_align = match cs.align_self {
            AlignSelf::Auto => style.align_items,
            AlignSelf::FlexStart => AlignItems::FlexStart,
            AlignSelf::Center => AlignItems::Center,
            AlignSelf::FlexEnd => AlignItems::FlexEnd,
            AlignSelf::Stretch => AlignItems::Stretch,
        };
        let cross_offset = if is_row {
            let avail_c = (inner_size.height - cs.margin.top - cs.margin.bottom).max(0.0);
            match eff_align {
                AlignItems::Center => cs.margin.top + ((avail_c - ch) * 0.5).max(0.0),
                AlignItems::FlexEnd => cs.margin.top + (avail_c - ch).max(0.0),
                _ => cs.margin.top,
            }
        } else {
            let avail_c = (inner_size.width - cs.margin.left - cs.margin.right).max(0.0);
            match eff_align {
                AlignItems::Center => cs.margin.left + ((avail_c - cw) * 0.5).max(0.0),
                AlignItems::FlexEnd => cs.margin.left + (avail_c - cw).max(0.0),
                _ => cs.margin.left,
            }
        };

        let child_origin = if is_row {
            let cx = if is_rtl {
                inner_origin.x + inner_size.width - current_offset - cs.margin.right - cw
            } else {
                inner_origin.x + current_offset + cs.margin.left
            };
            Point::new(cx, inner_origin.y + cross_offset)
        } else {
            Point::new(
                inner_origin.x + cross_offset,
                inner_origin.y + current_offset + cs.margin.top,
            )
        };

        LayoutEngine::compute_node_layout(
            arena,
            child,
            child_origin,
            Size::new(cw, ch),
            LayoutScope::Child,
        );

        let gap = if idx + 1 < num_children {
            effective_spacing
        } else {
            0.0
        };
        current_offset += if is_row {
            cs.margin.left + cw + cs.margin.right + gap
        } else {
            cs.margin.top + ch + cs.margin.bottom + gap
        };
    }
}
