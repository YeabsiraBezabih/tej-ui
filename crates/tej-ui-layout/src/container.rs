//! Responsive container queries and adaptive breakpoint evaluation.
//!
//! Enables layout styles and widget trees to adapt based on immediate parent container dimensions
//! rather than fixed global window boundaries.

use crate::style::{Dimension, FlexDirection, FlexWrap};

/// Standard responsive container breakpoint classifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Breakpoint {
    /// Compact form factor (e.g. mobile or narrow sidebar: width < 480pt).
    #[default]
    Compact,
    /// Medium form factor (e.g. tablet or split pane: 480pt <= width < 840pt).
    Medium,
    /// Expanded form factor (e.g. desktop full-screen: width >= 840pt).
    Expanded,
}

impl Breakpoint {
    /// Classifies an available container width into a standard breakpoint tier.
    #[inline]
    #[must_use]
    pub const fn from_width(width: f32) -> Self {
        if width < 480.0 {
            Self::Compact
        } else if width < 840.0 {
            Self::Medium
        } else {
            Self::Expanded
        }
    }

    /// Classifies an available container size into a breakpoint tier.
    #[inline]
    #[must_use]
    pub const fn from_size(width: f32, _height: f32) -> Self {
        Self::from_width(width)
    }

    /// Returns `true` if this breakpoint is [`Compact`](Self::Compact).
    #[inline]
    #[must_use]
    pub const fn is_compact(&self) -> bool {
        matches!(self, Self::Compact)
    }

    /// Returns `true` if this breakpoint is [`Medium`](Self::Medium).
    #[inline]
    #[must_use]
    pub const fn is_medium(&self) -> bool {
        matches!(self, Self::Medium)
    }

    /// Returns `true` if this breakpoint is [`Expanded`](Self::Expanded).
    #[inline]
    #[must_use]
    pub const fn is_expanded(&self) -> bool {
        matches!(self, Self::Expanded)
    }
}

/// Largest `f32` strictly less than `480.0`, avoiding any dead zone or overlap with `>= 480.0`.
const COMPACT_MAX_WIDTH: f32 = f32::from_bits(480.0_f32.to_bits() - 1);

/// Largest `f32` strictly less than `840.0`, avoiding any dead zone or overlap with `>= 840.0`.
const MEDIUM_MAX_WIDTH: f32 = f32::from_bits(840.0_f32.to_bits() - 1);

/// A container query specifying dimension constraints and adaptive style overrides.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ContainerQuery {
    /// Minimum container width threshold (inclusive).
    pub min_width: Option<f32>,
    /// Maximum container width threshold (inclusive).
    pub max_width: Option<f32>,
    /// Minimum container height threshold (inclusive).
    pub min_height: Option<f32>,
    /// Maximum container height threshold (inclusive).
    pub max_height: Option<f32>,
    /// Overridden flex direction when container matches constraints.
    pub direction_override: Option<FlexDirection>,
    /// Overridden flex wrap behavior when container matches constraints.
    pub wrap_override: Option<FlexWrap>,
    /// Overridden width dimension when container matches constraints.
    pub width_override: Option<Dimension>,
    /// Overridden height dimension when container matches constraints.
    pub height_override: Option<Dimension>,
    /// Overridden child spacing when container matches constraints.
    pub spacing_override: Option<f32>,
}

impl ContainerQuery {
    /// Creates an empty container query with no constraints.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            direction_override: None,
            wrap_override: None,
            width_override: None,
            height_override: None,
            spacing_override: None,
        }
    }

    /// Creates a query targeting compact container widths (`width < 480pt`).
    #[must_use]
    pub const fn compact() -> Self {
        Self::new().max_width(COMPACT_MAX_WIDTH)
    }

    /// Creates a query targeting medium container widths (`480pt <= width < 840pt`).
    #[must_use]
    pub const fn medium() -> Self {
        Self::new().min_width(480.0).max_width(MEDIUM_MAX_WIDTH)
    }

    /// Creates a query targeting expanded container widths (`width >= 840pt`).
    #[must_use]
    pub const fn expanded() -> Self {
        Self::new().min_width(840.0)
    }

    /// Sets maximum container width constraint (inclusive).
    #[must_use]
    pub const fn max_width(mut self, max: f32) -> Self {
        self.max_width = Some(max);
        self
    }

    /// Sets minimum container width constraint (inclusive).
    #[must_use]
    pub const fn min_width(mut self, min: f32) -> Self {
        self.min_width = Some(min);
        self
    }

    /// Sets maximum container height constraint.
    #[must_use]
    pub const fn max_height(mut self, max: f32) -> Self {
        self.max_height = Some(max);
        self
    }

    /// Sets minimum container height constraint.
    #[must_use]
    pub const fn min_height(mut self, min: f32) -> Self {
        self.min_height = Some(min);
        self
    }

    /// Sets flex direction override when query matches.
    #[must_use]
    pub const fn direction(mut self, direction: FlexDirection) -> Self {
        self.direction_override = Some(direction);
        self
    }

    /// Sets flex wrap override when query matches.
    #[must_use]
    pub const fn flex_wrap(mut self, wrap: FlexWrap) -> Self {
        self.wrap_override = Some(wrap);
        self
    }

    /// Sets width override when query matches.
    #[must_use]
    pub const fn width(mut self, width: Dimension) -> Self {
        self.width_override = Some(width);
        self
    }

    /// Sets height override when query matches.
    #[must_use]
    pub const fn height(mut self, height: Dimension) -> Self {
        self.height_override = Some(height);
        self
    }

    /// Sets spacing override when query matches.
    #[must_use]
    pub const fn spacing(mut self, spacing: f32) -> Self {
        self.spacing_override = Some(spacing);
        self
    }

    /// Evaluates whether the container query conditions are met for given container dimensions.
    #[inline]
    #[must_use]
    pub fn matches(&self, container_width: f32, container_height: f32) -> bool {
        if let Some(min_w) = self.min_width {
            if container_width < min_w {
                return false;
            }
        }
        if let Some(max_w) = self.max_width {
            if container_width > max_w {
                return false;
            }
        }
        if let Some(min_h) = self.min_height {
            if container_height < min_h {
                return false;
            }
        }
        if let Some(max_h) = self.max_height {
            if container_height > max_h {
                return false;
            }
        }
        true
    }
}
