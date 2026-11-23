//! Scroll snapping, overscroll behavior, and scrollbar styling tokens.

/// Strictness and axis configuration for scroll snap containers.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ScrollSnapType {
    /// No scroll snapping.
    #[default]
    None,
    /// Mandatory snapping along horizontal X axis.
    XMandatory,
    /// Mandatory snapping along vertical Y axis.
    YMandatory,
    /// Proximity snapping along horizontal X axis.
    XProximity,
    /// Proximity snapping along vertical Y axis.
    YProximity,
    /// Mandatory snapping along both X and Y axes.
    BothMandatory,
}

/// Alignment of child elements inside a scroll snap container.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ScrollSnapAlign {
    /// Element does not snap.
    #[default]
    None,
    /// Element snaps to the start edge of the scrollport.
    Start,
    /// Element snaps to the center of the scrollport.
    Center,
    /// Element snaps to the end edge of the scrollport.
    End,
}

/// Gesture behavior when swiping across multiple snap points.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ScrollSnapStop {
    /// Scroll passes through snap points smoothly.
    #[default]
    Normal,
    /// Scroll stops at the very next snap point (paging carousels).
    Always,
}

/// Behavior when scrolling reaches container boundaries (prevents scroll chaining).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum OverscrollBehavior {
    /// Default platform scrolling and scroll chaining to parent.
    #[default]
    Auto,
    /// Prevents scroll chaining into ancestor containers.
    Contain,
    /// Prevents scroll chaining and native bounce / rubber-banding effects.
    None,
}

/// Visual width mode for scrollbars.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ScrollbarWidth {
    /// Default OS platform scrollbar width.
    #[default]
    Auto,
    /// Sleek narrow scrollbars.
    Thin,
    /// Hides scrollbars while preserving scrollability.
    None,
}

/// Layout space reservation for scrollbars to prevent layout shift.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ScrollbarGutter {
    /// Space is reserved only when scrollbars are visible.
    #[default]
    Auto,
    /// Gutter space is permanently reserved to prevent horizontal jumping.
    Stable,
}
