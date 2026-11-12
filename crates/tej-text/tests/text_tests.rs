use tej_text::{SystemFont, TextMeasurer};

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
