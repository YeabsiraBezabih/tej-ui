use tej_ui_hal::Color;
use tej_ui_view::{
    CaretStyle, FillRule, FontVariantNumeric, StrokeLineCap, StrokeLineJoin, SvgPaintStyle,
    TextUnderlineStyle, TextWrap, VectorEffect,
};

#[test]
fn test_svg_paint_style_constructors() {
    let brand = Color::from_rgb_u32(0x007F_56D9);

    let solid = SvgPaintStyle::solid_fill(brand);
    assert_eq!(solid.fill, Some(brand));
    assert_eq!(solid.fill_opacity, 1.0);
    assert_eq!(solid.fill_rule, FillRule::NonZero);
    assert_eq!(solid.stroke, None);

    let stroke = SvgPaintStyle::stroked(brand, 2.0);
    assert_eq!(stroke.fill, None);
    assert_eq!(stroke.stroke, Some(brand));
    assert_eq!(stroke.stroke_width, 2.0);
    assert_eq!(stroke.stroke_linecap, StrokeLineCap::Round);
    assert_eq!(stroke.stroke_linejoin, StrokeLineJoin::Round);
    assert_eq!(stroke.vector_effect, VectorEffect::NonScalingStroke);
}

#[test]
fn test_typography_extended_tokens() {
    assert_eq!(TextWrap::default(), TextWrap::Wrap);
    assert_eq!(FontVariantNumeric::default(), FontVariantNumeric::Normal);

    let caret = CaretStyle::default();
    assert_eq!(caret.width, 2.0);

    let underline = TextUnderlineStyle::new(2.0, 1.5, Some(Color::from_rgb_u32(0x003B_82F6)));
    assert_eq!(underline.offset, 2.0);
    assert_eq!(underline.thickness, 1.5);
    assert_eq!(underline.color, Some(Color::from_rgb_u32(0x003B_82F6)));
}
