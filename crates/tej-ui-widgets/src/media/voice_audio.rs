//! Voice audio waveform visualizer, audio recorder, and AI Voice Orb.
//! Styled with Tej's live audio level bars, mic status badges, and pulsating radial gradients.

use tej_ui_hal::Color;
use tej_ui_layout::Rect;

/// Audio amplitude waveform buffer.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AudioWaveformState {
    pub samples: Vec<f32>,      // 0.0 to 1.0 normalized amplitudes
    pub playback_position: f32, // 0.0 to 1.0
}

impl AudioWaveformState {
    #[must_use]
    pub fn new(samples: Vec<f32>) -> Self {
        Self {
            samples,
            playback_position: 0.0,
        }
    }
}

/// State for a live Voice AI session Orb (pulsating with speech activity).
#[derive(Debug, Clone, PartialEq)]
pub struct VoiceOrbState {
    pub is_listening: bool,
    pub is_speaking: bool,
    pub pulse_phase: f32,  // 0.0 to 2*PI
    pub audio_energy: f32, // 0.0 to 1.0
}

impl Default for VoiceOrbState {
    fn default() -> Self {
        Self {
            is_listening: false,
            is_speaking: false,
            pulse_phase: 0.0,
            audio_energy: 0.0,
        }
    }
}

/// Tej styled Audio Waveform component.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioWaveform {
    pub bounds: Rect,
    pub bar_width: f32,
    pub bar_gap: f32,
    pub active_bar_color: Color,
    pub inactive_bar_color: Color,
    pub corner_radius: f32,
}

impl Default for AudioWaveform {
    fn default() -> Self {
        Self {
            bounds: Rect::ZERO,
            bar_width: 3.0,
            bar_gap: 2.0,
            active_bar_color: Color::from_rgb_u32(0x007F_56D9), // Tej Brand 600
            inactive_bar_color: Color::from_rgb_u32(0x00D0_D5DD), // Tej Gray 300
            corner_radius: 1.5,
        }
    }
}
