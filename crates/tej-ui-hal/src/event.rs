use tej_ui_layout::Point;

/// Mouse button identifier.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Unified cross-platform UI event enumeration.
#[derive(Clone, PartialEq, Debug)]
pub enum UiEvent {
    MouseMove {
        position: Point,
    },
    MouseDown {
        button: MouseButton,
        position: Point,
    },
    MouseUp {
        button: MouseButton,
        position: Point,
    },
    Scroll {
        delta_x: f32,
        delta_y: f32,
        position: Point,
    },
    KeyDown {
        key_code: u32,
    },
    KeyUp {
        key_code: u32,
    },
    TextInput {
        text: String,
    },
    Resize {
        width: f32,
        height: f32,
    },
    /// Monitor DPI scaling factor changed (e.g. window dragged to a high-DPI display).
    ScaleFactorChanged {
        scale_factor: f32,
    },
    FocusGained,
    FocusLost,
    CloseRequested,
    /// Request to redraw/present the frame.
    RedrawRequested,
    /// Native IME pre-edit composition updated.
    ImePreedit {
        text: String,
        cursor: Option<(usize, usize)>,
    },
    /// Native IME composition committed.
    ImeCommit {
        text: String,
    },
}
