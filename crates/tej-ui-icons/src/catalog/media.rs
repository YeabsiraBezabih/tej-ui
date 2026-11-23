//! Audio, video, and multimedia icons (Lucide catalog).

/// Media icon variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaIcon {
    Play,
    Pause,
    Square,
    SkipForward,
    SkipBack,
    Volume,
    Volume1,
    Volume2,
    VolumeX,
    Mic,
    MicOff,
    Video,
    VideoOff,
    Camera,
    Image,
    Music,
}

impl MediaIcon {
    /// Returns the SVG vector path data in 24x24 coordinate space.
    #[must_use]
    pub const fn svg_path(self) -> &'static str {
        match self {
            Self::Play => "M5 3l14 9-14 9V3z",
            Self::Pause => "M6 4h4v16H6z M14 4h4v16h-4z",
            Self::Square => "M3 3h18v18H3z",
            Self::SkipForward => "M5 4l10 8-10 8V4z M19 5v14",
            Self::SkipBack => "M19 20L9 12l10-8v16z M5 19V5",
            Self::Volume => "M11 5L6 9H2v6h4l5 4V5z",
            Self::Volume1 => "M11 5L6 9H2v6h4l5 4V5z M15.54 8.46a5 5 0 0 1 0 7.07",
            Self::Volume2 => {
                "M11 5L6 9H2v6h4l5 4V5z M19.07 4.93a10 10 0 0 1 0 14.14 M15.54 8.46a5 5 0 0 1 0 7.07"
            }
            Self::VolumeX => "M11 5L6 9H2v6h4l5 4V5z M23 9l-6 6 M17 9l6 6",
            Self::Mic => {
                "M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z M19 10v2a7 7 0 0 1-14 0v-2 M12 19v4 M8 23h8"
            }
            Self::MicOff => {
                "M1 1l22 22 M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6 M17 16.95A7 7 0 0 1 5 12v-2m14 0v2a7 7 0 0 1-.11 1.23 M12 19v4 M8 23h8"
            }
            Self::Video => {
                "M23 7l-7 5 7 5V7z M14 5H3a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h11a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2z"
            }
            Self::VideoOff => {
                "M16 16v1a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h1m5 0h5a2 2 0 0 1 2 2v1l7-5v10l-2.5-1.79 M1 1l22 22"
            }
            Self::Camera => {
                "M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z M12 17a4 4 0 1 0 0-8 4 4 0 0 0 0 8z"
            }
            Self::Image => {
                "M19 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2z M8.5 10a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z M21 15l-5-5L5 21"
            }
            Self::Music => {
                "M9 18V5l12-2v13 M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0z M21 16a3 3 0 1 1-6 0 3 3 0 0 1 6 0z"
            }
        }
    }
}
