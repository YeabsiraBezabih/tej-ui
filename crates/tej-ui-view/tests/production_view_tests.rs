use std::sync::{Arc, Mutex};
use tej_ui_a11y::{A11yNode, A11yRole, A11yTree};
use tej_ui_core::Signal;
use tej_ui_hal::{Color, MouseButton, SurfaceCompositor, UiEvent};
use tej_ui_layout::{LayoutArena, LayoutEngine, Point, Rect, Size};
use tej_ui_view::{
    Button, Checkbox, ErrorBoundary, EventDispatcher, Grid, HStack, Input, PaintContext, Text,
    Theme, VStack, View, ViewExt,
};

#[derive(Debug, Default)]
struct RecordingCompositor {
    draw_log: Arc<Mutex<Vec<String>>>,
}

impl SurfaceCompositor for RecordingCompositor {
    fn begin_frame(&mut self, _dirty_rects: &[Rect]) {}

    fn fill_rect(&mut self, rect: Rect, _color: Color) {
        self.draw_log.lock().unwrap().push(format!(
            "rect:{:.0},{:.0},{:.0}x{:.0}",
            rect.origin.x, rect.origin.y, rect.size.width, rect.size.height
        ));
    }

    fn fill_rounded_rect(&mut self, rect: Rect, _radius: f32, _color: Color) {
        self.draw_log.lock().unwrap().push(format!(
            "rrect:{:.0},{:.0},{:.0}x{:.0}",
            rect.origin.x, rect.origin.y, rect.size.width, rect.size.height
        ));
    }

    fn draw_line(&mut self, _p1: Point, _p2: Point, _width: f32, _color: Color) {}

    fn draw_text(&mut self, text: &str, origin: Point, _font_size: f32, _color: Color) {
        self.draw_log
            .lock()
            .unwrap()
            .push(format!("text:{text}@{:.0},{:.1}", origin.x, origin.y));
    }

    fn end_frame(&mut self) {}
}

#[test]
fn test_wrapped_text_layout_and_multiline_paint() {
    let text = Text::new("The quick brown fox jumps over the lazy dog").max_width(110.0);
    let mut state = text.build();
    let mut arena = LayoutArena::new();

    let node = text.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, node, Size::new(400.0, 400.0));

    assert!(
        state.lines.len() >= 3,
        "expected wrapped multi-line state, got {:?}",
        state.lines
    );
    let bounds = arena.computed_bounds[node as usize];
    assert!(bounds.size.width <= 110.0 + 0.01);
    assert!(bounds.size.height > 40.0);

    let mut compositor = RecordingCompositor::default();
    let theme = Theme::dark();
    let mut ctx = PaintContext::new(&mut compositor, &arena, &theme, &[]);
    text.paint(&state, &mut ctx);

    let log = compositor.draw_log.lock().unwrap();
    assert_eq!(log.len(), state.lines.len());
    // Subsequent lines must be offset downward along Y
    assert!(log[0].contains("@0,0.0"));
    assert!(!log[1].contains("@0,0.0"));
}

#[test]
fn test_input_grapheme_safe_backspace() {
    let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
    let accented = "e\u{0301}";
    let signal = Signal::new(format!("Hi {accented}{family}"));
    let input = Input::new(signal);

    let mut state = input.build();
    let mut arena = LayoutArena::new();
    let root = input.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 60.0));

    let mut dispatcher = EventDispatcher::new();
    input.register_events(&state, &mut dispatcher);
    // Focus the input node via MouseDown
    dispatcher.dispatch(
        &UiEvent::MouseDown {
            button: MouseButton::Left,
            position: Point::new(10.0, 10.0),
        },
        &arena,
        root,
    );
    assert!(dispatcher.is_focused(root));

    // First Backspace removes the entire 7-scalar ZWJ family emoji cluster in one step
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 8 }, &arena, root);
    assert_eq!(signal.get(), format!("Hi {accented}"));

    // Second Backspace removes the entire combining accent cluster ("e" + U+0301)
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 8 }, &arena, root);
    assert_eq!(signal.get(), "Hi ");
}

#[test]
fn test_view_accessibility_tree_emission() {
    let input_sig = Signal::new(String::from("alice@example.com"));
    let checked_sig = Signal::new(true);

    let view = VStack::new((
        Text::new("Settings"),
        HStack::new((Button::new(Text::new("Save")), Checkbox::new(checked_sig))),
        Input::new(input_sig).placeholder("Email"),
        Grid::new((Text::new("Cell A"), Text::new("Cell B"))).columns(2),
        ErrorBoundary::new(Text::new("Safe").padding(4.0), |e| Text::new(e.to_string())),
    ));

    let mut state = view.build();
    let mut arena = LayoutArena::new();
    let root_node = view.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root_node, Size::new(800.0, 600.0));

    let mut tree = A11yTree::new();
    let win_id = tree.allocate_id();
    tree.insert_node(A11yNode::new(win_id, A11yRole::Window));

    view.accessibility(&state, &arena, &mut tree, Some(win_id));

    let roles: Vec<A11yRole> = tree.iter().map(|(_, n)| n.role).collect();
    assert!(roles.contains(&A11yRole::Window));
    assert!(roles.contains(&A11yRole::Button));
    assert!(roles.contains(&A11yRole::Checkbox));
    assert!(roles.contains(&A11yRole::TextInput));
    assert!(roles.contains(&A11yRole::Grid));
    assert!(roles.contains(&A11yRole::Label));

    let checkbox_node = tree
        .iter()
        .find_map(|(_, n)| (n.role == A11yRole::Checkbox).then_some(n))
        .expect("checkbox a11y node present");
    assert_eq!(checkbox_node.is_checked, Some(true));

    let input_node = tree
        .iter()
        .find_map(|(_, n)| (n.role == A11yRole::TextInput).then_some(n))
        .expect("input a11y node present");
    assert_eq!(input_node.label.as_deref(), Some("Email"));
    assert_eq!(input_node.value.as_deref(), Some("alice@example.com"));

    // Window root should have the VStack attached as its child
    let win = tree.get_node(win_id).unwrap();
    assert_eq!(win.children.len(), 1);
}

#[test]
fn test_paint_context_deferred_overlay_z_ordering() {
    let arena = LayoutArena::new();
    let theme = Theme::dark();
    let mut compositor = RecordingCompositor::default();
    let mut ctx = PaintContext::new(&mut compositor, &arena, &theme, &[]);

    // Base element 1 paints, then schedules a popover overlay
    ctx.draw_text("base-1", Point::new(0.0, 0.0), 14.0, Color::WHITE);
    ctx.defer_overlay(Box::new(|overlay_ctx| {
        overlay_ctx.fill_rect(Rect::new(10.0, 10.0, 100.0, 50.0), Color::BLACK);
        overlay_ctx.draw_text("modal-overlay", Point::new(12.0, 12.0), 14.0, Color::WHITE);
        // Nested tooltip overlay deferred from inside the modal
        overlay_ctx.defer_overlay(Box::new(|nested_ctx| {
            nested_ctx.draw_text("nested-tooltip", Point::new(20.0, 20.0), 12.0, Color::WHITE);
        }));
    }));
    // Base element 2 paints after defer_overlay was queued
    ctx.draw_text("base-2", Point::new(0.0, 30.0), 14.0, Color::WHITE);

    assert_eq!(ctx.pending_overlay_count(), 1);
    ctx.flush_overlays();
    assert_eq!(ctx.pending_overlay_count(), 0);

    let log = compositor.draw_log.lock().unwrap();
    assert_eq!(log.len(), 5);
    assert!(log[0].starts_with("text:base-1"));
    assert!(log[1].starts_with("text:base-2"));
    assert!(log[2].starts_with("rect:10,10,100x50"));
    assert!(log[3].starts_with("text:modal-overlay"));
    assert!(log[4].starts_with("text:nested-tooltip"));
}
