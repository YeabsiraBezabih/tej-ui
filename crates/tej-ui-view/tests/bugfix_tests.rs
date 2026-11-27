use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tej_ui_hal::{Color, MouseButton, SurfaceCompositor, UiEvent};
use tej_ui_layout::{Dimension, LayoutArena, LayoutEngine, Point, Rect, Size};
use tej_ui_view::css::{BordersEffectsProps, CssStyleSheet, MotionProps};
use tej_ui_view::{
    Button, ErrorBoundary, EventDispatcher, Grid, HStack, PaintContext, StyleProps, Text, Theme,
    VStack, View, ViewExt,
};

#[derive(Debug, Default)]
struct RecordingCompositor {
    rects: Vec<(Rect, Color)>,
    rounded_rects: Vec<(Rect, f32, Color)>,
}

impl SurfaceCompositor for RecordingCompositor {
    fn begin_frame(&mut self, _dirty_rects: &[Rect]) {}

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.rects.push((rect, color));
    }

    fn fill_rounded_rect(&mut self, rect: Rect, radius: f32, color: Color) {
        self.rounded_rects.push((rect, radius, color));
    }

    fn draw_line(&mut self, _p1: Point, _p2: Point, _width: f32, _color: Color) {}

    fn draw_text(&mut self, _text: &str, _origin: Point, _font_size: f32, _color: Color) {}

    fn end_frame(&mut self) {}
}

#[test]
fn test_styled_border_outline_does_not_overwrite_interior() {
    let bg = Color::rgb(10, 20, 30);
    let border = Color::rgb(200, 50, 50);

    // 1. Rounded border + background: outer border drawn first, inset interior fill drawn second
    let rounded_view = Text::new("Hi")
        .width(Dimension::Points(100.0))
        .height(Dimension::Points(40.0))
        .background(bg)
        .corner_radius(8.0)
        .border(2.0, border);

    let mut state = rounded_view.build();
    let mut arena = LayoutArena::new();
    let root = rounded_view.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(200.0, 200.0));

    let mut comp = RecordingCompositor::default();
    let theme = Theme::dark();
    let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &[]);
    rounded_view.paint(&state, &mut ctx);

    assert_eq!(comp.rounded_rects.len(), 2);
    // First is outer border at full bounds
    assert_eq!(comp.rounded_rects[0].2, border);
    assert_eq!(comp.rounded_rects[0].0.size, Size::new(100.0, 40.0));
    // Second is inset interior background (not overwritten by border!)
    assert_eq!(comp.rounded_rects[1].2, bg);
    assert_eq!(comp.rounded_rects[1].0.origin, Point::new(2.0, 2.0));
    assert_eq!(comp.rounded_rects[1].0.size, Size::new(96.0, 36.0));

    // 2. Square border without corner_radius: draws background + 4 perimeter edge strips
    let square_view = Text::new("Hi")
        .width(Dimension::Points(100.0))
        .height(Dimension::Points(40.0))
        .background(bg)
        .border(2.0, border);

    let mut state2 = square_view.build();
    let mut arena2 = LayoutArena::new();
    let root2 = square_view.layout(&mut state2, &mut arena2);
    LayoutEngine::compute_layout(&mut arena2, root2, Size::new(200.0, 200.0));

    let mut comp2 = RecordingCompositor::default();
    let mut ctx2 = PaintContext::new(&mut comp2, &arena2, &theme, &[]);
    square_view.paint(&state2, &mut ctx2);

    // 1 background rect + 4 thin border edges = 5 rects, none of the border rects cover the full interior
    assert_eq!(comp2.rects.len(), 5);
    assert_eq!(comp2.rects[0].1, bg);
    for (edge_rect, edge_col) in &comp2.rects[1..] {
        assert_eq!(*edge_col, border);
        assert!(edge_rect.size.width <= 2.0 || edge_rect.size.height <= 2.0);
    }
}

#[test]
fn test_stacks_and_grid_allocate_dedicated_container_nodes() {
    // VStack with single child preserves child's own Text layout style
    let vstack = VStack::new(Text::new("Solo")).padding(12).spacing(8);
    let mut v_state = vstack.build();
    let mut arena = LayoutArena::new();
    let v_node = vstack.layout(&mut v_state, &mut arena);
    let text_node = v_state.child_state.node;

    assert_ne!(v_node, text_node);
    assert_eq!(arena.children[v_node as usize].as_slice(), &[text_node]);
    assert_eq!(arena.styles[text_node as usize].padding.top, 0.0);
    assert_eq!(arena.styles[v_node as usize].padding.top, 12.0);

    // HStack with single child
    let hstack = HStack::new(Text::new("Solo")).padding(6);
    let mut h_state = hstack.build();
    let h_node = hstack.layout(&mut h_state, &mut arena);
    let h_text_node = h_state.child_state.node;
    assert_ne!(h_node, h_text_node);
    assert_eq!(arena.children[h_node as usize].as_slice(), &[h_text_node]);

    // Grid with single child
    let grid = Grid::new(Text::new("Cell")).columns(4).padding(10);
    let mut g_state = grid.build();
    let g_node = grid.layout(&mut g_state, &mut arena);
    let g_text_node = g_state.child_state.node;
    assert_ne!(g_node, g_text_node);
    assert_eq!(arena.children[g_node as usize].as_slice(), &[g_text_node]);
    assert_eq!(arena.styles[g_node as usize].grid_columns, 4);
}

#[test]
fn test_error_boundary_register_events_delegates_to_inner_view() {
    let clicked = Arc::new(AtomicBool::new(false));
    let clicked_c = Arc::clone(&clicked);

    let boundary = ErrorBoundary::new(
        Button::new(Text::new("Tap")).on_click(move || {
            clicked_c.store(true, Ordering::SeqCst);
        }),
        |err| Text::new(format!("Fallback: {err}")),
    );

    let mut state = boundary.build();
    let mut arena = LayoutArena::new();
    let root = boundary.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(200.0, 100.0));

    let mut dispatcher = EventDispatcher::new();
    boundary.register_events(&state, &mut dispatcher);

    let bounds = arena.computed_bounds[root as usize];
    let center = Point::new(
        bounds.origin.x + bounds.size.width * 0.5,
        bounds.origin.y + bounds.size.height * 0.5,
    );

    dispatcher.dispatch(
        &UiEvent::MouseDown {
            button: MouseButton::Left,
            position: center,
        },
        &arena,
        root,
    );
    dispatcher.dispatch(
        &UiEvent::MouseUp {
            button: MouseButton::Left,
            position: center,
        },
        &arena,
        root,
    );

    assert!(
        clicked.load(Ordering::SeqCst),
        "ErrorBoundary must delegate register_events to inner view"
    );
}

#[test]
fn test_paint_context_scroll_viewport_no_hardcoded_gallery_bounds() {
    let arena = LayoutArena::new();
    let theme = Theme::dark();
    let mut comp = RecordingCompositor::default();

    // Default min_scroll_x = 0.0, min_scroll_y = 0.0 -> rect at (10, 20) IS scrolled (not ignored by y >= 50)
    let mut ctx = PaintContext::new(&mut comp, &arena, &theme, &[]).with_scroll(10.0, 0.0);
    ctx.fill_rect(Rect::new(10.0, 20.0, 50.0, 30.0), Color::WHITE);

    assert_eq!(comp.rects.len(), 1);
    assert_eq!(comp.rects[0].0.origin, Point::new(10.0, 10.0));

    // Configurable viewport bounds: items above min_scroll_y are not scrolled
    let mut comp2 = RecordingCompositor::default();
    let mut ctx2 =
        PaintContext::new(&mut comp2, &arena, &theme, &[]).with_scroll_viewport(15.0, 0.0, 40.0);
    ctx2.fill_rect(Rect::new(0.0, 10.0, 100.0, 20.0), Color::WHITE);
    ctx2.fill_rect(Rect::new(0.0, 60.0, 100.0, 20.0), Color::WHITE);

    assert_eq!(comp2.rects.len(), 2);
    assert_eq!(comp2.rects[0].0.origin.y, 10.0);
    assert_eq!(comp2.rects[1].0.origin.y, 45.0);
}

#[test]
fn test_css_defaults_and_zero_opacity_preservation() {
    let borders_default = BordersEffectsProps::default();
    assert_eq!(borders_default.opacity, 1.0);

    let motion_default = MotionProps::default();
    assert_eq!(motion_default.transform_scale, 1.0);

    // Applying a default stylesheet leaves StyleProps.opacity untouched (None)
    let mut props_default = StyleProps::default();
    let default_sheet = CssStyleSheet::default();
    props_default.apply_css(&default_sheet);
    assert_eq!(props_default.opacity, None);

    // Applying opacity == 0.0 must preserve Some(0.0) instead of dropping it
    let mut sheet_hidden = CssStyleSheet::default();
    sheet_hidden.borders_effects.opacity = 0.0;
    let mut props_hidden = StyleProps::default();
    props_hidden.apply_css(&sheet_hidden);
    assert_eq!(props_hidden.opacity, Some(0.0));
}

#[test]
fn test_text_layout_uses_char_count_for_utf8() {
    // 5 Japanese characters (15 UTF-8 bytes)
    let text = Text::new("こんにちは");
    let mut state = text.build();
    let mut arena = LayoutArena::new();
    let node = text.layout(&mut state, &mut arena);

    let expected_w = tej_ui_text::TextMeasurer::new()
        .measure_slice(
            "こんにちは",
            tej_ui_text::SystemFont::Body,
            Text::DEFAULT_FONT_SIZE,
        )
        .width;
    assert_eq!(
        arena.styles[node as usize].width,
        Dimension::Points(expected_w)
    );
}
