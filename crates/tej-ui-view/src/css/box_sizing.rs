//! Box model sizing, aspect ratio, and logical dimensions.

use super::types::LengthPercentage;

/// Box sizing model mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BoxSizing {
    /// Border and padding included inside declared dimensions.
    #[default]
    BorderBox,
    /// Dimensions exclude padding and border.
    ContentBox,
}

/// Proportional aspect ratio constraint (width / height).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AspectRatio {
    pub width: f32,
    pub height: f32,
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
        }
    }
}

impl AspectRatio {
    /// 16:9 widescreen ratio.
    pub const RATIO_16_9: Self = Self {
        width: 16.0,
        height: 9.0,
    };
    /// 4:3 standard ratio.
    pub const RATIO_4_3: Self = Self {
        width: 4.0,
        height: 3.0,
    };
    /// 1:1 square ratio.
    pub const SQUARE: Self = Self {
        width: 1.0,
        height: 1.0,
    };

    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// Set of box model sizing and dimension properties.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct BoxSizingProps {
    pub box_sizing: BoxSizing,
    pub width: LengthPercentage,
    pub height: LengthPercentage,
    pub min_width: LengthPercentage,
    pub max_width: LengthPercentage,
    pub min_height: LengthPercentage,
    pub max_height: LengthPercentage,
    pub aspect_ratio: Option<AspectRatio>,
    pub inline_size: LengthPercentage,
    pub block_size: LengthPercentage,
    pub min_inline_size: LengthPercentage,
    pub max_inline_size: LengthPercentage,
    pub min_block_size: LengthPercentage,
    pub max_block_size: LengthPercentage,
}
