//! Typography, text formatting, and document editor icons (Lucide catalog).

/// Rich text editor and formatting icon variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorIcon {
    Bold,
    Italic,
    Underline,
    AlignLeft,
    AlignCenter,
    AlignRight,
    AlignJustify,
    List,
    Code,
    Link,
    Link2,
    Scissors,
    Hash,
}

impl EditorIcon {
    /// Returns the SVG vector path data in 24x24 coordinate space.
    #[must_use]
    pub const fn svg_path(self) -> &'static str {
        match self {
            Self::Bold => {
                "M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z"
            }
            Self::Italic => "M19 4h-9 M14 20H5 M15 4L9 20",
            Self::Underline => "M6 3v7a6 6 0 0 0 6 6 6 6 0 0 0 6-6V3 M4 21h16",
            Self::AlignLeft => "M17 10H3 M21 6H3 M21 14H3 M17 18H3",
            Self::AlignCenter => "M18 10H6 M21 6H3 M21 14H3 M18 18H6",
            Self::AlignRight => "M21 10H7 M21 6H3 M21 14H3 M21 18H7",
            Self::AlignJustify => "M21 10H3 M21 6H3 M21 14H3 M21 18H3",
            Self::List => "M8 6h13 M8 12h13 M8 18h13 M3 6h.01 M3 12h.01 M3 18h.01",
            Self::Code => "M16 18l6-6-6-6 M8 6l-6 6 6 6",
            Self::Link => {
                "M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71 M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"
            }
            Self::Link2 => {
                "M15 7h3a5 5 0 0 1 5 5 5 5 0 0 1-5 5h-3m-6 0H6a5 5 0 0 1-5-5 5 5 0 0 1 5-5h3 M8 12h8"
            }
            Self::Scissors => {
                "M6 9a3 3 0 1 0 0-6 3 3 0 0 0 0 6z M6 21a3 3 0 1 0 0-6 3 3 0 0 0 0 6z M20 4L8.12 15.88 M14.47 14.48L20 20 M8.12 8.12L12 12"
            }
            Self::Hash => "M4 9h16 M4 15h16 M10 3L8 21 M16 3l-2 18",
        }
    }
}
