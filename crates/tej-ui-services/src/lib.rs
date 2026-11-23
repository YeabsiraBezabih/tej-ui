//! `tej-services`: Platform operating system services for `tej-ui`.
//!
//! Provides native file dialogs, system clipboard with MIME support, drag-and-drop,
//! global shortcut hooks, system tray status items, and OS keychain credential storage.

pub mod clipboard;
pub mod dialogs;
pub mod dnd;
pub mod dnd_protocol;
pub mod error;
pub mod keychain;
pub mod l10n;
pub mod menu;
pub mod notification;
pub mod packaging;
pub mod pdf_print;
pub mod persistence;
pub mod plugin_sandbox;
pub mod security;
pub mod shortcuts;
pub mod telemetry;
pub mod tray;
pub mod updater;

pub use clipboard::{Clipboard, ClipboardData, ClipboardProvider};
pub use dialogs::{FileDialog, FileDialogProvider, FileFilter};
pub use dnd::{DragDropEvent, DropPayload};
pub use dnd_protocol::{DragPreview, DragSession, DropAction, MimeNegotiator};
pub use error::{Result, ServiceError};
pub use keychain::{BiometricAuthResult, BiometricPromptConfig, SecureKeyStore};
pub use l10n::{L10nManager, Locale, MessageCatalog, MessageEntry, PluralCategory};
pub use menu::{AppBadgeController, MenuBar, MenuItem, MenuItemRole, TaskbarState};
pub use notification::{
    Notification, NotificationAction, NotificationBuilder, SharePayload, ShareSheetDispatcher,
};
pub use packaging::{EnterprisePolicyManager, FlatpakManifest, MacAppBundle, MsiConfig};
pub use pdf_print::{PageLayout, PageMargins, PaginatedPage, PaperSize, PdfPrintSpooler};
pub use persistence::{TabState, WindowGeometry, WorkspaceLayout, WorkspaceStateStore};
pub use plugin_sandbox::{PluginCapability, PluginSandboxHost, SbomComponent, SbomGenerator};
pub use security::{Credential, CredentialStore};
pub use shortcuts::{
    Modifiers, Shortcut, ShortcutChord, ShortcutHandler, ShortcutManager, ShortcutScope,
};
pub use telemetry::{CrashReport, CrashReporter, TelemetrySpan};
pub use tray::{TrayEvent, TrayIcon};
pub use updater::{OtaUpdater, ReleaseChannel, UpdateManifest, UpdateState};
