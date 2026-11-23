//! Extended grid alignment, auto-flow, and modern anchor positioning tokens.

use crate::style::{AlignItems, JustifyContent};

/// Cross-axis item alignment inside grid cells.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum JustifyItems {
    /// Stretches item to fill grid cell horizontally.
    #[default]
    Stretch,
    /// Aligns item to start of grid cell.
    Start,
    /// Centers item horizontally in grid cell.
    Center,
    /// Aligns item to end of grid cell.
    End,
}

/// Self-alignment override for a single grid child.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum JustifySelf {
    /// Inherits from container's justify-items.
    #[default]
    Auto,
    /// Stretches item to fill grid cell.
    Stretch,
    /// Aligns item to start of cell.
    Start,
    /// Centers item in cell.
    Center,
    /// Aligns item to end of cell.
    End,
}

/// Shorthand alignment for both cross-axis and main-axis track distribution.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PlaceContent {
    /// Alignment along cross axis.
    pub align: AlignItems,
    /// Alignment along main axis.
    pub justify: JustifyContent,
}

impl PlaceContent {
    /// Centered placement along both axes.
    pub const CENTER: Self = Self {
        align: AlignItems::Center,
        justify: JustifyContent::Center,
    };

    /// Creates a place-content descriptor.
    #[inline]
    #[must_use]
    pub const fn new(align: AlignItems, justify: JustifyContent) -> Self {
        Self { align, justify }
    }
}

/// Auto-placement packing direction for implicit grid tracks.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GridAutoFlow {
    /// Fills rows sequentially.
    #[default]
    Row,
    /// Fills columns sequentially.
    Column,
    /// Densely packs items into earlier empty row slots.
    DenseRow,
    /// Densely packs items into earlier empty column slots.
    DenseColumn,
}

/// Modern anchor-positioning areas for tethered floating popovers and tooltips.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AnchorPositionArea {
    /// Positioned above the anchor element.
    Top,
    /// Positioned below the anchor element.
    #[default]
    Bottom,
    /// Positioned to the left of the anchor element.
    Left,
    /// Positioned to the right of the anchor element.
    Right,
    /// Centered over the anchor element.
    Center,
    /// Above anchor, aligned to start edge.
    TopStart,
    /// Above anchor, aligned to end edge.
    TopEnd,
    /// Below anchor, aligned to start edge.
    BottomStart,
    /// Below anchor, aligned to end edge.
    BottomEnd,
}

/// Interactive panel resize constraints.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ResizeMode {
    /// Element is not resizable by the user.
    #[default]
    None,
    /// Resizable along both horizontal and vertical axes.
    Both,
    /// Resizable horizontally.
    Horizontal,
    /// Resizable vertically.
    Vertical,
}
