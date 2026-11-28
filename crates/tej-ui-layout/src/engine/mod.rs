//! Zero-allocation flat-SoA Flexbox and CSS Grid layout engine.

mod flex;
mod grid;
mod measure;

use crate::geometry::{Point, Rect, Size};
use crate::storage::LayoutArena;
use crate::style::{Dimension, DisplayMode, LayoutStyle};

/// Scope of a layout pass distinguishing the root viewport from child nodes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum LayoutScope {
    Root,
    Child,
}

/// Axis along which intrinsic content size is measured.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum MeasureAxis {
    Width,
    Height,
}

/// Lossless conversion for bounded UI child/grid counts (`<= 65_535`) to `f32`.
#[inline]
pub(crate) fn count_to_f32(count: usize) -> f32 {
    f32::from(u16::try_from(count).unwrap_or(u16::MAX))
}

/// High-performance, zero-allocation flex layout engine operating over flat SoA storage.
#[derive(Debug)]
pub struct LayoutEngine;

impl LayoutEngine {
    /// Compute layout for root node given available window/parent bounds.
    pub fn compute_layout(arena: &mut LayoutArena, root: u32, available_size: Size) {
        if (root as usize) >= arena.len() {
            return;
        }
        Self::compute_node_layout(arena, root, Point::ZERO, available_size, LayoutScope::Root);
    }

    #[inline]
    pub(crate) fn content_insets(style: &LayoutStyle) -> (f32, f32, f32, f32) {
        (
            style.padding.top + style.border.top,
            style.padding.right + style.border.right,
            style.padding.bottom + style.border.bottom,
            style.padding.left + style.border.left,
        )
    }

    #[inline]
    pub(crate) fn resolve_min(min_dim: Dimension, basis: f32) -> f32 {
        match min_dim {
            Dimension::Points(p) => p.max(0.0),
            Dimension::Percent(pct) => (basis * pct).max(0.0),
            Dimension::Fill => basis.max(0.0),
            Dimension::Auto => 0.0,
        }
    }

    #[inline]
    pub(crate) fn resolve_max(max_dim: Dimension, basis: f32) -> f32 {
        match max_dim {
            Dimension::Points(p) => p.max(0.0),
            Dimension::Percent(pct) => (basis * pct).max(0.0),
            Dimension::Fill => basis.max(0.0),
            Dimension::Auto => f32::INFINITY,
        }
    }

    #[inline]
    pub(crate) fn clamp_dimension(
        value: f32,
        min_dim: Dimension,
        max_dim: Dimension,
        basis: f32,
    ) -> f32 {
        let min_val = Self::resolve_min(min_dim, basis);
        let max_val = Self::resolve_max(max_dim, basis);
        // CSS precedence: min wins over max if min > max
        value.min(max_val).max(min_val).max(0.0)
    }

    pub(crate) fn zero_subtree(arena: &mut LayoutArena, node: u32) {
        if (node as usize) >= arena.len() {
            return;
        }
        arena.computed_bounds[node as usize] = Rect::ZERO;
        arena.dirty[node as usize] = false;
        let children = arena.children[node as usize].clone();
        for child in children {
            Self::zero_subtree(arena, child);
        }
    }

    pub(crate) fn compute_node_layout(
        arena: &mut LayoutArena,
        node: u32,
        origin: Point,
        parent_size: Size,
        scope: LayoutScope,
    ) {
        let mut style = arena.styles[node as usize];
        if style.display == DisplayMode::None {
            Self::zero_subtree(arena, node);
            return;
        }

        let (width_overridden, height_overridden) = apply_container_query(&mut style, parent_size);
        let scope_src = if scope == LayoutScope::Root {
            ExtentSource::Parent
        } else {
            ExtentSource::Direct
        };
        let width = resolve_node_extent(
            if width_overridden {
                ExtentSource::Parent
            } else {
                scope_src
            },
            style.width,
            style.min_width,
            style.max_width,
            parent_size.width,
        );
        let height = resolve_node_extent(
            if height_overridden {
                ExtentSource::Parent
            } else {
                scope_src
            },
            style.height,
            style.min_height,
            style.max_height,
            parent_size.height,
        );

        arena.computed_bounds[node as usize] = Rect::new(origin.x, origin.y, width, height);
        if arena.children[node as usize].is_empty() {
            arena.dirty[node as usize] = false;
            return;
        }

        let mut children = collect_visible_children(arena, node);
        if children.is_empty() {
            arena.dirty[node as usize] = false;
            return;
        }
        if children
            .iter()
            .any(|&c| arena.styles[c as usize].order != 0)
        {
            children.sort_by_key(|&c| arena.styles[c as usize].order);
        }

        let (inset_top, inset_right, inset_bottom, inset_left) = Self::content_insets(&style);
        let inner_origin = Point::new(origin.x + inset_left, origin.y + inset_top);
        let inner_width = (width - inset_left - inset_right).max(0.0);
        let inner_height = (height - inset_top - inset_bottom).max(0.0);

        if style.display == DisplayMode::Grid {
            grid::compute_grid_layout(
                arena,
                &children,
                style,
                inner_origin,
                inner_width,
                inner_height,
            );
        } else {
            flex::compute_flex_layout(
                arena,
                &children,
                style,
                inner_origin,
                Size::new(inner_width, inner_height),
            );
        }

        arena.dirty[node as usize] = false;
    }
}

fn apply_container_query(style: &mut LayoutStyle, parent_size: Size) -> (bool, bool) {
    let mut width_overridden = false;
    let mut height_overridden = false;
    if let Some(ref q) = style.container_query
        && q.matches(parent_size.width, parent_size.height)
    {
        if let Some(dir) = q.direction_override {
            style.direction = dir;
        }
        if let Some(wrap) = q.wrap_override {
            style.flex_wrap = wrap;
        }
        if let Some(w) = q.width_override {
            style.width = w;
            width_overridden = true;
        }
        if let Some(h) = q.height_override {
            style.height = h;
            height_overridden = true;
        }
        if let Some(sp) = q.spacing_override {
            style.spacing = sp;
        }
    }
    (width_overridden, height_overridden)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExtentSource {
    Parent,
    Direct,
}

fn resolve_node_extent(
    source: ExtentSource,
    dim: Dimension,
    min_dim: Dimension,
    max_dim: Dimension,
    parent_extent: f32,
) -> f32 {
    if source == ExtentSource::Parent {
        let raw = match dim {
            Dimension::Points(p) => p,
            Dimension::Percent(pct) => parent_extent * pct,
            Dimension::Auto | Dimension::Fill => parent_extent,
        };
        LayoutEngine::clamp_dimension(raw, min_dim, max_dim, parent_extent)
    } else {
        parent_extent.max(0.0)
    }
}

fn collect_visible_children(arena: &mut LayoutArena, node: u32) -> smallvec::SmallVec<[u32; 8]> {
    let raw_children = arena.children[node as usize].clone();
    let mut children = smallvec::SmallVec::with_capacity(raw_children.len());
    for child in raw_children {
        if arena.styles[child as usize].display == DisplayMode::None {
            LayoutEngine::zero_subtree(arena, child);
        } else {
            children.push(child);
        }
    }
    children
}
