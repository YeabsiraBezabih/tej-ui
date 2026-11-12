use tej::prelude::*;

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
