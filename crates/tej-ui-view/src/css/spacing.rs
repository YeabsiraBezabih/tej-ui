//! Spacing, margins, paddings, logical offsets, and positioned insets.

use super::types::LengthPercentage;

/// Element layout positioning scheme in document/container flow.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PositionScheme {
    /// In normal document flow.
    #[default]
    Static,
    /// Offset relative to its normal in-flow position.
    Relative,
    /// Removed from flow, positioned relative to nearest positioned ancestor.
    Absolute,
    /// Positioned relative to viewport.
    Fixed,
    /// Toggles between relative and fixed based on scroll offset.
    Sticky,
}

/// Margin and padding spacing constraints (physical and logical).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct SpacingProps {
    // Physical Margins
    pub margin_top: LengthPercentage,
    pub margin_bottom: LengthPercentage,
    pub margin_left: LengthPercentage,
    pub margin_right: LengthPercentage,

    // Logical Margins
    pub margin_block_start: LengthPercentage,
    pub margin_block_end: LengthPercentage,
    pub margin_inline_start: LengthPercentage,
    pub margin_inline_end: LengthPercentage,

    // Physical Paddings
    pub padding_top: LengthPercentage,
    pub padding_bottom: LengthPercentage,
    pub padding_left: LengthPercentage,
    pub padding_right: LengthPercentage,

    // Logical Paddings
    pub padding_block_start: LengthPercentage,
    pub padding_block_end: LengthPercentage,
    pub padding_inline_start: LengthPercentage,
    pub padding_inline_end: LengthPercentage,

    // Positioning Offsets & Insets
    pub position: PositionScheme,
    pub top: LengthPercentage,
    pub right: LengthPercentage,
    pub bottom: LengthPercentage,
    pub left: LengthPercentage,
    pub inset_block_start: LengthPercentage,
    pub inset_block_end: LengthPercentage,
    pub inset_inline_start: LengthPercentage,
    pub inset_inline_end: LengthPercentage,
    pub z_index: Option<i32>,
}
