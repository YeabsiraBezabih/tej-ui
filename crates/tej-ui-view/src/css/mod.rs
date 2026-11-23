//! Comprehensive 231 CSS property type system and style sheet container for `tej-ui`.
//!
//! Provides strongly-typed, zero-allocation representations of all 231 foundational,
//! responsive, layout, typography, SVG, animation, and modern CSS properties.

pub mod backgrounds;
pub mod borders_effects;
pub mod box_sizing;
pub mod flex_grid;
pub mod interaction;
pub mod modern;
pub mod motion;
pub mod spacing;
pub mod types;
pub mod typography;

pub use backgrounds::*;
pub use borders_effects::*;
pub use box_sizing::*;
pub use flex_grid::*;
pub use interaction::*;
pub use modern::*;
pub use motion::*;
pub use spacing::*;
pub use types::*;
pub use typography::*;

/// Total number of CSS properties strictly defined and typed in this module.
pub const TOTAL_CSS_PROPERTIES: usize = 231;

/// Complete CSS stylesheet representing all 231 properties for a component.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct CssStyleSheet {
    pub box_sizing: BoxSizingProps,
    pub spacing: SpacingProps,
    pub flex_grid: FlexGridProps,
    pub typography: TypographyProps,
    pub borders_effects: BordersEffectsProps,
    pub backgrounds: BackgroundProps,
    pub motion: MotionProps,
    pub interaction: InteractionProps,
    pub modern: ModernProps,
}

impl CssStyleSheet {
    /// Creates a new default stylesheet with all 231 properties initialized.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Total count of recognized CSS properties.
    #[must_use]
    pub const fn property_count() -> usize {
        TOTAL_CSS_PROPERTIES
    }

    /// Sets element width in points.
    #[must_use]
    pub fn with_width(mut self, points: f32) -> Self {
        self.box_sizing.width = LengthPercentage::Points(points);
        self
    }

    /// Sets element height in points.
    #[must_use]
    pub fn with_height(mut self, points: f32) -> Self {
        self.box_sizing.height = LengthPercentage::Points(points);
        self
    }

    /// Sets padding on all four sides.
    #[must_use]
    pub fn with_padding(mut self, points: f32) -> Self {
        self.spacing.padding_top = LengthPercentage::Points(points);
        self.spacing.padding_bottom = LengthPercentage::Points(points);
        self.spacing.padding_left = LengthPercentage::Points(points);
        self.spacing.padding_right = LengthPercentage::Points(points);
        self
    }

    /// Sets font size.
    #[must_use]
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.typography.font_size = size;
        self
    }
}
