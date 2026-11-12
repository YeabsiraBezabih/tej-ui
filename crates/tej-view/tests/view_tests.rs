use tej_core::Signal;
use tej_hal::Color;
use tej_layout::Dimension;
use tej_view::{Button, Canvas, Checkbox, HStack, If, Text, VStack, View, ViewExt};

#[test]
fn test_view_ext_styling_modifiers() {
    let styled_button = Button::new(Text::new("Submit"))
        .padding(12.0)
        .background(Color::rgb(45, 55, 72))
        .corner_radius(8.0)
        .border(1.5, Color::rgb(66, 153, 225))
        .width(Dimension::Points(200.0));

    let state = styled_button.build();
    assert_eq!(state.style.corner_radius, 8.0);
    assert_eq!(state.style.border_width, 1.5);
    assert_eq!(state.style.padding.top, 12.0);
}

#[test]
fn test_canvas_view_rendering() {
    let canvas = Canvas::new(|compositor, bounds| {
        compositor.fill_rect(bounds, Color::rgb(30, 30, 40));
    });

    let mut state = canvas.build();
    assert_eq!(state.redraw_count, 0);

    canvas.patch(&mut state, 1);
    assert_eq!(state.redraw_count, 1);
}

#[test]
fn test_static_text_view() {
    let t = Text::new("Hello World");
    let state = t.build();
    assert_eq!(state.current_text, "Hello World");
}

#[test]
fn test_reactive_text_view() {
    let msg = Signal::new(String::from("Initial"));
    let t = Text::reactive(msg);
    let mut state = t.build();
    assert_eq!(state.current_text, "Initial");

    msg.set(String::from("Updated"));
    t.patch(&mut state, 1);
    assert_eq!(state.current_text, "Updated");
}

#[test]
fn test_tuple_combinator_view() {
    let view = (
        Text::new("Title"),
        Text::new("Subtitle"),
        Button::new(Text::new("Click Me")),
    );

    let state = view.build();
    assert_eq!(state.0.current_text, "Title");
    assert_eq!(state.1.current_text, "Subtitle");
}

#[test]
fn test_vstack_and_hstack() {
    let count = Signal::new(0);
    let view = VStack::new((
        Text::new("Counter"),
        HStack::new((
            Button::new(Text::new("+")).on_click(move || count.update(|c| *c += 1)),
            Checkbox::new(Signal::new(true)),
        )),
    ))
    .padding(16)
    .spacing(8);

    let state = view.build();
    assert_eq!(state.padding, 16);
    assert_eq!(state.spacing, 8);
    assert_eq!(state.child_state.0.current_text, "Counter");
}

#[test]
fn test_conditional_if_view() {
    let show_details = Signal::new(false);
    let view = If::new(
        show_details,
        Text::new("Detailed Information"),
        Text::new("Summary"),
    );

    let mut state = view.build();
    assert!(!state.current_cond);

    show_details.set(true);
    view.patch(&mut state, 1);
    assert!(state.current_cond);
}
