//! Metadata definitions and aggregate catalog for all 231 CSS properties in `tej-ui`.

pub use crate::views::css::props_borders_effects::PROPS_BORDERS_EFFECTS;
pub use crate::views::css::props_box_spacing::PROPS_BOX_SPACING;
pub use crate::views::css::props_flex_grid::PROPS_FLEX_GRID;
pub use crate::views::css::props_modern::PROPS_MODERN;
pub use crate::views::css::props_typography::PROPS_TYPOGRAPHY;

/// Metadata definition for a strongly-typed CSS property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CssPropertyMeta {
    pub name: &'static str,
    pub category: &'static str,
    pub syntax: &'static str,
    pub description: &'static str,
}

/// Total count of strictly typed and documented CSS properties in `tej-ui`.
pub const TOTAL_CSS_PROPERTIES: usize = 231;
