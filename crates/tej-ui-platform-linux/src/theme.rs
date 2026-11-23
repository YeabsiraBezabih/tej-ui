//! Linux desktop theme and XDG Desktop Portal color-scheme synchronization.

use tej_ui_hal::Color;

/// XDG Desktop Portal color-scheme values (`org.freedesktop.appearance.color-scheme`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum XdgColorScheme {
    #[default]
    NoPreference = 0,
    PreferDark = 1,
    PreferLight = 2,
}

impl XdgColorScheme {
    pub fn from_u32(val: u32) -> Self {
        match val {
            1 => Self::PreferDark,
            2 => Self::PreferLight,
            _ => Self::NoPreference,
        }
    }
}

/// Linux desktop environment theme and accent configuration (GNOME / KDE / XFCE).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxDesktopTheme {
    pub color_scheme: XdgColorScheme,
    pub accent_color: Color,
    pub high_contrast: bool,
}

impl LinuxDesktopTheme {
    /// GNOME 45+ Adwaita Dark default with GNOME Blue accent (`#3584E4`).
    pub fn adwaita_dark() -> Self {
        Self {
            color_scheme: XdgColorScheme::PreferDark,
            accent_color: Color::rgb(53, 132, 228), // GNOME Blue
            high_contrast: false,
        }
    }

    /// GNOME 45+ Adwaita Light default.
    pub fn adwaita_light() -> Self {
        Self {
            color_scheme: XdgColorScheme::PreferLight,
            accent_color: Color::rgb(53, 132, 228),
            high_contrast: false,
        }
    }

    /// True if dark mode is preferred.
    pub fn is_dark(&self) -> bool {
        self.color_scheme == XdgColorScheme::PreferDark
    }
}

impl Default for LinuxDesktopTheme {
    fn default() -> Self {
        Self::adwaita_dark()
    }
}
