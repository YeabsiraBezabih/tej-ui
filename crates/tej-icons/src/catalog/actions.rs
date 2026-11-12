//! Action and command icons (Lucide catalog).

/// Action icon variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionIcon {
    Plus,
    Minus,
    Check,
    X,
    Trash,
    Trash2,
    Edit,
    Edit2,
    Copy,
    Save,
    Share,
    Download,
    Upload,
    RefreshCw,
    RefreshCcw,
    Filter,
    ZoomIn,
    ZoomOut,
    Maximize,
    Minimize,
}

impl ActionIcon {
    /// Returns the SVG vector path data in 24x24 coordinate space.
    #[must_use]
    pub const fn svg_path(self) -> &'static str {
        match self {
            Self::Plus => "M12 5v14 M5 12h14",
            Self::Minus => "M5 12h14",
            Self::Check => "M20 6L9 17l-5-5",
            Self::X => "M18 6L6 18 M6 6l12 12",
            Self::Trash => {
                "M3 6h18 M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6 M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
            }
            Self::Trash2 => "M3 6h18 M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6 M10 11v6 M14 11v6",
            Self::Edit => {
                "M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7 M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"
            }
            Self::Edit2 => "M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z",
            Self::Copy => {
                "M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2 M16 4h2a2 2 0 0 1 2 2v4 M8 4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2"
            }
            Self::Save => {
                "M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z M17 21v-8H7v8 M7 3v5h8"
            }
            Self::Share => "M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8 M16 6l-4-4-4 4 M12 2v13",
            Self::Download => "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4 M7 10l5 5 5-5 M12 15V3",
            Self::Upload => "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4 M17 8l-5-5-5 5 M12 3v12",
            Self::RefreshCw => {
                "M23 4v6h-6 M1 20v-6h6 M3.51 9a9 9 0 0 1 14.85-3.36L23 10 M1 14l4.64 4.36A9 9 0 0 0 20.49 15"
            }
            Self::RefreshCcw => {
                "M1 4v6h6 M23 20v-6h-6 M20.49 9A9 9 0 0 0 5.64 5.64L1 10 M23 14l-4.64 4.36A9 9 0 0 1 3.51 15"
            }
            Self::Filter => "M22 3H2l8 9.46V19l4 2v-8.54L22 3z",
            Self::ZoomIn => {
                "M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16z M21 21l-4.35-4.35 M11 8v6 M8 11h6"
            }
            Self::ZoomOut => "M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16z M21 21l-4.35-4.35 M8 11h6",
            Self::Maximize => {
                "M8 3H5a2 2 0 0 0-2 2v3 M21 8V5a2 2 0 0 0-2-2h-3 M3 16v3a2 2 0 0 0 2 2h3 M16 21h3a2 2 0 0 0 2-2v-3"
            }
            Self::Minimize => "M4 14h6v6 M20 10h-6V4 M14 10l7-7 M3 21l7-7",
        }
    }
}
