//! Creative & Continuous Controls Suite.
//!
//! Provides production-ready widgets for Rotary Knobs, Audio Level Meters,
//! Histogram Range Sliders, Multi-Stop Gradient Editors, and Aspect Ratio Pickers.

pub mod color_extended;
pub mod knob;
pub mod sliders_extended;

pub use color_extended::{AspectRatioPicker, AspectRatioPreset, ColorStop, GradientEditorState};
pub use knob::{KnobState, RotaryKnob};
pub use sliders_extended::{HistogramSlider, HistogramSliderState};
