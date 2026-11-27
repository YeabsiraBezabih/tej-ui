use tej_ui_text::{
    BidiClassifier, FontRasterizer, GlyphStreamer, PositionedGlyph, ShapedTextRun, SystemFont,
    TextDirection, TextMeasurer,
};

#[test]
fn test_text_measurer_content_hash_prevents_aba_cache_poisoning() {
    let measurer = TextMeasurer::new();

    // Mutate the same heap buffer in-place so pointer address AND byte length (4 bytes) stay identical,
    // while character count changes from 4 ASCII chars ("aaaa") to 2 multi-byte UTF-8 chars ("éé").
    let mut buf = String::with_capacity(16);
    buf.push_str("aaaa");
    assert_eq!(buf.len(), 4);
    let size_ascii = measurer.measure_slice(&buf, SystemFont::Body, 16.0);

    buf.clear();
    buf.push_str("éé");
    assert_eq!(buf.len(), 4);
    let size_utf8 = measurer.measure_slice(&buf, SystemFont::Body, 16.0);

    // Must not return cached 4-char width for the 2-char string
    assert!(
        (size_ascii.width - size_utf8.width * 2.0).abs() < 0.01,
        "ABA cache poisoning detected: ascii={:?}, utf8={:?}",
        size_ascii,
        size_utf8
    );
}

#[test]
fn test_bidi_bom_and_arabic_indic_digits_not_strong_rtl() {
    // U+FEFF (BOM) must NOT be classified as strong RTL
    assert!(!BidiClassifier::is_rtl_char('\u{FEFF}'));

    // Arabic-Indic digits (0x0660..=0x0669) and Extended Arabic-Indic digits (0x06F0..=0x06F9)
    for code in 0x0660..=0x0669 {
        let ch = char::from_u32(code).unwrap();
        assert!(
            !BidiClassifier::is_rtl_char(ch),
            "Arabic-Indic digit U+{code:04X} must not be strong RTL"
        );
    }
    for code in 0x06F0..=0x06F9 {
        let ch = char::from_u32(code).unwrap();
        assert!(
            !BidiClassifier::is_rtl_char(ch),
            "Extended Arabic-Indic digit U+{code:04X} must not be strong RTL"
        );
    }

    // Actual Arabic and Hebrew letters remain strong RTL
    assert!(BidiClassifier::is_rtl_char('م'));
    assert!(BidiClassifier::is_rtl_char('ש'));

    // A string prefixed with BOM followed by Latin text must detect LTR
    assert_eq!(
        BidiClassifier::detect_base_direction("\u{FEFF}Hello"),
        TextDirection::LeftToRight
    );
}

#[test]
fn test_rasterizer_guards_non_positive_size_and_bounds_cache() {
    let candidates = [
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];

    for path in &candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let rasterizer = FontRasterizer::from_bytes(&bytes).unwrap();

            // Non-positive and non-finite sizes must be clamped to 1.0 without panicking
            let g_zero = rasterizer.rasterize('A', 0.0);
            let g_neg = rasterizer.rasterize('A', -12.0);
            let g_nan = rasterizer.rasterize('A', f32::NAN);
            assert!(g_zero.metrics.advance_width.is_finite());
            assert!(g_neg.metrics.advance_width.is_finite());
            assert!(g_nan.metrics.advance_width.is_finite());

            let m_zero = rasterizer.measure_text("Test", 0.0);
            assert!(m_zero.width.is_finite() && m_zero.height > 0.0);

            // Cache size must remain bounded at <= 4096 entries
            for i in 0..4200u32 {
                let sz = 10.0 + (i as f32) * 0.1;
                let _ = rasterizer.rasterize('A', sz);
            }
            assert!(rasterizer.cache_len() <= 4096);
            break;
        }
    }
}

#[test]
fn test_glyph_streamer_hit_test() {
    let streamer = GlyphStreamer::new();
    let run = ShapedTextRun {
        direction: TextDirection::LeftToRight,
        total_width: 20.0,
        font_size: 16.0,
        glyphs: vec![
            PositionedGlyph {
                glyph_id: 1,
                cluster_index: 0,
                advance: 10.0,
                x_offset: 0.0,
                y_offset: 0.0,
            },
            PositionedGlyph {
                glyph_id: 2,
                cluster_index: 1,
                advance: 10.0,
                x_offset: 0.0,
                y_offset: 0.0,
            },
        ],
    };

    let hit_start = streamer.hit_test_x(&run, 2.0);
    assert_eq!(hit_start.cluster_index, 0);
    assert!(!hit_start.is_trailing);

    let hit_mid = streamer.hit_test_x(&run, 17.0);
    assert_eq!(hit_mid.cluster_index, 1);
    assert!(hit_mid.is_trailing);
}
