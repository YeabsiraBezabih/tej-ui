//! Bidirectional layout direction (LTR and RTL) for Arabic, Hebrew, and Persian scripts.

/// Text and layout flow direction.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum LayoutDirection {
    /// Left-to-Right layout direction (default for Latin, Cyrillic, Greek, etc.).
    #[default]
    Ltr,
    /// Right-to-Left layout direction (standard for Arabic, Hebrew, Persian, Urdu).
    Rtl,
}

impl LayoutDirection {
    /// Check if the layout direction is Right-to-Left.
    #[must_use]
    pub const fn is_rtl(self) -> bool {
        matches!(self, Self::Rtl)
    }

    /// Check if the layout direction is Left-to-Right.
    #[must_use]
    pub const fn is_ltr(self) -> bool {
        matches!(self, Self::Ltr)
    }

    /// Invert / mirror the direction.
    #[must_use]
    pub const fn mirror(self) -> Self {
        match self {
            Self::Ltr => Self::Rtl,
            Self::Rtl => Self::Ltr,
        }
    }
}
