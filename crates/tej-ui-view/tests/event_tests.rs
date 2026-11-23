use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tej_ui_core::Signal;
use tej_ui_hal::{MouseButton, UiEvent};
use tej_ui_layout::{LayoutArena, LayoutEngine, Point, Size};
use tej_ui_view::{Button, CursorIcon, EventDispatcher, Input, Text, VStack, View};

#[test]
fn test_hit_testing_hierarchy() {
    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(tej_ui_layout::LayoutStyle {
        width: tej_ui_layout::Dimension::Points(500.0),
        height: tej_ui_layout::Dimension::Points(400.0),
        direction: tej_ui_layout::FlexDirection::Column,
        ..Default::default()
    });

    let child1 = arena.allocate_node(tej_ui_layout::LayoutStyle {
        width: tej_ui_layout::Dimension::Points(500.0),
        height: tej_ui_layout::Dimension::Points(100.0),
        ..Default::default()
    });
    let child2 = arena.allocate_node(tej_ui_layout::LayoutStyle {
        width: tej_ui_layout::Dimension::Points(500.0),
        height: tej_ui_layout::Dimension::Points(100.0),
        ..Default::default()
    });

    arena.add_child(root, child1);
    arena.add_child(root, child2);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(500.0, 400.0));

    // Hit test inside child1 (y = 50.0)
    assert_eq!(arena.hit_test(root, Point::new(100.0, 50.0)), Some(child1));

    // Hit test inside child2 (y = 150.0)
    assert_eq!(arena.hit_test(root, Point::new(100.0, 150.0)), Some(child2));

    // Hit test outside
    assert_eq!(arena.hit_test(root, Point::new(600.0, 500.0)), None);
}

#[test]
fn test_click_dispatch_and_cursor() {
    let clicked = Arc::new(AtomicBool::new(false));
    let cl_clone = Arc::clone(&clicked);

    let view = Button::new(Text::new("Submit")).on_click(move || {
        cl_clone.store(true, Ordering::SeqCst);
    });

    let mut state = view.build();
    let mut arena = LayoutArena::new();
    let root = view.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(200.0, 50.0));

    let mut dispatcher = EventDispatcher::new();
    view.register_events(&state, &mut dispatcher);

    // Mouse move over button -> cursor should be Pointer
    let res = dispatcher.dispatch(
        &UiEvent::MouseMove {
            position: Point::new(50.0, 25.0),
        },
        &arena,
        root,
    );
    assert_eq!(res.cursor, CursorIcon::Pointer);
    assert!(dispatcher.is_hovered(&arena, root));

    // Mouse down and mouse up at button position
    dispatcher.dispatch(
        &UiEvent::MouseDown {
            button: MouseButton::Left,
            position: Point::new(50.0, 25.0),
        },
        &arena,
        root,
    );
    assert!(dispatcher.is_pressed(root));

    dispatcher.dispatch(
        &UiEvent::MouseUp {
            button: MouseButton::Left,
            position: Point::new(50.0, 25.0),
        },
        &arena,
        root,
    );
    assert!(!dispatcher.is_pressed(root));

    // Callback must have fired
    assert!(clicked.load(Ordering::SeqCst));
}

#[test]
fn test_tab_navigation_and_keyboard_activation() {
    let btn1_clicks = Arc::new(AtomicUsize::new(0));
    let b1_clone = Arc::clone(&btn1_clicks);

    let btn2_clicks = Arc::new(AtomicUsize::new(0));
    let b2_clone = Arc::clone(&btn2_clicks);

    let view = VStack::new((
        Button::new(Text::new("First")).on_click(move || {
            b1_clone.fetch_add(1, Ordering::SeqCst);
        }),
        Button::new(Text::new("Second")).on_click(move || {
            b2_clone.fetch_add(1, Ordering::SeqCst);
        }),
    ));

    let mut state = view.build();
    let mut arena = LayoutArena::new();
    let root = view.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 200.0));

    let mut dispatcher = EventDispatcher::new();
    view.register_events(&state, &mut dispatcher);

    // Press Tab (key_code 9) -> moves focus to First button
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 9 }, &arena, root);
    assert!(dispatcher.focused_node.is_some());
    let focused_first = dispatcher.focused_node.unwrap();

    // Press Enter (key_code 13) -> activates First button
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 13 }, &arena, root);
    assert_eq!(btn1_clicks.load(Ordering::SeqCst), 1);
    assert_eq!(btn2_clicks.load(Ordering::SeqCst), 0);

    // Press Tab again -> shifts focus to Second button
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 9 }, &arena, root);
    assert_ne!(dispatcher.focused_node, Some(focused_first));

    // Press Space (key_code 32) -> activates Second button
    dispatcher.dispatch(&UiEvent::KeyDown { key_code: 32 }, &arena, root);
    assert_eq!(btn1_clicks.load(Ordering::SeqCst), 1);
    assert_eq!(btn2_clicks.load(Ordering::SeqCst), 1);
}

#[test]
fn test_text_input_event_routing() {
    let text = Signal::new(String::new());
    let input = Input::new(text);

    let mut state = input.build();
    let mut arena = LayoutArena::new();
    let root = input.layout(&mut state, &mut arena);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 50.0));

    let mut dispatcher = EventDispatcher::new();
    input.register_events(&state, &mut dispatcher);

    // Click on input to focus
    dispatcher.dispatch(
        &UiEvent::MouseDown {
            button: MouseButton::Left,
            position: Point::new(10.0, 10.0),
        },
        &arena,
        root,
    );
    assert!(dispatcher.is_focused(root));

    // Send TextInput
    dispatcher.dispatch(
        &UiEvent::TextInput {
            text: "Hello Rust".into(),
        },
        &arena,
        root,
    );

    assert_eq!(text.get(), "Hello Rust");
}

#[test]
fn test_two_phase_event_routing_capture_and_bubble() {
    use std::sync::Mutex;
    use tej_ui_view::{EventPhase, EventPropagation};

    let log = Arc::new(Mutex::new(Vec::new()));
    let log_clone = Arc::clone(&log);

    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(tej_ui_layout::LayoutStyle {
        width: tej_ui_layout::Dimension::Points(400.0),
        height: tej_ui_layout::Dimension::Points(400.0),
        ..Default::default()
    });
    let child = arena.allocate_node(tej_ui_layout::LayoutStyle {
        width: tej_ui_layout::Dimension::Points(200.0),
        height: tej_ui_layout::Dimension::Points(200.0),
        ..Default::default()
    });
    arena.add_child(root, child);
    LayoutEngine::compute_layout(&mut arena, root, Size::new(400.0, 400.0));

    let mut dispatcher = EventDispatcher::new();

    // Register capturing listener on root
    let log_c1 = Arc::clone(&log_clone);
    dispatcher.register_capture(
        root,
        Arc::new(move |_ev, phase, node| {
            assert_eq!(phase, EventPhase::Capturing);
            log_c1.lock().unwrap().push(format!("root_capture:{node}"));
            EventPropagation::Continue
        }),
    );

    // Register bubbling listener on root
    let log_b1 = Arc::clone(&log_clone);
    dispatcher.register_bubble(
        root,
        Arc::new(move |_ev, phase, node| {
            assert_eq!(phase, EventPhase::Bubbling);
            log_b1.lock().unwrap().push(format!("root_bubble:{node}"));
            EventPropagation::Continue
        }),
    );

    // Register click handler on child
    let log_ch = Arc::clone(&log_clone);
    dispatcher.register_click(
        child,
        Arc::new(move || {
            log_ch.lock().unwrap().push("child_click".into());
        }),
    );

    // Dispatch MouseDown and MouseUp on child
    dispatcher.dispatch(
        &UiEvent::MouseDown {
            button: MouseButton::Left,
            position: Point::new(50.0, 50.0),
        },
        &arena,
        root,
    );

    dispatcher.dispatch(
        &UiEvent::MouseUp {
            button: MouseButton::Left,
            position: Point::new(50.0, 50.0),
        },
        &arena,
        root,
    );

    let events = log.lock().unwrap().clone();
    // Verification: capturing runs first on root, then child fires click, then root bubbles
    assert!(events.contains(&format!("root_capture:{root}")));
    assert!(events.contains(&"child_click".to_string()));
    assert!(events.contains(&format!("root_bubble:{root}")));

    // Test capture cancellation: interceptor returns EventPropagation::Stop
    let mut dispatcher2 = EventDispatcher::new();
    let child_hit = Arc::new(AtomicBool::new(false));
    let ch_clone = Arc::clone(&child_hit);

    dispatcher2.register_capture(
        root,
        Arc::new(|_ev, _phase, _node| {
            // Cancel event at parent container
            EventPropagation::Stop
        }),
    );
    dispatcher2.register_click(
        child,
        Arc::new(move || {
            ch_clone.store(true, Ordering::SeqCst);
        }),
    );

    dispatcher2.dispatch(
        &UiEvent::MouseDown {
            button: MouseButton::Left,
            position: Point::new(50.0, 50.0),
        },
        &arena,
        root,
    );
    dispatcher2.dispatch(
        &UiEvent::MouseUp {
            button: MouseButton::Left,
            position: Point::new(50.0, 50.0),
        },
        &arena,
        root,
    );

    // Child click should NOT have fired because root capture cancelled the event
    assert!(!child_hit.load(Ordering::SeqCst));
}
