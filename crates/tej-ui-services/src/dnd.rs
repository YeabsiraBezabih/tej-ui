use std::path::PathBuf;

/// Payload delivered during a drag-and-drop operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DropPayload {
    /// Files dragged from the OS desktop or file explorer.
    Files(Vec<PathBuf>),
    /// Plain text string.
    Text(String),
    /// Custom internal drag token.
    Custom { type_id: String, data: Vec<u8> },
}

/// Drag-and-drop lifecycle events.
#[derive(Clone, Debug, PartialEq)]
pub enum DragDropEvent {
    DragEntered {
        position: (f32, f32),
        payload: DropPayload,
    },
    DragMoved {
        position: (f32, f32),
    },
    DragLeft,
    Dropped {
        position: (f32, f32),
        payload: DropPayload,
    },
}
