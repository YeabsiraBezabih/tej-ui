//! macOS system appearance (Aqua, Dark Aqua) and Apple accent color palette.

use tej_ui_hal::Color;

/// Standard Apple system appearances (`NSAppearanceName`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacAppearance {
    #[default]
    Aqua,
    DarkAqua,
    AccessibilityHighContrastAqua,
    AccessibilityHighContrastDarkAqua,
}

/// Official macOS system accent colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppleAccentColor {
    Blue,
    Purple,
    Pink,
    Red,
    Orange,
    Yellow,
    Green,
    Graphite,
}

impl AppleAccentColor {
    /// Authentic Apple sRGB hex color values.
    pub fn color(&self) -> Color {
        match self {
            Self::Blue => Color::rgb(0, 122, 255),       // #007AFF
            Self::Purple => Color::rgb(175, 82, 222),    // #AF52DE
            Self::Pink => Color::rgb(255, 45, 85),       // #FF2D55
            Self::Red => Color::rgb(255, 59, 48),        // #FF3B30
            Self::Orange => Color::rgb(255, 149, 0),     // #FF9500
            Self::Yellow => Color::rgb(255, 204, 0),     // #FFCC00
            Self::Green => Color::rgb(52, 199, 89),      // #34C759
            Self::Graphite => Color::rgb(142, 142, 147), // #8E8E93
        }
    }
}

/// macOS system appearance manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacosAppearanceManager {
    pub appearance: MacAppearance,
    pub accent: AppleAccentColor,
}

impl MacosAppearanceManager {
    pub fn new(appearance: MacAppearance, accent: AppleAccentColor) -> Self {
        Self { appearance, accent }
    }

    /// True if Dark Aqua appearance is active.
    pub fn is_dark(&self) -> bool {
        matches!(
            self.appearance,
            MacAppearance::DarkAqua | MacAppearance::AccessibilityHighContrastDarkAqua
        )
    }

    /// True if an accessibility high contrast appearance is active.
    pub fn is_high_contrast(&self) -> bool {
        matches!(
            self.appearance,
            MacAppearance::AccessibilityHighContrastAqua
                | MacAppearance::AccessibilityHighContrastDarkAqua
        )
    }

    /// Background window chrome color.
    pub fn window_background(&self) -> Color {
        if self.is_dark() {
            Color::rgb(30, 30, 30)
        } else {
            Color::rgb(246, 246, 246)
        }
    }
}

impl Default for MacosAppearanceManager {
    fn default() -> Self {
        Self::new(MacAppearance::DarkAqua, AppleAccentColor::Blue)
    }
}
