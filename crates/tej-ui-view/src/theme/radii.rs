//! Corner radius and spacing scale tokens.

/// Standardized corner rounding presets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadiiTokens {
    pub none: f32,
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
    pub full: f32,
}

impl Default for RadiiTokens {
    fn default() -> Self {
        Self {
            none: 0.0,
            xs: 2.0,
            sm: 4.0,
            md: 6.0,
            lg: 8.0,
            xl: 12.0,
            xxl: 16.0,
            full: 9999.0,
        }
    }
}

impl RadiiTokens {
    /// Official Tej geometric radius scale.
    #[must_use]
    pub const fn tej() -> Self {
        Self {
            none: 0.0,
            xs: 4.0,      // Micro tags & badges
            sm: 6.0,      // Small controls & checkboxes
            md: 8.0,      // Standard buttons & input fields
            lg: 10.0,     // Dropdown popovers & tooltips
            xl: 12.0,     // Cards & sheets
            xxl: 16.0,    // Modals & large dialogs
            full: 9999.0, // Avatars & pill tags
        }
    }
}

/// 4px/8px design grid spacing scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpacingTokens {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

impl Default for SpacingTokens {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: 48.0,
        }
    }
}
