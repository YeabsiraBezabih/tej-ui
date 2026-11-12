//! Multi-line rich text editor submodule with undo/redo and IME support.

pub mod buffer;
pub mod editor_widget;
pub mod history;
pub mod ime;
pub mod selection;

pub use buffer::TextBuffer;
pub use editor_widget::{RichTextEditor, RichTextEditorState};
pub use history::{EditAction, UndoStack};
pub use ime::ImeState;
pub use selection::{CursorPos, TextSelection};
