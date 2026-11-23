//! Unified Design Token and Theming Engine for `tej-ui`.
//!
//! Provides standardized semantic colors, typography scale, spacing grids,
//! corner radii, elevation shadows, and dynamic dark/light theme switching.

pub mod colors;
pub mod elevation;
pub mod radii;
pub mod spec;
pub mod typography;

pub use colors::ColorTokens;
pub use elevation::{ElevationTokens, Shadow};
pub use radii::{RadiiTokens, SpacingTokens};
pub use spec::Theme;
pub use typography::{TextStyle, TypographyTokens};

use tej_ui_core::{provide_context, use_context};

/// Retrieves the active ambient design theme, defaulting to [`Theme::dark()`].
#[must_use]
pub fn use_theme() -> Theme {
    use_context::<Theme>().unwrap_or_else(Theme::dark)
}

/// Provides an ambient theme for the current thread or scope.
pub fn provide_theme(theme: Theme) {
    provide_context(theme);
}
