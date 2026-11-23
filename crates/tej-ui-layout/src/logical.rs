//! Logical dimensions, logical insets, and writing-mode aware layout coordinates.
//!
//! Provides internationalized layout properties (inline/block sizing, logical padding/margin,
//! and directional corner radii) with zero-cost conversion to physical coordinate spaces.

use crate::direction::LayoutDirection;
use crate::geometry::Insets;
use crate::style::Dimension;

/// Logical insets around block flow and inline flow axes.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct LogicalInsets {
    /// Inset at start of block flow (top in horizontal-tb).
    pub block_start: f32,
    /// Inset at end of inline flow (right in LTR, left in RTL).
    pub inline_end: f32,
    /// Inset at end of block flow (bottom in horizontal-tb).
    pub block_end: f32,
    /// Inset at start of inline flow (left in LTR, right in RTL).
    pub inline_start: f32,
}

impl LogicalInsets {
    /// Zero insets.
    pub const ZERO: Self = Self {
        block_start: 0.0,
        inline_end: 0.0,
        block_end: 0.0,
        inline_start: 0.0,
    };

    /// Uniform insets on all four logical edges.
    #[inline]
    #[must_use]
    pub const fn all(value: f32) -> Self {
        Self {
            block_start: value,
            inline_end: value,
            block_end: value,
            inline_start: value,
        }
    }

    /// Symmetric insets along block and inline axes.
    #[inline]
    #[must_use]
    pub const fn symmetric(block: f32, inline: f32) -> Self {
        Self {
            block_start: block,
            inline_end: inline,
            block_end: block,
            inline_start: inline,
        }
    }

    /// Insets along block flow axis only.
    #[inline]
    #[must_use]
    pub const fn block(value: f32) -> Self {
        Self {
            block_start: value,
            inline_end: 0.0,
            block_end: value,
            inline_start: 0.0,
        }
    }

    /// Insets along inline flow axis only.
    #[inline]
    #[must_use]
    pub const fn inline(value: f32) -> Self {
        Self {
            block_start: 0.0,
            inline_end: value,
            block_end: 0.0,
            inline_start: value,
        }
    }

    /// Converts logical insets to physical insets based on reading direction.
    #[must_use]
    pub fn to_physical(&self, direction: LayoutDirection) -> Insets {
        match direction {
            LayoutDirection::Ltr => Insets {
                top: self.block_start,
                right: self.inline_end,
                bottom: self.block_end,
                left: self.inline_start,
            },
            LayoutDirection::Rtl => Insets {
                top: self.block_start,
                right: self.inline_start,
                bottom: self.block_end,
                left: self.inline_end,
            },
        }
    }

    /// Constructs logical insets from physical insets and reading direction.
    #[must_use]
    pub fn from_physical(insets: Insets, direction: LayoutDirection) -> Self {
        match direction {
            LayoutDirection::Ltr => Self {
                block_start: insets.top,
                inline_end: insets.right,
                block_end: insets.bottom,
                inline_start: insets.left,
            },
            LayoutDirection::Rtl => Self {
                block_start: insets.top,
                inline_end: insets.left,
                block_end: insets.bottom,
                inline_start: insets.right,
            },
        }
    }
}

/// Logical dimension sizing constraint (inline-size and block-size).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct LogicalSize {
    /// Dimension along inline flow axis (width in horizontal writing mode).
    pub inline_size: Dimension,
    /// Dimension along block flow axis (height in horizontal writing mode).
    pub block_size: Dimension,
}

impl LogicalSize {
    /// Creates a new logical size descriptor.
    #[inline]
    #[must_use]
    pub const fn new(inline_size: Dimension, block_size: Dimension) -> Self {
        Self {
            inline_size,
            block_size,
        }
    }

    /// Resolves logical dimensions to physical (width, height) constraints.
    #[must_use]
    pub const fn to_physical(&self, is_vertical_writing_mode: bool) -> (Dimension, Dimension) {
        if is_vertical_writing_mode {
            (self.block_size, self.inline_size)
        } else {
            (self.inline_size, self.block_size)
        }
    }
}

/// Directional corner radii for logical internationalized surfaces (e.g. chat bubbles).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct LogicalCornerRadii {
    /// Radius at start of block flow and start of inline flow (top-left in LTR).
    pub start_start: f32,
    /// Radius at start of block flow and end of inline flow (top-right in LTR).
    pub start_end: f32,
    /// Radius at end of block flow and end of inline flow (bottom-right in LTR).
    pub end_end: f32,
    /// Radius at end of block flow and start of inline flow (bottom-left in LTR).
    pub end_start: f32,
}

impl LogicalCornerRadii {
    /// Uniform radii for all four logical corners.
    #[inline]
    #[must_use]
    pub const fn uniform(radius: f32) -> Self {
        Self {
            start_start: radius,
            start_end: radius,
            end_end: radius,
            end_start: radius,
        }
    }

    /// Resolves logical radii to physical (top-left, top-right, bottom-right, bottom-left) tuple.
    #[must_use]
    pub fn to_physical(&self, direction: LayoutDirection) -> (f32, f32, f32, f32) {
        match direction {
            LayoutDirection::Ltr => (
                self.start_start,
                self.start_end,
                self.end_end,
                self.end_start,
            ),
            LayoutDirection::Rtl => (
                self.start_end,
                self.start_start,
                self.end_start,
                self.end_end,
            ),
        }
    }
}
