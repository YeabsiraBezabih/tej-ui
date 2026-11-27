use crate::geometry::{Point, Rect, Size};
use crate::storage::LayoutArena;
use crate::style::{AlignItems, Dimension, DisplayMode, FlexDirection, FlexWrap, JustifyContent};

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
        let inner_size = Size::new(inner_width, inner_height);

        // CSS Grid Layout Mode (supports col_span, row_span, and child margins)
        if style.display == DisplayMode::Grid {
            Self::compute_grid_layout(
                arena,
                node,
                &children,
                style,
                inner_origin,
                inner_width,
                inner_height,
            );
            arena.dirty[node as usize] = false;
            return;
        }

        let is_row = style.direction == FlexDirection::Row;
        let is_rtl = style.layout_direction.is_rtl();
        let spacing = style.spacing;
        let num_children = children.len();
        let total_spacing = spacing * (num_children.saturating_sub(1) as f32);

        // First pass: sum fixed/percent main-axis extents (including margins) and count flex/auto items
        let mut fixed_main_extent = 0.0f32;
        let mut auto_count = 0usize;
        let mut total_flex_grow = 0.0f32;

        for &child in &children {
            let cs = arena.styles[child as usize];
            let main_margin = if is_row {
                cs.margin.left + cs.margin.right
            } else {
                cs.margin.top + cs.margin.bottom
            };
            fixed_main_extent += main_margin;

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
                            Self::measure_intrinsic_extent(arena, child, true, inner_size)
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
                            Self::measure_intrinsic_extent(arena, child, false, inner_size)
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

        // Distribute remaining main space without collapsing Auto siblings when flex_grow > 0
        let total_flex_weight = total_flex_grow + (auto_count as f32);
        let auto_extent = if auto_count > 0 && total_flex_weight > 0.0 {
            remaining_main / total_flex_weight
        } else {
            0.0
        };

        if is_row && style.flex_wrap == FlexWrap::Wrap {
            let mut line_x = 0.0f32;
            let mut line_y = 0.0f32;
            let mut line_height = 0.0f32;

            for &child in children.iter() {
                let cs = arena.styles[child as usize];
                let child_w = match cs.width {
                    Dimension::Points(p) => p,
                    Dimension::Percent(pct) => inner_width * pct,
                    Dimension::Auto => {
                        Self::measure_intrinsic_extent(arena, child, true, inner_size)
                            .unwrap_or(auto_extent.max(30.0))
                    }
                };
                let child_h = match cs.height {
                    Dimension::Points(p) => p,
                    Dimension::Percent(pct) => inner_height * pct,
                    Dimension::Auto => {
                        Self::measure_intrinsic_extent(arena, child, false, inner_size)
                            .unwrap_or(24.0)
                    }
                };

                let outer_w = child_w + cs.margin.left + cs.margin.right;
                let outer_h = child_h + cs.margin.top + cs.margin.bottom;

                if line_x > 0.0 && line_x + outer_w > inner_width {
                    line_x = 0.0;
                    line_y += line_height + spacing;
                    line_height = 0.0;
                }

                let child_origin = if is_rtl {
                    Point::new(
                        inner_origin.x + inner_width - line_x - cs.margin.right - child_w,
                        inner_origin.y + line_y + cs.margin.top,
                    )
                } else {
                    Point::new(
                        inner_origin.x + line_x + cs.margin.left,
                        inner_origin.y + line_y + cs.margin.top,
                    )
                };

                Self::compute_node_layout(arena, child, child_origin, Size::new(child_w, child_h));

                line_height = line_height.max(outer_h);
                line_x += outer_w + spacing;
            }
        } else if !is_row && style.flex_wrap == FlexWrap::Wrap {
            let mut line_x = 0.0f32;
            let mut line_y = 0.0f32;
            let mut line_width = 0.0f32;

            for &child in children.iter() {
                let cs = arena.styles[child as usize];
                let child_w = match cs.width {
                    Dimension::Points(p) => p,
                    Dimension::Percent(pct) => inner_width * pct,
                    Dimension::Auto => {
                        Self::measure_intrinsic_extent(arena, child, true, inner_size)
                            .unwrap_or(30.0)
                    }
                };
                let child_h = match cs.height {
                    Dimension::Points(p) => p,
                    Dimension::Percent(pct) => inner_height * pct,
                    Dimension::Auto => {
                        Self::measure_intrinsic_extent(arena, child, false, inner_size)
                            .unwrap_or(auto_extent.max(24.0))
                    }
                };

                let outer_w = child_w + cs.margin.left + cs.margin.right;
                let outer_h = child_h + cs.margin.top + cs.margin.bottom;

                if line_y > 0.0 && line_y + outer_h > inner_height {
                    line_y = 0.0;
                    line_x += line_width + spacing;
                    line_width = 0.0;
                }

                let child_origin = if is_rtl {
                    Point::new(
                        inner_origin.x + inner_width - line_x - cs.margin.right - child_w,
                        inner_origin.y + line_y + cs.margin.top,
                    )
                } else {
                    Point::new(
                        inner_origin.x + line_x + cs.margin.left,
                        inner_origin.y + line_y + cs.margin.top,
                    )
                };

                Self::compute_node_layout(arena, child, child_origin, Size::new(child_w, child_h));

                line_width = line_width.max(outer_w);
                line_y += outer_h + spacing;
            }
        } else {
            // Compute resolved child sizes first so justify_content and align_items can position accurately
            let mut child_sizes: smallvec::SmallVec<[(f32, f32); 8]> =
                smallvec::SmallVec::with_capacity(num_children);
            let mut total_children_main = 0.0f32;

            for &child in children.iter() {
                let cs = arena.styles[child as usize];
                let margin_h = cs.margin.left + cs.margin.right;
                let margin_v = cs.margin.top + cs.margin.bottom;

                let child_w = if is_row && cs.flex_grow > 0.0 && total_flex_weight > 0.0 {
                    (remaining_main * (cs.flex_grow / total_flex_weight)).max(0.0)
                } else {
                    match cs.width {
                        Dimension::Points(p) => p,
                        Dimension::Percent(pct) => inner_width * pct,
                        Dimension::Auto => {
                            if is_row {
                                Self::measure_intrinsic_extent(arena, child, true, inner_size)
                                    .unwrap_or(auto_extent)
                            } else {
                                (inner_width - margin_h).max(0.0)
                            }
                        }
                    }
                };

                let child_h = if !is_row && cs.flex_grow > 0.0 && total_flex_weight > 0.0 {
                    (remaining_main * (cs.flex_grow / total_flex_weight)).max(0.0)
                } else {
                    match cs.height {
                        Dimension::Points(p) => p,
                        Dimension::Percent(pct) => inner_height * pct,
                        Dimension::Auto => {
                            if is_row {
                                (inner_height - margin_v).max(0.0)
                            } else {
                                Self::measure_intrinsic_extent(arena, child, false, inner_size)
                                    .unwrap_or(auto_extent)
                            }
                        }
                    }
                };

                if is_row {
                    total_children_main += child_w + margin_h;
                } else {
                    total_children_main += child_h + margin_v;
                }
                child_sizes.push((child_w, child_h));
            }

            // Apply justify_content along main axis
            let free_main = (avail_main_extent - total_children_main - total_spacing).max(0.0);
            let (start_offset, effective_spacing) = match style.justify_content {
                JustifyContent::FlexStart => (0.0, spacing),
                JustifyContent::Center => (free_main * 0.5, spacing),
                JustifyContent::FlexEnd => (free_main, spacing),
                JustifyContent::SpaceBetween => {
                    if num_children > 1 {
                        (0.0, spacing + free_main / ((num_children - 1) as f32))
                    } else {
                        (0.0, spacing)
                    }
                }
                JustifyContent::SpaceAround => {
                    if num_children > 0 {
                        let per_item = free_main / (num_children as f32);
                        (per_item * 0.5, spacing + per_item)
                    } else {
                        (0.0, spacing)
                    }
                }
            };

            let mut current_offset = start_offset;

            for (idx, (&child, &(mut child_w, mut child_h))) in
                children.iter().zip(child_sizes.iter()).enumerate()
            {
                let cs = arena.styles[child as usize];

                // Apply align_items along cross axis
                let cross_offset = if is_row {
                    if style.align_items == AlignItems::Stretch && cs.height == Dimension::Auto {
                        child_h = (inner_height - cs.margin.top - cs.margin.bottom).max(0.0);
                    }
                    let free_cross =
                        (inner_height - child_h - cs.margin.top - cs.margin.bottom).max(0.0);
                    match style.align_items {
                        AlignItems::FlexStart | AlignItems::Stretch => cs.margin.top,
                        AlignItems::Center => cs.margin.top + free_cross * 0.5,
                        AlignItems::FlexEnd => cs.margin.top + free_cross,
                    }
                } else {
                    if style.align_items == AlignItems::Stretch && cs.width == Dimension::Auto {
                        child_w = (inner_width - cs.margin.left - cs.margin.right).max(0.0);
                    }
                    let free_cross =
                        (inner_width - child_w - cs.margin.left - cs.margin.right).max(0.0);
                    match style.align_items {
                        AlignItems::FlexStart | AlignItems::Stretch => cs.margin.left,
                        AlignItems::Center => cs.margin.left + free_cross * 0.5,
                        AlignItems::FlexEnd => cs.margin.left + free_cross,
                    }
                };

                let child_origin = if is_row {
                    if is_rtl {
                        let child_x = inner_origin.x + inner_width
                            - current_offset
                            - cs.margin.right
                            - child_w;
                        Point::new(child_x, inner_origin.y + cross_offset)
                    } else {
                        Point::new(
                            inner_origin.x + current_offset + cs.margin.left,
                            inner_origin.y + cross_offset,
                        )
                    }
                } else {
                    Point::new(
                        inner_origin.x + cross_offset,
                        inner_origin.y + current_offset + cs.margin.top,
                    )
                };

                Self::compute_node_layout(arena, child, child_origin, Size::new(child_w, child_h));

                let gap = if idx + 1 < num_children {
                    effective_spacing
                } else {
                    0.0
                };
                if is_row {
                    current_offset += cs.margin.left + child_w + cs.margin.right + gap;
                } else {
                    current_offset += cs.margin.top + child_h + cs.margin.bottom + gap;
                }
            }
        }

        arena.dirty[node as usize] = false;
    }

    fn compute_grid_layout(
        arena: &mut LayoutArena,
        _node: u32,
        children: &[u32],
        style: crate::style::LayoutStyle,
        inner_origin: Point,
        inner_width: f32,
        inner_height: f32,
    ) {
        let num_cols = style.grid_columns.max(1) as usize;
        let col_gap = style.grid_column_gap.max(0.0);
        let row_gap = style.grid_row_gap.max(0.0);
        let total_col_gaps = col_gap * ((num_cols as f32) - 1.0).max(0.0);
        let col_width = ((inner_width - total_col_gaps) / (num_cols as f32)).max(0.0);

        // Pass 1: 2D occupancy grid placement supporting both col_span and row_span
        let mut occupied: Vec<Vec<bool>> = Vec::new();
        let mut row_heights: Vec<f32> = Vec::new();

        struct PlacedItem {
            child: u32,
            row: usize,
            col: usize,
            col_span: usize,
            row_span: usize,
            measured_h: f32,
        }

        let mut placements: Vec<PlacedItem> = Vec::with_capacity(children.len());

        for &child in children {
            let cs = arena.styles[child as usize];
            let c_span = (cs.col_span.max(1) as usize).min(num_cols);
            let r_span = cs.row_span.max(1) as usize;

            // Find first (row, col) where the c_span x r_span block is unoccupied
            let mut placed_row = 0usize;
            let mut placed_col;
            'search: loop {
                while occupied.len() < placed_row + r_span {
                    occupied.push(vec![false; num_cols]);
                    row_heights.push(0.0);
                }
                for col in 0..=(num_cols - c_span) {
                    let mut fits = true;
                    'check: for r in placed_row..(placed_row + r_span) {
                        for c in col..(col + c_span) {
                            if occupied[r][c] {
                                fits = false;
                                break 'check;
                            }
                        }
                    }
                    if fits {
                        placed_col = col;
                        break 'search;
                    }
                }
                placed_row += 1;
            }

            for r in placed_row..(placed_row + r_span) {
                for c in placed_col..(placed_col + c_span) {
                    occupied[r][c] = true;
                }
            }

            let cell_w = (c_span as f32) * col_width + ((c_span - 1) as f32) * col_gap;
            let content_w = (cell_w - cs.margin.left - cs.margin.right).max(0.0);
            let measured_h = match cs.height {
                Dimension::Points(p) => p,
                Dimension::Percent(pct) => inner_height * pct,
                Dimension::Auto => Self::measure_intrinsic_extent(
                    arena,
                    child,
                    false,
                    Size::new(content_w, inner_height),
                )
                .unwrap_or(24.0),
            };
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

        // Ensure multi-row-span items have sufficient combined track height across their spanned rows
        for item in &placements {
            if item.row_span > 1 {
                let cs = arena.styles[item.child as usize];
                let outer_h = item.measured_h + cs.margin.top + cs.margin.bottom;
                let span_gaps = row_gap * ((item.row_span - 1) as f32);
                let current_span_h: f32 = row_heights[item.row..(item.row + item.row_span)]
                    .iter()
                    .sum::<f32>()
                    + span_gaps;
                if outer_h > current_span_h {
                    let deficit_per_row = (outer_h - current_span_h) / (item.row_span as f32);
                    for h in &mut row_heights[item.row..(item.row + item.row_span)] {
                        *h += deficit_per_row;
                    }
                }
            }
        }

        // Compute cumulative Y offsets for each grid row
        let mut row_y_offsets = Vec::with_capacity(row_heights.len());
        let mut acc_y = 0.0f32;
        for &rh in &row_heights {
            row_y_offsets.push(acc_y);
            acc_y += rh + row_gap;
        }

        // Pass 2: Layout each placed item in its resolved grid area
        for item in placements {
            let cs = arena.styles[item.child as usize];
            let cell_x = inner_origin.x + (item.col as f32) * (col_width + col_gap);
            let cell_w =
                (item.col_span as f32) * col_width + ((item.col_span - 1) as f32) * col_gap;
            let cell_y = inner_origin.y + row_y_offsets[item.row];
            let cell_h = if item.row_span > 1 {
                row_heights[item.row..(item.row + item.row_span)]
                    .iter()
                    .sum::<f32>()
                    + row_gap * ((item.row_span - 1) as f32)
            } else {
                item.measured_h + cs.margin.top + cs.margin.bottom
            };

            let child_x = cell_x + cs.margin.left;
            let child_y = cell_y + cs.margin.top;
            let child_w = (cell_w - cs.margin.left - cs.margin.right).max(0.0);
            let child_h = (cell_h - cs.margin.top - cs.margin.bottom).max(0.0);

            Self::compute_node_layout(
                arena,
                item.child,
                Point::new(child_x, child_y),
                Size::new(child_w, child_h),
            );
        }
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
                            let cm = arena.styles[child as usize].margin;
                            if let Some(w) =
                                Self::measure_intrinsic_extent(arena, child, true, parent_size)
                            {
                                sum += w + cm.left + cm.right;
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
                            let cm = arena.styles[child as usize].margin;
                            if let Some(w) =
                                Self::measure_intrinsic_extent(arena, child, true, parent_size)
                            {
                                max_w = max_w.max(w + cm.left + cm.right);
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
                        let cm = arena.styles[child as usize].margin;
                        if let Some(h) =
                            Self::measure_intrinsic_extent(arena, child, false, parent_size)
                        {
                            max_h = max_h.max(h + cm.top + cm.bottom);
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
                        let cm = arena.styles[child as usize].margin;
                        if let Some(h) =
                            Self::measure_intrinsic_extent(arena, child, false, parent_size)
                        {
                            sum += h + cm.top + cm.bottom;
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
