//! Windows system backdrop effects (Mica, Acrylic) and DWM integration.

/// Translucent system backdrop material effects for Windows 11 / Windows 10.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowBackdrop {
    /// Standard opaque window surface.
    #[default]
    None,
    /// Windows 11 Mica material (subtle wallpaper tinting for titlebar and app canvas).
    Mica,
    /// Windows 11 Mica Alt material (tabbed applications / multi-document apps).
    MicaAlt,
    /// Acrylic blur-behind material.
    Acrylic,
}

/// Desktop Window Manager (DWM) window attributes configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DwmAttributes {
    /// Window backdrop effect.
    pub backdrop: WindowBackdrop,
    /// Whether system dark mode theme is forced for the titlebar and chrome.
    pub use_dark_mode: bool,
    /// Custom border color in 0x00RRGGBB format (None uses OS default).
    pub border_color: Option<u32>,
    /// Custom titlebar color in 0x00RRGGBB format.
    pub titlebar_color: Option<u32>,
}

impl DwmAttributes {
    /// Creates a default DWM attribute configuration with Mica and dark mode.
    #[must_use]
    pub const fn mica_dark() -> Self {
        Self {
            backdrop: WindowBackdrop::Mica,
            use_dark_mode: true,
            border_color: None,
            titlebar_color: None,
        }
    }
}
