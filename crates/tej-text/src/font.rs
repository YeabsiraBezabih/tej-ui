/// Semantic OS system font categories.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum SystemFont {
    #[default]
    Body,
    Caption,
    Title,
    Headline,
    Monospace,
}

impl SystemFont {
    /// Nominal base point size for standard DPI displays.
    #[must_use]
    pub const fn default_point_size(self) -> f32 {
        match self {
            Self::Caption => 11.0,
            Self::Body => 14.0,
            Self::Title => 18.0,
            Self::Headline => 24.0,
            Self::Monospace => 13.0,
        }
    }

    /// Approximate average character advance width ratio for fast estimating.
    #[must_use]
    pub const fn avg_char_width_ratio(self) -> f32 {
        match self {
            Self::Monospace => 0.60,
            _ => 0.52,
        }
    }
}

/// Font weight options.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum FontWeight {
    Thin,
    Light,
    #[default]
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
}
