//! Markdown compiling, layout, and rendering engine with native LaTeX, Mermaid, and SVG support.

pub mod latex;
pub mod mermaid;
pub mod svg;
#[cfg(test)]
mod tests;

pub use latex::render_latex_block;
pub use mermaid::render_mermaid_diagram;
pub use svg::render_svg_block;

use tej_hal::{Color, SurfaceCompositor};
use tej_layout::{Point, Rect};
use tej_platform_linux::LinuxSurface;

const COLOR_TEXT: Color = Color::rgb(205, 214, 244);
const COLOR_TEXT_DIM: Color = Color::rgb(108, 112, 134);
const COLOR_ACCENT: Color = Color::rgb(137, 180, 250);
const COLOR_GREEN: Color = Color::rgb(166, 227, 161);
const COLOR_YELLOW: Color = Color::rgb(249, 226, 175);
const COLOR_BORDER: Color = Color::rgb(49, 50, 68);
const COLOR_CODE_BG: Color = Color::rgb(24, 25, 38);

/// Render compiled Markdown document into the Linux surface.
pub fn render_markdown_document(
    surface: &mut LinuxSurface,
    x: f32,
    y: f32,
    w: f32,
    max_h: f32,
    content: &str,
) -> f32 {
    let mut cur_y = y;
    let limit_y = y + max_h;
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if cur_y > limit_y - 20.0 {
            break;
        }

        let line = lines[i];

        // 1. Mermaid Fenced Block
        if line.starts_with("```mermaid") {
            let mut mermaid_lines = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].starts_with("```") {
                mermaid_lines.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1;
            }
            let mermaid_str = mermaid_lines.join("\n");
            let rendered_h =
                render_mermaid_diagram(surface, x + 6.0, cur_y, w - 12.0, &mermaid_str);
            cur_y += rendered_h;
            continue;
        }

        // 2. SVG Fenced Block or Raw SVG Tag
        if line.starts_with("```svg") || line.starts_with("```xml") {
            let mut svg_lines = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].starts_with("```") {
                svg_lines.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1;
            }
            let svg_str = svg_lines.join("\n");
            let rendered_h = render_svg_block(surface, x + 6.0, cur_y, w - 12.0, &svg_str);
            cur_y += rendered_h;
            continue;
        }

        if line.trim().starts_with("<svg") {
            let mut svg_lines = vec![line];
            while i + 1 < lines.len() && !lines[i].contains("</svg>") {
                i += 1;
                svg_lines.push(lines[i]);
            }
            i += 1;
            let svg_str = svg_lines.join("\n");
            let rendered_h = render_svg_block(surface, x + 6.0, cur_y, w - 12.0, &svg_str);
            cur_y += rendered_h;
            continue;
        }

        // 3. Display LaTeX Block ($$...$$)
        if line.trim().starts_with("$$") {
            let mut latex_str = line.trim().to_string();
            if !line.trim().ends_with("$$") || line.trim() == "$$" {
                i += 1;
                while i < lines.len() {
                    latex_str.push(' ');
                    latex_str.push_str(lines[i].trim());
                    if lines[i].trim().ends_with("$$") {
                        break;
                    }
                    i += 1;
                }
            }
            i += 1;
            let rendered_h = render_latex_block(surface, x + 6.0, cur_y, w - 12.0, &latex_str);
            cur_y += rendered_h;
            continue;
        }

        // 4. Code Block
        if line.starts_with("```") {
            let mut code_lines = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].starts_with("```") {
                code_lines.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1;
            }
            let code_h = (code_lines.len() as f32 * 20.0 + 16.0).max(36.0);
            surface.fill_rect(Rect::new(x + 6.0, cur_y, w - 12.0, code_h), COLOR_CODE_BG);
            surface.fill_rect(Rect::new(x + 6.0, cur_y, w - 12.0, 1.0), COLOR_BORDER);
            surface.fill_rect(
                Rect::new(x + 6.0, cur_y + code_h - 1.0, w - 12.0, 1.0),
                COLOR_BORDER,
            );

            let mut code_y = cur_y + 10.0;
            for cl in code_lines {
                surface.draw_text(cl, Point::new(x + 18.0, code_y), 11.0, COLOR_YELLOW);
                code_y += 20.0;
            }
            cur_y += code_h + 12.0;
            continue;
        }

        // 5. Headings
        if let Some(rest) = line.strip_prefix("# ") {
            surface.draw_text(rest, Point::new(x + 6.0, cur_y), 18.0, COLOR_TEXT);
            cur_y += 26.0;
            surface.fill_rect(Rect::new(x + 6.0, cur_y, w - 12.0, 1.0), COLOR_BORDER);
            cur_y += 10.0;
            i += 1;
            continue;
        }

        if let Some(rest) = line.strip_prefix("## ") {
            surface.draw_text(rest, Point::new(x + 6.0, cur_y), 15.0, COLOR_ACCENT);
            cur_y += 24.0;
            i += 1;
            continue;
        }

        if let Some(rest) = line.strip_prefix("### ") {
            surface.draw_text(rest, Point::new(x + 6.0, cur_y), 13.0, COLOR_ACCENT);
            cur_y += 22.0;
            i += 1;
            continue;
        }

        // 6. Checklists and List Items
        if let Some(stripped) = line.strip_prefix("- [ ]") {
            surface.fill_rect(Rect::new(x + 8.0, cur_y + 3.0, 12.0, 12.0), COLOR_BORDER);
            surface.draw_text(stripped, Point::new(x + 28.0, cur_y), 12.0, COLOR_TEXT);
            cur_y += 22.0;
            i += 1;
            continue;
        }

        if let Some(stripped) = line
            .strip_prefix("- [x]")
            .or_else(|| line.strip_prefix("- [X]"))
        {
            surface.fill_rect(Rect::new(x + 8.0, cur_y + 3.0, 12.0, 12.0), COLOR_GREEN);
            surface.draw_text("v", Point::new(x + 11.0, cur_y + 3.0), 9.0, Color::BLACK);
            surface.draw_text(stripped, Point::new(x + 28.0, cur_y), 12.0, COLOR_TEXT_DIM);
            cur_y += 22.0;
            i += 1;
            continue;
        }

        if line.starts_with('-') || line.starts_with('*') {
            surface.fill_rect(Rect::new(x + 10.0, cur_y + 6.0, 5.0, 5.0), COLOR_ACCENT);
            surface.draw_text(&line[1..], Point::new(x + 24.0, cur_y), 12.0, COLOR_TEXT);
            cur_y += 22.0;
            i += 1;
            continue;
        }

        // 7. Regular paragraph text or inline LaTeX
        if !line.is_empty() {
            surface.draw_text(line, Point::new(x + 6.0, cur_y), 12.0, COLOR_TEXT);
            cur_y += 20.0;
        } else {
            cur_y += 10.0;
        }

        i += 1;
    }

    cur_y - y
}
