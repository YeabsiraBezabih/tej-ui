use tej_ui_text::{FontFamilyKind, FontRegistry, FontSlant, SystemFont, TextMeasurer};

#[test]
fn test_horizontal_line_metrics_positive_ascent_and_sensible_line_height() {
    let registry = FontRegistry::global();
    if let Some((_, rast)) = registry.rasterizer(FontFamilyKind::SansSerif, 400, FontSlant::Normal)
    {
        let size_px = 16.0;
        let (ascent, descent, new_line_size) = rast.horizontal_line_metrics(size_px);

        assert!(
            ascent.is_finite() && ascent > 0.0,
            "expected positive ascent, got {ascent}"
        );
        assert!(
            descent.is_finite() && descent <= 0.0,
            "expected non-positive descent, got {descent}"
        );
        assert!(
            new_line_size.is_finite() && new_line_size >= ascent,
            "expected sensible line height >= ascent ({ascent}), got {new_line_size}"
        );
        assert!(
            new_line_size >= size_px * 0.9 && new_line_size <= size_px * 2.5,
            "expected line height within sensible bounds for {size_px}px, got {new_line_size}"
        );

        // Non-positive and NaN sizes must sanitize cleanly without panicking
        let (a_zero, _d_zero, h_zero) = rast.horizontal_line_metrics(0.0);
        let (a_nan, _d_nan, h_nan) = rast.horizontal_line_metrics(f32::NAN);
        assert!(a_zero.is_finite() && a_zero > 0.0 && h_zero.is_finite() && h_zero > 0.0);
        assert!(a_nan.is_finite() && a_nan > 0.0 && h_nan.is_finite() && h_nan > 0.0);
    }
}

#[test]
fn test_font_registry_global_resolves_faces_and_weight_buckets() {
    assert_eq!(FontRegistry::weight_bucket(300), 400);
    assert_eq!(FontRegistry::weight_bucket(400), 400);
    assert_eq!(FontRegistry::weight_bucket(500), 600);
    assert_eq!(FontRegistry::weight_bucket(600), 600);
    assert_eq!(FontRegistry::weight_bucket(650), 700);
    assert_eq!(FontRegistry::weight_bucket(700), 700);
    assert_eq!(FontRegistry::weight_bucket(900), 700);

    let registry = FontRegistry::global();

    let regular_a = registry.rasterizer(FontFamilyKind::SansSerif, 400, FontSlant::Normal);
    let regular_b = registry.rasterizer(FontFamilyKind::SansSerif, 350, FontSlant::Normal);
    let bold = registry.rasterizer(FontFamilyKind::SansSerif, 700, FontSlant::Normal);
    let italic = registry.rasterizer(FontFamilyKind::SansSerif, 400, FontSlant::Italic);
    let mono = registry.rasterizer(FontFamilyKind::Monospace, 400, FontSlant::Normal);

    if let (Some((id_a, rast_a)), Some((id_b, rast_b))) = (regular_a, regular_b) {
        // Weight 350 and 400 snap to the same bucket (400) -> identical font_id & Arc
        assert_eq!(id_a, id_b);
        assert!(std::sync::Arc::ptr_eq(&rast_a, &rast_b));
        assert!(id_a > 0);

        if let Some((id_bold, _)) = bold {
            assert_ne!(id_a, id_bold, "distinct weight bucket gets unique font_id");
        }
        if let Some((id_italic, _)) = italic {
            assert_ne!(id_a, id_italic, "italic face gets unique font_id");
        }
        if let Some((id_mono, _)) = mono {
            assert_ne!(id_a, id_mono, "monospace face gets unique font_id");
        }
    }
}

#[test]
fn test_measure_styled_matches_font_rasterizer_when_system_font_present() {
    let registry = FontRegistry::global();
    let measurer = TextMeasurer::new();
    let sample = "Hello, tej-ui typography 0123456789!";

    for (family_kind, weight) in [
        (FontFamilyKind::SansSerif, 400),
        (FontFamilyKind::SansSerif, 700),
        (FontFamilyKind::Monospace, 400),
    ] {
        let measured = measurer.measure_styled(sample, 16.0, family_kind, weight);
        assert!(measured.width > 0.0);
        assert!(measured.height > 0.0);

        // Repeat call hits content-hashed cache and returns identical size
        let cached = measurer.measure_styled(sample, 16.0, family_kind, weight);
        assert_eq!(measured, cached);

        if let Some((_, rast)) = registry.rasterizer(family_kind, weight, FontSlant::Normal) {
            let direct = rast.measure_text(sample, 16.0);
            assert!(
                (measured.width - direct.width).abs() < 1e-4,
                "width mismatch for family={family_kind:?}, weight={weight}: measured={measured:?}, direct={direct:?}"
            );
            assert!(
                (measured.height - direct.height).abs() < 1e-4,
                "height mismatch for family={family_kind:?}, weight={weight}: measured={measured:?}, direct={direct:?}"
            );
        }
    }

    // measure_slice also agrees with measure_styled for Body (proportional, 400) and Monospace (400)
    assert_eq!(
        measurer.measure_slice(sample, SystemFont::Body, 14.0),
        measurer.measure_styled(sample, 14.0, FontFamilyKind::SansSerif, 400)
    );
    assert_eq!(
        measurer.measure_slice(sample, SystemFont::Monospace, 14.0),
        measurer.measure_styled(sample, 14.0, FontFamilyKind::Monospace, 400)
    );
}

#[test]
fn test_empty_registry_uses_calibrated_fallback_ratios() {
    let empty = FontRegistry::new_empty();
    assert!(
        empty
            .rasterizer(FontFamilyKind::SansSerif, 400, FontSlant::Normal)
            .is_none()
    );

    let prop = empty.measure_text("abcd", 10.0, FontFamilyKind::SansSerif, 400);
    let mono = empty.measure_text("abcd", 10.0, FontFamilyKind::Monospace, 400);

    assert!((prop.width - 4.0 * 10.0 * 0.58).abs() < 1e-4);
    assert!((prop.height - 10.0 * 1.35).abs() < 1e-4);
    assert!((mono.width - 4.0 * 10.0 * 0.62).abs() < 1e-4);
    assert!((mono.height - 10.0 * 1.35).abs() < 1e-4);
}
