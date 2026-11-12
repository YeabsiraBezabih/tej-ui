//! Curated high-frequency Lucide / Feather icon definitions.

use crate::path::VectorPath;

/// Canonical standard UI icons matching the Lucide icon library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LucideIcon {
    // Navigation
    ChevronDown,
    ChevronRight,
    ChevronUp,
    ChevronLeft,
    ArrowRight,
    ArrowLeft,
    Home,
    Menu,

    // Actions & Editing
    Plus,
    Minus,
    Check,
    X,
    Trash,
    Edit,
    Copy,
    RefreshCw,
    Download,
    Upload,
    ExternalLink,

    // Search & Filter
    Search,
    Filter,

    // Settings & Security
    Settings,
    Lock,
    Eye,
    EyeOff,

    // Files & Data
    File,
    Folder,

    // Identity & Communication
    User,
    Users,
    Bell,
    Mail,
    Calendar,

    // Theme & Status
    Sun,
    Moon,
    Info,
    AlertCircle,
    CheckCircle,
    HelpCircle,
}

impl LucideIcon {
    /// Returns the SVG vector path command string in standard 24x24 viewbox.
    #[must_use]
    pub const fn svg_path(self) -> &'static str {
        match self {
            // Navigation
            Self::ChevronDown => "M6 9l6 6 6-6",
            Self::ChevronRight => "M9 18l6-6-6-6",
            Self::ChevronUp => "M18 15l-6-6-6 6",
            Self::ChevronLeft => "M15 18l-6-6 6-6",
            Self::ArrowRight => "M5 12h14 M12 5l7 7-7 7",
            Self::ArrowLeft => "M19 12H5 M12 19l-7-7 7-7",
            Self::Home => "M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z M9 22V12h6v10",
            Self::Menu => "M3 12h18 M3 6h18 M3 18h18",

            // Actions
            Self::Plus => "M12 5v14 M5 12h14",
            Self::Minus => "M5 12h14",
            Self::Check => "M20 6L9 17l-5-5",
            Self::X => "M18 6L6 18 M6 6l12 12",
            Self::Trash => {
                "M3 6h18 M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6 M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
            }
            Self::Edit => {
                "M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7 M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"
            }
            Self::Copy => {
                "M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2 M16 4h2a2 2 0 0 1 2 2v4 M8 4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2"
            }
            Self::RefreshCw => {
                "M23 4v6h-6 M1 20v-6h6 M3.51 9a9 9 0 0 1 14.85-3.36L23 10 M1 14l4.64 4.36A9 9 0 0 0 20.49 15"
            }
            Self::Download => "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4 M7 10l5 5 5-5 M12 15V3",
            Self::Upload => "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4 M17 8l-5-5-5 5 M12 3v12",
            Self::ExternalLink => {
                "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6 M15 3h6v6 M10 14L21 3"
            }

            // Search & Filter
            Self::Search => "M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16z M21 21l-4.35-4.35",
            Self::Filter => "M22 3H2l8 9.46V19l4 2v-8.54L22 3z",

            // Settings & Security
            Self::Settings => {
                "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
            }
            Self::Lock => {
                "M19 11H5a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7a2 2 0 0 0-2-2z M7 11V7a5 5 0 0 1 10 0v4"
            }
            Self::Eye => {
                "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z"
            }
            Self::EyeOff => {
                "M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24 M1 1l22 22"
            }

            // Files & Folders
            Self::File => "M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z M13 2v7h7",
            Self::Folder => {
                "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
            }

            // Identity & Communication
            Self::User => {
                "M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2 M12 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z"
            }
            Self::Users => {
                "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2 M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z M23 21v-2a4 4 0 0 0-3-3.87 M16 3.13a4 4 0 0 1 0 7.75"
            }
            Self::Bell => "M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9 M13.73 21a2 2 0 0 1-3.46 0",
            Self::Mail => {
                "M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z M22 6l-10 7L2 6"
            }
            Self::Calendar => {
                "M19 4H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2z M16 2v4 M8 2v4 M3 10h18"
            }

            // Theme & Status
            Self::Sun => {
                "M12 1v2 M12 21v2 M4.22 4.22l1.42 1.42 M18.36 18.36l1.42 1.42 M1 12h2 M21 12h2 M4.22 19.78l1.42-1.42 M18.36 5.64l1.42-1.42 M12 17a5 5 0 1 0 0-10 5 5 0 0 0 0 10z"
            }
            Self::Moon => "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z",
            Self::Info => "M12 16v-4 M12 8h.01 M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z",
            Self::AlertCircle => "M12 8v4 M12 16h.01 M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z",
            Self::CheckCircle => "M22 11.08V12a10 10 0 1 1-5.93-9.14 M22 4L12 14.01l-3-3",
            Self::HelpCircle => {
                "M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3 M12 17h.01 M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z"
            }
        }
    }

    /// Converts this Lucide icon into a cached VectorPath.
    #[must_use]
    pub fn to_vector_path(self) -> VectorPath {
        VectorPath::parse_svg(self.svg_path())
    }
}
