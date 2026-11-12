//! Native desktop GUI runner for the Obsidian Clone using `tej-platform-linux`.
//!
//! Provides ultra-responsive input handling, dynamic window resizing,
//! interactive markdown editing, live compilation/preview, and low CPU/RAM consumption.

use crate::app_view::ObsidianState;
use crate::db::VaultDatabase;
use crate::gui_renderer::{RenderContext, calculate_tab_rects, render_frame};
use crate::models::NoteItem;
use std::f32::consts::PI;
use std::fs;
use std::thread::sleep;
use std::time::Duration;
use tej_hal::{PlatformWindow, UiEvent, WindowConfig};
use tej_layout::{Point, Rect, Size};
use tej_platform_linux::{LinuxSurface, LinuxWindow};

/// Run the interactive native desktop GUI application.
pub fn run_desktop_gui(vault_db: VaultDatabase, mut notes: Vec<NoteItem>) {
    let window_size = Size::new(1280.0, 800.0);
    let mut window = LinuxWindow::new(WindowConfig {
        title: "Obsidian — tej-ui Native Studio".to_string(),
        size: window_size,
        ..Default::default()
    });

    if !window.is_open() {
        println!("  [Headless Environment] No native display server detected.");
        println!("  (Web GUI is accessible at http://127.0.0.1:4000)");
        return;
    }

    println!("  ✓ Native desktop GUI window opened.");
    println!("  ✓ Interactive Markdown Editing enabled: Type directly in editor.");
    println!("  ✓ Toggle Live Preview (LaTeX, Mermaid, SVGs) with [Mode: Preview] or Ctrl+E.");

    let mut surface = LinuxSurface::new(window_size);
    let mut state = ObsidianState::new();
    let mut selected_note_index = 0usize;
    let mut hovered_note = None;
    let mut hovered_tab = None;
    let mut graph_angle = 0.0f32;
    let mut is_preview_mode = false;
    let mut cursor = (0usize, 0usize);
    let mut frame_count = 0u64;

    while window.is_open() {
        frame_count += 1;
        let cursor_blink = (frame_count / 25) % 2 == 0;

        // 0. Ensure surface buffer matches physical window size 1:1
        let win_size = window.size();
        if surface.size() != win_size && win_size.width > 0.0 && win_size.height > 0.0 {
            surface.resize(win_size);
        }
        #[allow(clippy::cast_precision_loss)]
        let win_w = surface.width() as f32;

        // 1. Process Hardware Events with zero-latency dispatch
        for event in window.poll_events() {
            match event {
                UiEvent::Resize { width, height } => {
                    surface.resize(Size::new(width, height));
                }
                UiEvent::TextInput { text } => {
                    if !is_preview_mode && state.active_tab.get() == 0 && !notes.is_empty() {
                        cursor = insert_char_at(
                            &mut notes[selected_note_index].content,
                            cursor.0,
                            cursor.1,
                            &text,
                        );
                    }
                }
                UiEvent::MouseMove { position } => {
                    update_hover_states(
                        position,
                        win_w,
                        &state,
                        notes.len(),
                        &mut hovered_note,
                        &mut hovered_tab,
                    );
                }
                UiEvent::MouseDown { position, .. } => {
                    handle_click(
                        position,
                        win_w,
                        &mut state,
                        &notes,
                        &mut selected_note_index,
                        &mut is_preview_mode,
                        &mut cursor,
                    );
                }
                UiEvent::KeyDown { key_code, .. } => match key_code {
                    27 => return,                             // Esc key to exit
                    49 => state.active_tab.set(0),            // '1' Editor tab
                    50 => state.active_tab.set(1),            // '2' 3D Graph tab
                    51 => state.active_tab.set(2),            // '3' Database Records tab
                    9 => state.toggle_sidebar(),              // Tab key toggles sidebar
                    69 => is_preview_mode = !is_preview_mode, // 'E' toggles Edit/Preview mode
                    8 => {
                        // Backspace
                        if !is_preview_mode && state.active_tab.get() == 0 && !notes.is_empty() {
                            cursor = delete_before_cursor(
                                &mut notes[selected_note_index].content,
                                cursor.0,
                                cursor.1,
                            );
                        }
                    }
                    13 => {
                        // Enter / Return
                        if !is_preview_mode && state.active_tab.get() == 0 && !notes.is_empty() {
                            cursor = insert_newline_at(
                                &mut notes[selected_note_index].content,
                                cursor.0,
                                cursor.1,
                            );
                        }
                    }
                    37 => {
                        // Left Arrow
                        if cursor.1 > 0 {
                            cursor.1 -= 1;
                        } else if cursor.0 > 0 {
                            cursor.0 -= 1;
                            cursor.1 = line_length(&notes[selected_note_index].content, cursor.0);
                        }
                    }
                    39 => {
                        // Right Arrow
                        let len = line_length(&notes[selected_note_index].content, cursor.0);
                        if cursor.1 < len {
                            cursor.1 += 1;
                        } else if cursor.0 + 1 < total_lines(&notes[selected_note_index].content) {
                            cursor.0 += 1;
                            cursor.1 = 0;
                        }
                    }
                    38 if cursor.0 > 0 => {
                        // Up Arrow
                        cursor.0 -= 1;
                        cursor.1 = cursor
                            .1
                            .min(line_length(&notes[selected_note_index].content, cursor.0));
                    }
                    40 if cursor.0 + 1 < total_lines(&notes[selected_note_index].content) => {
                        // Down Arrow
                        cursor.0 += 1;
                        cursor.1 = cursor
                            .1
                            .min(line_length(&notes[selected_note_index].content, cursor.0));
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // 2. Physics & State Updates
        let is_animating = !state.sidebar_width_spring.is_at_rest();
        if is_animating {
            state.sidebar_width_spring.step(1.0 / 60.0);
        }

        let is_graph_active = state.active_tab.get() == 1;
        if is_graph_active {
            graph_angle += 0.018;
            if graph_angle > 2.0 * PI {
                graph_angle -= 2.0 * PI;
            }
        }

        // 3. Read Process Memory Telemetry
        let ram_mb = read_process_rss_mb();

        // 4. Render Complete Frame with Antialiased Typography
        let render_ctx = RenderContext {
            state: &state,
            notes: &notes,
            selected_idx: selected_note_index,
            hovered_note,
            hovered_tab,
            graph_angle,
            ram_mb,
            vault_db: &vault_db,
            is_preview: is_preview_mode,
            cursor,
            cursor_blink,
        };
        render_frame(&mut surface, &render_ctx);

        // 5. Blit to Native Platform Window (1:1 physical pixels)
        window.update_surface(&surface);

        // 6. Adaptive Resource-Efficient Sleeping
        if is_graph_active || is_animating || !is_preview_mode {
            sleep(Duration::from_millis(16)); // 60 FPS for active interaction
        } else {
            sleep(Duration::from_millis(25)); // 40 FPS adaptive power save
        }
    }
}

fn update_hover_states(
    pos: Point,
    win_w: f32,
    state: &ObsidianState,
    total_notes: usize,
    hovered_note: &mut Option<usize>,
    hovered_tab: &mut Option<usize>,
) {
    *hovered_tab = None;
    let tab_rects = calculate_tab_rects(win_w);
    for (i, rect) in tab_rects.iter().enumerate() {
        if rect.contains(pos) {
            *hovered_tab = Some(i);
            break;
        }
    }

    *hovered_note = None;
    let sidebar_w = state.sidebar_width_spring.get();
    if pos.x >= 8.0 && pos.x <= (sidebar_w - 8.0) && pos.y >= 104.0 {
        for i in 0..total_notes {
            let y = 104.0 + (i as f32) * 34.0;
            let rect = Rect::new(8.0, y, sidebar_w - 16.0, 30.0);
            if rect.contains(pos) {
                *hovered_note = Some(i);
                break;
            }
        }
    }
}

fn handle_click(
    pos: Point,
    win_w: f32,
    state: &mut ObsidianState,
    notes: &[NoteItem],
    selected_idx: &mut usize,
    is_preview: &mut bool,
    cursor: &mut (usize, usize),
) {
    // 1. Header Tab Clicks
    let tab_rects = calculate_tab_rects(win_w);
    for (i, rect) in tab_rects.iter().enumerate() {
        if rect.contains(pos) {
            state.active_tab.set(i);
            return;
        }
    }

    // 2. Sidebar Toggle Button Click
    let toggle_rect = Rect::new(8.0, 6.0, 28.0, 26.0);
    if toggle_rect.contains(pos) {
        state.toggle_sidebar();
        return;
    }

    // 3. Sidebar Note Items Hit Test
    let sidebar_w = state.sidebar_width_spring.get();
    if pos.x >= 8.0 && pos.x <= (sidebar_w - 8.0) && pos.y >= 104.0 {
        for (i, note) in notes.iter().enumerate() {
            let y = 104.0 + (i as f32) * 34.0;
            let rect = Rect::new(8.0, y, sidebar_w - 16.0, 30.0);
            if rect.contains(pos) {
                *selected_idx = i;
                state.selected_note_id.set(Some(note.id.to_string()));
                state.active_tab.set(0);
                *cursor = (0, 0);
                return;
            }
        }
    }

    // 4. Editor Mode Toggle Button (Edit vs Preview)
    let content_x = sidebar_w + 14.0;
    let content_w = (win_w - content_x - 14.0).max(80.0);
    let btn_x = (content_x + content_w - 170.0).max(content_x + 180.0);
    let btn_rect = Rect::new(btn_x, 50.0, 160.0, 26.0);
    if btn_rect.contains(pos) {
        *is_preview = !*is_preview;
        return;
    }

    // 5. Click in text editor to position cursor
    if !*is_preview && pos.x >= (content_x + 36.0) && pos.y >= 110.0 {
        let clicked_line = ((pos.y - 110.0) / 20.0) as usize;
        let clicked_col = ((pos.x - content_x - 38.0).max(0.0) / 7.5) as usize;
        if *selected_idx < notes.len() {
            let total = total_lines(&notes[*selected_idx].content);
            let line = clicked_line.min(total.saturating_sub(1));
            let max_col = line_length(&notes[*selected_idx].content, line);
            *cursor = (line, clicked_col.min(max_col));
        }
    }
}

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

fn line_length(content: &str, line_idx: usize) -> usize {
    content
        .lines()
        .nth(line_idx)
        .map_or(0, |l| l.chars().count())
}

fn total_lines(content: &str) -> usize {
    content.lines().count().max(1)
}

fn read_process_rss_mb() -> f32 {
    fs::read_to_string("/proc/self/statm")
        .ok()
        .and_then(|content| {
            let mut parts = content.split_whitespace();
            let _total_pages = parts.next()?;
            let rss_pages: u64 = parts.next()?.parse().ok()?;
            Some((rss_pages as f32) * 4096.0 / (1024.0 * 1024.0))
        })
        .unwrap_or(16.0)
}
