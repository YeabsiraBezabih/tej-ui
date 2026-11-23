use tej_ui_view::TOTAL_CSS_PROPERTIES;
use tej_ui_view::css::*;

#[test]
fn test_total_css_properties_count() {
    assert_eq!(TOTAL_CSS_PROPERTIES, 231);
    assert_eq!(CssStyleSheet::property_count(), 231);
}

#[test]
fn test_css_stylesheet_instantiation_and_builders() {
    let sheet = CssStyleSheet::new()
        .with_width(320.0)
        .with_height(240.0)
        .with_padding(16.0)
        .with_font_size(14.0);

    assert_eq!(sheet.box_sizing.width, LengthPercentage::Points(320.0));
    assert_eq!(sheet.box_sizing.height, LengthPercentage::Points(240.0));
    assert_eq!(sheet.spacing.padding_top, LengthPercentage::Points(16.0));
    assert_eq!(sheet.typography.font_size, 14.0);
}

#[test]
fn test_all_modules_initialization() {
    let sheet = CssStyleSheet::default();

    // Box Sizing & Sizing Props
    assert_eq!(sheet.box_sizing.box_sizing, BoxSizing::BorderBox);
    assert_eq!(sheet.box_sizing.aspect_ratio, None);

    // Spacing Props
    assert_eq!(sheet.spacing.position, PositionScheme::Static);
    assert_eq!(sheet.spacing.z_index, None);

    // Flex & Grid Props
    assert_eq!(sheet.flex_grid.display, Display::Flex);
    assert_eq!(sheet.flex_grid.grid_template_columns.len(), 0);

    // Typography Props
    assert_eq!(sheet.typography.text_align, TextAlign::Start);
    assert_eq!(sheet.typography.font_style, FontStyle::Normal);

    // Borders & Effects Props
    assert_eq!(sheet.borders_effects.visibility, Visibility::Visible);
    assert_eq!(sheet.borders_effects.mix_blend_mode, BlendMode::Normal);

    // Backgrounds Props
    assert_eq!(sheet.backgrounds.background_size, BackgroundSize::Auto);
    assert_eq!(sheet.backgrounds.background_clip, BoxClip::BorderBox);

    // Motion Props
    assert_eq!(sheet.motion.transform_style, TransformStyle::Flat);
    assert_eq!(sheet.motion.animation_direction, AnimationDirection::Normal);

    // Interaction Props
    assert_eq!(sheet.interaction.overflow_x, Overflow::Visible);
    assert_eq!(sheet.interaction.scroll_behavior, ScrollBehavior::Auto);

    // Modern Props
    assert_eq!(sheet.modern.object_fit, ObjectFit::Fill);
    assert_eq!(sheet.modern.table_layout, TableLayout::Auto);
}
