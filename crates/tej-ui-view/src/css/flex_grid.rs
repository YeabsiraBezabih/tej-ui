//! Flexbox and CSS Grid layout properties.

use super::types::LengthPercentage;
use smallvec::SmallVec;
use tej_ui_layout::{
    AlignItems, FlexDirection, FlexWrap, GridAutoFlow, JustifyContent, JustifyItems, JustifySelf,
    PlaceContent,
};

/// Inner/outer element display layout type.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Display {
    #[default]
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
    Block,
    InlineBlock,
    Inline,
    None,
    Contents,
}

/// Grid line or span descriptor.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct GridPlacement {
    pub start: i16,
    pub span: u16,
}

/// Sizing track descriptor for CSS Grid tracks (columns/rows).
#[derive(Clone, PartialEq, Debug)]
pub enum GridTrack {
    Fr(f32),
    Points(f32),
    Percent(f32),
    Auto,
    MinMax(LengthPercentage, LengthPercentage),
    Repeat(u16, Box<GridTrack>),
}

/// Properties governing Flexbox and CSS Grid layout containers and children.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct FlexGridProps {
    pub display: Display,
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: Option<AlignItems>,
    pub justify_items: JustifyItems,
    pub justify_self: JustifySelf,
    pub place_content: Option<PlaceContent>,
    pub flex_wrap: FlexWrap,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: LengthPercentage,
    pub align_self: Option<AlignItems>,
    pub order: i32,
    pub row_gap: f32,
    pub column_gap: f32,
    pub grid_template_columns: SmallVec<[GridTrack; 4]>,
    pub grid_template_rows: SmallVec<[GridTrack; 4]>,
    pub grid_column: GridPlacement,
    pub grid_row: GridPlacement,
    pub grid_auto_flow: GridAutoFlow,
    pub grid_auto_columns: Option<LengthPercentage>,
    pub grid_auto_rows: Option<LengthPercentage>,
}
