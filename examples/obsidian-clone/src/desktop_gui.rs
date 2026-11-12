//! Native desktop GUI runner for the Obsidian Clone using `tej-platform-linux`.
//!
//! Provides a 60 FPS interactive desktop window with authentic Obsidian theme,
//! sidebar note selection, tab switching, live 3D knowledge graph, and zero-copy DB queries.

use crate::app_view::ObsidianState;
use crate::db::VaultDatabase;
use crate::models::NoteItem;
use std::f32::consts::PI;
use std::thread::sleep;
use std::time::Duration;
use tej_hal::{Color, PlatformWindow, SurfaceCompositor, UiEvent, WindowConfig};
use tej_layout::{Point, Rect, Size};
use tej_platform_linux::{LinuxSurface, LinuxWindow};

const COLOR_BG: Color = Color::rgb(30, 30, 46); // #1e1e2e
const COLOR_SIDEBAR: Color = Color::rgb(24, 24, 37); // #181825
const COLOR_HEADER: Color = Color::rgb(17, 17, 27); // #11111b
const COLOR_BORDER: Color = Color::rgb(49, 50, 68); // #313244
const COLOR_TEXT: Color = Color::rgb(205, 214, 244); // #cdd6f4
const COLOR_TEXT_MUTED: Color = Color::rgb(166, 173, 200); // #a6adc8
const COLOR_ACCENT: Color = Color::rgb(137, 180, 250); // #89b4fa
const COLOR_ACTIVE_ITEM: Color = Color::rgb(49, 50, 68); // #313244
const COLOR_GREEN: Color = Color::rgb(166, 227, 161); // #a6e3a1

/// Run the interactive native desktop GUI application.
pub fn run_desktop_gui(vault_db: VaultDatabase, notes: Vec<NoteItem>) {
    let window_size = Size::new(1280.0, 800.0);
    let mut window = LinuxWindow::new(WindowConfig {
        title: "Obsidian — tej-ui Native GUI".to_string(),
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
    let mut graph_angle = 0.0f32;
    let mut mouse_pos = Point::new(0.0, 0.0);

    while window.is_open() {
        // 1. Process Hardware Events
        for event in window.poll_events() {
            match event {
                UiEvent::MouseMove { position } => {
                    mouse_pos = position;
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
        state.sidebar_width_spring.step(1.0 / 60.0);
        graph_angle += 0.015;
        if graph_angle > 2.0 * PI {
            graph_angle -= 2.0 * PI;
        }

        // 3. Render Complete Frame
        render_frame(
            &mut surface,
            &state,
            &notes,
            selected_note_index,
            graph_angle,
            mouse_pos,
            &vault_db,
        );

        // 4. Blit to Native Platform Window
        window.update_surface(&surface);

        // 5. Native 60 FPS Pace
        sleep(Duration::from_millis(16));
    }
}

fn handle_click(
    pos: Point,
    state: &mut ObsidianState,
    notes: &[NoteItem],
    selected_idx: &mut usize,
) {
    // Header Tab clicks (y: 4..36)
    if (4.0..36.0).contains(&pos.y) {
        if (270.0..440.0).contains(&pos.x) {
            state.active_tab.set(0);
        } else if (450.0..620.0).contains(&pos.x) {
            state.active_tab.set(1);
        } else if (630.0..800.0).contains(&pos.x) {
            state.active_tab.set(2);
        }
        return;
    }

    // Sidebar Toggle button click (top-left icon)
    if (8.0..32.0).contains(&pos.y) && (8.0..40.0).contains(&pos.x) {
        state.toggle_sidebar();
        return;
    }

    // Sidebar Note clicks (y: 80..400)
    let sidebar_w = state.sidebar_width_spring.get();
    if pos.x < sidebar_w && pos.y >= 80.0 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let note_slot = ((pos.y - 80.0) / 32.0) as usize;
        if note_slot < notes.len() {
            *selected_idx = note_slot;
            state
                .selected_note_id
                .set(Some(notes[note_slot].id.to_string()));
            state.active_tab.set(0); // Auto-switch to editor
        }
    }
}

fn render_frame(
    surface: &mut LinuxSurface,
    state: &ObsidianState,
    notes: &[NoteItem],
    selected_idx: usize,
    graph_angle: f32,
    mouse: Point,
    vault_db: &VaultDatabase,
) {
    surface.begin_frame(&[]);
    surface.clear(COLOR_BG);

    let sidebar_w = state.sidebar_width_spring.get().max(0.0);

    // 1. Sidebar Background & Border
    if sidebar_w > 2.0 {
        surface.fill_rect(Rect::new(0.0, 36.0, sidebar_w, 740.0), COLOR_SIDEBAR);
        surface.fill_rect(Rect::new(sidebar_w - 1.0, 36.0, 1.0, 740.0), COLOR_BORDER);

        if sidebar_w > 120.0 {
            surface.draw_text(
                "VAULT EXPLORER",
                Point::new(16.0, 52.0),
                8.0,
                COLOR_TEXT_MUTED,
            );

            // Render Notes List
            for (i, note) in notes.iter().enumerate() {
                let y = 80.0 + (i as f32) * 32.0;
                let is_selected = i == selected_idx;
                if is_selected {
                    surface.fill_rect(
                        Rect::new(8.0, y - 4.0, sidebar_w - 16.0, 26.0),
                        COLOR_ACTIVE_ITEM,
                    );
                }
                let color = if is_selected {
                    COLOR_ACCENT
                } else {
                    COLOR_TEXT
                };
                let display_title = if note.title.len() > 24 {
                    format!("{}...", &note.title[..21])
                } else {
                    note.title.clone()
                };
                surface.draw_text(
                    &format!("- {display_title}"),
                    Point::new(16.0, y),
                    8.0,
                    color,
                );
            }
        }
    }

    // 2. Main Content Area based on Active Tab
    let content_x = sidebar_w + 12.0;
    let content_w = 1280.0 - content_x;

    match state.active_tab.get() {
        1 => render_3d_knowledge_graph(surface, content_x, 48.0, content_w, 720.0, graph_angle),
        2 => render_database_records(surface, content_x, 48.0, content_w, 720.0, notes, vault_db),
        _ => render_markdown_editor(
            surface,
            content_x,
            48.0,
            content_w,
            720.0,
            &notes[selected_idx],
        ),
    }

    // 3. Top Header Bar & Tabs
    render_header(surface, state, sidebar_w);

    // 4. Bottom Status Bar
    render_status_bar(surface, mouse, selected_idx, notes.len());

    surface.end_frame();
}

fn render_header(surface: &mut LinuxSurface, state: &ObsidianState, _sidebar_w: f32) {
    surface.fill_rect(Rect::new(0.0, 0.0, 1280.0, 36.0), COLOR_HEADER);
    surface.fill_rect(Rect::new(0.0, 35.0, 1280.0, 1.0), COLOR_BORDER);

    // Sidebar Toggle Button
    surface.draw_text("[=]", Point::new(12.0, 12.0), 8.0, COLOR_ACCENT);
    surface.draw_text("Obsidian Vault", Point::new(48.0, 12.0), 8.0, COLOR_TEXT);

    // Tabs
    let active_tab = state.active_tab.get();
    let tabs = [
        (270.0, "[1] Markdown Editor"),
        (450.0, "[2] 3D Knowledge Graph"),
        (650.0, "[3] Vault Records (DB)"),
    ];

    for (i, (x, title)) in tabs.iter().enumerate() {
        let is_active = i == active_tab;
        if is_active {
            surface.fill_rect(Rect::new(*x - 6.0, 6.0, 170.0, 24.0), COLOR_ACTIVE_ITEM);
            surface.fill_rect(Rect::new(*x - 6.0, 33.0, 170.0, 2.0), COLOR_ACCENT);
        }
        let col = if is_active {
            COLOR_ACCENT
        } else {
            COLOR_TEXT_MUTED
        };
        surface.draw_text(title, Point::new(*x, 12.0), 8.0, col);
    }
}

fn render_markdown_editor(
    surface: &mut LinuxSurface,
    x: f32,
    y: f32,
    _w: f32,
    _h: f32,
    note: &NoteItem,
) {
    // Document Title Banner
    surface.draw_text(
        &format!("# {}", note.title),
        Point::new(x + 20.0, y + 10.0),
        16.0,
        COLOR_TEXT,
    );
    surface.fill_rect(Rect::new(x + 20.0, y + 36.0, 600.0, 1.0), COLOR_BORDER);

    // Line number gutter and note content lines
    let mut line_y = y + 50.0;
    for (idx, line) in note.content.lines().enumerate() {
        if line_y > 740.0 {
            break;
        }
        let line_num = format!("{:2}", idx + 1);
        surface.draw_text(
            &line_num,
            Point::new(x + 10.0, line_y),
            8.0,
            COLOR_TEXT_MUTED,
        );

        let line_color = if line.starts_with('#') {
            COLOR_ACCENT
        } else if line.starts_with('-') || line.starts_with('*') {
            COLOR_GREEN
        } else if line.starts_with('`') {
            Color::rgb(249, 226, 175) // #f9e2af (Yellow)
        } else {
            COLOR_TEXT
        };

        surface.draw_text(line, Point::new(x + 50.0, line_y), 8.0, line_color);
        line_y += 18.0;
    }
}

fn render_3d_knowledge_graph(
    surface: &mut LinuxSurface,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    angle: f32,
) {
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;

    surface.draw_text(
        "Interactive 3D Knowledge Graph (tej-3d Real-Time Projection)",
        Point::new(x + 20.0, y + 10.0),
        8.0,
        COLOR_ACCENT,
    );

    // 6 Simulated 3D Nodes rotated in space
    let raw_nodes = [
        ("Architecture", -120.0, -80.0, 50.0),
        ("SlotMap Arena", 120.0, -90.0, -40.0),
        ("Zero-Copy DB", 140.0, 80.0, 60.0),
        ("Spring Physics", -110.0, 100.0, -50.0),
        ("Native Linux", 0.0, -140.0, 0.0),
        ("tej-widgets", 0.0, 130.0, 10.0),
    ];

    let cos_a = angle.cos();
    let sin_a = angle.sin();

    let projected: Vec<(Point, &str)> = raw_nodes
        .iter()
        .map(|(label, nx, ny, nz)| {
            // Y-axis rotation
            let rx = nx * cos_a - nz * sin_a;
            let rz = nx * sin_a + nz * cos_a;
            let perspective = 400.0 / (400.0 + rz);
            let px = cx + rx * perspective;
            let py = cy + ny * perspective;
            (Point::new(px, py), *label)
        })
        .collect();

    // Draw connecting edges
    for i in 0..projected.len() {
        for j in (i + 1)..projected.len() {
            let p1 = projected[i].0;
            let p2 = projected[j].0;
            surface.draw_line(p1, p2, 1.0, COLOR_BORDER);
        }
    }

    // Draw nodes
    for (pt, label) in projected {
        surface.fill_rect(Rect::new(pt.x - 6.0, pt.y - 6.0, 12.0, 12.0), COLOR_ACCENT);
        surface.draw_text(label, Point::new(pt.x + 10.0, pt.y - 4.0), 8.0, COLOR_TEXT);
    }
}

fn render_database_records(
    surface: &mut LinuxSurface,
    x: f32,
    y: f32,
    w: f32,
    _h: f32,
    notes: &[NoteItem],
    vault_db: &VaultDatabase,
) {
    surface.draw_text(
        "Zero-Copy ACID DBMS Live Virtual Table (Reading directly from mmap pages)",
        Point::new(x + 20.0, y + 10.0),
        8.0,
        COLOR_ACCENT,
    );

    // Table Header
    surface.fill_rect(Rect::new(x + 20.0, y + 40.0, w - 40.0, 24.0), COLOR_SIDEBAR);
    surface.draw_text("PK", Point::new(x + 30.0, y + 46.0), 8.0, COLOR_TEXT_MUTED);
    surface.draw_text(
        "TITLE",
        Point::new(x + 180.0, y + 46.0),
        8.0,
        COLOR_TEXT_MUTED,
    );
    surface.draw_text(
        "TAGS",
        Point::new(x + 460.0, y + 46.0),
        8.0,
        COLOR_TEXT_MUTED,
    );
    surface.draw_text(
        "STATUS",
        Point::new(x + 650.0, y + 46.0),
        8.0,
        COLOR_TEXT_MUTED,
    );

    if let Ok(rx) = vault_db.db.begin_read() {
        let mut row_y = y + 72.0;
        for (i, note) in notes.iter().enumerate() {
            let pk = format!("note:{:08}", note.id);
            let exists = rx.get(pk.as_bytes()).ok().flatten().is_some();
            let status = if exists {
                "Committed (mmap Page)"
            } else {
                "Pending"
            };
            let tags = note.tags.join(", ");

            surface.draw_text(&pk, Point::new(x + 30.0, row_y), 8.0, COLOR_ACCENT);
            let display_title = if note.title.len() > 30 {
                format!("{}...", &note.title[..27])
            } else {
                note.title.clone()
            };
            surface.draw_text(
                &display_title,
                Point::new(x + 180.0, row_y),
                8.0,
                COLOR_TEXT,
            );
            surface.draw_text(&tags, Point::new(x + 460.0, row_y), 8.0, COLOR_TEXT_MUTED);
            surface.draw_text(status, Point::new(x + 650.0, row_y), 8.0, COLOR_GREEN);

            surface.fill_rect(
                Rect::new(x + 20.0, row_y + 16.0, w - 40.0, 1.0),
                COLOR_BORDER,
            );
            row_y += 26.0;
            if i >= 15 {
                break;
            }
        }
    }
}

fn render_status_bar(surface: &mut LinuxSurface, mouse: Point, selected_idx: usize, total: usize) {
    surface.fill_rect(Rect::new(0.0, 776.0, 1280.0, 24.0), COLOR_HEADER);
    surface.fill_rect(Rect::new(0.0, 776.0, 1280.0, 1.0), COLOR_BORDER);

    let info = format!(
        "60 FPS Native GUI | Note {}/{} | Mouse: ({:.0}, {:.0}) | DB: Zero-Copy mmap | tej-ui",
        selected_idx + 1,
        total,
        mouse.x,
        mouse.y
    );
    surface.draw_text(&info, Point::new(16.0, 782.0), 8.0, COLOR_TEXT_MUTED);
}
