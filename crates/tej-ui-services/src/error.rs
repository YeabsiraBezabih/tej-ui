use thiserror::Error;

/// Operating system service errors.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ServiceError {
    /// Clipboard operation failed.
    #[error("Clipboard error: {0}")]
    ClipboardError(String),

    /// File dialog cancelled or failed.
    #[error("File dialog error: {0}")]
    DialogError(String),

    /// Credential storage error.
    #[error("Credential store error: {0}")]
    SecurityError(String),

    /// Shortcut registration error.
    #[error("Global shortcut error: {0}")]
    ShortcutError(String),

    /// System tray error.
    #[error("System tray error: {0}")]
    TrayError(String),

    /// Persistence / serialization error.
    #[error("Persistence error: {0}")]
    PersistenceError(String),
}

/// Specialized result type for `tej-services`.
pub type Result<T> = std::result::Result<T, ServiceError>;
