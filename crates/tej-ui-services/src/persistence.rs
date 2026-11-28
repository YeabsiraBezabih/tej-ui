//! Window geometry and workspace state persistence across application launches.

use crate::error::{Result, ServiceError};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Serialized geometry and screen placement of a top-level window.
#[derive(Clone, PartialEq, Debug)]
pub struct WindowGeometry {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub is_maximized: bool,
    pub is_fullscreen: bool,
    pub display_id: u32,
}

impl Default for WindowGeometry {
    fn default() -> Self {
        Self {
            x: 100.0,
            y: 100.0,
            width: 1280.0,
            height: 800.0,
            is_maximized: false,
            is_fullscreen: false,
            display_id: 0,
        }
    }
}

/// State of an individual workspace tab.
#[derive(Clone, PartialEq, Debug)]
pub struct TabState {
    pub id: String,
    pub title: String,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub is_pinned: bool,
}

impl TabState {
    /// Create a new tab state.
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            scroll_x: 0.0,
            scroll_y: 0.0,
            is_pinned: false,
        }
    }
}

/// Workspace layout containing open tabs, split positions, and unsaved drafts.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct WorkspaceLayout {
    pub active_tab_index: usize,
    pub tabs: Vec<TabState>,
    pub split_ratio: f32,
    pub drafts: HashMap<String, String>,
}

/// Root persistent application state snapshot.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct WorkspaceStateStore {
    pub window: WindowGeometry,
    pub workspace: WorkspaceLayout,
}

impl WorkspaceStateStore {
    /// Create a new empty workspace state store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Serialize this state to a clean, deterministic JSON string.
    #[must_use]
    pub fn to_json_string(&self) -> String {
        use std::fmt::Write;
        let mut out = String::with_capacity(1024);
        out.push_str("{\n");

        // Window geometry
        out.push_str("  \"window\": {\n");
        let _ = writeln!(out, "    \"x\": {},", self.window.x);
        let _ = writeln!(out, "    \"y\": {},", self.window.y);
        let _ = writeln!(out, "    \"width\": {},", self.window.width);
        let _ = writeln!(out, "    \"height\": {},", self.window.height);
        let _ = writeln!(out, "    \"is_maximized\": {},", self.window.is_maximized);
        let _ = writeln!(out, "    \"is_fullscreen\": {},", self.window.is_fullscreen);
        let _ = writeln!(out, "    \"display_id\": {}", self.window.display_id);
        out.push_str("  },\n");

        // Workspace
        out.push_str("  \"workspace\": {\n");
        let _ = writeln!(
            out,
            "    \"active_tab_index\": {},",
            self.workspace.active_tab_index
        );
        let _ = writeln!(out, "    \"split_ratio\": {},", self.workspace.split_ratio);

        // Tabs
        out.push_str("    \"tabs\": [\n");
        for (i, tab) in self.workspace.tabs.iter().enumerate() {
            out.push_str("      {\n");
            let _ = writeln!(out, "        \"id\": \"{}\",", escape_json(&tab.id));
            let _ = writeln!(out, "        \"title\": \"{}\",", escape_json(&tab.title));
            let _ = writeln!(out, "        \"scroll_x\": {},", tab.scroll_x);
            let _ = writeln!(out, "        \"scroll_y\": {},", tab.scroll_y);
            let _ = writeln!(out, "        \"is_pinned\": {}", tab.is_pinned);
            if i + 1 < self.workspace.tabs.len() {
                out.push_str("      },\n");
            } else {
                out.push_str("      }\n");
            }
        }
        out.push_str("    ],\n");

        // Drafts
        out.push_str("    \"drafts\": {\n");
        let drafts_len = self.workspace.drafts.len();
        for (i, (k, v)) in self.workspace.drafts.iter().enumerate() {
            let suffix = if i + 1 < drafts_len { "," } else { "" };
            let _ = writeln!(
                out,
                "      \"{}\": \"{}\"{suffix}",
                escape_json(k),
                escape_json(v),
            );
        }
        out.push_str("    }\n");

        out.push_str("  }\n");
        out.push('}');
        out
    }

    /// Parse a serialized JSON string back into a `WorkspaceStateStore`.
    pub fn parse_json(json: &str) -> Result<Self> {
        let mut store = Self::default();

        // Extract window section
        if let Some(win_block) = extract_block(json, "\"window\":") {
            if let Some(val) = extract_f32(&win_block, "\"x\":") {
                store.window.x = val;
            }
            if let Some(val) = extract_f32(&win_block, "\"y\":") {
                store.window.y = val;
            }
            if let Some(val) = extract_f32(&win_block, "\"width\":") {
                store.window.width = val;
            }
            if let Some(val) = extract_f32(&win_block, "\"height\":") {
                store.window.height = val;
            }
            if let Some(val) = extract_bool(&win_block, "\"is_maximized\":") {
                store.window.is_maximized = val;
            }
            if let Some(val) = extract_bool(&win_block, "\"is_fullscreen\":") {
                store.window.is_fullscreen = val;
            }
            if let Some(val) = extract_u32(&win_block, "\"display_id\":") {
                store.window.display_id = val;
            }
        }

        // Extract workspace section
        if let Some(ws_block) = extract_block(json, "\"workspace\":") {
            if let Some(val) = extract_usize(&ws_block, "\"active_tab_index\":") {
                store.workspace.active_tab_index = val;
            }
            if let Some(val) = extract_f32(&ws_block, "\"split_ratio\":") {
                store.workspace.split_ratio = val;
            }

            // Extract tabs array
            if let Some(tabs_block) = extract_array(&ws_block, "\"tabs\":") {
                store.workspace.tabs = parse_tabs(&tabs_block);
            }

            // Extract drafts map
            if let Some(drafts_block) = extract_block(&ws_block, "\"drafts\":") {
                store.workspace.drafts = parse_key_value_map(&drafts_block);
            }
        }

        Ok(store)
    }

    /// Atomically persist state to disk via a temporary file.
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let serialized = self.to_json_string();
        let tmp_path = path.with_extension("tmp");

        fs::write(&tmp_path, serialized)
            .map_err(|e| ServiceError::PersistenceError(format!("Failed to write state: {e}")))?;
        fs::rename(&tmp_path, path)
            .map_err(|e| ServiceError::PersistenceError(format!("Failed to commit state: {e}")))?;

        Ok(())
    }

    /// Load persisted state from a file.
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|e| {
            ServiceError::PersistenceError(format!("Failed to read state file: {e}"))
        })?;
        Self::parse_json(&text)
    }
}

fn escape_json(raw: &str) -> String {
    raw.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn unescape_json(raw: &str) -> String {
    raw.replace("\\t", "\t")
        .replace("\\r", "\r")
        .replace("\\n", "\n")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

fn extract_block(text: &str, key: &str) -> Option<String> {
    let key_idx = text.find(key)?;
    let brace_idx = text[key_idx..].find('{')? + key_idx;
    let mut depth = 0;
    let mut end_idx = 0;

    for (i, ch) in text[brace_idx..].char_indices() {
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                end_idx = brace_idx + i + 1;
                break;
            }
        }
    }

    if end_idx > brace_idx {
        Some(text[brace_idx..end_idx].to_string())
    } else {
        None
    }
}

fn extract_array(text: &str, key: &str) -> Option<String> {
    let key_idx = text.find(key)?;
    let bracket_idx = text[key_idx..].find('[')? + key_idx;
    let mut depth = 0;
    let mut end_idx = 0;

    for (i, ch) in text[bracket_idx..].char_indices() {
        if ch == '[' {
            depth += 1;
        } else if ch == ']' {
            depth -= 1;
            if depth == 0 {
                end_idx = bracket_idx + i + 1;
                break;
            }
        }
    }

    if end_idx > bracket_idx {
        Some(text[bracket_idx..end_idx].to_string())
    } else {
        None
    }
}

fn extract_f32(text: &str, key: &str) -> Option<f32> {
    let key_idx = text.find(key)?;
    let after = text[key_idx + key.len()..].trim_start();
    let val_str = after.split(&[',', '\n', '}', ']'][..]).next()?.trim();
    val_str.parse::<f32>().ok()
}

fn extract_u32(text: &str, key: &str) -> Option<u32> {
    let key_idx = text.find(key)?;
    let after = text[key_idx + key.len()..].trim_start();
    let val_str = after.split(&[',', '\n', '}', ']'][..]).next()?.trim();
    val_str.parse::<u32>().ok()
}

fn extract_usize(text: &str, key: &str) -> Option<usize> {
    let key_idx = text.find(key)?;
    let after = text[key_idx + key.len()..].trim_start();
    let val_str = after.split(&[',', '\n', '}', ']'][..]).next()?.trim();
    val_str.parse::<usize>().ok()
}

fn extract_bool(text: &str, key: &str) -> Option<bool> {
    let key_idx = text.find(key)?;
    let after = text[key_idx + key.len()..].trim_start();
    let val_str = after.split(&[',', '\n', '}', ']'][..]).next()?.trim();
    match val_str {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_tabs(array_str: &str) -> Vec<TabState> {
    let mut tabs = Vec::new();
    let mut remaining = array_str;

    while let Some(open) = remaining.find('{') {
        let close = match remaining[open..].find('}') {
            Some(c) => open + c,
            None => break,
        };
        let item_str = &remaining[open..=close];

        let id = extract_quoted_string(item_str, "\"id\":").unwrap_or_default();
        let title = extract_quoted_string(item_str, "\"title\":").unwrap_or_default();
        let scroll_x = extract_f32(item_str, "\"scroll_x\":").unwrap_or(0.0);
        let scroll_y = extract_f32(item_str, "\"scroll_y\":").unwrap_or(0.0);
        let is_pinned = extract_bool(item_str, "\"is_pinned\":").unwrap_or(false);

        tabs.push(TabState {
            id,
            title,
            scroll_x,
            scroll_y,
            is_pinned,
        });

        remaining = &remaining[close + 1..];
    }

    tabs
}

fn parse_key_value_map(block: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in block.lines() {
        let trimmed = line.trim().trim_end_matches(',');
        if let Some(colon) = trimmed.find(':') {
            let k_part = trimmed[..colon].trim();
            let v_part = trimmed[colon + 1..].trim();
            if k_part.starts_with('"')
                && k_part.ends_with('"')
                && v_part.starts_with('"')
                && v_part.ends_with('"')
            {
                let key = unescape_json(&k_part[1..k_part.len() - 1]);
                let val = unescape_json(&v_part[1..v_part.len() - 1]);
                map.insert(key, val);
            }
        }
    }
    map
}

fn extract_quoted_string(text: &str, key: &str) -> Option<String> {
    let key_idx = text.find(key)?;
    let after = text[key_idx + key.len()..].trim_start();
    if let Some(stripped) = after.strip_prefix('"') {
        let quote_end = stripped.find('"')?;
        Some(unescape_json(&stripped[..quote_end]))
    } else {
        None
    }
}
