//! Semantic color palette tokens for application themes.

use tej_ui_hal::Color;

/// Complete semantic color palette based on modern design systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorTokens {
    pub background: Color,
    pub foreground: Color,
    pub card: Color,
    pub card_foreground: Color,
    pub popover: Color,
    pub popover_foreground: Color,
    pub primary: Color,
    pub primary_foreground: Color,
    pub secondary: Color,
    pub secondary_foreground: Color,
    pub muted: Color,
    pub muted_foreground: Color,
    pub accent: Color,
    pub accent_foreground: Color,
    pub destructive: Color,
    pub destructive_foreground: Color,
    pub border: Color,
    pub input: Color,
    pub ring: Color,
}

impl ColorTokens {
    /// Zinc-based dark mode palette.
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            background: Color::from_rgb_u32(0x09090B),
            foreground: Color::from_rgb_u32(0xFAFAFA),
            card: Color::from_rgb_u32(0x121215),
            card_foreground: Color::from_rgb_u32(0xFAFAFA),
            popover: Color::from_rgb_u32(0x18181B),
            popover_foreground: Color::from_rgb_u32(0xFAFAFA),
            primary: Color::from_rgb_u32(0x3B82F6), // Indigo / Sky blue accent
            primary_foreground: Color::from_rgb_u32(0xFFFFFF),
            secondary: Color::from_rgb_u32(0x27272A),
            secondary_foreground: Color::from_rgb_u32(0xFAFAFA),
            muted: Color::from_rgb_u32(0x27272A),
            muted_foreground: Color::from_rgb_u32(0xA1A1AA),
            accent: Color::from_rgb_u32(0x27272A),
            accent_foreground: Color::from_rgb_u32(0xFAFAFA),
            destructive: Color::from_rgb_u32(0xEF4444),
            destructive_foreground: Color::from_rgb_u32(0xFFFFFF),
            border: Color::from_rgb_u32(0x27272A),
            input: Color::from_rgb_u32(0x27272A),
            ring: Color::from_rgb_u32(0x3B82F6),
        }
    }

    /// Clean enterprise light mode palette.
    #[must_use]
    pub const fn light() -> Self {
        Self {
            background: Color::from_rgb_u32(0xFFFFFF),
            foreground: Color::from_rgb_u32(0x09090B),
            card: Color::from_rgb_u32(0xFFFFFF),
            card_foreground: Color::from_rgb_u32(0x09090B),
            popover: Color::from_rgb_u32(0xFFFFFF),
            popover_foreground: Color::from_rgb_u32(0x09090B),
            primary: Color::from_rgb_u32(0x2563EB),
            primary_foreground: Color::from_rgb_u32(0xFFFFFF),
            secondary: Color::from_rgb_u32(0xF4F4F5),
            secondary_foreground: Color::from_rgb_u32(0x18181B),
            muted: Color::from_rgb_u32(0xF4F4F5),
            muted_foreground: Color::from_rgb_u32(0x71717A),
            accent: Color::from_rgb_u32(0xF4F4F5),
            accent_foreground: Color::from_rgb_u32(0x18181B),
            destructive: Color::from_rgb_u32(0xDC2626),
            destructive_foreground: Color::from_rgb_u32(0xFFFFFF),
            border: Color::from_rgb_u32(0xE4E4E7),
            input: Color::from_rgb_u32(0xE4E4E7),
            ring: Color::from_rgb_u32(0x2563EB),
        }
    }

    /// High contrast accessibility palette.
    #[must_use]
    pub const fn high_contrast() -> Self {
        Self {
            background: Color::BLACK,
            foreground: Color::WHITE,
            card: Color::from_rgb_u32(0x101010),
            card_foreground: Color::WHITE,
            popover: Color::from_rgb_u32(0x151515),
            popover_foreground: Color::WHITE,
            primary: Color::from_rgb_u32(0x60A5FA),
            primary_foreground: Color::BLACK,
            secondary: Color::from_rgb_u32(0x303030),
            secondary_foreground: Color::WHITE,
            muted: Color::from_rgb_u32(0x202020),
            muted_foreground: Color::from_rgb_u32(0xE0E0E0),
            accent: Color::from_rgb_u32(0x404040),
            accent_foreground: Color::WHITE,
            destructive: Color::from_rgb_u32(0xFF3333),
            destructive_foreground: Color::WHITE,
            border: Color::WHITE,
            input: Color::WHITE,
            ring: Color::from_rgb_u32(0x93C5FD),
        }
    }

    /// Official Tej enterprise light theme palette (brand violet + neutral gray).
    #[must_use]
    pub const fn tej_light() -> Self {
        Self {
            background: Color::from_rgb_u32(0xFFFFFF),
            foreground: Color::from_rgb_u32(0x101828), // Gray 900
            card: Color::from_rgb_u32(0xFFFFFF),
            card_foreground: Color::from_rgb_u32(0x101828),
            popover: Color::from_rgb_u32(0xFFFFFF),
            popover_foreground: Color::from_rgb_u32(0x101828),
            primary: Color::from_rgb_u32(0x7F56D9), // Brand 600
            primary_foreground: Color::from_rgb_u32(0xFFFFFF),
            secondary: Color::from_rgb_u32(0xF9FAFB), // Gray 50
            secondary_foreground: Color::from_rgb_u32(0x344054), // Gray 700
            muted: Color::from_rgb_u32(0xF2F4F7),     // Gray 100
            muted_foreground: Color::from_rgb_u32(0x667085), // Gray 500
            accent: Color::from_rgb_u32(0xF9F5FF),    // Brand 50
            accent_foreground: Color::from_rgb_u32(0x6941C6), // Brand 700
            destructive: Color::from_rgb_u32(0xD92D20), // Error 600
            destructive_foreground: Color::from_rgb_u32(0xFFFFFF),
            border: Color::from_rgb_u32(0xEAECF0), // Gray 200
            input: Color::from_rgb_u32(0xD0D5DD),  // Gray 300
            ring: Color::from_rgb_u32(0x7F56D9),   // Brand 600
        }
    }

    /// Official Tej enterprise dark theme palette (dark surface + brand violet accents).
    #[must_use]
    pub const fn tej_dark() -> Self {
        Self {
            background: Color::from_rgb_u32(0x0C111D), // Gray 950
            foreground: Color::from_rgb_u32(0xF9FAFB), // Gray 50
            card: Color::from_rgb_u32(0x101828),       // Gray 900
            card_foreground: Color::from_rgb_u32(0xF9FAFB),
            popover: Color::from_rgb_u32(0x1D2939), // Gray 800
            popover_foreground: Color::from_rgb_u32(0xF9FAFB),
            primary: Color::from_rgb_u32(0x7F56D9), // Brand 600
            primary_foreground: Color::from_rgb_u32(0xFFFFFF),
            secondary: Color::from_rgb_u32(0x1D2939), // Gray 800
            secondary_foreground: Color::from_rgb_u32(0xEAECF0), // Gray 200
            muted: Color::from_rgb_u32(0x1D2939),     // Gray 800
            muted_foreground: Color::from_rgb_u32(0x98A2B3), // Gray 400
            accent: Color::from_rgb_u32(0x271F3D),    // Deep brand tint
            accent_foreground: Color::from_rgb_u32(0xD6BBFB), // Brand 300
            destructive: Color::from_rgb_u32(0xF04438), // Error 500
            destructive_foreground: Color::from_rgb_u32(0xFFFFFF),
            border: Color::from_rgb_u32(0x344054), // Gray 700
            input: Color::from_rgb_u32(0x344054),  // Gray 700
            ring: Color::from_rgb_u32(0x9E77ED),   // Brand 500
        }
    }
}

impl Default for ColorTokens {
    fn default() -> Self {
        Self::dark()
    }
}
