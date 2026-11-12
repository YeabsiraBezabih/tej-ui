//! `tej-text`: System font discovery, text shaping, and zero-copy string slice measurement.
//!
//! Provides system font abstractions and cached bounding-box measurement without string clones.

pub mod font;
pub mod measurement;

pub use font::{FontWeight, SystemFont};
pub use measurement::TextMeasurer;
