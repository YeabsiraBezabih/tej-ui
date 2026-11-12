//! System, user profile, security, and status icons (Lucide catalog).

/// System, identity, and status icon variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemIcon {
    Search,
    Settings,
    Tool,
    User,
    Users,
    UserPlus,
    UserCheck,
    UserX,
    Lock,
    Unlock,
    Key,
    Shield,
    ShieldCheck,
    Eye,
    EyeOff,
    Sun,
    Moon,
    Info,
    HelpCircle,
    AlertCircle,
    AlertTriangle,
    CheckCircle,
    XCircle,
    Clock,
    Calendar,
    Bookmark,
    Star,
    Heart,
    Flag,
}

impl SystemIcon {
    /// Returns the SVG vector path data in 24x24 coordinate space.
    #[must_use]
    pub const fn svg_path(self) -> &'static str {
        match self {
            Self::Search => "M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16z M21 21l-4.35-4.35",
            Self::Settings => {
                "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
            }
            Self::Tool => {
                "M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"
            }
            Self::User => {
                "M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2 M12 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z"
            }
            Self::Users => {
                "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2 M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z M23 21v-2a4 4 0 0 0-3-3.87 M16 3.13a4 4 0 0 1 0 7.75"
            }
            Self::UserPlus => {
                "M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2 M8.5 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z M20 8v6 M23 11h-6"
            }
            Self::UserCheck => {
                "M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2 M8.5 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z M17 11l2 2 4-4"
            }
            Self::UserX => {
                "M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2 M8.5 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8z M18 8l5 5 M23 8l-5 5"
            }
            Self::Lock => {
                "M19 11H5a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7a2 2 0 0 0-2-2z M7 11V7a5 5 0 0 1 10 0v4"
            }
            Self::Unlock => {
                "M19 11H5a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7a2 2 0 0 0-2-2z M7 11V7a5 5 0 0 1 9.9-1"
            }
            Self::Key => {
                "M21 2l-2 2m-1.5 1.5L16 7l-2-2-1.5 1.5 2 2-3 3a6.5 6.5 0 1 1-1.41-1.41l3-3-2-2 1.5-1.5 2 2 1.5-1.5L22 4l-1-2z"
            }
            Self::Shield => "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z",
            Self::ShieldCheck => "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z M9 12l2 2 4-4",
            Self::Eye => {
                "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z"
            }
            Self::EyeOff => {
                "M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24 M1 1l22 22"
            }
            Self::Sun => {
                "M12 1v2 M12 21v2 M4.22 4.22l1.42 1.42 M18.36 18.36l1.42 1.42 M1 12h2 M21 12h2 M4.22 19.78l1.42-1.42 M18.36 5.64l1.42-1.42 M12 17a5 5 0 1 0 0-10 5 5 0 0 0 0 10z"
            }
            Self::Moon => "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z",
            Self::Info => "M12 16v-4 M12 8h.01 M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z",
            Self::HelpCircle => {
                "M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3 M12 17h.01 M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z"
            }
            Self::AlertCircle => "M12 8v4 M12 16h.01 M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z",
            Self::AlertTriangle => {
                "M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z M12 9v4 M12 17h.01"
            }
            Self::CheckCircle => "M22 11.08V12a10 10 0 1 1-5.93-9.14 M22 4L12 14.01l-3-3",
            Self::XCircle => "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z M15 9l-6 6 M9 9l6 6",
            Self::Clock => "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z M12 6v6l4 2",
            Self::Calendar => {
                "M19 4H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2z M16 2v4 M8 2v4 M3 10h18"
            }
            Self::Bookmark => "M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z",
            Self::Star => {
                "M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
            }
            Self::Heart => {
                "M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"
            }
            Self::Flag => "M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z M4 22v-7",
        }
    }
}
