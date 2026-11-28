//! Automated tests for SystemFontDb, OpenTypeShaper, and GlyphStreamer.

use tej_ui_text::{
    CaretHit, FontSlant, GlyphStreamer, OpenTypeShaper, PositionedGlyph, ShapedTextRun,
    ShaperFeatures, SystemFontDb, TextDirection,
};

#[test]
fn test_system_font_db_discovery() {
    let db = SystemFontDb::new();
    // System font discovery queries OS font database
    let count = db.family_count();
    println!("Discovered {count} system font families on host");

    // Query generic families
    let sans = db.query_font("sans-serif", 400, FontSlant::Normal);
    if count > 0 {
        assert!(sans.is_some(), "Generic sans-serif font should resolve");
    }
}

#[test]
fn test_glyph_streamer_hit_testing() {
    let run = ShapedTextRun {
        direction: TextDirection::LeftToRight,
        total_width: 100.0,
        font_size: 16.0,
        glyphs: vec![
            PositionedGlyph {
                glyph_id: 1,
                cluster_index: 0,
                advance: 20.0,
                x_offset: 0.0,
                y_offset: 0.0,
            },
            PositionedGlyph {
                glyph_id: 2,
                cluster_index: 1,
                advance: 25.0,
                x_offset: 0.0,
                y_offset: 0.0,
            },
            PositionedGlyph {
                glyph_id: 3,
                cluster_index: 2,
                advance: 30.0,
                x_offset: 0.0,
                y_offset: 0.0,
            },
        ],
    };

    let streamer = GlyphStreamer::new();

    // Hit test at start (x = 5.0 in first glyph advance 20.0, mid = 10.0) -> cluster 0, leading
    assert_eq!(
        streamer.hit_test_x(&run, 5.0),
        CaretHit {
            cluster_index: 0,
            is_trailing: false
        }
    );

    // Hit test past mid of first glyph (x = 15.0) -> cluster 0, trailing
    assert_eq!(
        streamer.hit_test_x(&run, 15.0),
        CaretHit {
            cluster_index: 0,
            is_trailing: true
        }
    );

    // Hit test in second glyph (x = 30.0, glyph 2 starts at 20.0, advance 25.0, mid = 32.5) -> cluster 1, leading
    assert_eq!(
        streamer.hit_test_x(&run, 30.0),
        CaretHit {
            cluster_index: 1,
            is_trailing: false
        }
    );

    // Hit test past end (x = 150.0) -> last cluster trailing
    assert_eq!(
        streamer.hit_test_x(&run, 150.0),
        CaretHit {
            cluster_index: 2,
            is_trailing: true
        }
    );
}

#[test]
fn test_opentype_shaper_instantiation_and_features() {
    let shaper = OpenTypeShaper::new();
    let features = ShaperFeatures::default();
    assert!(features.kerning);
    assert!(features.standard_ligatures);
    assert!(!features.discretionary_ligatures);

    // Test with invalid data gracefully returns None
    assert!(
        shaper
            .shape_text(&[], 0, "Hello", 16.0, TextDirection::LeftToRight, features)
            .is_none()
    );
}
