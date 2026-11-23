use tej_ui_text::{
    BidiClassifier, FontCategory, FontFallbackChain, FontRasterizer, SystemFont, TextDirection,
    TextMeasurer, TextShaper,
};

#[test]
fn test_zero_copy_measurement() {
    let measurer = TextMeasurer::new();
    let text = "Performance Benchmark Text";

    let size1 = measurer.measure_slice(text, SystemFont::Body, 14.0);
    assert!(size1.width > 0.0);
    assert!(size1.height > 0.0);

    // Cache hit should return identical measurement
    let size2 = measurer.measure_slice(text, SystemFont::Body, 14.0);
    assert_eq!(size1, size2);
}

#[test]
fn test_monospace_font_scaling() {
    let measurer = TextMeasurer::new();
    let code = "fn main() {}";

    let mono_size = measurer.measure_slice(code, SystemFont::Monospace, 12.0);
    let body_size = measurer.measure_slice(code, SystemFont::Body, 12.0);

    assert!(mono_size.width > 0.0);
    assert!(body_size.width > 0.0);
}

#[test]
fn test_bidi_segmentation_mixed_script() {
    let classifier = BidiClassifier::new();

    // Mixed LTR (English) and RTL (Arabic/Hebrew)
    let mixed = "Hello مرحبا World";
    let runs = classifier.segment(mixed);

    assert_eq!(runs.len(), 3);
    assert_eq!(runs[0].text, "Hello ");
    assert_eq!(runs[0].direction, TextDirection::LeftToRight);
    assert_eq!(runs[1].text, "مرحبا ");
    assert_eq!(runs[1].direction, TextDirection::RightToLeft);
    assert_eq!(runs[2].text, "World");
    assert_eq!(runs[2].direction, TextDirection::LeftToRight);

    // Visual ordering for RTL-first context
    let rtl_first = "مرحبا Hello";
    let runs_rtl = classifier.segment(rtl_first);
    let visual = classifier.visual_order(runs_rtl);
    assert_eq!(visual[0].direction, TextDirection::LeftToRight);
    assert_eq!(visual[1].direction, TextDirection::RightToLeft);
}

#[test]
fn test_shaper_ligatures_and_cjk() {
    let shaper = TextShaper::new();
    let font = SystemFont::system_ui();

    // Programming ligatures
    let code = "let res = a -> b == c;";
    let shaped = shaper.shape(code, font, 16.0, TextDirection::LeftToRight);

    // Ensure ligature clusters were formed
    let ligatures: Vec<_> = shaped.glyphs.iter().filter(|g| g.is_ligature).collect();
    assert_eq!(ligatures.len(), 2); // "->" and "=="
    assert_eq!(ligatures[0].char_count, 2);
    assert_eq!(ligatures[1].char_count, 2);

    // CJK wide characters
    let cjk = "こんにちは"; // 5 full-width Japanese hiragana chars
    let shaped_cjk = shaper.shape(cjk, font, 16.0, TextDirection::LeftToRight);
    assert_eq!(shaped_cjk.glyphs.len(), 5);
    for g in shaped_cjk.glyphs {
        assert_eq!(g.advance, 16.0 * 1.0); // Full width advance
    }
}

#[test]
fn test_fallback_chain_and_emoji() {
    let fallback = FontFallbackChain::default();

    // Alphanumeric uses primary font
    assert_eq!(
        FontFallbackChain::categorize_char('A'),
        FontCategory::Primary
    );
    assert_eq!(fallback.font_for_char('A'), &fallback.primary);

    // CJK uses CJK fallback font
    assert_eq!(FontFallbackChain::categorize_char('漢'), FontCategory::Cjk);
    assert_eq!(fallback.font_for_char('漢'), &fallback.cjk_fallback);

    // Arabic uses RTL fallback font
    assert_eq!(
        FontFallbackChain::categorize_char('م'),
        FontCategory::ComplexScript
    );
    assert_eq!(fallback.font_for_char('م'), &fallback.rtl_fallback);

    // Emoji uses Emoji fallback font
    assert_eq!(
        FontFallbackChain::categorize_char('🚀'),
        FontCategory::Emoji
    );
    assert_eq!(fallback.font_for_char('🚀'), &fallback.emoji_fallback);
}

#[test]
fn test_font_rasterizer_system() {
    let candidates = [
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
    ];
    for path in &candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let rasterizer = FontRasterizer::from_bytes(&bytes).unwrap();
            let glyph = rasterizer.rasterize('A', 16.0);
            assert!(glyph.metrics.advance_width > 0.0);
            let size = rasterizer.measure_text("Hello tej-ui", 14.0);
            assert!(size.width > 0.0);
            assert!(size.height > 0.0);
            break;
        }
    }
}

#[test]
fn test_complex_script_and_cursive_joining() {
    use tej_ui_text::{
        ArabicJoiningForm, CursiveScriptJoiner, EmojiPresentation, OpenTypeFeatures,
    };

    // 1. OpenType features flags
    let features = OpenTypeFeatures::default();
    assert!(features.standard_ligatures);
    assert!(features.contextual_alternates);
    assert!(features.kerning);

    // 2. Arabic cursive joining analysis
    let beh = 'ب'; // Arabic Beh joins on both sides
    let alef = 'ا'; // Alef only joins from the right, not left

    assert!(CursiveScriptJoiner::is_arabic(beh));
    assert!(CursiveScriptJoiner::is_arabic(alef));
    assert!(!CursiveScriptJoiner::is_arabic('Z'));

    // Isolated (no neighbors)
    assert_eq!(
        CursiveScriptJoiner::resolve_form(None, beh, None),
        ArabicJoiningForm::Isolated
    );

    // Initial (followed by another Arabic letter)
    assert_eq!(
        CursiveScriptJoiner::resolve_form(None, beh, Some(alef)),
        ArabicJoiningForm::Initial
    );

    // Medial (between two joinable letters)
    assert_eq!(
        CursiveScriptJoiner::resolve_form(Some(beh), beh, Some(beh)),
        ArabicJoiningForm::Medial
    );

    // Final (preceded by joinable letter, end of word)
    assert_eq!(
        CursiveScriptJoiner::resolve_form(Some(beh), alef, None),
        ArabicJoiningForm::Final
    );

    // 3. Emoji presentation variation selector
    assert_eq!(
        EmojiPresentation::detect('🎉', None),
        EmojiPresentation::ColorEmoji
    );
    assert_eq!(
        EmojiPresentation::detect('#', Some('\u{FE0F}')),
        EmojiPresentation::ColorEmoji
    );
    assert_eq!(
        EmojiPresentation::detect('A', None),
        EmojiPresentation::Text
    );
}
