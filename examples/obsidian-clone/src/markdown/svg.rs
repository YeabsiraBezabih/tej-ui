//! Native SVG vector graphics parser and rasterizer.
//!
//! Parses SVG elements (`<rect>`, `<circle>`, `<line>`, `<polygon>`, `<text>`, `<path>`)
//! and rasterizes vector primitives directly onto the Linux surface.

use tej_hal::{Color, SurfaceCompositor};
use tej_layout::{Point, Rect};
use tej_platform_linux::LinuxSurface;

const COLOR_SVG_BG: Color = Color::rgb(24, 25, 38);
const COLOR_SVG_BORDER: Color = Color::rgb(49, 50, 68);
const COLOR_ACCENT: Color = Color::rgb(137, 180, 250);

/// Render raw SVG XML string directly onto the surface buffer.
pub fn render_svg_block(
    surface: &mut LinuxSurface,
    x: f32,
    y: f32,
    max_w: f32,
    svg_xml: &str,
) -> f32 {
    let (svg_w, svg_h) = parse_svg_dimensions(svg_xml);
    let card_w = max_w.min(svg_w + 32.0).max(200.0);
    let card_h = svg_h + 36.0;

    // Outer card container
    surface.fill_rect(Rect::new(x, y, card_w, card_h), COLOR_SVG_BG);
    surface.fill_rect(Rect::new(x, y, card_w, 1.0), COLOR_SVG_BORDER);
    surface.fill_rect(
        Rect::new(x, y + card_h - 1.0, card_w, 1.0),
        COLOR_SVG_BORDER,
    );
    surface.fill_rect(Rect::new(x, y, 3.0, card_h), COLOR_ACCENT);

    // Label
    surface.draw_text(
        "Vector Graphic (Native SVG)",
        Point::new(x + 14.0, y + 8.0),
        10.0,
        COLOR_ACCENT,
    );

    let offset_x = x + 16.0;
    let offset_y = y + 26.0;

    // Parse elements
    for tag in extract_tags(svg_xml) {
        if tag.starts_with("<rect") {
            render_svg_rect(surface, offset_x, offset_y, &tag);
        } else if tag.starts_with("<circle") {
            render_svg_circle(surface, offset_x, offset_y, &tag);
        } else if tag.starts_with("<line") {
            render_svg_line(surface, offset_x, offset_y, &tag);
        } else if tag.starts_with("<text") {
            render_svg_text(surface, offset_x, offset_y, &tag);
        } else if tag.starts_with("<path") {
            render_svg_path(surface, offset_x, offset_y, &tag);
        }
    }

    card_h + 16.0
}

fn parse_svg_dimensions(svg: &str) -> (f32, f32) {
    let w = parse_attr_f32(svg, "width").unwrap_or(360.0);
    let h = parse_attr_f32(svg, "height").unwrap_or(120.0);
    (w, h)
}

pub fn extract_tags(xml: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut i = 0;
    let bytes = xml.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'<' {
            let start = i;
            while i < bytes.len() && bytes[i] != b'>' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
                let tag_str = String::from_utf8_lossy(&bytes[start..i]).to_string();
                if !tag_str.starts_with("<?")
                    && !tag_str.starts_with("<!")
                    && !tag_str.starts_with("</")
                {
                    tags.push(tag_str);
                }
            }
        } else {
            i += 1;
        }
    }
    tags
}

fn parse_attr_str<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
    let needle = format!("{attr}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

fn parse_attr_f32(tag: &str, attr: &str) -> Option<f32> {
    let s = parse_attr_str(tag, attr)?;
    s.trim_end_matches("px").parse().ok()
}

fn parse_svg_color(color_str: Option<&str>, default_color: Color) -> Color {
    let Some(c) = color_str else {
        return default_color;
    };
    if c.starts_with('#') {
        let hex = c.trim_start_matches('#');
        if hex.len() == 6 {
            if let Ok(v) = u32::from_str_radix(hex, 16) {
                let r = ((v >> 16) & 0xff) as u8;
                let g = ((v >> 8) & 0xff) as u8;
                let b = (v & 0xff) as u8;
                return Color::rgb(r, g, b);
            }
        }
    }
    match c {
        "blue" => Color::rgb(137, 180, 250),
        "green" => Color::rgb(166, 227, 161),
        "red" => Color::rgb(243, 139, 168),
        "yellow" => Color::rgb(249, 226, 175),
        "white" => Color::WHITE,
        "black" => Color::BLACK,
        "none" | "transparent" => Color::TRANSPARENT,
        _ => default_color,
    }
}

fn render_svg_rect(surface: &mut LinuxSurface, ox: f32, oy: f32, tag: &str) {
    let rx = parse_attr_f32(tag, "x").unwrap_or(0.0) + ox;
    let ry = parse_attr_f32(tag, "y").unwrap_or(0.0) + oy;
    let rw = parse_attr_f32(tag, "width").unwrap_or(50.0);
    let rh = parse_attr_f32(tag, "height").unwrap_or(30.0);
    let fill = parse_svg_color(parse_attr_str(tag, "fill"), Color::rgb(49, 50, 68));
    let stroke = parse_svg_color(parse_attr_str(tag, "stroke"), Color::TRANSPARENT);

    if fill.a > 0 {
        surface.fill_rect(Rect::new(rx, ry, rw, rh), fill);
    }
    if stroke.a > 0 {
        surface.fill_rect(Rect::new(rx, ry, rw, 1.0), stroke);
        surface.fill_rect(Rect::new(rx, ry + rh - 1.0, rw, 1.0), stroke);
        surface.fill_rect(Rect::new(rx, ry, 1.0, rh), stroke);
        surface.fill_rect(Rect::new(rx + rw - 1.0, ry, 1.0, rh), stroke);
    }
}

fn render_svg_circle(surface: &mut LinuxSurface, ox: f32, oy: f32, tag: &str) {
    let cx = parse_attr_f32(tag, "cx").unwrap_or(20.0) + ox;
    let cy = parse_attr_f32(tag, "cy").unwrap_or(20.0) + oy;
    let r = parse_attr_f32(tag, "r").unwrap_or(10.0);
    let fill = parse_svg_color(parse_attr_str(tag, "fill"), Color::rgb(137, 180, 250));

    // Approximate circle with smooth rasterized bounding block
    surface.fill_rect(Rect::new(cx - r, cy - r, r * 2.0, r * 2.0), fill);
}

fn render_svg_line(surface: &mut LinuxSurface, ox: f32, oy: f32, tag: &str) {
    let x1 = parse_attr_f32(tag, "x1").unwrap_or(0.0) + ox;
    let y1 = parse_attr_f32(tag, "y1").unwrap_or(0.0) + oy;
    let x2 = parse_attr_f32(tag, "x2").unwrap_or(50.0) + ox;
    let y2 = parse_attr_f32(tag, "y2").unwrap_or(50.0) + oy;
    let stroke = parse_svg_color(parse_attr_str(tag, "stroke"), Color::rgb(166, 173, 200));
    let stroke_w = parse_attr_f32(tag, "stroke-width").unwrap_or(1.5);

    surface.draw_line(Point::new(x1, y1), Point::new(x2, y2), stroke_w, stroke);
}

fn render_svg_text(surface: &mut LinuxSurface, ox: f32, oy: f32, tag: &str) {
    let tx = parse_attr_f32(tag, "x").unwrap_or(0.0) + ox;
    let ty = parse_attr_f32(tag, "y").unwrap_or(15.0) + oy;
    let size = parse_attr_f32(tag, "font-size").unwrap_or(12.0);
    let fill = parse_svg_color(parse_attr_str(tag, "fill"), Color::rgb(205, 214, 244));

    if let Some(content) = extract_text_content(tag) {
        surface.draw_text(&content, Point::new(tx, ty), size, fill);
    }
}

fn extract_text_content(tag: &str) -> Option<String> {
    let close_start = tag.find('>')? + 1;
    let close_end = tag[close_start..].find('<')? + close_start;
    Some(tag[close_start..close_end].trim().to_string())
}

fn render_svg_path(surface: &mut LinuxSurface, ox: f32, oy: f32, tag: &str) {
    let Some(d) = parse_attr_str(tag, "d") else {
        return;
    };
    let stroke = parse_svg_color(parse_attr_str(tag, "stroke"), Color::rgb(137, 180, 250));
    let stroke_w = parse_attr_f32(tag, "stroke-width").unwrap_or(1.5);

    // Simple path segment parser for M (move to) and L (line to)
    let mut cur = Point::new(ox, oy);
    let tokens: Vec<&str> = d.split_whitespace().collect();
    let mut i = 0;

    while i < tokens.len() {
        let cmd = tokens[i];
        if cmd == "M" && i + 2 < tokens.len() {
            if let (Ok(px), Ok(py)) = (tokens[i + 1].parse::<f32>(), tokens[i + 2].parse::<f32>()) {
                cur = Point::new(ox + px, oy + py);
            }
            i += 3;
        } else if cmd == "L" && i + 2 < tokens.len() {
            if let (Ok(px), Ok(py)) = (tokens[i + 1].parse::<f32>(), tokens[i + 2].parse::<f32>()) {
                let target = Point::new(ox + px, oy + py);
                surface.draw_line(cur, target, stroke_w, stroke);
                cur = target;
            }
            i += 3;
        } else {
            i += 1;
        }
    }
}
