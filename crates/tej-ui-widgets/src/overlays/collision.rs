//! Collision-aware anchoring and auto-flip placement engine for overlays and popovers.

use std::fmt::Debug;
use tej_ui_layout::{Point, Rect, Size};

use crate::overlays::popover::PopoverPlacement;

/// Cross-axis alignment for floating overlays relative to an anchor.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PopoverAlignment {
    /// Align to the start edge of the anchor (e.g. left-aligned for top/bottom placement).
    Start,
    /// Center along the cross-axis of the anchor.
    #[default]
    Center,
    /// Align to the end edge of the anchor (e.g. right-aligned for top/bottom placement).
    End,
}

/// Parameters for computing collision-aware placement.
#[derive(Clone, Copy, Debug)]
pub struct CollisionPlacementOptions {
    /// Preferred placement direction.
    pub preferred_placement: PopoverPlacement,
    /// Preferred cross-axis alignment.
    pub alignment: PopoverAlignment,
    /// Offset distance between anchor and overlay along the main axis.
    pub offset: f32,
    /// Minimum clearance from viewport boundaries.
    pub viewport_padding: f32,
    /// Whether auto-flip is enabled.
    pub auto_flip: bool,
}

impl Default for CollisionPlacementOptions {
    fn default() -> Self {
        Self {
            preferred_placement: PopoverPlacement::Bottom,
            alignment: PopoverAlignment::Center,
            offset: 8.0,
            viewport_padding: 8.0,
            auto_flip: true,
        }
    }
}

/// Result of a collision-aware placement calculation.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct OverlayPlacementResult {
    /// Resolved placement direction (may differ from preferred if flipped).
    pub actual_placement: PopoverPlacement,
    /// Final positioned rectangle of the overlay.
    pub rect: Rect,
    /// Whether the overlay flipped from its preferred placement.
    pub is_flipped: bool,
    /// Cross-axis pixel offset of the indicator arrow from the overlay's origin, pointing at anchor center.
    pub arrow_offset: f32,
}

/// Calculate the collision-aware position of an overlay given anchor bounds and viewport constraints.
#[must_use]
pub fn compute_collision_placement(
    anchor: Rect,
    overlay_size: Size,
    viewport: Rect,
    opts: CollisionPlacementOptions,
) -> OverlayPlacementResult {
    let preferred = opts.preferred_placement;
    let (chosen_placement, is_flipped) = if opts.auto_flip {
        resolve_auto_flip(
            anchor,
            overlay_size,
            viewport,
            preferred,
            opts.offset,
            opts.viewport_padding,
        )
    } else {
        (preferred, false)
    };

    let (mut x, mut y) = calculate_raw_coordinates(
        anchor,
        overlay_size,
        chosen_placement,
        opts.alignment,
        opts.offset,
    );

    // Clamp coordinates within viewport padding bounds
    let min_x = viewport.origin.x + opts.viewport_padding;
    let max_x =
        (viewport.origin.x + viewport.size.width - opts.viewport_padding - overlay_size.width)
            .max(min_x);
    let min_y = viewport.origin.y + opts.viewport_padding;
    let max_y =
        (viewport.origin.y + viewport.size.height - opts.viewport_padding - overlay_size.height)
            .max(min_y);

    x = x.clamp(min_x, max_x);
    y = y.clamp(min_y, max_y);

    let arrow_offset = calculate_arrow_offset(anchor, x, y, overlay_size, chosen_placement);

    OverlayPlacementResult {
        actual_placement: chosen_placement,
        rect: Rect {
            origin: Point::new(x, y),
            size: overlay_size,
        },
        is_flipped,
        arrow_offset,
    }
}

fn resolve_auto_flip(
    anchor: Rect,
    overlay_size: Size,
    viewport: Rect,
    preferred: PopoverPlacement,
    offset: f32,
    padding: f32,
) -> (PopoverPlacement, bool) {
    let top_limit = viewport.origin.y + padding;
    let bottom_limit = viewport.origin.y + viewport.size.height - padding;
    let left_limit = viewport.origin.x + padding;
    let right_limit = viewport.origin.x + viewport.size.width - padding;

    let opposite = match preferred {
        PopoverPlacement::Top => PopoverPlacement::Bottom,
        PopoverPlacement::Bottom => PopoverPlacement::Top,
        PopoverPlacement::Left => PopoverPlacement::Right,
        PopoverPlacement::Right => PopoverPlacement::Left,
    };

    let preferred_fits = match preferred {
        PopoverPlacement::Bottom => {
            anchor.origin.y + anchor.size.height + offset + overlay_size.height <= bottom_limit
        }
        PopoverPlacement::Top => anchor.origin.y - offset - overlay_size.height >= top_limit,
        PopoverPlacement::Right => {
            anchor.origin.x + anchor.size.width + offset + overlay_size.width <= right_limit
        }
        PopoverPlacement::Left => anchor.origin.x - offset - overlay_size.width >= left_limit,
    };

    if preferred_fits {
        return (preferred, false);
    }

    let opposite_fits = match opposite {
        PopoverPlacement::Bottom => {
            anchor.origin.y + anchor.size.height + offset + overlay_size.height <= bottom_limit
        }
        PopoverPlacement::Top => anchor.origin.y - offset - overlay_size.height >= top_limit,
        PopoverPlacement::Right => {
            anchor.origin.x + anchor.size.width + offset + overlay_size.width <= right_limit
        }
        PopoverPlacement::Left => anchor.origin.x - offset - overlay_size.width >= left_limit,
    };

    if opposite_fits {
        return (opposite, true);
    }

    // Neither fits fully: choose side with greatest available room
    let clearance_preferred = match preferred {
        PopoverPlacement::Bottom => bottom_limit - (anchor.origin.y + anchor.size.height + offset),
        PopoverPlacement::Top => (anchor.origin.y - offset) - top_limit,
        PopoverPlacement::Right => right_limit - (anchor.origin.x + anchor.size.width + offset),
        PopoverPlacement::Left => (anchor.origin.x - offset) - left_limit,
    };

    let clearance_opposite = match opposite {
        PopoverPlacement::Bottom => bottom_limit - (anchor.origin.y + anchor.size.height + offset),
        PopoverPlacement::Top => (anchor.origin.y - offset) - top_limit,
        PopoverPlacement::Right => right_limit - (anchor.origin.x + anchor.size.width + offset),
        PopoverPlacement::Left => (anchor.origin.x - offset) - left_limit,
    };

    if clearance_opposite > clearance_preferred {
        (opposite, true)
    } else {
        (preferred, false)
    }
}

fn calculate_raw_coordinates(
    anchor: Rect,
    overlay_size: Size,
    placement: PopoverPlacement,
    alignment: PopoverAlignment,
    offset: f32,
) -> (f32, f32) {
    match placement {
        PopoverPlacement::Bottom => {
            let y = anchor.origin.y + anchor.size.height + offset;
            let x = align_cross_axis(
                anchor.origin.x,
                anchor.size.width,
                overlay_size.width,
                alignment,
            );
            (x, y)
        }
        PopoverPlacement::Top => {
            let y = anchor.origin.y - overlay_size.height - offset;
            let x = align_cross_axis(
                anchor.origin.x,
                anchor.size.width,
                overlay_size.width,
                alignment,
            );
            (x, y)
        }
        PopoverPlacement::Right => {
            let x = anchor.origin.x + anchor.size.width + offset;
            let y = align_cross_axis(
                anchor.origin.y,
                anchor.size.height,
                overlay_size.height,
                alignment,
            );
            (x, y)
        }
        PopoverPlacement::Left => {
            let x = anchor.origin.x - overlay_size.width - offset;
            let y = align_cross_axis(
                anchor.origin.y,
                anchor.size.height,
                overlay_size.height,
                alignment,
            );
            (x, y)
        }
    }
}

fn align_cross_axis(
    anchor_start: f32,
    anchor_len: f32,
    overlay_len: f32,
    alignment: PopoverAlignment,
) -> f32 {
    match alignment {
        PopoverAlignment::Start => anchor_start,
        PopoverAlignment::Center => anchor_start + (anchor_len - overlay_len) * 0.5,
        PopoverAlignment::End => anchor_start + anchor_len - overlay_len,
    }
}

fn calculate_arrow_offset(
    anchor: Rect,
    popover_x: f32,
    popover_y: f32,
    overlay_size: Size,
    placement: PopoverPlacement,
) -> f32 {
    match placement {
        PopoverPlacement::Top | PopoverPlacement::Bottom => {
            let anchor_center_x = anchor.origin.x + anchor.size.width * 0.5;
            let raw_arrow = anchor_center_x - popover_x;
            raw_arrow.clamp(12.0, (overlay_size.width - 12.0).max(12.0))
        }
        PopoverPlacement::Left | PopoverPlacement::Right => {
            let anchor_center_y = anchor.origin.y + anchor.size.height * 0.5;
            let raw_arrow = anchor_center_y - popover_y;
            raw_arrow.clamp(12.0, (overlay_size.height - 12.0).max(12.0))
        }
    }
}
