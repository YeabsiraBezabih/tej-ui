//! System clipboard service managing inter-process text, HTML, image, and custom data exchange.

use crate::error::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Data format representation on the system clipboard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipboardData {
    Text(String),
    Html {
        html: String,
        alt_text: Option<String>,
    },
    ImageRgba {
        width: u32,
        height: u32,
        bytes: Vec<u8>,
    },
    Custom {
        mime_type: String,
        payload: Vec<u8>,
    },
}

/// Abstract contract for asynchronous and synchronous platform clipboard providers.
pub trait ClipboardProvider: std::fmt::Debug + Send + Sync {
    /// Read plain text string from system clipboard.
    fn get_text(&self) -> Result<Option<String>>;

    /// Write plain text string to system clipboard.
    fn set_text(&self, text: &str) -> Result<()>;

    /// Read rich HTML markup from system clipboard.
    fn get_html(&self) -> Result<Option<String>>;

    /// Write rich HTML markup and optional fallback plain text.
    fn set_html(&self, html: &str, alt_text: Option<&str>) -> Result<()>;

    /// Read uncompressed RGBA pixel buffer `(width, height, bytes)`.
    fn get_image(&self) -> Result<Option<(u32, u32, Vec<u8>)>>;

    /// Write RGBA pixel buffer to clipboard.
    fn set_image(&self, width: u32, height: u32, bytes: Vec<u8>) -> Result<()>;

    /// Clear all formats and content currently on the clipboard.
    fn clear(&self) -> Result<()>;

    /// List all MIME types currently available on clipboard.
    fn formats(&self) -> Vec<String>;
}

/// System clipboard service managing inter-process text, image, and custom data exchange.
#[derive(Debug, Default, Clone)]
pub struct Clipboard {
    storage: Arc<RwLock<HashMap<String, ClipboardData>>>,
}

impl Clipboard {
    /// Create a new clipboard handle.
    #[must_use]
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Read text from clipboard.
    pub fn get_text(&self) -> Result<Option<String>> {
        let read = self.storage.read();
        if let Some(ClipboardData::Text(s)) = read.get("text/plain") {
            Ok(Some(s.clone()))
        } else {
            Ok(None)
        }
    }

    /// Write text to clipboard.
    pub fn set_text(&self, text: impl Into<String>) -> Result<()> {
        let text_str = text.into();
        let mut write = self.storage.write();
        write.insert("text/plain".into(), ClipboardData::Text(text_str));
        Ok(())
    }

    /// Read HTML markup from clipboard.
    pub fn get_html(&self) -> Result<Option<String>> {
        let read = self.storage.read();
        if let Some(ClipboardData::Html { html, .. }) = read.get("text/html") {
            Ok(Some(html.clone()))
        } else {
            Ok(None)
        }
    }

    /// Write HTML markup to clipboard with optional fallback plain text.
    pub fn set_html(&self, html: impl Into<String>, alt_text: Option<&str>) -> Result<()> {
        let html_str = html.into();
        let mut write = self.storage.write();
        if let Some(alt) = alt_text {
            write.insert("text/plain".into(), ClipboardData::Text(alt.to_string()));
        }
        write.insert(
            "text/html".into(),
            ClipboardData::Html {
                html: html_str,
                alt_text: alt_text.map(ToString::to_string),
            },
        );
        Ok(())
    }

    /// Read image RGBA pixel buffer.
    pub fn get_image(&self) -> Result<Option<(u32, u32, Vec<u8>)>> {
        let read = self.storage.read();
        if let Some(ClipboardData::ImageRgba {
            width,
            height,
            bytes,
        }) = read.get("image/png")
        {
            Ok(Some((*width, *height, bytes.clone())))
        } else {
            Ok(None)
        }
    }

    /// Write RGBA pixel buffer to clipboard.
    pub fn set_image(&self, width: u32, height: u32, bytes: Vec<u8>) -> Result<()> {
        let mut write = self.storage.write();
        write.insert(
            "image/png".into(),
            ClipboardData::ImageRgba {
                width,
                height,
                bytes,
            },
        );
        Ok(())
    }

    /// Read custom MIME payload.
    pub fn get_custom(&self, mime_type: &str) -> Result<Option<Vec<u8>>> {
        let read = self.storage.read();
        if let Some(ClipboardData::Custom { payload, .. }) = read.get(mime_type) {
            Ok(Some(payload.clone()))
        } else {
            Ok(None)
        }
    }

    /// Set custom MIME payload.
    pub fn set_custom(&self, mime_type: impl Into<String>, payload: Vec<u8>) -> Result<()> {
        let mime = mime_type.into();
        let mut write = self.storage.write();
        write.insert(
            mime.clone(),
            ClipboardData::Custom {
                mime_type: mime,
                payload,
            },
        );
        Ok(())
    }

    /// Clear all clipboard formats.
    pub fn clear(&self) -> Result<()> {
        self.storage.write().clear();
        Ok(())
    }

    /// List available clipboard formats.
    #[must_use]
    pub fn formats(&self) -> Vec<String> {
        self.storage.read().keys().cloned().collect()
    }
}

impl ClipboardProvider for Clipboard {
    fn get_text(&self) -> Result<Option<String>> {
        self.get_text()
    }

    fn set_text(&self, text: &str) -> Result<()> {
        self.set_text(text)
    }

    fn get_html(&self) -> Result<Option<String>> {
        self.get_html()
    }

    fn set_html(&self, html: &str, alt_text: Option<&str>) -> Result<()> {
        self.set_html(html, alt_text)
    }

    fn get_image(&self) -> Result<Option<(u32, u32, Vec<u8>)>> {
        self.get_image()
    }

    fn set_image(&self, width: u32, height: u32, bytes: Vec<u8>) -> Result<()> {
        self.set_image(width, height, bytes)
    }

    fn clear(&self) -> Result<()> {
        self.clear()
    }

    fn formats(&self) -> Vec<String> {
        self.formats()
    }
}
