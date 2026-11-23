//! Elevation and box shadow token definitions.

use tej_ui_hal::Color;

/// Visual drop shadow specification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl Shadow {
    pub const NONE: Self = Self {
        offset_x: 0.0,
        offset_y: 0.0,
        blur: 0.0,
        spread: 0.0,
        color: Color::TRANSPARENT,
    };
}

/// Elevation presets for layering surfaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElevationTokens {
    pub flat: Shadow,
    pub raised: Shadow,
    pub overlay: Shadow,
    pub floating: Shadow,
}

impl Default for ElevationTokens {
    fn default() -> Self {
        Self::tej()
    }
}

impl ElevationTokens {
    /// Official Tej multi-layered elevation depth tokens.
    #[must_use]
    pub const fn tej() -> Self {
        Self {
            flat: Shadow::NONE,
            raised: Shadow {
                offset_x: 0.0,
                offset_y: 1.0,
                blur: 2.0,
                spread: 0.0,
                color: Color::rgba(16, 24, 40, 15),
            },
            overlay: Shadow {
                offset_x: 0.0,
                offset_y: 4.0,
                blur: 8.0,
                spread: -2.0,
                color: Color::rgba(16, 24, 40, 25),
            },
            floating: Shadow {
                offset_x: 0.0,
                offset_y: 12.0,
                blur: 16.0,
                spread: -4.0,
                color: Color::rgba(16, 24, 40, 35),
            },
        }
    }
}
