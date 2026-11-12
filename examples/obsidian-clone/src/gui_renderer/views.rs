//! Content view rendering passes for editor, 3D graph, and database virtual table.

use super::theme::{
    COLOR_ACCENT, COLOR_ACTIVE, COLOR_BORDER, COLOR_CARD, COLOR_GREEN, COLOR_TEXT, COLOR_TEXT_DIM,
    COLOR_TEXT_MUTED, COLOR_YELLOW,
};
use crate::db::VaultDatabase;
use crate::models::NoteItem;
use tej_hal::{Color, SurfaceCompositor};
use tej_layout::{Point, Rect};
use tej_platform_linux::LinuxSurface;

/// Configuration parameters for rendering the markdown editor and preview.
#[derive(Clone, Copy, Debug)]
pub struct EditorViewConfig<'a> {
    pub note: &'a NoteItem,
    pub is_preview: bool,
    pub cursor: (usize, usize),
    pub cursor_blink: bool,
}

/// Render Markdown document panel supporting interactive Editing and compiled Live Preview.
pub fn render_markdown_editor(
    surface: &mut LinuxSurface,
    bounds: Rect,
    config: EditorViewConfig<'_>,
) {
    let x = bounds.origin.x;
    let y = bounds.origin.y;
    let w = bounds.size.width;
    let h = bounds.size.height;
    let note = config.note;
    let is_preview = config.is_preview;
    let cursor = config.cursor;
    let cursor_blink = config.cursor_blink;

    // 1. Breadcrumb navigation
    surface.draw_text(
        &format!("Vault > {} > {}", note.folder, note.title),
        Point::new(x + 6.0, y + 6.0),
        10.0,
        COLOR_TEXT_DIM,
    );

    // 2. Mode Toggle Button (Edit vs Preview)
    let btn_w = 160.0;
    let btn_x = (x + w - btn_w - 10.0).max(x + 180.0);
    let btn_rect = Rect::new(btn_x, y + 4.0, btn_w, 24.0);

    if is_preview {
        surface.fill_rect(btn_rect, COLOR_CARD);
        surface.fill_rect(
            Rect::new(btn_rect.origin.x, btn_rect.origin.y, btn_w, 1.0),
            COLOR_ACCENT,
        );
        surface.fill_rect(
            Rect::new(btn_rect.origin.x, btn_rect.origin.y + 23.0, btn_w, 1.0),
            COLOR_ACCENT,
        );
        surface.fill_rect(
            Rect::new(btn_rect.origin.x, btn_rect.origin.y, 1.0, 24.0),
            COLOR_ACCENT,
        );
        surface.fill_rect(
            Rect::new(
                btn_rect.origin.x + btn_w - 1.0,
                btn_rect.origin.y,
                1.0,
                24.0,
            ),
            COLOR_ACCENT,
        );
        surface.draw_text(
            "[Mode: Preview] (Edit)",
            Point::new(btn_x + 8.0, y + 9.0),
            10.0,
            COLOR_ACCENT,
        );
    } else {
        surface.fill_rect(btn_rect, COLOR_ACTIVE);
        surface.fill_rect(
            Rect::new(btn_rect.origin.x, btn_rect.origin.y, btn_w, 1.0),
            COLOR_GREEN,
        );
        surface.fill_rect(
            Rect::new(btn_rect.origin.x, btn_rect.origin.y + 23.0, btn_w, 1.0),
            COLOR_GREEN,
        );
        surface.fill_rect(
            Rect::new(btn_rect.origin.x, btn_rect.origin.y, 1.0, 24.0),
            COLOR_GREEN,
        );
        surface.fill_rect(
            Rect::new(
                btn_rect.origin.x + btn_w - 1.0,
                btn_rect.origin.y,
                1.0,
                24.0,
            ),
            COLOR_GREEN,
        );
        surface.draw_text(
            "[Mode: Edit] (Preview)",
            Point::new(btn_x + 8.0, y + 9.0),
            10.0,
            COLOR_GREEN,
        );
    }

    // 3. Render content based on mode
    if is_preview {
        // Live Preview: Native Markdown, LaTeX math, Mermaid diagrams, and SVGs!
        crate::markdown::render_markdown_document(
            surface,
            x + 6.0,
            y + 36.0,
            w - 12.0,
            h - 40.0,
            &note.content,
        );
    } else {
        // Interactive Edit Mode: Line numbers, active line tint, and blinking cursor
        surface.draw_text(&note.title, Point::new(x + 6.0, y + 28.0), 18.0, COLOR_TEXT);
        surface.fill_rect(
            Rect::new(x + 6.0, y + 52.0, w.min(800.0), 1.0),
            COLOR_BORDER,
        );

        let max_y = y + h - 10.0;
        let mut line_y = y + 66.0;

        for (idx, line) in note.content.lines().enumerate() {
            if line_y > max_y {
                break;
            }

            let is_current_line = cursor.0 == idx;
            if is_current_line {
                surface.fill_rect(
                    Rect::new(x + 36.0, line_y - 2.0, w - 46.0, 20.0),
                    Color::rgb(35, 36, 52),
                );
            }

            let num_str = format!("{:2}", idx + 1);
            let num_color = if is_current_line {
                COLOR_ACCENT
            } else {
                COLOR_TEXT_DIM
            };
            surface.draw_text(&num_str, Point::new(x + 6.0, line_y), 10.0, num_color);

            let line_color = if line.starts_with('#') {
                COLOR_ACCENT
            } else if line.starts_with('-') || line.starts_with('*') {
                COLOR_GREEN
            } else if line.starts_with("```") || line.starts_with("$$") {
                COLOR_YELLOW
            } else {
                COLOR_TEXT
            };

            surface.draw_text(line, Point::new(x + 38.0, line_y), 12.0, line_color);

            // Blinking cursor
            if is_current_line && cursor_blink {
                let cursor_x = x + 38.0 + (cursor.1 as f32) * 7.5;
                surface.fill_rect(Rect::new(cursor_x, line_y, 2.0, 16.0), COLOR_ACCENT);
            }

            line_y += 20.0;
        }
    }
}

/// Render 3D Knowledge Graph perspective projection.
pub fn render_3d_knowledge_graph(
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
        "Interactive 3D Knowledge Graph (Perspective Camera)",
        Point::new(x + 8.0, y + 8.0),
        11.0,
        COLOR_ACCENT,
    );

    let scale_factor = (w / 600.0).min(h / 500.0).clamp(0.45, 1.0);

    let raw_nodes = [
        (
            "Architecture",
            -130.0 * scale_factor,
            -90.0 * scale_factor,
            50.0 * scale_factor,
        ),
        (
            "SlotMap Arena",
            130.0 * scale_factor,
            -100.0 * scale_factor,
            -40.0 * scale_factor,
        ),
        (
            "Zero-Copy DB",
            150.0 * scale_factor,
            90.0 * scale_factor,
            60.0 * scale_factor,
        ),
        (
            "Spring Physics",
            -120.0 * scale_factor,
            110.0 * scale_factor,
            -50.0 * scale_factor,
        ),
        ("Native Linux", 0.0, -140.0 * scale_factor, 0.0),
        (
            "tej-widgets",
            0.0,
            130.0 * scale_factor,
            10.0 * scale_factor,
        ),
    ];

    let cos_a = angle.cos();
    let sin_a = angle.sin();

    let projected: Vec<(Point, &str)> = raw_nodes
        .iter()
        .map(|(label, nx, ny, nz)| {
            let rx = nx * cos_a - nz * sin_a;
            let rz = nx * sin_a + nz * cos_a;
            let perspective = 400.0 / (400.0 + rz);
            let px = cx + rx * perspective;
            let py = cy + ny * perspective;
            (Point::new(px, py), *label)
        })
        .collect();

    for i in 0..projected.len() {
        for j in (i + 1)..projected.len() {
            surface.draw_line(projected[i].0, projected[j].0, 1.0, COLOR_BORDER);
        }
    }

    for (pt, label) in projected {
        surface.fill_rect(Rect::new(pt.x - 5.0, pt.y - 5.0, 10.0, 10.0), COLOR_ACCENT);
        if scale_factor > 0.55 {
            surface.draw_text(label, Point::new(pt.x + 8.0, pt.y - 5.0), 10.0, COLOR_TEXT);
        }
    }
}

/// Render Zero-Copy Database live record table.
pub fn render_database_records(
    surface: &mut LinuxSurface,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    notes: &[NoteItem],
    vault_db: &VaultDatabase,
) {
    surface.draw_text(
        "Zero-Copy ACID DBMS Live Virtual Table",
        Point::new(x + 8.0, y + 8.0),
        12.0,
        COLOR_ACCENT,
    );

    let is_compact = w < 550.0;
    let is_mid = (550.0..750.0).contains(&w);

    surface.fill_rect(Rect::new(x + 6.0, y + 32.0, w - 12.0, 26.0), COLOR_CARD);
    surface.draw_text("PK", Point::new(x + 14.0, y + 38.0), 10.0, COLOR_TEXT_MUTED);
    surface.draw_text(
        "TITLE",
        Point::new(x + 120.0, y + 38.0),
        10.0,
        COLOR_TEXT_MUTED,
    );
    if is_mid || !is_compact {
        let col_x = if is_mid { x + 340.0 } else { x + 440.0 };
        surface.draw_text(
            "STATUS",
            Point::new(col_x, y + 38.0),
            10.0,
            COLOR_TEXT_MUTED,
        );
    }

    if let Ok(rx) = vault_db.db.begin_read() {
        let max_y = y + h - 10.0;
        let mut row_y = y + 64.0;
        for (i, note) in notes.iter().enumerate() {
            if row_y + 24.0 > max_y {
                break;
            }
            let pk = format!("note:{:04}", note.id);
            let exists = rx.get(pk.as_bytes()).ok().flatten().is_some();
            let status = if exists {
                "Committed (mmap)"
            } else {
                "Pending"
            };

            surface.draw_text(&pk, Point::new(x + 14.0, row_y + 3.0), 11.0, COLOR_ACCENT);

            let title_len = if is_compact {
                18
            } else if is_mid {
                26
            } else {
                38
            };
            let display_title = if note.title.len() > title_len {
                format!("{}...", &note.title[..title_len.saturating_sub(3)])
            } else {
                note.title.clone()
            };
            surface.draw_text(
                &display_title,
                Point::new(x + 120.0, row_y + 3.0),
                11.0,
                COLOR_TEXT,
            );

            if is_mid || !is_compact {
                let col_x = if is_mid { x + 340.0 } else { x + 440.0 };
                surface.draw_text(status, Point::new(col_x, row_y + 3.0), 10.0, COLOR_GREEN);
            }

            surface.fill_rect(
                Rect::new(x + 6.0, row_y + 22.0, w - 12.0, 1.0),
                COLOR_BORDER,
            );
            row_y += 28.0;
            if i >= 20 {
                break;
            }
        }
    }
}
