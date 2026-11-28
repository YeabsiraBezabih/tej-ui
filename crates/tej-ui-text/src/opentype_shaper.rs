//! Full OpenType GSUB/GPOS complex script shaping using `swash`.

use crate::bidi::TextDirection;
use parking_lot::Mutex;
use swash::FontRef;
use swash::shape::ShapeContext;

/// A positioned glyph cluster resulting from OpenType shaping.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedGlyph {
    /// Glyph identifier in the font.
    pub glyph_id: u16,
    /// Byte offset in the source string.
    pub cluster_index: u32,
    /// Advance width along text progression axis.
    pub advance: f32,
    /// Horizontal offset from current pen position.
    pub x_offset: f32,
    /// Vertical offset from text baseline.
    pub y_offset: f32,
}

/// A shaped run containing positioned glyph clusters and layout metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedTextRun {
    pub direction: TextDirection,
    pub total_width: f32,
    pub font_size: f32,
    pub glyphs: Vec<PositionedGlyph>,
}

/// OpenType feature flags to toggle during shaping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Public OpenType shaper toggle struct; callers and tests access boolean fields directly"
)]
pub struct ShaperFeatures {
    /// Pair kerning (`kern`).
    pub kerning: bool,
    /// Standard ligatures (`liga`).
    pub standard_ligatures: bool,
    /// Discretionary ligatures (`dlig`).
    pub discretionary_ligatures: bool,
    /// Contextual alternates (`calt`).
    pub contextual_alternates: bool,
    /// Tabular figures (`tnum`).
    pub tabular_numbers: bool,
}

impl Default for ShaperFeatures {
    fn default() -> Self {
        Self {
            kerning: true,
            standard_ligatures: true,
            discretionary_ligatures: false,
            contextual_alternates: true,
            tabular_numbers: false,
        }
    }
}

/// Thread-safe OpenType text shaper utilizing `swash` for complex script layout.
#[derive(Default)]
pub struct OpenTypeShaper {
    context: Mutex<ShapeContext>,
}

impl std::fmt::Debug for OpenTypeShaper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenTypeShaper").finish()
    }
}

impl OpenTypeShaper {
    /// Constructs a new OpenType text shaper.
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: Mutex::new(ShapeContext::new()),
        }
    }

    /// Shapes a text slice using raw font data with full OpenType features and Bidi direction.
    pub fn shape_text(
        &self,
        font_data: &[u8],
        font_index: usize,
        text: &str,
        font_size: f32,
        direction: TextDirection,
        features: ShaperFeatures,
    ) -> Option<ShapedTextRun> {
        let font_ref = FontRef::from_index(font_data, font_index)?;

        let mut swash_features = Vec::with_capacity(8);
        if features.kerning {
            swash_features.push(("kern", 1));
        }
        if features.standard_ligatures {
            swash_features.push(("liga", 1));
        }
        if features.discretionary_ligatures {
            swash_features.push(("dlig", 1));
        }
        if features.contextual_alternates {
            swash_features.push(("calt", 1));
        }
        if features.tabular_numbers {
            swash_features.push(("tnum", 1));
        }

        let swash_dir = match direction {
            TextDirection::LeftToRight | TextDirection::Auto => {
                swash::shape::Direction::LeftToRight
            }
            TextDirection::RightToLeft => swash::shape::Direction::RightToLeft,
        };

        let mut context = self.context.lock();
        let mut shaper = context
            .builder(font_ref)
            .size(font_size)
            .direction(swash_dir)
            .features(&swash_features)
            .build();

        shaper.add_str(text);

        let mut glyphs = Vec::with_capacity(text.len());
        let mut total_width = 0.0f32;

        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
                total_width += glyph.advance;
                glyphs.push(PositionedGlyph {
                    glyph_id: glyph.id,
                    cluster_index: cluster.source.start,
                    advance: glyph.advance,
                    x_offset: glyph.x,
                    y_offset: glyph.y,
                });
            }
        });

        Some(ShapedTextRun {
            direction,
            total_width,
            font_size,
            glyphs,
        })
    }
}
