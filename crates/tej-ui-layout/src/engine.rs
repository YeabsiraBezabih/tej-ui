use crate::geometry::{Point, Rect, Size};
use crate::storage::LayoutArena;
use crate::style::{Dimension, DisplayMode, FlexDirection, FlexWrap};

/// High-performance, zero-allocation flex layout engine operating over flat SoA storage.
#[derive(Debug)]
pub struct LayoutEngine;

impl LayoutEngine {
    /// Compute layout for root node given available window/parent bounds.
    pub fn compute_layout(arena: &mut LayoutArena, root: u32, available_size: Size) {
        if root as usize >= arena.len() {
            return;
        }

        Self::compute_node_layout(arena, root, Point::ZERO, available_size);
    }

    fn compute_node_layout(arena: &mut LayoutArena, node: u32, origin: Point, parent_size: Size) {
        let mut style = arena.styles[node as usize];

        // Evaluate responsive container queries against parent container dimensions
        if let Some(ref q) = style.container_query {
            if q.matches(parent_size.width, parent_size.height) {
                if let Some(dir) = q.direction_override {
                    style.direction = dir;
                }
                if let Some(wrap) = q.wrap_override {
                    style.flex_wrap = wrap;
                }
                if let Some(w) = q.width_override {
                    style.width = w;
                }
                if let Some(h) = q.height_override {
                    style.height = h;
                }
                if let Some(sp) = q.spacing_override {
                    style.spacing = sp;
                }
            }
        }

        // Resolve width and height from allocated parent constraints
        let width = match style.width {
            Dimension::Points(p) => p,
            Dimension::Percent(pct) => parent_size.width * pct,
            Dimension::Auto => parent_size.width,
        };

        let height = match style.height {
            Dimension::Points(p) => p,
            Dimension::Percent(pct) => parent_size.height * pct,
            Dimension::Auto => parent_size.height,
        };

        let rect = Rect::new(origin.x, origin.y, width, height);
        arena.computed_bounds[node as usize] = rect;

        if arena.children[node as usize].is_empty() {
            arena.dirty[node as usize] = false;
            return;
        }

        // Clone SmallVec<[u32; 4]> (zero-allocation for nodes with <= 4 children)
        let children = arena.children[node as usize].clone();

        let padding = style.padding;
        let inner_origin = Point::new(origin.x + padding.left, origin.y + padding.top);
        let inner_width = (width - padding.left - padding.right).max(0.0);
        let inner_height = (height - padding.top - padding.bottom).max(0.0);
        let _inner_size = Size::new(inner_width, inner_height);

        // CSS Grid Layout Mode
        if style.display == DisplayMode::Grid {
            let cols = style.grid_columns.max(1) as f32;
            let col_gap = style.grid_column_gap.max(0.0);
            let row_gap = style.grid_row_gap.max(0.0);
            let total_col_gaps = col_gap * (cols - 1.0).max(0.0);
            let col_width = ((inner_width - total_col_gaps) / cols).max(0.0);

            let mut col_cursor = 0u16;
            let mut row_y = 0.0f32;
            let mut row_h = 0.0f32;

            for &child in &children {
                let cs = arena.styles[child as usize];
                let span = cs.col_span.max(1).min(style.grid_columns.max(1));

                if col_cursor > 0 && col_cursor + span > style.grid_columns.max(1) {
                    col_cursor = 0;
                    row_y += row_h + row_gap;
                    row_h = 0.0;
                }

                let child_x = inner_origin.x + (col_cursor as f32) * (col_width + col_gap);
                let child_w = (span as f32) * col_width + ((span - 1) as f32) * col_gap;
                let child_y = inner_origin.y + row_y;

                let child_h = match cs.height {
                    Dimension::Points(p) => p,
                    Dimension::Percent(pct) => inner_height * pct,
                    Dimension::Auto => Self::measure_intrinsic_extent(
                        arena,
                        child,
                        false,
                        Size::new(child_w, inner_height),
                    )
                    .unwrap_or(24.0),
                };

                Self::compute_node_layout(
                    arena,
                    child,
                    Point::new(child_x, child_y),
                    Size::new(child_w, child_h),
                );

                row_h = row_h.max(child_h);
                col_cursor += span;
            }

            arena.dirty[node as usize] = false;
            return;
        }

        let is_row = style.direction == FlexDirection::Row;
        let is_rtl = style.layout_direction.is_rtl();
        let spacing = style.spacing;
        let num_children = children.len();
        let total_spacing = spacing * (num_children.saturating_sub(1) as f32);

        // First pass: sum fixed/percent main-axis extents and count flex (Auto) items
        let mut fixed_main_extent = 0.0f32;
        let mut auto_count = 0usize;
        let mut total_flex_grow = 0.0f32;

        for &child in &children {
            let cs = arena.styles[child as usize];
            if cs.flex_grow > 0.0 {
                total_flex_grow += cs.flex_grow;
                continue;
            }
            if is_row {
                match cs.width {
                    Dimension::Points(p) => fixed_main_extent += p,
                    Dimension::Percent(pct) => fixed_main_extent += inner_width * pct,
                    Dimension::Auto => {
                        if let Some(w) =
                            Self::measure_intrinsic_extent(arena, child, true, _inner_size)
                        {
                            fixed_main_extent += w;
                        } else {
                            auto_count += 1;
                        }
                    }
                }
            } else {
                match cs.height {
                    Dimension::Points(p) => fixed_main_extent += p,
                    Dimension::Percent(pct) => fixed_main_extent += inner_height * pct,
                    Dimension::Auto => {
                        if let Some(h) =
                            Self::measure_intrinsic_extent(arena, child, false, _inner_size)
                        {
                            fixed_main_extent += h;
                        } else {
                            auto_count += 1;
                        }
                    }
                }
            }
        }

        let avail_main_extent = if is_row { inner_width } else { inner_height };
        let remaining_main = (avail_main_extent - fixed_main_extent - total_spacing).max(0.0);
        let auto_extent = if total_flex_grow > 0.0 {
            0.0
        } else if auto_count > 0 {
            remaining_main / (auto_count as f32)
        } else {
            0.0
        };

        if is_row && style.flex_wrap == FlexWrap::Wrap {
            let mut line_x = 0.0f32;
            let mut line_y = 0.0f32;
            let mut line_height = 0.0f32;

            for &child in children.iter() {
                let child_style = arena.styles[child as usize];
                let child_w = match child_style.width {
                    Dimension::Points(p) => p,
                    Dimension::Percent(pct) => inner_width * pct,
                    Dimension::Auto => {
                        Self::measure_intrinsic_extent(arena, child, true, _inner_size)
                            .unwrap_or(auto_extent.max(30.0))
                    }
                };
                let child_h = match child_style.height {
                    Dimension::Points(p) => p,
                    Dimension::Percent(pct) => inner_height * pct,
                    Dimension::Auto => {
                        Self::measure_intrinsic_extent(arena, child, false, _inner_size)
                            .unwrap_or(24.0)
                    }
                };

                if line_x > 0.0 && line_x + child_w > inner_width {
                    line_x = 0.0;
                    line_y += line_height + spacing;
                    line_height = 0.0;
                }

                let child_origin = if is_rtl {
                    Point::new(
                        inner_origin.x + inner_width - line_x - child_w,
                        inner_origin.y + line_y,
                    )
                } else {
                    Point::new(inner_origin.x + line_x, inner_origin.y + line_y)
                };

                Self::compute_node_layout(arena, child, child_origin, Size::new(child_w, child_h));

                line_height = line_height.max(child_h);
                line_x += child_w + spacing;
            }
        } else {
            let mut current_offset = 0.0;

            for (idx, &child) in children.iter().enumerate() {
                let child_style = arena.styles[child as usize];

                let child_w = if is_row && child_style.flex_grow > 0.0 && total_flex_grow > 0.0 {
                    (remaining_main * (child_style.flex_grow / total_flex_grow)).max(0.0)
                } else {
                    match child_style.width {
                        Dimension::Points(p) => p,
                        Dimension::Percent(pct) => inner_width * pct,
                        Dimension::Auto => {
                            if is_row {
                                Self::measure_intrinsic_extent(arena, child, true, _inner_size)
                                    .unwrap_or(auto_extent)
                            } else {
                                inner_width
                            }
                        }
                    }
                };

                let child_h = if !is_row && child_style.flex_grow > 0.0 && total_flex_grow > 0.0 {
                    (remaining_main * (child_style.flex_grow / total_flex_grow)).max(0.0)
                } else {
                    match child_style.height {
                        Dimension::Points(p) => p,
                        Dimension::Percent(pct) => inner_height * pct,
                        Dimension::Auto => {
                            if is_row {
                                inner_height
                            } else {
                                Self::measure_intrinsic_extent(arena, child, false, _inner_size)
                                    .unwrap_or(auto_extent)
                            }
                        }
                    }
                };

                let child_origin = if is_row {
                    if is_rtl {
                        let child_x = inner_origin.x + inner_width - current_offset - child_w;
                        Point::new(child_x, inner_origin.y)
                    } else {
                        Point::new(inner_origin.x + current_offset, inner_origin.y)
                    }
                } else {
                    Point::new(inner_origin.x, inner_origin.y + current_offset)
                };

                Self::compute_node_layout(arena, child, child_origin, Size::new(child_w, child_h));

                if is_row {
                    current_offset += child_w + if idx + 1 < num_children { spacing } else { 0.0 };
                } else {
                    current_offset += child_h + if idx + 1 < num_children { spacing } else { 0.0 };
                }
            }
        }

        arena.dirty[node as usize] = false;
    }

    /// Recursively calculate intrinsic content extents for a node without fixed dimensions.
    fn measure_intrinsic_extent(
        arena: &LayoutArena,
        node: u32,
        is_width: bool,
        parent_size: Size,
    ) -> Option<f32> {
        let style = arena.styles.get(node as usize)?;
        let dim = if is_width { style.width } else { style.height };
        match dim {
            Dimension::Points(p) => Some(p),
            Dimension::Percent(pct) => {
                let p = if is_width {
                    parent_size.width
                } else {
                    parent_size.height
                };
                Some(p * pct)
            }
            Dimension::Auto => {
                let is_row = style.direction == FlexDirection::Row;
                if (is_width && is_row && style.flex_grow > 0.0)
                    || (!is_width && !is_row && style.flex_grow > 0.0)
                {
                    return None;
                }
                let children = arena.children.get(node as usize)?;
                if children.is_empty() {
                    return None;
                }
                let padding = style.padding;
                let spacing = style.spacing;
                let num_children = children.len();
                let total_spacing = spacing * (num_children.saturating_sub(1) as f32);

                if is_width {
                    if is_row {
                        let mut sum = 0.0;
                        let mut has_any = false;
                        for &child in children {
                            if let Some(w) =
                                Self::measure_intrinsic_extent(arena, child, true, parent_size)
                            {
                                sum += w;
                                has_any = true;
                            }
                        }
                        if has_any {
                            Some(sum + total_spacing + padding.left + padding.right)
                        } else if padding.left + padding.right > 0.0 {
                            Some(padding.left + padding.right)
                        } else {
                            None
                        }
                    } else {
                        let mut max_w: f32 = 0.0;
                        let mut has_any = false;
                        for &child in children {
                            if let Some(w) =
                                Self::measure_intrinsic_extent(arena, child, true, parent_size)
                            {
                                max_w = max_w.max(w);
                                has_any = true;
                            }
                        }
                        if has_any {
                            Some(max_w + padding.left + padding.right)
                        } else if padding.left + padding.right > 0.0 {
                            Some(padding.left + padding.right)
                        } else {
                            None
                        }
                    }
                } else if is_row {
                    let mut max_h: f32 = 0.0;
                    let mut has_any = false;
                    for &child in children {
                        if let Some(h) =
                            Self::measure_intrinsic_extent(arena, child, false, parent_size)
                        {
                            max_h = max_h.max(h);
                            has_any = true;
                        }
                    }
                    if has_any {
                        Some(max_h + padding.top + padding.bottom)
                    } else if padding.top + padding.bottom > 0.0 {
                        Some(padding.top + padding.bottom)
                    } else {
                        None
                    }
                } else {
                    let mut sum = 0.0;
                    let mut has_any = false;
                    for &child in children {
                        if let Some(h) =
                            Self::measure_intrinsic_extent(arena, child, false, parent_size)
                        {
                            sum += h;
                            has_any = true;
                        }
                    }
                    if has_any {
                        Some(sum + total_spacing + padding.top + padding.bottom)
                    } else if padding.top + padding.bottom > 0.0 {
                        Some(padding.top + padding.bottom)
                    } else {
                        None
                    }
                }
            }
        }
    }
}
