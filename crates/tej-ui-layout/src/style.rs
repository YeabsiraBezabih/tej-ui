use crate::container::ContainerQuery;
use crate::direction::LayoutDirection;
use crate::geometry::Insets;
use crate::grid_extended::{GridAutoFlow, JustifyItems, JustifySelf, PlaceContent};

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
    /// Fill available space along main axis (like `flex_grow = 1.0`) or inner cross extent.
    Fill,
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

/// Per-item cross-axis alignment override.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AlignSelf {
    #[default]
    Auto,
    FlexStart,
    Center,
    FlexEnd,
    Stretch,
}

/// Overflow behavior along an axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
    Auto,
}

/// Layout display mode: Flexbox, CSS Grid, or hidden (`None`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DisplayMode {
    #[default]
    Flex,
    Grid,
    None,
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
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LayoutStyle {
    pub display: DisplayMode,
    pub direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub layout_direction: LayoutDirection,
    pub align_items: AlignItems,
    pub align_self: AlignSelf,
    pub align_content: Option<AlignItems>,
    pub justify_content: JustifyContent,
    pub justify_items: JustifyItems,
    pub justify_self: JustifySelf,
    pub place_content: Option<PlaceContent>,
    pub order: i32,
    pub z_index: i32,
    pub width: Dimension,
    pub height: Dimension,
    pub min_width: Dimension,
    pub max_width: Dimension,
    pub min_height: Dimension,
    pub max_height: Dimension,
    pub padding: Insets,
    pub border: Insets,
    pub margin: Insets,
    pub spacing: f32,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Dimension,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub grid_columns: u16,
    pub grid_rows: u16,
    pub grid_auto_flow: GridAutoFlow,
    pub grid_column_gap: f32,
    pub grid_row_gap: f32,
    pub col_span: u16,
    pub row_span: u16,
    pub container_query: Option<ContainerQuery>,
}

impl Default for LayoutStyle {
    fn default() -> Self {
        Self {
            display: DisplayMode::Flex,
            direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            layout_direction: LayoutDirection::Ltr,
            align_items: AlignItems::FlexStart,
            align_self: AlignSelf::Auto,
            align_content: None,
            justify_content: JustifyContent::FlexStart,
            justify_items: JustifyItems::Stretch,
            justify_self: JustifySelf::Auto,
            place_content: None,
            order: 0,
            z_index: 0,
            width: Dimension::Auto,
            height: Dimension::Auto,
            min_width: Dimension::Auto,
            max_width: Dimension::Auto,
            min_height: Dimension::Auto,
            max_height: Dimension::Auto,
            padding: Insets::ZERO,
            border: Insets::ZERO,
            margin: Insets::ZERO,
            spacing: 0.0,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Dimension::Auto,
            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,
            grid_columns: 0,
            grid_rows: 0,
            grid_auto_flow: GridAutoFlow::Row,
            grid_column_gap: 0.0,
            grid_row_gap: 0.0,
            col_span: 0,
            row_span: 0,
            container_query: None,
        }
    }
}
