use tej_ui_layout::{
    AnchorPositionArea, Dimension, GridAutoFlow, JustifyItems, JustifySelf, LayoutDirection,
    LogicalCornerRadii, LogicalInsets, LogicalSize, OverscrollBehavior, PlaceContent, ResizeMode,
    ScrollSnapAlign, ScrollSnapStop, ScrollSnapType, ScrollbarGutter, ScrollbarWidth,
};

#[test]
fn test_logical_insets_ltr_and_rtl() {
    let logical = LogicalInsets {
        block_start: 10.0,
        inline_end: 20.0,
        block_end: 30.0,
        inline_start: 40.0,
    };

    // In LTR: top = block_start (10), right = inline_end (20), bottom = block_end (30), left = inline_start (40)
    let physical_ltr = logical.to_physical(LayoutDirection::Ltr);
    assert_eq!(physical_ltr.top, 10.0);
    assert_eq!(physical_ltr.right, 20.0);
    assert_eq!(physical_ltr.bottom, 30.0);
    assert_eq!(physical_ltr.left, 40.0);

    // In RTL: top = block_start (10), right = inline_start (40), bottom = block_end (30), left = inline_end (20)
    let physical_rtl = logical.to_physical(LayoutDirection::Rtl);
    assert_eq!(physical_rtl.top, 10.0);
    assert_eq!(physical_rtl.right, 40.0);
    assert_eq!(physical_rtl.bottom, 30.0);
    assert_eq!(physical_rtl.left, 20.0);

    // Roundtrip from physical
    let roundtrip = LogicalInsets::from_physical(physical_rtl, LayoutDirection::Rtl);
    assert_eq!(roundtrip, logical);
}

#[test]
fn test_logical_corner_radii() {
    let radii = LogicalCornerRadii {
        start_start: 12.0, // Top-Left in LTR, Top-Right in RTL
        start_end: 0.0,    // Top-Right in LTR, Top-Left in RTL
        end_end: 12.0,
        end_start: 12.0,
    };

    let (tl_ltr, tr_ltr, br_ltr, bl_ltr) = radii.to_physical(LayoutDirection::Ltr);
    assert_eq!(tl_ltr, 12.0);
    assert_eq!(tr_ltr, 0.0);
    assert_eq!(br_ltr, 12.0);
    assert_eq!(bl_ltr, 12.0);

    let (tl_rtl, tr_rtl, br_rtl, bl_rtl) = radii.to_physical(LayoutDirection::Rtl);
    assert_eq!(tl_rtl, 0.0);
    assert_eq!(tr_rtl, 12.0);
    assert_eq!(br_rtl, 12.0);
    assert_eq!(bl_rtl, 12.0);
}

#[test]
fn test_logical_size_resolution() {
    let size = LogicalSize::new(Dimension::Points(300.0), Dimension::Points(100.0));

    // Horizontal text flow: width = inline (300), height = block (100)
    let (w_h, h_h) = size.to_physical(false);
    assert_eq!(w_h, Dimension::Points(300.0));
    assert_eq!(h_h, Dimension::Points(100.0));

    // Vertical text flow: width = block (100), height = inline (300)
    let (w_v, h_v) = size.to_physical(true);
    assert_eq!(w_v, Dimension::Points(100.0));
    assert_eq!(h_v, Dimension::Points(300.0));
}

#[test]
fn test_scroll_snap_and_grid_tokens() {
    assert_eq!(ScrollSnapType::default(), ScrollSnapType::None);
    assert_eq!(ScrollSnapAlign::default(), ScrollSnapAlign::None);
    assert_eq!(ScrollSnapStop::default(), ScrollSnapStop::Normal);
    assert_eq!(OverscrollBehavior::default(), OverscrollBehavior::Auto);
    assert_eq!(ScrollbarWidth::default(), ScrollbarWidth::Auto);
    assert_eq!(ScrollbarGutter::default(), ScrollbarGutter::Auto);

    assert_eq!(JustifyItems::default(), JustifyItems::Stretch);
    assert_eq!(JustifySelf::default(), JustifySelf::Auto);
    assert_eq!(GridAutoFlow::default(), GridAutoFlow::Row);
    assert_eq!(AnchorPositionArea::default(), AnchorPositionArea::Bottom);
    assert_eq!(ResizeMode::default(), ResizeMode::None);

    let place_center = PlaceContent::CENTER;
    assert_eq!(place_center.align, tej_ui_layout::AlignItems::Center);
    assert_eq!(place_center.justify, tej_ui_layout::JustifyContent::Center);
}
