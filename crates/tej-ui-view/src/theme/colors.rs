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
    pub primary_hover: Color,
    pub secondary: Color,
    pub secondary_foreground: Color,
    pub muted: Color,
    pub muted_foreground: Color,
    pub accent: Color,
    pub accent_foreground: Color,
    pub destructive: Color,
    pub destructive_foreground: Color,
    pub destructive_hover: Color,
    pub border: Color,
    pub input: Color,
    pub input_bg: Color,
    pub input_border: Color,
    pub ring: Color,
    pub success: Color,
    pub success_foreground: Color,
    pub warning: Color,
    pub warning_foreground: Color,
    pub info: Color,
    pub info_foreground: Color,
    pub scrim: Color,
}

impl ColorTokens {
    /// Zinc-based dark mode palette.
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            background: Color::from_rgb_u32(0x0009_090B),
            foreground: Color::from_rgb_u32(0x00FA_FAFA),
            card: Color::from_rgb_u32(0x0012_1215),
            card_foreground: Color::from_rgb_u32(0x00FA_FAFA),
            popover: Color::from_rgb_u32(0x0018_181B),
            popover_foreground: Color::from_rgb_u32(0x00FA_FAFA),
            primary: Color::from_rgb_u32(0x003B_82F6), // Indigo / Sky blue accent
            primary_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            primary_hover: Color::from_rgb_u32(0x0025_63EB),
            secondary: Color::from_rgb_u32(0x0027_272A),
            secondary_foreground: Color::from_rgb_u32(0x00FA_FAFA),
            muted: Color::from_rgb_u32(0x0027_272A),
            muted_foreground: Color::from_rgb_u32(0x00A1_A1AA),
            accent: Color::from_rgb_u32(0x0027_272A),
            accent_foreground: Color::from_rgb_u32(0x00FA_FAFA),
            destructive: Color::from_rgb_u32(0x00EF_4444),
            destructive_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            destructive_hover: Color::from_rgb_u32(0x00DC_2626),
            border: Color::from_rgb_u32(0x0027_272A),
            input: Color::from_rgb_u32(0x0027_272A),
            input_bg: Color::from_rgb_u32(0x0018_181B),
            input_border: Color::from_rgb_u32(0x003F_3F46),
            ring: Color::from_rgb_u32(0x003B_82F6),
            success: Color::from_rgb_u32(0x0022_C55E),
            success_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            warning: Color::from_rgb_u32(0x00F5_9E0B),
            warning_foreground: Color::from_rgb_u32(0x0009_090B),
            info: Color::from_rgb_u32(0x0038_BDF8),
            info_foreground: Color::from_rgb_u32(0x0009_090B),
            scrim: Color::rgba(0, 0, 0, 140),
        }
    }

    /// Clean enterprise light mode palette.
    #[must_use]
    pub const fn light() -> Self {
        Self {
            background: Color::from_rgb_u32(0x00FF_FFFF),
            foreground: Color::from_rgb_u32(0x0009_090B),
            card: Color::from_rgb_u32(0x00FF_FFFF),
            card_foreground: Color::from_rgb_u32(0x0009_090B),
            popover: Color::from_rgb_u32(0x00FF_FFFF),
            popover_foreground: Color::from_rgb_u32(0x0009_090B),
            primary: Color::from_rgb_u32(0x0025_63EB),
            primary_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            primary_hover: Color::from_rgb_u32(0x001D_4ED8),
            secondary: Color::from_rgb_u32(0x00F4_F4F5),
            secondary_foreground: Color::from_rgb_u32(0x0018_181B),
            muted: Color::from_rgb_u32(0x00F4_F4F5),
            muted_foreground: Color::from_rgb_u32(0x0071_717A),
            accent: Color::from_rgb_u32(0x00F4_F4F5),
            accent_foreground: Color::from_rgb_u32(0x0018_181B),
            destructive: Color::from_rgb_u32(0x00DC_2626),
            destructive_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            destructive_hover: Color::from_rgb_u32(0x00B9_1C1C),
            border: Color::from_rgb_u32(0x00E4_E4E7),
            input: Color::from_rgb_u32(0x00E4_E4E7),
            input_bg: Color::from_rgb_u32(0x00FF_FFFF),
            input_border: Color::from_rgb_u32(0x00D4_D4D8),
            ring: Color::from_rgb_u32(0x0025_63EB),
            success: Color::from_rgb_u32(0x0016_A34A),
            success_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            warning: Color::from_rgb_u32(0x00D9_7706),
            warning_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            info: Color::from_rgb_u32(0x0002_84C7),
            info_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            scrim: Color::rgba(0, 0, 0, 140),
        }
    }

    /// High contrast accessibility palette.
    #[must_use]
    pub const fn high_contrast() -> Self {
        Self {
            background: Color::BLACK,
            foreground: Color::WHITE,
            card: Color::from_rgb_u32(0x0010_1010),
            card_foreground: Color::WHITE,
            popover: Color::from_rgb_u32(0x0015_1515),
            popover_foreground: Color::WHITE,
            primary: Color::from_rgb_u32(0x0060_A5FA),
            primary_foreground: Color::BLACK,
            primary_hover: Color::from_rgb_u32(0x0093_C5FD),
            secondary: Color::from_rgb_u32(0x0030_3030),
            secondary_foreground: Color::WHITE,
            muted: Color::from_rgb_u32(0x0020_2020),
            muted_foreground: Color::from_rgb_u32(0x00E0_E0E0),
            accent: Color::from_rgb_u32(0x0040_4040),
            accent_foreground: Color::WHITE,
            destructive: Color::from_rgb_u32(0x00FF_3333),
            destructive_foreground: Color::WHITE,
            destructive_hover: Color::from_rgb_u32(0x00FF_6666),
            border: Color::WHITE,
            input: Color::WHITE,
            input_bg: Color::BLACK,
            input_border: Color::WHITE,
            ring: Color::from_rgb_u32(0x0093_C5FD),
            success: Color::from_rgb_u32(0x004A_DE80),
            success_foreground: Color::BLACK,
            warning: Color::from_rgb_u32(0x00FA_CC15),
            warning_foreground: Color::BLACK,
            info: Color::from_rgb_u32(0x0038_BDF8),
            info_foreground: Color::BLACK,
            scrim: Color::rgba(0, 0, 0, 140),
        }
    }

    /// Official Tej enterprise light theme palette (brand violet + neutral gray).
    #[must_use]
    pub const fn tej_light() -> Self {
        Self {
            background: Color::from_rgb_u32(0x00FF_FFFF),
            foreground: Color::from_rgb_u32(0x0010_1828), // Gray 900
            card: Color::from_rgb_u32(0x00FF_FFFF),
            card_foreground: Color::from_rgb_u32(0x0010_1828),
            popover: Color::from_rgb_u32(0x00FF_FFFF),
            popover_foreground: Color::from_rgb_u32(0x0010_1828),
            primary: Color::from_rgb_u32(0x007F_56D9), // Brand 600
            primary_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            primary_hover: Color::from_rgb_u32(0x0069_41C6), // Brand 700
            secondary: Color::from_rgb_u32(0x00F9_FAFB),     // Gray 50
            secondary_foreground: Color::from_rgb_u32(0x0034_4054), // Gray 700
            muted: Color::from_rgb_u32(0x00F2_F4F7),         // Gray 100
            muted_foreground: Color::from_rgb_u32(0x0066_7085), // Gray 500
            accent: Color::from_rgb_u32(0x00F9_F5FF),        // Brand 50
            accent_foreground: Color::from_rgb_u32(0x0069_41C6), // Brand 700
            destructive: Color::from_rgb_u32(0x00D9_2D20),   // Error 600
            destructive_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            destructive_hover: Color::from_rgb_u32(0x00B4_2318), // Error 700
            border: Color::from_rgb_u32(0x00EA_ECF0),            // Gray 200
            input: Color::from_rgb_u32(0x00D0_D5DD),             // Gray 300
            input_bg: Color::from_rgb_u32(0x00FF_FFFF),
            input_border: Color::from_rgb_u32(0x00D0_D5DD), // Gray 300
            ring: Color::from_rgb_u32(0x007F_56D9),         // Brand 600
            success: Color::from_rgb_u32(0x0003_9855),      // Success 600
            success_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            warning: Color::from_rgb_u32(0x00DC_6803), // Warning 600
            warning_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            info: Color::from_rgb_u32(0x0015_70EF), // Blue 600
            info_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            scrim: Color::rgba(0, 0, 0, 140),
        }
    }

    /// Official Tej enterprise dark theme palette (dark surface + brand violet accents).
    #[must_use]
    pub const fn tej_dark() -> Self {
        Self {
            background: Color::from_rgb_u32(0x000C_111D), // Gray 950
            foreground: Color::from_rgb_u32(0x00F9_FAFB), // Gray 50
            card: Color::from_rgb_u32(0x0010_1828),       // Gray 900
            card_foreground: Color::from_rgb_u32(0x00F9_FAFB),
            popover: Color::from_rgb_u32(0x001D_2939), // Gray 800
            popover_foreground: Color::from_rgb_u32(0x00F9_FAFB),
            primary: Color::from_rgb_u32(0x007F_56D9), // Brand 600
            primary_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            primary_hover: Color::from_rgb_u32(0x0069_41C6), // Brand 700
            secondary: Color::from_rgb_u32(0x001D_2939),     // Gray 800
            secondary_foreground: Color::from_rgb_u32(0x00EA_ECF0), // Gray 200
            muted: Color::from_rgb_u32(0x001D_2939),         // Gray 800
            muted_foreground: Color::from_rgb_u32(0x0098_A2B3), // Gray 400
            accent: Color::from_rgb_u32(0x0027_1F3D),        // Deep brand tint
            accent_foreground: Color::from_rgb_u32(0x00D6_BBFB), // Brand 300
            destructive: Color::from_rgb_u32(0x00F0_4438),   // Error 500
            destructive_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            destructive_hover: Color::from_rgb_u32(0x00D9_2D20), // Error 600
            border: Color::from_rgb_u32(0x0034_4054),            // Gray 700
            input: Color::from_rgb_u32(0x0034_4054),             // Gray 700
            input_bg: Color::from_rgb_u32(0x0010_1828),          // Gray 900
            input_border: Color::from_rgb_u32(0x0034_4054),      // Gray 700
            ring: Color::from_rgb_u32(0x009E_77ED),              // Brand 500
            success: Color::from_rgb_u32(0x0012_B76A),           // Success 500
            success_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            warning: Color::from_rgb_u32(0x00F7_9009), // Warning 500
            warning_foreground: Color::from_rgb_u32(0x000C_111D),
            info: Color::from_rgb_u32(0x002E_90FA), // Blue 500
            info_foreground: Color::from_rgb_u32(0x00FF_FFFF),
            scrim: Color::rgba(0, 0, 0, 140),
        }
    }
}

impl Default for ColorTokens {
    fn default() -> Self {
        Self::dark()
    }
}
