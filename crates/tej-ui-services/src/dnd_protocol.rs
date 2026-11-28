//! Cross-window and OS-native Drag-and-Drop protocol negotiation.

use crate::dnd::DropPayload;
use std::collections::HashMap;
use std::path::PathBuf;

/// Action negotiated between drag source and drop target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DropAction {
    Copy,
    Move,
    Link,
    None,
}

/// Visual preview metadata rendered under the drag cursor.
#[derive(Debug, Clone)]
pub struct DragPreview {
    pub label: String,
    pub width: f32,
    pub height: f32,
    pub cursor_offset_x: f32,
    pub cursor_offset_y: f32,
}

impl DragPreview {
    pub fn new(label: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            label: label.into(),
            width,
            height,
            cursor_offset_x: width * 0.5,
            cursor_offset_y: height * 0.5,
        }
    }
}

/// Active cross-window drag session tracking source, payload, and allowed actions.
#[derive(Debug, Clone)]
pub struct DragSession {
    session_id: u64,
    source_window_id: Option<u64>,
    payload: DropPayload,
    preview: Option<DragPreview>,
    allowed_actions: Vec<DropAction>,
    selected_action: DropAction,
}

impl DragSession {
    pub fn new(
        session_id: u64,
        source_window_id: Option<u64>,
        payload: DropPayload,
        allowed_actions: Vec<DropAction>,
    ) -> Self {
        Self {
            session_id,
            source_window_id,
            payload,
            preview: None,
            allowed_actions,
            selected_action: DropAction::None,
        }
    }

    #[must_use]
    pub fn with_preview(mut self, preview: DragPreview) -> Self {
        self.preview = Some(preview);
        self
    }

    pub fn session_id(&self) -> u64 {
        self.session_id
    }

    pub fn source_window_id(&self) -> Option<u64> {
        self.source_window_id
    }

    pub fn payload(&self) -> &DropPayload {
        &self.payload
    }

    pub fn preview(&self) -> Option<&DragPreview> {
        self.preview.as_ref()
    }

    pub fn allowed_actions(&self) -> &[DropAction] {
        &self.allowed_actions
    }

    pub fn selected_action(&self) -> DropAction {
        self.selected_action
    }

    pub fn set_selected_action(&mut self, action: DropAction) {
        if self.allowed_actions.contains(&action) {
            self.selected_action = action;
        } else {
            self.selected_action = DropAction::None;
        }
    }
}

/// MIME type converter between OS native formats (`text/uri-list`, etc.) and `DropPayload`.
#[derive(Debug, Clone, Default)]
pub struct MimeNegotiator {
    custom_decoders: HashMap<String, String>,
}

impl MimeNegotiator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Converts raw MIME payload from OS into high-level `DropPayload`.
    pub fn parse_os_payload(mime_type: &str, raw_data: &[u8]) -> Option<DropPayload> {
        match mime_type {
            "text/plain" | "text/plain;charset=utf-8" => String::from_utf8(raw_data.to_vec())
                .ok()
                .map(DropPayload::Text),
            "text/uri-list" => {
                let text = String::from_utf8(raw_data.to_vec()).ok()?;
                let paths: Vec<PathBuf> = text
                    .lines()
                    .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
                    .map(|l| {
                        let path = l.trim().strip_prefix("file://").unwrap_or(l.trim());
                        PathBuf::from(path)
                    })
                    .collect();
                if paths.is_empty() {
                    None
                } else {
                    Some(DropPayload::Files(paths))
                }
            }
            custom => Some(DropPayload::Custom {
                type_id: custom.to_string(),
                data: raw_data.to_vec(),
            }),
        }
    }

    /// Registers a custom MIME type alias.
    pub fn register_custom_type(&mut self, mime: impl Into<String>, alias: impl Into<String>) {
        self.custom_decoders.insert(mime.into(), alias.into());
    }
}
