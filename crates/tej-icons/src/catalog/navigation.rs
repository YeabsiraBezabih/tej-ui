//! Navigation and directional icons (Lucide catalog).

/// Navigation icon variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavigationIcon {
    ChevronDown,
    ChevronUp,
    ChevronLeft,
    ChevronRight,
    ChevronsDown,
    ChevronsUp,
    ChevronsLeft,
    ChevronsRight,
    ArrowDown,
    ArrowUp,
    ArrowLeft,
    ArrowRight,
    ArrowUpRight,
    ArrowDownRight,
    CornerUpLeft,
    CornerDownRight,
    Home,
    Menu,
    MoreHorizontal,
    MoreVertical,
    ExternalLink,
    Compass,
    MapPin,
}

impl NavigationIcon {
    /// Returns the SVG vector path data in 24x24 coordinate space.
    #[must_use]
    pub const fn svg_path(self) -> &'static str {
        match self {
            Self::ChevronDown => "M6 9l6 6 6-6",
            Self::ChevronUp => "M18 15l-6-6-6 6",
            Self::ChevronLeft => "M15 18l-6-6 6-6",
            Self::ChevronRight => "M9 18l6-6-6-6",
            Self::ChevronsDown => "M7 13l5 5 5-5 M7 6l5 5 5-5",
            Self::ChevronsUp => "M17 11l-5-5-5 5 M17 18l-5-5-5 5",
            Self::ChevronsLeft => "M11 17l-5-5 5-5 M18 17l-5-5 5-5",
            Self::ChevronsRight => "M13 17l5-5-5-5 M6 17l5-5-5-5",
            Self::ArrowDown => "M12 5v14 M19 12l-7 7-7-7",
            Self::ArrowUp => "M12 19V5 M5 12l7-7 7 7",
            Self::ArrowLeft => "M19 12H5 M12 19l-7-7 7-7",
            Self::ArrowRight => "M5 12h14 M12 5l7 7-7 7",
            Self::ArrowUpRight => "M7 17L17 7 M7 7h10v10",
            Self::ArrowDownRight => "M7 7l10 10 M17 7v10H7",
            Self::CornerUpLeft => "M9 14L4 9l5-5 M20 20v-7a4 4 0 0 0-4-4H4",
            Self::CornerDownRight => "M15 10l5 5-5 5 M4 4v7a4 4 0 0 0 4 4h12",
            Self::Home => "M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z M9 22V12h6v10",
            Self::Menu => "M3 12h18 M3 6h18 M3 18h18",
            Self::MoreHorizontal => {
                "M12 13a1 1 0 1 0 0-2 1 1 0 0 0 0 2z M19 13a1 1 0 1 0 0-2 1 1 0 0 0 0 2z M5 13a1 1 0 1 0 0-2 1 1 0 0 0 0 2z"
            }
            Self::MoreVertical => {
                "M12 13a1 1 0 1 0 0-2 1 1 0 0 0 0 2z M12 6a1 1 0 1 0 0-2 1 1 0 0 0 0 2z M12 20a1 1 0 1 0 0-2 1 1 0 0 0 0 2z"
            }
            Self::ExternalLink => {
                "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6 M15 3h6v6 M10 14L21 3"
            }
            Self::Compass => {
                "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z M16.24 7.76l-2.12 6.36-6.36 2.12 2.12-6.36 6.36-2.12z"
            }
            Self::MapPin => {
                "M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z M12 13a3 3 0 1 0 0-6 3 3 0 0 0 0 6z"
            }
        }
    }
}
