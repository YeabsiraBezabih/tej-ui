use crate::geometry::Insets;

/// Dimension constraint for width or height.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Dimension {
    /// Exact length in display points/pixels.
    Points(f32),
    /// Percentage relative to parent dimension (0.0 to 1.0).
    Percent(f32),
    /// Automatic sizing based on intrinsic content.
    #[default]
    Auto,
}

/// Direction of flex layout axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
}

/// Alignment along cross axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AlignItems {
    #[default]
    FlexStart,
    Center,
    FlexEnd,
    Stretch,
}

/// Alignment along main axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum JustifyContent {
    #[default]
    FlexStart,
    Center,
    FlexEnd,
    SpaceBetween,
    SpaceAround,
}

/// Layout style configuration stored compactly.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct LayoutStyle {
    pub direction: FlexDirection,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,
    pub width: Dimension,
    pub height: Dimension,
    pub padding: Insets,
    pub margin: Insets,
    pub spacing: f32,
    pub flex_grow: f32,
}
