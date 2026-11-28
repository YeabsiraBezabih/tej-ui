use tej_ui_text::{
    BidiClassifier, TextDirection, TextMeasurer, grapheme_count, next_grapheme_boundary,
    next_word_boundary, pop_grapheme, prev_grapheme_boundary, prev_word_boundary, wrap_text,
};

#[test]
fn test_uax9_bidi_hebrew_arabic_and_mixed_latin_digits() {
    let classifier = BidiClassifier::new();

    // Base direction detection via UAX #9 P2/P3
    assert_eq!(
        BidiClassifier::detect_base_direction("שלום עולם"),
        TextDirection::RightToLeft
    );
    assert_eq!(
        BidiClassifier::detect_base_direction("مرحبا بالعالم"),
        TextDirection::RightToLeft
    );
    assert_eq!(
        BidiClassifier::detect_base_direction("Hello 123 שלום"),
        TextDirection::LeftToRight
    );

    // Mixed Hebrew + European digits in RTL paragraph:
    // UAX #9 resolves Hebrew to level 1 (RTL) and digits to level 2 (LTR).
    let hebrew_digits = "שלום 123 עולם";
    let logical = classifier.analyze(hebrew_digits, TextDirection::Auto);
    assert_eq!(logical.len(), 3);
    assert_eq!(logical[0].direction, TextDirection::RightToLeft);
    assert_eq!(logical[0].level, 1);
    assert_eq!(logical[1].text, "123");
    assert_eq!(logical[1].direction, TextDirection::LeftToRight);
    assert_eq!(logical[1].level, 2);
    assert_eq!(logical[2].direction, TextDirection::RightToLeft);
    assert_eq!(logical[2].level, 1);

    // Visual reordering swaps the two Hebrew words around the LTR digits
    let visual = classifier.visual_runs(hebrew_digits, TextDirection::Auto);
    assert_eq!(visual.len(), 3);
    assert!(visual[0].text.contains("עולם"));
    assert_eq!(visual[1].text, "123");
    assert_eq!(visual[1].direction, TextDirection::LeftToRight);
    assert!(visual[2].text.contains("שלום"));

    // Mixed Arabic + Latin + digits in LTR paragraph
    let mixed_arabic = "Order مرحبا 2026 done";
    let runs = classifier.analyze(mixed_arabic, TextDirection::LeftToRight);
    assert!(
        runs.iter()
            .any(|r| r.direction == TextDirection::RightToLeft && r.text.contains("مرحبا"))
    );
    assert!(
        runs.iter()
            .any(|r| r.direction == TextDirection::LeftToRight && r.text.contains("2026"))
    );

    // visual_order (Rule L2) agrees with visual_runs on RTL + digits + RTL
    let reordered = classifier.visual_order(logical);
    assert_eq!(reordered[0].text, visual[0].text);
    assert_eq!(reordered[1].text, "123");
    assert_eq!(reordered[2].text, visual[2].text);
}

#[test]
fn test_uax29_zwj_emoji_and_combining_grapheme_operations() {
    // Family ZWJ emoji is 7 Unicode scalar values but 1 extended grapheme cluster
    let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
    assert_eq!(grapheme_count(family), 1);

    // Combining acute accent: "e" + U+0301 = 1 grapheme cluster
    let accented = "cafe\u{0301}";
    assert_eq!(grapheme_count(accented), 4);

    // Popping a ZWJ emoji removes the entire cluster in one Backspace
    let mut text = format!("Hi {family}");
    assert_eq!(grapheme_count(&text), 4);
    let popped = pop_grapheme(&mut text);
    assert_eq!(popped.as_deref(), Some(family));
    assert_eq!(text, "Hi ");

    // Popping an accented combining cluster removes base + combining mark together
    let mut word = String::from(accented);
    let popped_accent = pop_grapheme(&mut word);
    assert_eq!(popped_accent.as_deref(), Some("e\u{0301}"));
    assert_eq!(word, "caf");

    // Grapheme boundary stepping forward and backward across ZWJ emoji
    let sample = format!("A{family}B");
    let b1 = next_grapheme_boundary(&sample, 0);
    assert_eq!(&sample[..b1], "A");
    let b2 = next_grapheme_boundary(&sample, b1);
    assert_eq!(&sample[b1..b2], family);
    let b3 = next_grapheme_boundary(&sample, b2);
    assert_eq!(b3, sample.len());

    assert_eq!(prev_grapheme_boundary(&sample, sample.len()), b2);
    assert_eq!(prev_grapheme_boundary(&sample, b2), b1);
    assert_eq!(prev_grapheme_boundary(&sample, b1), 0);
    assert_eq!(prev_grapheme_boundary(&sample, 0), 0);

    // Word boundary stepping
    let sentence = "hello world  tej";
    let w1 = next_word_boundary(sentence, 0);
    assert_eq!(&sentence[..w1], "hello");
    let w2 = next_word_boundary(sentence, w1);
    assert_eq!(&sentence[..w2], "hello world");
    let w3 = next_word_boundary(sentence, w2);
    assert_eq!(w3, sentence.len());

    let p1 = prev_word_boundary(sentence, sentence.len());
    assert_eq!(&sentence[p1..], "tej");
    let p2 = prev_word_boundary(sentence, p1);
    assert_eq!(&sentence[p2..], "world  tej");
    let p3 = prev_word_boundary(sentence, p2);
    assert_eq!(p3, 0);
}

#[test]
fn test_multiline_word_wrapping() {
    let measurer = TextMeasurer::new();
    let paragraph = "The quick brown fox jumps over the lazy dog";

    // Wrap at a narrow width (~100px at 14px font size)
    let wrapped = wrap_text(paragraph, 100.0, 14.0, &measurer);
    assert!(
        wrapped.lines.len() >= 3,
        "expected at least 3 wrapped lines, got {:?}",
        wrapped.lines
    );
    assert!(wrapped.width <= 100.0 + 0.01);
    assert!(wrapped.line_height > 0.0);
    let line_count = f32::from(u16::try_from(wrapped.lines.len()).unwrap_or(u16::MAX));
    assert!((wrapped.height - line_count * wrapped.line_height).abs() < 0.01);
    // Rejoined words match original text
    assert_eq!(wrapped.lines.join(" "), paragraph);

    // Explicit newlines are preserved
    let with_newlines = "Line one\nLine two\nLine three";
    let wrapped_nl = wrap_text(with_newlines, 500.0, 14.0, &measurer);
    assert_eq!(wrapped_nl.lines, vec!["Line one", "Line two", "Line three"]);

    // Overlong unbreakable word wraps on grapheme boundaries without panicking
    let emoji_word = "👨\u{200D}👩\u{200D}👧\u{200D}👦👨\u{200D}👩\u{200D}👧\u{200D}👦👨\u{200D}👩\u{200D}👧\u{200D}👦";
    let wrapped_emoji = wrap_text(emoji_word, 20.0, 14.0, &measurer);
    assert!(wrapped_emoji.lines.len() >= 2);
    assert_eq!(wrapped_emoji.lines.concat(), emoji_word);
}
