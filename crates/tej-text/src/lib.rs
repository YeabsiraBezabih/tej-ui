//! `tej-text`: System font discovery, text shaping, and zero-copy string slice measurement.
//!
//! Provides system font abstractions, Unicode bidirectional text (UAX #9), complex script
//! shaping with ligature detection, and system font fallback chains for CJK and color emoji.

pub mod bidi;
pub mod fallback;
pub mod font;
pub mod measurement;
pub mod rasterizer;
pub mod shaper;

pub use bidi::{BidiClassifier, BidiRun, TextDirection};
pub use fallback::{FontCategory, FontFallbackChain};
pub use font::{FontWeight, SystemFont};
pub use measurement::TextMeasurer;
pub use rasterizer::{FontRasterizer, RasterizedGlyph};
pub use shaper::{GlyphCluster, ShapedRun, TextShaper};
