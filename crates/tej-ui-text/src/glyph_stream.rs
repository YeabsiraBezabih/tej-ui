//! Subpixel glyph rasterizer and streaming bridge into `tej-hal`'s VRAM texture atlas.

use crate::opentype_shaper::ShapedTextRun;
use parking_lot::Mutex;
use swash::FontRef;
use swash::scale::{Render, ScaleContext, Source};
use tej_ui_hal::gpu::{AtlasAllocation, GlyphInstance, GlyphKey, PremulColor, VramAtlas};

/// Caret hit-test result within a shaped text run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaretHit {
    /// Byte offset within the original string.
    pub cluster_index: u32,
    /// Whether the hit coordinate falls in the trailing half of the glyph.
    pub is_trailing: bool,
}

/// Target parameters for streaming a shaped run into GPU glyph instances.
#[derive(Debug, Clone, Copy)]
pub struct GlyphStreamConfig<'a> {
    pub font_data: &'a [u8],
    pub font_index: usize,
    pub font_id: u64,
    pub origin_x: f32,
    pub origin_y: f32,
    pub color: PremulColor,
    pub clip_bounds: [f32; 4],
}

/// Subpixel-aware glyph rasterizer streaming rendered glyphs directly into the GPU texture atlas.
pub struct GlyphStreamer {
    scale_context: Mutex<ScaleContext>,
}

impl Default for GlyphStreamer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for GlyphStreamer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphStreamer").finish()
    }
}

impl GlyphStreamer {
    /// Constructs a new glyph streamer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            scale_context: Mutex::new(ScaleContext::new()),
        }
    }

    /// Streams a shaped text run into GPU glyph instances, uploading missing glyphs into VRAM atlas.
    pub fn stream_run(
        &self,
        config: GlyphStreamConfig<'_>,
        run: &ShapedTextRun,
        atlas: &mut VramAtlas,
        out_instances: &mut Vec<GlyphInstance>,
    ) {
        let font_ref = match FontRef::from_index(config.font_data, config.font_index) {
            Some(f) => f,
            None => return,
        };

        let mut scale_context = self.scale_context.lock();
        let mut scaler = scale_context
            .builder(font_ref)
            .size(run.font_size)
            .hint(true)
            .build();

        let mut pen_x = config.origin_x;

        for g in &run.glyphs {
            // Compute 1/4 subpixel horizontal bin (0..3)
            let subpixel_bin = ((pen_x + g.x_offset).fract().abs() * 4.0).floor() as u8;
            let size_px = run.font_size.round() as u16;

            let key = GlyphKey::new(config.font_id, u32::from(g.glyph_id), subpixel_bin, size_px);

            let alloc = match atlas.get(&key) {
                Some(a) => a,
                None => {
                    // Rasterize glyph using swash scaler
                    if let Some(image) = Render::new(&[
                        Source::ColorOutline(0),
                        Source::ColorBitmap(swash::scale::StrikeWith::BestFit),
                        Source::Outline,
                    ])
                    .format(swash::zeno::Format::Alpha)
                    .render(&mut scaler, g.glyph_id)
                    {
                        let w = image.placement.width;
                        let h = image.placement.height;

                        if w > 0 && h > 0 {
                            atlas
                                .insert(key, w, h, &image.data)
                                .unwrap_or(AtlasAllocation {
                                    uv_bounds: [0.0; 4],
                                    pixel_rect: [0, 0, w, h],
                                    last_used_epoch: 0,
                                })
                        } else {
                            pen_x += g.advance;
                            continue;
                        }
                    } else {
                        pen_x += g.advance;
                        continue;
                    }
                }
            };

            let glyph_w = alloc.pixel_rect[2] as f32;
            let glyph_h = alloc.pixel_rect[3] as f32;
            let quad_x = pen_x + g.x_offset;
            let quad_y = config.origin_y + g.y_offset;

            out_instances.push(GlyphInstance::new(
                [quad_x, quad_y, glyph_w, glyph_h],
                alloc.uv_bounds,
                config.color,
                config.clip_bounds,
                false,
            ));

            pen_x += g.advance;
        }
    }

    /// Hit-tests a horizontal pixel offset against a shaped text run to determine caret index.
    #[must_use]
    pub fn hit_test_x(&self, run: &ShapedTextRun, x_offset: f32) -> CaretHit {
        if run.glyphs.is_empty() || x_offset <= 0.0 {
            return CaretHit {
                cluster_index: 0,
                is_trailing: false,
            };
        }

        let mut current_x = 0.0f32;

        for g in &run.glyphs {
            let next_x = current_x + g.advance;
            let mid_x = current_x + g.advance * 0.5;

            if x_offset >= current_x && x_offset < next_x {
                return CaretHit {
                    cluster_index: g.cluster_index,
                    is_trailing: x_offset >= mid_x,
                };
            }
            current_x = next_x;
        }

        let last_cluster = run.glyphs.last().map(|g| g.cluster_index).unwrap_or(0);
        CaretHit {
            cluster_index: last_cluster,
            is_trailing: true,
        }
    }
}
