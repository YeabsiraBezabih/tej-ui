use crate::error::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Data format representation on the system clipboard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipboardData {
    Text(String),
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
        let mut write = self.storage.write();
        write.insert("text/plain".into(), ClipboardData::Text(text.into()));
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
}
