//! Native desktop GUI runner for the Obsidian Clone using `tej::App` and declarative `View`.

use crate::app_view::{ObsidianState, build_obsidian_view};
use crate::db::VaultDatabase;
use crate::models::NoteItem;
use tej_ui::App;
use tej_ui_hal::WindowConfig;
use tej_ui_layout::Size;

/// Run the interactive native desktop GUI application via declarative `App::new(view).run()`.
pub fn run_desktop_gui(_vault_db: VaultDatabase, notes: Vec<NoteItem>) {
    let state = ObsidianState::new();
    let view = build_obsidian_view(&state, &notes);

    let app = App::new(view).window(WindowConfig {
        title: "Obsidian — tej-ui Native Studio".to_string(),
        size: Size::new(1280.0, 800.0),
        ..Default::default()
    });

    println!("  ✓ Starting declarative native desktop app via tej::App runtime...");
    if let Err(e) = app.run() {
        eprintln!("Application exited with error: {e}");
    }
}

#[allow(dead_code)]
pub(crate) fn insert_char_at(
    content: &mut String,
    line_idx: usize,
    col_idx: usize,
    text: &str,
) -> (usize, usize) {
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    if lines.is_empty() {
        lines.push(String::new());
    }
    let line = line_idx.min(lines.len().saturating_sub(1));
    let col = col_idx.min(lines[line].chars().count());
    let byte_offset: usize = lines[line].chars().take(col).map(|c| c.len_utf8()).sum();
    lines[line].insert_str(byte_offset, text);
    *content = lines.join("\n");
    (line, col + text.chars().count())
}

#[allow(dead_code)]
pub(crate) fn delete_before_cursor(
    content: &mut String,
    line_idx: usize,
    col_idx: usize,
) -> (usize, usize) {
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    if lines.is_empty() {
        return (0, 0);
    }
    let line = line_idx.min(lines.len().saturating_sub(1));
    if col_idx > 0 {
        let col = col_idx.min(lines[line].chars().count());
        let start_byte: usize = lines[line]
            .chars()
            .take(col - 1)
            .map(|c| c.len_utf8())
            .sum();
        let char_len = lines[line].chars().nth(col - 1).map_or(1, |c| c.len_utf8());
        lines[line].drain(start_byte..start_byte + char_len);
        *content = lines.join("\n");
        (line, col - 1)
    } else if line > 0 {
        let prev_len = lines[line - 1].chars().count();
        let cur_line = lines.remove(line);
        lines[line - 1].push_str(&cur_line);
        *content = lines.join("\n");
        (line - 1, prev_len)
    } else {
        (0, 0)
    }
}

#[allow(dead_code)]
pub(crate) fn insert_newline_at(
    content: &mut String,
    line_idx: usize,
    col_idx: usize,
) -> (usize, usize) {
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    if lines.is_empty() {
        lines.push(String::new());
    }
    let line = line_idx.min(lines.len().saturating_sub(1));
    let col = col_idx.min(lines[line].chars().count());
    let byte_offset: usize = lines[line].chars().take(col).map(|c| c.len_utf8()).sum();
    let rest = lines[line].split_off(byte_offset);
    lines.insert(line + 1, rest);
    *content = lines.join("\n");
    (line + 1, 0)
}
