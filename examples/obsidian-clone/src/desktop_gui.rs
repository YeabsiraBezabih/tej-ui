//! Native desktop GUI runner for the Obsidian Clone using `tej-platform-linux`.
//!
//! Provides ultra-responsive input handling, pixel-accurate hit-testing,
//! silky spring physics, and low CPU/RAM resource consumption.

use crate::app_view::ObsidianState;
use crate::db::VaultDatabase;
use crate::gui_renderer::{RenderContext, TAB_RECTS, render_frame};
use crate::models::NoteItem;
use std::f32::consts::PI;
use std::fs;
use std::thread::sleep;
use std::time::Duration;
use tej_hal::{PlatformWindow, UiEvent, WindowConfig};
use tej_layout::{Point, Rect, Size};
use tej_platform_linux::{LinuxSurface, LinuxWindow};

/// Run the interactive native desktop GUI application.
pub fn run_desktop_gui(vault_db: VaultDatabase, notes: Vec<NoteItem>) {
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

    println!("  ✓ Native desktop GUI window opened (1280x800).");
    println!(
        "  ✓ Interactivity active: Click notes in sidebar, switch tabs, or press 1-3 / Tab / Esc."
    );

    let mut surface = LinuxSurface::new(window_size);
    let mut state = ObsidianState::new();
    let mut selected_note_index = 0usize;
    let mut hovered_note = None;
    let mut hovered_tab = None;
    let mut graph_angle = 0.0f32;

    while window.is_open() {
        // 1. Process Hardware Events with zero-latency dispatch
        for event in window.poll_events() {
            match event {
                UiEvent::MouseMove { position } => {
                    update_hover_states(
                        position,
                        &state,
                        notes.len(),
                        &mut hovered_note,
                        &mut hovered_tab,
                    );
                }
                UiEvent::MouseDown { position, .. } => {
                    handle_click(position, &mut state, &notes, &mut selected_note_index);
                }
                UiEvent::KeyDown { key_code, .. } => match key_code {
                    27 => return,                  // Esc key to exit
                    49 => state.active_tab.set(0), // '1' Editor tab
                    50 => state.active_tab.set(1), // '2' 3D Graph tab
                    51 => state.active_tab.set(2), // '3' Database Records tab
                    9 => state.toggle_sidebar(),   // Tab key toggles sidebar
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
        };
        render_frame(&mut surface, &render_ctx);

        // 5. Blit to Native Platform Window
        window.update_surface(&surface);

        // 6. Adaptive Resource-Efficient Sleeping (Near 0% Idle CPU)
        if is_graph_active || is_animating {
            sleep(Duration::from_millis(16)); // 60 FPS for active motion
        } else {
            sleep(Duration::from_millis(25)); // 40 FPS adaptive power save
        }
    }
}

fn update_hover_states(
    pos: Point,
    state: &ObsidianState,
    total_notes: usize,
    hovered_note: &mut Option<usize>,
    hovered_tab: &mut Option<usize>,
) {
    // Check Tabs hover
    *hovered_tab = None;
    for (i, &rect) in TAB_RECTS.iter().enumerate() {
        if rect.contains(pos) {
            *hovered_tab = Some(i);
            break;
        }
    }

    // Check Sidebar Notes hover
    *hovered_note = None;
    let sidebar_w = state.sidebar_width_spring.get();
    if pos.x >= 12.0 && pos.x <= (sidebar_w - 12.0) && pos.y >= 110.0 {
        let note_idx = ((pos.y - 110.0) / 36.0) as usize;
        let rect_y = 110.0 + (note_idx as f32) * 36.0;
        if note_idx < total_notes && pos.y >= rect_y && pos.y <= (rect_y + 32.0) {
            *hovered_note = Some(note_idx);
        }
    }
}

fn handle_click(
    pos: Point,
    state: &mut ObsidianState,
    notes: &[NoteItem],
    selected_idx: &mut usize,
) {
    // 1. Header Tab Clicks
    for (i, &rect) in TAB_RECTS.iter().enumerate() {
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
    if pos.x >= 12.0 && pos.x <= (sidebar_w - 12.0) && pos.y >= 110.0 {
        let note_idx = ((pos.y - 110.0) / 36.0) as usize;
        let rect_y = 110.0 + (note_idx as f32) * 36.0;
        if note_idx < notes.len() && pos.y >= rect_y && pos.y <= (rect_y + 32.0) {
            *selected_idx = note_idx;
            state
                .selected_note_id
                .set(Some(notes[note_idx].id.to_string()));
            state.active_tab.set(0); // Instantly switch to editor
        }
    }
}

/// Read resident set size (RSS) in MB from Linux procfs.
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
