use std::sync::atomic::{AtomicUsize, Ordering};
use tej_ui_core::Signal;
use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{LayoutArena, LayoutEngine, Point, Rect, Size};
use tej_ui_view::{Button, Input, PaintContext, Text, Theme, VStack, View, ViewExt};

#[derive(Debug, Default)]
struct MockCompositor {
    rect_count: AtomicUsize,
    text_count: AtomicUsize,
}

impl SurfaceCompositor for MockCompositor {
    fn begin_frame(&mut self, _dirty_rects: &[Rect]) {}

    fn fill_rect(&mut self, _rect: Rect, _color: Color) {
        self.rect_count.fetch_add(1, Ordering::SeqCst);
    }

    fn fill_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: Color) {
        self.rect_count.fetch_add(1, Ordering::SeqCst);
    }

    fn draw_line(&mut self, _p1: Point, _p2: Point, _width: f32, _color: Color) {}

    fn draw_text(&mut self, _text: &str, _origin: Point, _font_size: f32, _color: Color) {
        self.text_count.fetch_add(1, Ordering::SeqCst);
    }

    fn end_frame(&mut self) {}
}

#[test]
fn test_view_tree_layout_and_paint_pipeline() {
    let text_signal = Signal::new(String::from("Search database"));
    let view = VStack::new((Button::new(Text::new("Click me")), Input::new(text_signal)))
        .padding(12)
        .background(Color::rgb(20, 20, 25));

    let mut state = view.build();
    let mut arena = LayoutArena::new();

    let root_node = view.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root_node, Size::new(800.0, 600.0));

    let root_bounds = arena.computed_bounds[root_node as usize];
    assert_eq!(root_bounds.size.width, 800.0);
    assert_eq!(root_bounds.size.height, 600.0);

    let mut compositor = MockCompositor::default();
    let theme = Theme::dark();
    let dirty_rects = [root_bounds];

    let mut ctx = PaintContext::new(&mut compositor, &arena, &theme, &dirty_rects);
    view.paint(&state, &mut ctx);

    // Bkg rect for Styled container + button rounded rect + input rounded rect = at least 3 rects
    assert!(compositor.rect_count.load(Ordering::SeqCst) >= 3);
    // Button text + input text = at least 2 text draw calls
    assert!(compositor.text_count.load(Ordering::SeqCst) >= 2);
}
