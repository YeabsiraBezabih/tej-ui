use tej_ui_core::Signal;
use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{
    AlignItems, Dimension, DisplayMode, FlexDirection, FlexWrap, Insets, JustifyContent,
    LayoutArena, LayoutEngine, Overflow, Point, Rect, Size,
};
use tej_ui_view::css::{CssStyleSheet, Display, LengthPercentage, Overflow as CssOverflow};
use tej_ui_view::{
    Button, ButtonVariant, Checkbox, ColorTokens, For, HStack, If, Input, PaintContext, StyleProps,
    Text, Theme, VStack, View, ViewExt,
};

#[derive(Debug, Default)]
struct SpyCompositor {
    rects: Vec<(Rect, Color)>,
    rounded_rects: Vec<(Rect, f32, Color)>,
    clips_pushed: Vec<(Rect, [f32; 4])>,
    clips_popped: usize,
    styled_texts: Vec<(String, Point, f32, u16, bool, Color)>,
    plain_texts: Vec<(String, Point, f32, Color)>,
}

impl SurfaceCompositor for SpyCompositor {
    fn begin_frame(&mut self, _dirty_rects: &[Rect]) {}

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.rects.push((rect, color));
    }

    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: Color) {
        self.rounded_rects.push((rect, radius, color));
    }

    fn draw_line(&mut self, _p1: Point, _p2: Point, _width: f32, _color: Color) {}

    fn draw_text(&mut self, text: &str, origin: Point, font_size: f32, color: Color) {
        self.plain_texts
            .push((text.to_string(), origin, font_size, color));
    }

    fn draw_text_styled(
        &mut self,
        text: &str,
        origin: Point,
        font_size: f32,
        weight: u16,
        family: tej_ui_hal::TextFamily,
        color: Color,
    ) {
        let monospace = matches!(family, tej_ui_hal::TextFamily::Monospace);
        self.styled_texts.push((
            text.to_string(),
            origin,
            font_size,
            weight,
            monospace,
            color,
        ));
    }

    fn push_clip(&mut self, rect: Rect, corner_radii: [f32; 4]) {
        self.clips_pushed.push((rect, corner_radii));
    }

    fn pop_clip(&mut self) {
        self.clips_popped += 1;
    }

    fn end_frame(&mut self) {}
}

#[test]
fn test_paint_context_foreground_stack_clip_and_styled_text() {
    let arena = LayoutArena::new();
    let theme = Theme::dark();
    let mut comp = SpyCompositor::default();
    let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &[]).with_scroll(10.0, 0.0);

    // Default foreground falls back to theme.colors.foreground
    assert_eq!(ctx.current_foreground(), theme.colors.foreground);
    let custom1 = Color::rgb(255, 0, 0);
    let custom2 = Color::rgb(0, 255, 0);
    ctx.push_foreground(custom1);
    assert_eq!(ctx.current_foreground(), custom1);
    ctx.push_foreground(custom2);
    assert_eq!(ctx.current_foreground(), custom2);
    ctx.pop_foreground();
    assert_eq!(ctx.current_foreground(), custom1);
    ctx.pop_foreground();
    assert_eq!(ctx.current_foreground(), theme.colors.foreground);

    // Clip forwarding applies scroll offset
    ctx.push_clip(Rect::new(5.0, 20.0, 100.0, 40.0), [4.0; 4]);
    ctx.draw_text_styled(
        "Hello",
        Point::new(8.0, 25.0),
        14.0,
        700,
        tej_ui_hal::TextFamily::Monospace,
        custom1,
    );
    ctx.pop_clip();

    assert_eq!(comp.clips_pushed.len(), 1);
    assert_eq!(comp.clips_pushed[0].0, Rect::new(5.0, 10.0, 100.0, 40.0));
    assert_eq!(comp.clips_pushed[0].1, [4.0; 4]);
    assert_eq!(comp.clips_popped, 1);

    assert_eq!(comp.styled_texts.len(), 1);
    let (txt, pt, fs, weight, mono, col) = &comp.styled_texts[0];
    assert_eq!(txt, "Hello");
    assert_eq!(*pt, Point::new(8.0, 15.0));
    assert_eq!(*fs, 14.0);
    assert_eq!(*weight, 700);
    assert!(*mono);
    assert_eq!(*col, custom1);
}

#[test]
fn test_color_tokens_new_semantic_fields_across_palettes() {
    for tokens in [
        ColorTokens::dark(),
        ColorTokens::light(),
        ColorTokens::high_contrast(),
        ColorTokens::tej_dark(),
        ColorTokens::tej_light(),
    ] {
        assert_eq!(tokens.scrim, Color::rgba(0, 0, 0, 140));
        assert_ne!(tokens.primary_hover, Color::TRANSPARENT);
        assert_ne!(tokens.destructive_hover, Color::TRANSPARENT);
        assert_ne!(tokens.input_bg, Color::TRANSPARENT);
        assert_ne!(tokens.input_border, Color::TRANSPARENT);
        assert_ne!(tokens.success, Color::TRANSPARENT);
        assert_ne!(tokens.success_foreground, Color::TRANSPARENT);
        assert_ne!(tokens.warning, Color::TRANSPARENT);
        assert_ne!(tokens.warning_foreground, Color::TRANSPARENT);
        assert_ne!(tokens.info, Color::TRANSPARENT);
        assert_ne!(tokens.info_foreground, Color::TRANSPARENT);
    }
}

#[test]
fn test_styled_layout_no_child_flex_grow_mutation_and_overflow_clipping() {
    let view = HStack::new(Text::new("Item"))
        .border(2.0, Color::WHITE)
        .corner_radius(6.0)
        .flex_shrink(0.0)
        .min_width(Dimension::Points(50.0))
        .max_width(Dimension::Points(200.0))
        .min_height(Dimension::Points(30.0))
        .max_height(Dimension::Points(100.0))
        .overflow(Overflow::Hidden);

    let mut state = view.build();
    let mut arena = LayoutArena::new();
    let root = view.layout(&mut state, &mut arena);

    let root_style = arena.styles[root as usize];
    assert_eq!(root_style.direction, FlexDirection::Column);
    assert_eq!(root_style.border, Insets::all(2.0));
    assert_eq!(root_style.flex_shrink, 0.0);
    assert_eq!(root_style.min_width, Dimension::Points(50.0));
    assert_eq!(root_style.max_width, Dimension::Points(200.0));
    assert_eq!(root_style.min_height, Dimension::Points(30.0));
    assert_eq!(root_style.max_height, Dimension::Points(100.0));
    assert_eq!(root_style.overflow_x, Overflow::Hidden);
    assert_eq!(root_style.overflow_y, Overflow::Hidden);

    // Child HStack has width == Auto and flex_grow == 0.0; Styled must NOT mutate it to 1.0
    let child_node = arena.children[root as usize][0];
    assert_eq!(arena.styles[child_node as usize].flex_grow, 0.0);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 200.0));
    let mut comp = SpyCompositor::default();
    let theme = Theme::dark();
    let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &[]);
    view.paint(&state, &mut ctx);

    assert_eq!(comp.clips_pushed.len(), 1);
    assert_eq!(comp.clips_pushed[0].1, [6.0; 4]);
    assert_eq!(comp.clips_popped, 1);
}

#[test]
fn test_style_props_apply_css_per_edge_and_percent_and_overflow() {
    let mut sheet = CssStyleSheet::default();
    sheet.box_sizing.width = LengthPercentage::Percent(0.5);
    sheet.box_sizing.height = LengthPercentage::Points(120.0);
    sheet.box_sizing.min_width = LengthPercentage::Points(80.0);
    sheet.box_sizing.max_width = LengthPercentage::Percent(1.0);
    sheet.spacing.padding_left = LengthPercentage::Points(16.0);
    sheet.spacing.margin_top = LengthPercentage::Points(8.0);
    sheet.flex_grid.display = Display::Grid;
    sheet.flex_grid.flex_direction = FlexDirection::Column;
    sheet.flex_grid.flex_wrap = FlexWrap::Wrap;
    sheet.interaction.overflow_x = CssOverflow::Hidden;
    sheet.interaction.overflow_y = CssOverflow::Scroll;

    let mut props = StyleProps::default();
    props.apply_css(&sheet);

    assert_eq!(props.width, Dimension::Percent(0.5));
    assert_eq!(props.height, Dimension::Points(120.0));
    assert_eq!(props.min_width, Dimension::Points(80.0));
    assert_eq!(props.max_width, Dimension::Percent(1.0));
    assert_eq!(props.padding, Insets::new(0.0, 0.0, 0.0, 16.0));
    assert_eq!(props.margin, Insets::new(8.0, 0.0, 0.0, 0.0));
    assert_eq!(props.display, Some(DisplayMode::Grid));
    assert_eq!(props.flex_direction, Some(FlexDirection::Column));
    assert_eq!(props.flex_wrap, Some(FlexWrap::Wrap));
    assert_eq!(props.overflow_x, Overflow::Hidden);
    assert_eq!(props.overflow_y, Overflow::Scroll);
}

#[test]
fn test_transparent_control_flow_and_stack_builders() {
    let mut arena = LayoutArena::new();

    // Direct layout of () and Option::None uses DisplayMode::None
    let unit_node = ().layout(&mut (), &mut arena);
    assert_eq!(arena.styles[unit_node as usize].display, DisplayMode::None);
    let none_view: Option<Text> = None;
    let mut none_state = none_view.build();
    let none_node = none_view.layout(&mut none_state, &mut arena);
    assert_eq!(arena.styles[none_node as usize].display, DisplayMode::None);

    // If with hidden () branch attaches 0 nodes to parent VStack
    let cond = Signal::new(false);
    let vstack = VStack::new((
        Text::new("First"),
        If::new(cond, (Text::new("A"), Text::new("B")), ()),
        none_view,
        Text::new("Last"),
    ))
    .align_items(AlignItems::Center)
    .justify_content(JustifyContent::SpaceBetween)
    .flex_shrink(0.0)
    .min_width(Dimension::Points(100.0))
    .max_width(Dimension::Points(400.0));

    let mut state = vstack.build();
    let root = vstack.layout(&mut state, &mut arena);
    assert_eq!(arena.children[root as usize].len(), 2);
    assert_eq!(arena.styles[root as usize].align_items, AlignItems::Center);
    assert_eq!(
        arena.styles[root as usize].justify_content,
        JustifyContent::SpaceBetween
    );
    assert_eq!(arena.styles[root as usize].flex_shrink, 0.0);
    assert_eq!(
        arena.styles[root as usize].min_width,
        Dimension::Points(100.0)
    );
    assert_eq!(
        arena.styles[root as usize].max_width,
        Dimension::Points(400.0)
    );

    // When condition becomes true, tuple branch attaches both items directly to parent VStack
    cond.set(true);
    vstack.rebuild(&mut state);
    let root_true = vstack.layout(&mut state, &mut arena);
    assert_eq!(arena.children[root_true as usize].len(), 4);

    // For attaches items directly into parent VStack without an intermediate Row wrapper
    let for_stack = VStack::new(For::new(
        vec![1, 2, 3],
        |x| *x,
        |x| Text::new(format!("Item {x}")),
    ));
    let mut for_state = for_stack.build();
    let for_root = for_stack.layout(&mut for_state, &mut arena);
    assert_eq!(arena.children[for_root as usize].len(), 3);
}

#[test]
fn test_button_foreground_inheritance_centering_clip_and_builders() {
    let btn = Button::new(Text::new("Save").bold()).full_width();
    let mut state = btn.build();
    let mut arena = LayoutArena::new();
    let root = btn.layout(&mut state, &mut arena);

    let style = arena.styles[root as usize];
    assert_eq!(style.align_items, AlignItems::Center);
    assert_eq!(style.justify_content, JustifyContent::Center);
    assert_eq!(style.width, Dimension::Fill);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(200.0, 40.0));

    let theme = Theme::light();
    let mut comp = SpyCompositor::default();
    let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &[]).with_interaction(
        Some(root),
        Some(root),
        None,
    );
    btn.paint(&state, &mut ctx);

    // Focus ring drawn first, then primary_hover fill
    assert_eq!(comp.rounded_rects.len(), 2);
    assert_eq!(comp.rounded_rects[0].2, theme.colors.ring);
    assert_eq!(comp.rounded_rects[1].2, theme.colors.primary_hover);
    // Child Text inherited primary_foreground (white) and bold weight (700)
    assert_eq!(comp.styled_texts.len(), 1);
    assert_eq!(comp.styled_texts[0].3, 700);
    assert_eq!(comp.styled_texts[0].5, theme.colors.primary_foreground);
    // Clip pushed & popped around child
    assert_eq!(comp.clips_pushed.len(), 1);
    assert_eq!(comp.clips_popped, 1);

    // Outline button draws focus ring BEFORE border + inner fill
    let outline_btn = Button::new(Text::new("Cancel")).outline();
    let mut o_state = outline_btn.build();
    let o_root = outline_btn.layout(&mut o_state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, o_root, Size::new(200.0, 40.0));
    let mut o_comp = SpyCompositor::default();
    let mut o_ctx = PaintContext::new(&mut o_comp, &arena, &theme, &[]).with_interaction(
        Some(o_root),
        None,
        None,
    );
    outline_btn.paint(&o_state, &mut o_ctx);
    assert_eq!(o_comp.rounded_rects.len(), 3);
    assert_eq!(o_comp.rounded_rects[0].2, theme.colors.ring);
    assert_eq!(o_comp.rounded_rects[1].2, theme.colors.border);
    assert_eq!(o_comp.rounded_rects[2].2, theme.colors.card);
    assert_eq!(o_comp.styled_texts[0].5, theme.colors.foreground);

    // Convenience builders check
    let _g = Button::new(Text::new("G")).ghost();
    let _s = Button::new(Text::new("S")).secondary();
    let _d = Button::new(Text::new("D")).variant(ButtonVariant::Destructive);
}

#[test]
fn test_input_border_bg_clip_caret_and_sizing_builders() {
    let sig = Signal::new(String::from("hello"));
    let input = Input::new(sig)
        .width(Dimension::Points(300.0))
        .flex_grow(1.0);
    let mut state = input.build();
    let mut arena = LayoutArena::new();
    let root = input.layout(&mut state, &mut arena);

    assert_eq!(arena.styles[root as usize].width, Dimension::Points(300.0));
    assert_eq!(arena.styles[root as usize].flex_grow, 1.0);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 36.0));

    let theme = Theme::light();

    // Unfocused: input_border + input_bg, no caret, clipped text
    let mut comp = SpyCompositor::default();
    let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &[]);
    input.paint(&state, &mut ctx);
    assert_eq!(comp.rounded_rects.len(), 2);
    assert_eq!(comp.rounded_rects[0].2, theme.colors.input_border);
    assert_eq!(comp.rounded_rects[1].2, theme.colors.input_bg);
    assert_eq!(comp.clips_pushed.len(), 1);
    assert_eq!(comp.clips_popped, 1);
    assert!(comp.rects.is_empty(), "No caret when unfocused");
    assert_eq!(comp.plain_texts.len(), 1);
    assert_eq!(comp.plain_texts[0].1.x, 12.0);
    let expected_y = (36.0 - theme.typography.body_small.font_size) * 0.5;
    assert!((comp.plain_texts[0].1.y - expected_y).abs() < 1e-4);

    // Focused: ring border + input_bg + 1.5px caret
    let mut f_comp = SpyCompositor::default();
    let mut f_ctx = PaintContext::new(&mut f_comp, &arena, &theme, &[]).with_interaction(
        Some(root),
        None,
        None,
    );
    input.paint(&state, &mut f_ctx);
    assert_eq!(f_comp.rounded_rects[0].2, theme.colors.ring);
    assert_eq!(f_comp.rects.len(), 1);
    assert_eq!(f_comp.rects[0].0.size.width, 1.5);
}

#[test]
fn test_checkbox_unchecked_border_and_focus_ring() {
    let checked = Signal::new(false);
    let cb = Checkbox::new(checked);
    let mut state = cb.build();
    let mut arena = LayoutArena::new();
    let root = cb.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(100.0, 100.0));

    let theme = Theme::light();
    let mut comp = SpyCompositor::default();
    let mut ctx =
        PaintContext::new(&mut comp, &arena, &theme, &[]).with_interaction(Some(root), None, None);
    cb.paint(&state, &mut ctx);

    // Focus ring + 1px input_border + inset input_bg
    assert_eq!(comp.rounded_rects.len(), 3);
    assert_eq!(comp.rounded_rects[0].2, theme.colors.ring);
    assert_eq!(comp.rounded_rects[1].2, theme.colors.input_border);
    assert_eq!(comp.rounded_rects[2].2, theme.colors.input_bg);
}

#[test]
fn test_text_weight_and_monospace_builders() {
    let t = Text::new("code_123").semibold().monospace();
    let mut state = t.build();
    let mut arena = LayoutArena::new();
    let root = t.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(200.0, 50.0));

    let theme = Theme::dark();
    let mut comp = SpyCompositor::default();
    let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &[]);
    t.paint(&state, &mut ctx);

    assert_eq!(comp.styled_texts.len(), 1);
    assert_eq!(comp.styled_texts[0].3, 600);
    assert!(comp.styled_texts[0].4);
    assert_eq!(comp.styled_texts[0].5, theme.colors.foreground);
}
