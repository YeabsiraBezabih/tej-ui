use tej_ui::prelude::*;

#[test]
fn test_app_headless_frame() {
    let count = Signal::new(0);

    let app = App::new(VStack::new((
        Text::new("Counter Title"),
        HStack::new((
            Button::new(Text::new("Increment")).on_click(move || count.update(|c| *c += 1)),
            Checkbox::new(Signal::new(true)),
        )),
    )));

    let state = app.run_headless_frame();
    assert_eq!(state.child_state.0.current_text, "Counter Title");
}

#[test]
fn test_headless_harness_interaction_and_clock() {
    let click_count = Signal::new(0usize);
    let view = VStack::new((
        Text::new("Dashboard Header"),
        Button::new(Text::new("Count Up")).on_click(move || click_count.update(|c| *c += 1)),
    ));

    let mut harness = TestHarness::new(view);
    assert_eq!(harness.frame_count(), 1);
    assert_eq!(harness.mock_time_millis(), 0);
    assert!(harness.node_count() >= 2);

    // 1. Advance mock time
    harness.advance_time(100);
    assert_eq!(harness.mock_time_millis(), 100);
    assert_eq!(harness.frame_count(), 2);

    // 2. Simulated interaction: Click the button
    let button_node_id = harness.state().child_state.1.node;
    let button_node = harness
        .get_node(button_node_id)
        .expect("Button node not found");

    assert!(button_node.bounds.size.width > 0.0);
    assert!(button_node.bounds.size.height > 0.0);

    // Click at the exact center of the button
    harness.click_node(button_node);
    assert_eq!(click_count.get(), 1);

    harness.click_node(button_node);
    assert_eq!(click_count.get(), 2);

    // Step frame ensures state consistency
    harness.step_frame();
    assert_eq!(harness.frame_count(), 5);
}
