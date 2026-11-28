//! Keyframe timeline editor with time rulers and animated diamonds.
//! Styled with Tej's timeline tick marks, track headers, and active playhead lines.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// An individual keyframe plotted on a track.
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe {
    pub time_seconds: f32,
    pub value: f32,
    pub is_selected: bool,
}

/// A track in the animation timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineTrack {
    pub name: String,
    pub keyframes: Vec<Keyframe>,
}

/// State for the animation timeline editor.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TimelineState {
    pub current_time_seconds: f32,
    pub total_duration_seconds: f32,
    pub tracks: Vec<TimelineTrack>,
    pub is_playing: bool,
}

/// Tej styled Timeline component.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineEditor {
    pub bounds: Rect,
    pub header_width: f32,
    pub track_height: f32,
    pub background: Color,
    pub playhead_color: Color,
}

impl Default for TimelineEditor {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            header_width: 160.0,
            track_height: 32.0,
            background: Color::from_rgb_u32(0x0010_1828), // Tej Gray 900
            playhead_color: Color::from_rgb_u32(0x00F0_4438), // Tej Error 500
        }
    }
}
