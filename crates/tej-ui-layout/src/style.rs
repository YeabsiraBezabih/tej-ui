use crate::container::ContainerQuery;
use crate::direction::LayoutDirection;
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

/// Flex wrapping mode along the cross axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
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

/// Layout display mode: Flexbox or CSS Grid.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DisplayMode {
    #[default]
    Flex,
    Grid,
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
    pub display: DisplayMode,
    pub direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub layout_direction: LayoutDirection,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,
    pub width: Dimension,
    pub height: Dimension,
    pub padding: Insets,
    pub margin: Insets,
    pub spacing: f32,
    pub flex_grow: f32,
    pub grid_columns: u16,
    pub grid_column_gap: f32,
    pub grid_row_gap: f32,
    pub col_span: u16,
    pub row_span: u16,
    pub container_query: Option<ContainerQuery>,
}
