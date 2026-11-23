//! Windows system theme, high contrast mode, and accent color synchronization.

use tej_ui_hal::Color;

/// Windows system theme mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Win32ThemeMode {
    #[default]
    Light,
    Dark,
    HighContrast,
}

/// Windows DWM accent colorization and system color settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Win32SystemTheme {
    pub mode: Win32ThemeMode,
    pub accent_color: Color,
    pub titlebar_text_color: Color,
}

impl Win32SystemTheme {
    /// Creates a Windows 11 Dark Mode default theme with Windows Blue accent (`#0078D4`).
    pub fn windows_11_dark() -> Self {
        Self {
            mode: Win32ThemeMode::Dark,
            accent_color: Color::rgb(0, 120, 212),
            titlebar_text_color: Color::rgb(255, 255, 255),
        }
    }

    /// Creates a Windows 11 Light Mode default theme.
    pub fn windows_11_light() -> Self {
        Self {
            mode: Win32ThemeMode::Light,
            accent_color: Color::rgb(0, 103, 192),
            titlebar_text_color: Color::rgb(25, 25, 25),
        }
    }

    /// Extracts DWM accent color from Win32 `ColorizationColor` (0xAARRGGBB format).
    pub fn from_colorization_argb(argb: u32, is_dark: bool) -> Self {
        let r = ((argb >> 16) & 0xFF) as u8;
        let g = ((argb >> 8) & 0xFF) as u8;
        let b = (argb & 0xFF) as u8;
        let accent = Color::rgb(r, g, b);

        let (mode, text_color) = if is_dark {
            (Win32ThemeMode::Dark, Color::rgb(255, 255, 255))
        } else {
            (Win32ThemeMode::Light, Color::rgb(25, 25, 25))
        };

        Self {
            mode,
            accent_color: accent,
            titlebar_text_color: text_color,
        }
    }

    /// True if high contrast theme is active.
    pub fn is_high_contrast(&self) -> bool {
        self.mode == Win32ThemeMode::HighContrast
    }
}

impl Default for Win32SystemTheme {
    fn default() -> Self {
        Self::windows_11_dark()
    }
}
