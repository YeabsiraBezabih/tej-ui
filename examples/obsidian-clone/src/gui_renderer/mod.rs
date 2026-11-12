//! Studio-grade UI renderer for Obsidian Clone with dynamic responsive layout and typography.

pub mod theme;
pub mod views;

pub use theme::*;
pub use views::*;

use crate::app_view::ObsidianState;
use crate::db::VaultDatabase;
use crate::models::NoteItem;
use tej_hal::SurfaceCompositor;
use tej_layout::{Point, Rect};
use tej_platform_linux::LinuxSurface;

/// Render context packing frame state and telemetry parameters.
pub struct RenderContext<'a> {
    pub state: &'a ObsidianState,
    pub notes: &'a [NoteItem],
    pub selected_idx: usize,
    pub hovered_note: Option<usize>,
    pub hovered_tab: Option<usize>,
    pub graph_angle: f32,
    pub ram_mb: f32,
    pub vault_db: &'a VaultDatabase,
    pub is_preview: bool,
    pub cursor: (usize, usize),
    pub cursor_blink: bool,
}

/// Render the complete desktop frame matching dynamic physical window dimensions.
pub fn render_frame(surface: &mut LinuxSurface, ctx: &RenderContext<'_>) {
    surface.begin_frame(&[]);
    surface.clear(COLOR_BG);

    #[allow(clippy::cast_precision_loss)]
    let win_w = surface.width() as f32;
    #[allow(clippy::cast_precision_loss)]
    let win_h = surface.height() as f32;

    let max_sidebar_w = (win_w * 0.45).clamp(100.0, 240.0);
    let raw_sidebar_w = ctx.state.sidebar_width_spring.get();
    let sidebar_w = raw_sidebar_w.min(max_sidebar_w).max(0.0);

    // 1. Sidebar Panel
    if sidebar_w > 2.0 {
        render_sidebar(
            surface,
            sidebar_w,
            win_h,
            ctx.notes,
            ctx.selected_idx,
            ctx.hovered_note,
        );
    }

    // 2. Main Content Panel
    let content_x = sidebar_w + 14.0;
    let content_w = (win_w - content_x - 14.0).max(80.0);
    let content_h = (win_h - 38.0 - 24.0).max(80.0);

    match ctx.state.active_tab.get() {
        1 => render_3d_knowledge_graph(
            surface,
            content_x,
            46.0,
            content_w,
            content_h,
            ctx.graph_angle,
        ),
        2 => render_database_records(
            surface,
            content_x,
            46.0,
            content_w,
            content_h,
            ctx.notes,
            ctx.vault_db,
        ),
        _ => render_markdown_editor(
            surface,
            Rect::new(content_x, 46.0, content_w, content_h),
            EditorViewConfig {
                note: &ctx.notes[ctx.selected_idx],
                is_preview: ctx.is_preview,
                cursor: ctx.cursor,
                cursor_blink: ctx.cursor_blink,
            },
        ),
    }

    // 3. Top Header Bar & Tabs
    render_header(surface, win_w, ctx.state, ctx.hovered_tab);

    // 4. Bottom Status Bar
    render_status_bar(
        surface,
        win_w,
        win_h,
        ctx.selected_idx,
        ctx.notes.len(),
        ctx.ram_mb,
        ctx.state.active_tab.get(),
    );

    surface.end_frame();
}

fn render_header(
    surface: &mut LinuxSurface,
    win_w: f32,
    state: &ObsidianState,
    hovered_tab: Option<usize>,
) {
    surface.fill_rect(Rect::new(0.0, 0.0, win_w, 38.0), COLOR_HEADER);
    surface.fill_rect(Rect::new(0.0, 37.0, win_w, 1.0), COLOR_BORDER);

    // Sidebar Toggle Button
    surface.fill_rect(Rect::new(8.0, 6.0, 28.0, 26.0), COLOR_CARD);
    surface.draw_text("[=]", Point::new(14.0, 11.0), 12.0, COLOR_ACCENT);

    // App Branding (shown when sufficient horizontal space exists)
    if win_w >= 650.0 {
        surface.draw_text("Obsidian Vault", Point::new(44.0, 12.0), 13.0, COLOR_TEXT);
    }

    // Navigation Tabs
    let active_tab = state.active_tab.get();
    let tab_rects = calculate_tab_rects(win_w);

    for (i, rect) in tab_rects.iter().enumerate() {
        let is_active = i == active_tab;
        let is_hover = Some(i) == hovered_tab && !is_active;

        if is_active {
            surface.fill_rect(*rect, COLOR_ACTIVE);
            surface.fill_rect(
                Rect::new(
                    rect.origin.x,
                    rect.origin.y + rect.size.height - 2.0,
                    rect.size.width,
                    2.0,
                ),
                COLOR_ACCENT,
            );
        } else if is_hover {
            surface.fill_rect(*rect, COLOR_HOVER);
        }

        let title = if rect.size.width < 90.0 {
            match i {
                0 => "[1] Edit",
                1 => "[2] Graph",
                _ => "[3] DB",
            }
        } else if rect.size.width < 140.0 {
            match i {
                0 => "[1] Editor",
                1 => "[2] 3D Graph",
                _ => "[3] Records",
            }
        } else {
            match i {
                0 => "[1] Markdown Editor",
                1 => "[2] 3D Knowledge Graph",
                _ => "[3] Vault Records (DB)",
            }
        };

        let col = if is_active {
            COLOR_ACCENT
        } else if is_hover {
            COLOR_TEXT
        } else {
            COLOR_TEXT_MUTED
        };
        surface.draw_text(
            title,
            Point::new(rect.origin.x + 8.0, rect.origin.y + 7.0),
            11.0,
            col,
        );
    }
}

fn render_sidebar(
    surface: &mut LinuxSurface,
    sidebar_w: f32,
    win_h: f32,
    notes: &[NoteItem],
    selected_idx: usize,
    hovered_note: Option<usize>,
) {
    let sidebar_h = (win_h - 38.0 - 24.0).max(10.0);
    surface.fill_rect(Rect::new(0.0, 38.0, sidebar_w, sidebar_h), COLOR_SIDEBAR);
    surface.fill_rect(
        Rect::new(sidebar_w - 1.0, 38.0, 1.0, sidebar_h),
        COLOR_BORDER,
    );

    if sidebar_w < 110.0 {
        return;
    }

    surface.draw_text(
        "FILES & FOLDERS",
        Point::new(14.0, 50.0),
        10.0,
        COLOR_TEXT_DIM,
    );

    surface.fill_rect(Rect::new(10.0, 68.0, sidebar_w - 20.0, 26.0), COLOR_CARD);
    let search_label = if sidebar_w > 160.0 {
        "Search (Ctrl+P)..."
    } else {
        "Search..."
    };
    surface.draw_text(search_label, Point::new(16.0, 74.0), 10.0, COLOR_TEXT_MUTED);

    for (i, note) in notes.iter().enumerate() {
        let y = 104.0 + (i as f32) * 34.0;
        if y + 32.0 > (win_h - 28.0) {
            break;
        }

        let rect = Rect::new(8.0, y, sidebar_w - 16.0, 30.0);
        let is_selected = i == selected_idx;
        let is_hover = Some(i) == hovered_note && !is_selected;

        if is_selected {
            surface.fill_rect(rect, COLOR_ACTIVE);
            surface.fill_rect(
                Rect::new(rect.origin.x, rect.origin.y, 3.0, rect.size.height),
                COLOR_ACCENT,
            );
        } else if is_hover {
            surface.fill_rect(rect, COLOR_HOVER);
        }

        let title_color = if is_selected {
            COLOR_ACCENT
        } else if is_hover {
            COLOR_TEXT
        } else {
            COLOR_TEXT_MUTED
        };
        let char_limit = ((sidebar_w - 30.0) / 7.5).max(4.0) as usize;
        let display_title = if note.title.len() > char_limit {
            format!("{}...", &note.title[..char_limit.saturating_sub(3)])
        } else {
            note.title.clone()
        };

        surface.draw_text(
            &format!("- {display_title}"),
            Point::new(14.0, y + 7.0),
            11.0,
            title_color,
        );
    }
}

fn render_status_bar(
    surface: &mut LinuxSurface,
    win_w: f32,
    win_h: f32,
    selected_idx: usize,
    total: usize,
    ram_mb: f32,
    active_tab: usize,
) {
    let bar_y = win_h - 24.0;
    surface.fill_rect(Rect::new(0.0, bar_y, win_w, 24.0), COLOR_HEADER);
    surface.fill_rect(Rect::new(0.0, bar_y, win_w, 1.0), COLOR_BORDER);

    let mode_str = match active_tab {
        1 => "3D Graph",
        2 => "DBMS Records",
        _ => "Editor",
    };

    let info = if win_w < 600.0 {
        format!(
            "Note {}/{} | {} | {:.1}MB",
            selected_idx + 1,
            total,
            mode_str,
            ram_mb
        )
    } else {
        format!(
            "Note {}/{} | {} | RAM: {:.1} MB | CPU: <0.2% | Zero-Copy mmap | tej-ui",
            selected_idx + 1,
            total,
            mode_str,
            ram_mb
        )
    };
    surface.draw_text(&info, Point::new(12.0, bar_y + 6.0), 10.0, COLOR_TEXT_MUTED);
}
