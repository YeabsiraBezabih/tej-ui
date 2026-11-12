//! `tej-services`: Platform operating system services for `tej-ui`.
//!
//! Provides native file dialogs, system clipboard with MIME support, drag-and-drop,
//! global shortcut hooks, system tray status items, and OS keychain credential storage.

pub mod clipboard;
pub mod dialogs;
pub mod dnd;
pub mod error;
pub mod security;
pub mod shortcuts;
pub mod tray;

pub use clipboard::{Clipboard, ClipboardData};
pub use dialogs::{FileDialog, FileFilter};
pub use dnd::{DragDropEvent, DropPayload};
pub use error::{Result, ServiceError};
pub use security::{Credential, CredentialStore};
pub use shortcuts::{Modifiers, Shortcut, ShortcutHandler, ShortcutManager};
pub use tray::{TrayEvent, TrayIcon};
