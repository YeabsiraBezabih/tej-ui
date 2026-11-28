//! Semantic OS system font categories and font weights.

/// Semantic OS system font categories and font families.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum SystemFont {
    /// Standard body text font (e.g. San Francisco, Segoe UI, Roboto, Ubuntu).
    #[default]
    Body,
    /// Smaller caption or secondary text font.
    Caption,
    /// Subheading / section title font.
    Title,
    /// Large display or headline font.
    Headline,
    /// Monospace code font (e.g. SF Mono, Cascadia Code, Consolas, JetBrains Mono).
    Monospace,
    /// Explicit named font family.
    Named(&'static str),
}

impl SystemFont {
    /// Creates a named font family descriptor.
    #[must_use]
    pub const fn named(name: &'static str) -> Self {
        Self::Named(name)
    }

    /// Creates a default system UI font.
    #[must_use]
    pub const fn system_ui() -> Self {
        Self::Body
    }

    /// Nominal base point size for standard DPI displays.
    #[must_use]
    pub const fn default_point_size(self) -> f32 {
        match self {
            Self::Caption => 11.0,
            Self::Title => 18.0,
            Self::Headline => 24.0,
            Self::Monospace => 13.0,
            Self::Body | Self::Named(_) => 14.0,
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

    /// Fast numeric identifier for cache keys.
    #[must_use]
    pub fn cache_id(self) -> u32 {
        match self {
            Self::Body => 0,
            Self::Caption => 1,
            Self::Title => 2,
            Self::Headline => 3,
            Self::Monospace => 4,
            Self::Named(name) => {
                let mut h = 0x811c_9dc5_u32;
                for b in name.as_bytes() {
                    h ^= u32::from(*b);
                    h = h.wrapping_mul(0x0100_0193);
                }
                h
            }
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
