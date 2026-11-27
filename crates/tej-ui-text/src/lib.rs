//! `tej-text`: System font discovery, text shaping, and zero-copy string slice measurement.
//!
//! Provides system font abstractions, Unicode bidirectional text (UAX #9), UAX #29 grapheme
//! segmentation, multi-line word wrapping, complex script shaping with OpenType GSUB/GPOS
//! features, system font discovery, and VRAM glyph streaming.

pub mod bidi;
pub mod complex;
pub mod fallback;
pub mod font;
pub mod font_db;
pub mod glyph_stream;
pub mod grapheme;
pub mod measurement;
pub mod opentype_shaper;
pub mod rasterizer;
pub mod shaper;
pub mod wrap;

pub use bidi::{BidiClassifier, BidiRun, TextDirection};
pub use complex::{ArabicJoiningForm, CursiveScriptJoiner, EmojiPresentation, OpenTypeFeatures};
pub use fallback::{FontCategory, FontFallbackChain};
pub use font::{FontWeight, SystemFont};
pub use font_db::SystemFontDb;
pub use glyph_stream::{CaretHit, GlyphStreamConfig, GlyphStreamer};
pub use grapheme::{
    grapheme_count, next_grapheme_boundary, next_word_boundary, pop_grapheme,
    prev_grapheme_boundary, prev_word_boundary,
};
pub use measurement::TextMeasurer;
pub use opentype_shaper::{OpenTypeShaper, PositionedGlyph, ShapedTextRun, ShaperFeatures};
pub use rasterizer::{FontRasterizer, RasterizedGlyph};
pub use shaper::{GlyphCluster, ShapedRun, TextShaper};
pub use wrap::{WrappedText, wrap_text};
