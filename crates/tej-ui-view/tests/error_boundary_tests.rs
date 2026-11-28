use std::sync::Arc;
use std::sync::Mutex;
use tej_ui_view::{ErrorBoundary, ErrorBoundaryState, Text, View};

#[derive(Clone, Debug)]
struct PanickingView {
    should_panic: bool,
}

impl View for PanickingView {
    type State = ();

    fn build(&self) -> Self::State {
        assert!(!self.should_panic, "Test component failure");
    }

    fn rebuild(&self, _state: &mut Self::State) {
        assert!(!self.should_panic, "Rebuild panic");
    }

    fn patch(&self, _state: &mut Self::State, _dirty_bits: u64) {
        assert!(!self.should_panic, "Patch panic");
    }

    fn layout(&self, _state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        arena.allocate_node(tej_ui_layout::LayoutStyle::default())
    }

    fn paint(&self, _state: &Self::State, _ctx: &mut tej_ui_view::PaintContext<'_>) {}
}

#[test]
fn test_error_boundary_normal_render() {
    let boundary = ErrorBoundary::new(Text::new("Normal Content"), |err| {
        Text::new(format!("Error: {err}"))
    });

    let state = boundary.build();
    match state {
        ErrorBoundaryState::Normal(text_state) => {
            assert_eq!(text_state.current_text, "Normal Content");
        }
        ErrorBoundaryState::Recovered { .. } => {
            panic!("Expected normal state");
        }
    }
}

#[test]
fn test_error_boundary_catches_panic_and_reports_telemetry() {
    let telemetry_log = Arc::new(Mutex::new(Vec::new()));
    let tel_clone = Arc::clone(&telemetry_log);

    let boundary = ErrorBoundary::new(PanickingView { should_panic: true }, |err| {
        Text::new(format!("Handled Crash: {err}"))
    })
    .on_error(move |err| {
        tel_clone.lock().unwrap().push(err.to_string());
    });

    let state = boundary.build();
    match state {
        ErrorBoundaryState::Recovered {
            error_message,
            fallback_state,
        } => {
            assert_eq!(error_message, "Test component failure");
            assert_eq!(
                fallback_state.current_text,
                "Handled Crash: Test component failure"
            );
        }
        ErrorBoundaryState::Normal(()) => {
            panic!("Expected recovered fallback state");
        }
    }

    let logs = telemetry_log.lock().unwrap().clone();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0], "Test component failure");
}
