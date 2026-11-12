use crate::geometry::{Point, Rect, Size};
use crate::storage::LayoutArena;
use crate::style::{Dimension, FlexDirection};

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
        let style = arena.styles[node as usize];

        // Resolve width and height
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

        let children = arena.children[node as usize].clone();
        if children.is_empty() {
            arena.dirty[node as usize] = false;
            return;
        }

        let padding = style.padding;
        let inner_origin = Point::new(origin.x + padding.left, origin.y + padding.top);
        let inner_width = (width - padding.left - padding.right).max(0.0);
        let inner_height = (height - padding.top - padding.bottom).max(0.0);
        let _inner_size = Size::new(inner_width, inner_height);

        let is_row = style.direction == FlexDirection::Row;
        let spacing = style.spacing;

        let mut current_offset = 0.0;
        let num_children = children.len();

        for (idx, &child) in children.iter().enumerate() {
            let child_style = arena.styles[child as usize];

            let child_w = match child_style.width {
                Dimension::Points(p) => p,
                Dimension::Percent(pct) => inner_width * pct,
                Dimension::Auto => {
                    if is_row {
                        (inner_width - (spacing * (num_children.saturating_sub(1) as f32)))
                            / (num_children as f32).max(1.0)
                    } else {
                        inner_width
                    }
                }
            };

            let child_h = match child_style.height {
                Dimension::Points(p) => p,
                Dimension::Percent(pct) => inner_height * pct,
                Dimension::Auto => {
                    if is_row {
                        inner_height
                    } else {
                        (inner_height - (spacing * (num_children.saturating_sub(1) as f32)))
                            / (num_children as f32).max(1.0)
                    }
                }
            };

            let child_origin = if is_row {
                Point::new(inner_origin.x + current_offset, inner_origin.y)
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

        arena.dirty[node as usize] = false;
    }
}
