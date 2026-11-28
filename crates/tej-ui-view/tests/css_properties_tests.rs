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

#[test]
fn test_all_nine_css_modules_wired_into_style_props_and_paint() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tej_ui_hal::{Color, SurfaceCompositor};
    use tej_ui_layout::{AlignSelf, LayoutArena, LayoutDirection, LayoutEngine, Point, Rect, Size};
    use tej_ui_view::{
        EventDispatcher, PaintContext, StyleProps, Text, Theme, VStack, View, ViewExt,
    };

    #[derive(Debug, Default)]
    struct MockCompositor {
        rects: AtomicUsize,
        lines: AtomicUsize,
        clips: AtomicUsize,
        texts: AtomicUsize,
    }

    impl SurfaceCompositor for MockCompositor {
        fn begin_frame(&mut self, _dirty_rects: &[Rect]) {}
        fn fill_rect(&mut self, _rect: Rect, _color: Color) {
            self.rects.fetch_add(1, Ordering::SeqCst);
        }
        fn fill_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: Color) {
            self.rects.fetch_add(1, Ordering::SeqCst);
        }
        fn draw_line(&mut self, _p1: Point, _p2: Point, _w: f32, _c: Color) {
            self.lines.fetch_add(1, Ordering::SeqCst);
        }
        fn draw_text(&mut self, _t: &str, _o: Point, _s: f32, _c: Color) {
            self.texts.fetch_add(1, Ordering::SeqCst);
        }
        fn push_clip(&mut self, _rect: Rect, _radii: [f32; 4]) {
            self.clips.fetch_add(1, Ordering::SeqCst);
        }
        fn pop_clip(&mut self) {}
        fn end_frame(&mut self) {}
    }

    let mut sheet = CssStyleSheet::new()
        .with_width(200.0)
        .with_height(100.0)
        .with_padding(12.0)
        .with_font_size(18.0);
    sheet.box_sizing.inline_size = LengthPercentage::Points(220.0);
    sheet.box_sizing.aspect_ratio = Some(AspectRatio::RATIO_16_9);
    sheet.spacing.margin_inline_start = LengthPercentage::Points(10.0);
    sheet.spacing.position = PositionScheme::Relative;
    sheet.spacing.left = LengthPercentage::Points(4.0);
    sheet.flex_grid.align_self = Some(tej_ui_layout::AlignItems::Center);
    sheet.flex_grid.flex_basis = LengthPercentage::Points(150.0);
    sheet
        .flex_grid
        .grid_template_columns
        .push(GridTrack::Fr(1.0));
    sheet
        .flex_grid
        .grid_template_columns
        .push(GridTrack::Fr(1.0));
    sheet.backgrounds.background_color = Some(Color::rgb(30, 30, 40));
    sheet.backgrounds.clip_path_inset = Some([2.0, 2.0, 2.0, 2.0]);
    sheet.borders_effects.border_left = BorderEdge {
        width: 2.0,
        style: BorderLineStyle::Solid,
        color: Color::rgb(255, 0, 0),
    };
    sheet.borders_effects.outline_width = 1.0;
    sheet.borders_effects.outline_color = Color::rgb(0, 255, 0);
    sheet.typography.color = ColorValue::Color(Color::rgb(240, 240, 240));
    sheet.typography.font_weight = 700;
    sheet.typography.text_decoration_line = TextDecorationLine::Underline;
    sheet.typography.text_transform = TextTransform::Uppercase;
    sheet.typography.direction = LayoutDirection::Rtl;
    sheet.interaction.pointer_events = PointerEvents::None;
    sheet.motion.transform_translate_x = 6.0;
    sheet.motion.transform_translate_y = 8.0;
    sheet.modern.accent_color = Some(Color::rgb(0, 128, 255));
    sheet.modern.color_scheme = ColorScheme::Dark;

    let mut props = StyleProps::default();
    props.apply_css(&sheet);

    assert_eq!(props.margin.right, 10.0); // RTL inline_start maps to right
    assert_eq!(props.align_self, AlignSelf::Center);
    assert_eq!(props.grid_columns, 2);
    assert_eq!(props.transform_translate_x, 6.0);
    assert_eq!(props.accent_color, Some(Color::rgb(0, 128, 255)));
    assert!(props.text_style.is_some());

    let view = VStack::new(Text::new("hello")).css(&sheet);
    let mut state = view.build();
    let mut arena = LayoutArena::new();
    let root = view.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(400.0, 300.0));

    let mut dispatcher = EventDispatcher::new();
    view.register_events(&state, &mut dispatcher);

    let mut compositor = MockCompositor::default();
    let theme = Theme::dark();
    let mut ctx = PaintContext::new(&mut compositor, &arena, &theme, &[]);
    view.paint(&state, &mut ctx);

    assert!(compositor.rects.load(Ordering::SeqCst) >= 5); // bg + border + 4 outline rects
    assert_eq!(compositor.clips.load(Ordering::SeqCst), 1); // clip_path_inset
    assert_eq!(compositor.lines.load(Ordering::SeqCst), 1); // underline decoration
    assert_eq!(compositor.texts.load(Ordering::SeqCst), 1);
    let _ = Arc::new(0);
}

#[test]
fn test_phase_a_order_grid_justify_and_content_visibility() {
    use tej_ui_hal::Color;
    use tej_ui_layout::{
        Dimension, DisplayMode, FlexDirection, JustifyItems, LayoutArena, LayoutEngine,
        LayoutStyle, Size,
    };

    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(LayoutStyle {
        direction: FlexDirection::Row,
        width: Dimension::Points(200.0),
        height: Dimension::Points(50.0),
        ..Default::default()
    });
    let first_dom = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(80.0),
        height: Dimension::Points(40.0),
        order: 2,
        ..Default::default()
    });
    let second_dom = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(80.0),
        height: Dimension::Points(40.0),
        order: 1,
        ..Default::default()
    });
    arena.add_child(root, first_dom);
    arena.add_child(root, second_dom);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(200.0, 50.0));
    assert!(
        arena.computed_bounds[second_dom as usize].origin.x
            < arena.computed_bounds[first_dom as usize].origin.x
    );

    // Grid justify_items Center centers smaller child inside cell
    let mut g_arena = LayoutArena::new();
    let grid = g_arena.allocate_node(LayoutStyle {
        display: DisplayMode::Grid,
        grid_columns: 1,
        justify_items: JustifyItems::Center,
        width: Dimension::Points(200.0),
        height: Dimension::Points(60.0),
        ..Default::default()
    });
    let item = g_arena.allocate_node(LayoutStyle {
        width: Dimension::Points(100.0),
        height: Dimension::Points(30.0),
        ..Default::default()
    });
    g_arena.add_child(grid, item);
    LayoutEngine::compute_layout(&mut g_arena, grid, Size::new(200.0, 60.0));
    assert_eq!(g_arena.computed_bounds[item as usize].origin.x, 50.0);

    let mut sheet = CssStyleSheet::new();
    sheet.borders_effects.content_visibility_auto = true;
    sheet.borders_effects.border_block_start = BorderEdge {
        width: 3.0,
        style: BorderLineStyle::Solid,
        color: Color::rgb(10, 20, 30),
    };
    sheet.flex_grid.place_content = Some(tej_ui_layout::PlaceContent::CENTER);
    let mut props = tej_ui_view::StyleProps::default();
    props.apply_css(&sheet);
    assert!(props.content_visibility_auto);
    assert_eq!(props.border_width, 3.0);
    assert_eq!(
        props.justify_content,
        Some(tej_ui_layout::JustifyContent::Center)
    );
}
