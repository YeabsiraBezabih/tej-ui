use tej_ui_core::Signal;
use tej_ui_hal::Color;
use tej_ui_layout::Dimension;
use tej_ui_view::{Button, Canvas, Checkbox, HStack, If, Text, VStack, View, ViewExt};

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

#[derive(Clone, Debug, PartialEq, Eq)]
struct SubtreeConfig {
    accent: String,
}

#[derive(Debug)]
struct ContextInspectView;

impl View for ContextInspectView {
    type State = Option<SubtreeConfig>;

    fn build(&self) -> Self::State {
        tej_ui_core::use_context::<SubtreeConfig>()
    }

    fn rebuild(&self, state: &mut Self::State) {
        *state = tej_ui_core::use_context::<SubtreeConfig>();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}

#[test]
fn test_context_provider_view_hierarchy() {
    assert_eq!(tej_ui_core::use_context::<SubtreeConfig>(), None);

    let view = ContextInspectView.provide_context(SubtreeConfig {
        accent: "emerald".into(),
    });

    let state = view.build();
    assert_eq!(
        state,
        Some(SubtreeConfig {
            accent: "emerald".into()
        })
    );

    // Outside the provider, ambient context remains unset
    assert_eq!(tej_ui_core::use_context::<SubtreeConfig>(), None);
}

#[test]
fn test_snapshot_testing_and_diff_comparator() {
    use tej_ui_view::{ImageBuffer, SnapshotComparator};

    // 1. Identical buffers comparison
    let mut golden = ImageBuffer::new(10, 10);
    for y in 0..10 {
        for x in 0..10 {
            golden.set_pixel(x, y, 0xFF00_FF00); // Solid green
        }
    }

    let actual = golden.clone();
    let comparator = SnapshotComparator::strict();
    let res = comparator.compare(&actual, &golden);
    assert!(res.is_match);
    assert_eq!(res.different_pixels, 0);
    assert_eq!(res.max_channel_diff, 0);

    // 2. Tolerance check: slight rounding delta in blue channel (delta = 2)
    let mut actual_slight_diff = golden.clone();
    actual_slight_diff.set_pixel(5, 5, 0xFF00_FF02);

    let default_comp = SnapshotComparator::default(); // tolerance = 2
    let res_tolerated = default_comp.compare(&actual_slight_diff, &golden);
    assert!(res_tolerated.is_match);
    assert_eq!(res_tolerated.different_pixels, 0);

    // 3. Significant failure: mismatched color (red vs green)
    let mut actual_bad = golden.clone();
    actual_bad.set_pixel(0, 0, 0xFFFF_0000); // Red
    actual_bad.set_pixel(1, 1, 0xFFFF_0000);

    let res_bad = comparator.compare(&actual_bad, &golden);
    assert!(!res_bad.is_match);
    assert_eq!(res_bad.different_pixels, 2);
    assert!(res_bad.max_channel_diff > 200);

    // Diff mask highlighted mismatched pixel in failure red
    assert_eq!(res_bad.diff_mask.get_pixel(0, 0), Some(0xFFFF_0000));

    // 4. PPM export format
    let ppm = golden.to_ppm_string();
    assert!(ppm.starts_with("P3\n10 10\n255\n"));
}

#[test]
fn test_event_stream_fuzzer_and_mount_stress() {
    use tej_ui_view::{Button, EventStreamFuzzer, MountStressReport, Text, VStack};

    // 1. Chaos event stream fuzzer
    let mut fuzzer = EventStreamFuzzer::new(42, 800.0, 600.0);
    let events = fuzzer.generate_batch(50);
    assert_eq!(events.len(), 50);

    // 2. High-iteration mount and unmount stress harness (10,000 cycles)
    let report = MountStressReport::execute_cycles(10_000, || {
        VStack::new((
            Text::new("Ephemeral Label"),
            Button::new(Text::new("Press")).on_click(|| {}),
        ))
    });

    assert!(report.successful);
    assert_eq!(report.completed_cycles, 10_000);
}
