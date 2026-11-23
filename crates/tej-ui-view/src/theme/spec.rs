//! Master theme definition aggregating colors, typography, radii, and spacing.

use crate::theme::colors::ColorTokens;
use crate::theme::elevation::ElevationTokens;
use crate::theme::radii::{RadiiTokens, SpacingTokens};
use crate::theme::typography::TypographyTokens;
use tej_ui_hal::Color;

/// Complete application visual design theme.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub name: String,
    pub is_dark: bool,
    pub colors: ColorTokens,
    pub typography: TypographyTokens,
    pub radii: RadiiTokens,
    pub spacing: SpacingTokens,
    pub elevation: ElevationTokens,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Creates the standard modern dark theme (Zinc).
    #[must_use]
    pub fn dark() -> Self {
        Self {
            name: "Zinc Dark".into(),
            is_dark: true,
            colors: ColorTokens::dark(),
            typography: TypographyTokens::default(),
            radii: RadiiTokens::default(),
            spacing: SpacingTokens::default(),
            elevation: ElevationTokens::default(),
        }
    }

    /// Creates the clean enterprise light theme.
    #[must_use]
    pub fn light() -> Self {
        Self {
            name: "Zinc Light".into(),
            is_dark: false,
            colors: ColorTokens::light(),
            typography: TypographyTokens::default(),
            radii: RadiiTokens::default(),
            spacing: SpacingTokens::default(),
            elevation: ElevationTokens::default(),
        }
    }

    /// Creates the high-contrast accessibility theme.
    #[must_use]
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".into(),
            is_dark: true,
            colors: ColorTokens::high_contrast(),
            typography: TypographyTokens::default(),
            radii: RadiiTokens::default(),
            spacing: SpacingTokens::default(),
            elevation: ElevationTokens::default(),
        }
    }

    /// Creates the official Tej enterprise light theme (brand violet + neutral gray).
    #[must_use]
    pub fn tej_light() -> Self {
        Self {
            name: "Tej Light".into(),
            is_dark: false,
            colors: ColorTokens::tej_light(),
            typography: TypographyTokens::default(),
            radii: RadiiTokens::tej(),
            spacing: SpacingTokens::default(),
            elevation: ElevationTokens::tej(),
        }
    }

    /// Creates the official Tej enterprise dark theme (brand violet + dark surface).
    #[must_use]
    pub fn tej_dark() -> Self {
        Self {
            name: "Tej Dark".into(),
            is_dark: true,
            colors: ColorTokens::tej_dark(),
            typography: TypographyTokens::default(),
            radii: RadiiTokens::tej(),
            spacing: SpacingTokens::default(),
            elevation: ElevationTokens::tej(),
        }
    }

    /// Overrides the primary brand color in this theme.
    #[must_use]
    pub fn with_primary(mut self, color: Color) -> Self {
        self.colors.primary = color;
        self.colors.ring = color;
        self
    }

    /// Sets the human-readable theme name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}
