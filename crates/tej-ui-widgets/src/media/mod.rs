//! Surfaces, Media & Audio Visualizers Suite.
//!
//! Provides production-ready widgets for Image Comparison sliders, Image Carousels,
//! Vector Signature Pads, File Dropzones, Spotlight Cards, and Voice Audio Waveforms.

pub mod carousel;
pub mod cropper;
pub mod spotlight;
pub mod voice_audio;

pub use carousel::{CarouselState, ImageCompare, ImageCompareState};
pub use cropper::{Dropzone, DropzoneState, SignaturePadState, SignatureStroke, StrokePoint};
pub use spotlight::{MetricCardState, MetricTrend, SpotlightCard};
pub use voice_audio::{AudioWaveform, AudioWaveformState, VoiceOrbState};
