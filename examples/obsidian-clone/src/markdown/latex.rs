//! Native LaTeX mathematical formula parser and rasterizer.
//!
//! Parses math expressions (fractions, overdots, superscripts, subscripts, radicals,
//! Greek letters, and operators) and renders formatted mathematical typography.

use tej_hal::{Color, SurfaceCompositor};
use tej_layout::{Point, Rect};
use tej_platform_linux::LinuxSurface;

const COLOR_MATH_BG: Color = Color::rgb(24, 25, 38);
const COLOR_MATH_BORDER: Color = Color::rgb(49, 50, 68);
const COLOR_MATH_TEXT: Color = Color::rgb(205, 214, 244);
const COLOR_TEXT_MUTED: Color = Color::rgb(166, 173, 200);
const COLOR_ACCENT: Color = Color::rgb(137, 180, 250);
const COLOR_SYMBOL: Color = Color::rgb(249, 226, 175);

/// Mathematical expression AST node.
#[derive(Clone, Debug)]
pub enum MathNode {
    Text(String),
    Symbol(String),
    Fraction(Vec<MathNode>, Vec<MathNode>),
    Overdot(String, usize),
    Superscript(String, String),
    Subscript(String, String),
    Sqrt(Vec<MathNode>),
}

/// Parse a LaTeX math string into a list of mathematical nodes.
#[must_use]
pub fn parse_latex(input: &str) -> Vec<MathNode> {
    let clean = input
        .trim()
        .trim_start_matches('$')
        .trim_end_matches('$')
        .trim();
    let mut nodes = Vec::new();
    let chars: Vec<char> = clean.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        // 1. LaTeX Commands
        if chars[i] == '\\' {
            i += 1;
            let start = i;
            while i < chars.len() && chars[i].is_alphabetic() {
                i += 1;
            }
            let cmd: String = chars[start..i].iter().collect();

            match cmd.as_str() {
                "frac" => {
                    let num = extract_braced(&chars, &mut i);
                    let den = extract_braced(&chars, &mut i);
                    nodes.push(MathNode::Fraction(parse_latex(&num), parse_latex(&den)));
                }
                "ddot" => {
                    let arg = extract_braced(&chars, &mut i);
                    nodes.push(MathNode::Overdot(arg, 2));
                }
                "dot" => {
                    let arg = extract_braced(&chars, &mut i);
                    nodes.push(MathNode::Overdot(arg, 1));
                }
                "sqrt" => {
                    let arg = extract_braced(&chars, &mut i);
                    nodes.push(MathNode::Sqrt(parse_latex(&arg)));
                }
                "alpha" => nodes.push(MathNode::Symbol("α".into())),
                "beta" => nodes.push(MathNode::Symbol("β".into())),
                "gamma" => nodes.push(MathNode::Symbol("γ".into())),
                "delta" => nodes.push(MathNode::Symbol("δ".into())),
                "Delta" => nodes.push(MathNode::Symbol("Δ".into())),
                "omega" => nodes.push(MathNode::Symbol("ω".into())),
                "pi" => nodes.push(MathNode::Symbol("π".into())),
                "sum" => nodes.push(MathNode::Symbol("∑".into())),
                "int" => nodes.push(MathNode::Symbol("∫".into())),
                "times" => nodes.push(MathNode::Symbol("×".into())),
                "cdot" => nodes.push(MathNode::Symbol("·".into())),
                "pm" => nodes.push(MathNode::Symbol("±".into())),
                _ => nodes.push(MathNode::Text(cmd)),
            }
            continue;
        }

        // 2. Subscript / Superscript
        if (chars[i] == '^' || chars[i] == '_') && !nodes.is_empty() {
            let is_sup = chars[i] == '^';
            i += 1;
            let script = if i < chars.len() && chars[i] == '{' {
                extract_braced(&chars, &mut i)
            } else if i < chars.len() {
                let s = chars[i].to_string();
                i += 1;
                s
            } else {
                String::new()
            };

            let prev = nodes.pop().unwrap_or(MathNode::Text(String::new()));
            let base = match prev {
                MathNode::Text(t) | MathNode::Symbol(t) => t,
                MathNode::Fraction(..) => "(frac)".to_string(),
                MathNode::Overdot(v, _) => v,
                MathNode::Superscript(b, s) => format!("{b}^{s}"),
                MathNode::Subscript(b, s) => format!("{b}_{s}"),
                MathNode::Sqrt(_) => "(sqrt)".to_string(),
            };

            if is_sup {
                nodes.push(MathNode::Superscript(base, script));
            } else {
                nodes.push(MathNode::Subscript(base, script));
            }
            continue;
        }

        if i + 1 < chars.len() && (chars[i + 1] == '^' || chars[i + 1] == '_') {
            let base = chars[i].to_string();
            let is_sup = chars[i + 1] == '^';
            i += 2;
            let script = if i < chars.len() && chars[i] == '{' {
                extract_braced(&chars, &mut i)
            } else if i < chars.len() {
                let s = chars[i].to_string();
                i += 1;
                s
            } else {
                String::new()
            };

            if is_sup {
                nodes.push(MathNode::Superscript(base, script));
            } else {
                nodes.push(MathNode::Subscript(base, script));
            }
            continue;
        }

        // 3. Regular characters and operators
        let c = chars[i];
        if c == '=' || c == '+' || c == '-' || c == '*' || c == '/' {
            nodes.push(MathNode::Symbol(format!(" {c} ")));
        } else {
            nodes.push(MathNode::Text(c.to_string()));
        }
        i += 1;
    }

    nodes
}

fn extract_braced(chars: &[char], idx: &mut usize) -> String {
    while *idx < chars.len() && chars[*idx].is_whitespace() {
        *idx += 1;
    }
    if *idx >= chars.len() || chars[*idx] != '{' {
        if *idx < chars.len() {
            let res = chars[*idx].to_string();
            *idx += 1;
            return res;
        }
        return String::new();
    }
    *idx += 1;
    let mut depth = 1;
    let mut res = String::new();
    while *idx < chars.len() && depth > 0 {
        if chars[*idx] == '{' {
            depth += 1;
        } else if chars[*idx] == '}' {
            depth -= 1;
            if depth == 0 {
                *idx += 1;
                break;
            }
        }
        res.push(chars[*idx]);
        *idx += 1;
    }
    res
}

/// Render a LaTeX math equation centered inside a styled math container card.
pub fn render_latex_block(
    surface: &mut LinuxSurface,
    x: f32,
    y: f32,
    max_w: f32,
    latex_str: &str,
) -> f32 {
    let nodes = parse_latex(latex_str);
    let card_w = max_w.min(720.0);
    let card_h = 56.0;

    surface.fill_rect(Rect::new(x, y, card_w, card_h), COLOR_MATH_BG);
    surface.fill_rect(Rect::new(x, y, card_w, 1.0), COLOR_MATH_BORDER);
    surface.fill_rect(
        Rect::new(x, y + card_h - 1.0, card_w, 1.0),
        COLOR_MATH_BORDER,
    );
    surface.fill_rect(Rect::new(x, y, 3.0, card_h), COLOR_ACCENT);

    let mut cur_x = x + 24.0;
    let baseline = y + 26.0;

    for node in &nodes {
        match node {
            MathNode::Text(t) => {
                surface.draw_text(t, Point::new(cur_x, baseline), 15.0, COLOR_MATH_TEXT);
                cur_x += (t.len() as f32) * 9.5;
            }
            MathNode::Symbol(s) => {
                surface.draw_text(s, Point::new(cur_x, baseline), 15.0, COLOR_SYMBOL);
                cur_x += (s.len() as f32) * 9.5;
            }
            MathNode::Overdot(var, count) => {
                let dots = if *count >= 2 { "¨" } else { "˙" };
                surface.draw_text(
                    dots,
                    Point::new(cur_x + 1.0, baseline - 10.0),
                    14.0,
                    COLOR_ACCENT,
                );
                surface.draw_text(var, Point::new(cur_x, baseline), 15.0, COLOR_MATH_TEXT);
                cur_x += 16.0;
            }
            MathNode::Superscript(base, sup) => {
                surface.draw_text(base, Point::new(cur_x, baseline), 15.0, COLOR_MATH_TEXT);
                cur_x += (base.len() as f32) * 9.5;
                surface.draw_text(sup, Point::new(cur_x, baseline - 6.0), 10.0, COLOR_ACCENT);
                cur_x += (sup.len() as f32) * 7.0 + 3.0;
            }
            MathNode::Subscript(base, sub) => {
                surface.draw_text(base, Point::new(cur_x, baseline), 15.0, COLOR_MATH_TEXT);
                cur_x += (base.len() as f32) * 9.5;
                surface.draw_text(
                    sub,
                    Point::new(cur_x, baseline + 5.0),
                    10.0,
                    COLOR_TEXT_MUTED,
                );
                cur_x += (sub.len() as f32) * 7.0 + 3.0;
            }
            MathNode::Fraction(num_nodes, den_nodes) => {
                let num_str = nodes_to_str(num_nodes);
                let den_str = nodes_to_str(den_nodes);
                let frac_w = ((num_str.len().max(den_str.len()) as f32) * 8.5).max(28.0);

                let num_x = cur_x + (frac_w - (num_str.len() as f32) * 8.0) / 2.0;
                surface.draw_text(
                    &num_str,
                    Point::new(num_x, baseline - 10.0),
                    12.0,
                    COLOR_MATH_TEXT,
                );

                surface.draw_line(
                    Point::new(cur_x, baseline + 8.0),
                    Point::new(cur_x + frac_w, baseline + 8.0),
                    1.5,
                    COLOR_ACCENT,
                );

                let den_x = cur_x + (frac_w - (den_str.len() as f32) * 8.0) / 2.0;
                surface.draw_text(
                    &den_str,
                    Point::new(den_x, baseline + 12.0),
                    12.0,
                    COLOR_MATH_TEXT,
                );

                cur_x += frac_w + 8.0;
            }
            MathNode::Sqrt(inner) => {
                surface.draw_text("√", Point::new(cur_x, baseline - 1.0), 16.0, COLOR_ACCENT);
                cur_x += 12.0;
                let inner_str = nodes_to_str(inner);
                let inner_w = (inner_str.len() as f32) * 8.5;
                surface.draw_line(
                    Point::new(cur_x, baseline - 4.0),
                    Point::new(cur_x + inner_w, baseline - 4.0),
                    1.0,
                    COLOR_ACCENT,
                );
                surface.draw_text(
                    &inner_str,
                    Point::new(cur_x, baseline),
                    13.0,
                    COLOR_MATH_TEXT,
                );
                cur_x += inner_w + 6.0;
            }
        }
    }

    card_h + 12.0
}

fn nodes_to_str(nodes: &[MathNode]) -> String {
    let mut out = String::new();
    for n in nodes {
        match n {
            MathNode::Text(t) | MathNode::Symbol(t) => out.push_str(t),
            MathNode::Overdot(v, 2) => {
                out.push_str("d^2 ");
                out.push_str(v);
            }
            MathNode::Overdot(v, _) => {
                out.push('d');
                out.push_str(v);
            }
            MathNode::Superscript(b, s) => {
                out.push_str(b);
                out.push('^');
                out.push_str(s);
            }
            MathNode::Subscript(b, s) => {
                out.push_str(b);
                out.push('_');
                out.push_str(s);
            }
            MathNode::Fraction(num, den) => {
                out.push_str(&nodes_to_str(num));
                out.push('/');
                out.push_str(&nodes_to_str(den));
            }
            MathNode::Sqrt(inner) => {
                out.push_str("sqrt(");
                out.push_str(&nodes_to_str(inner));
                out.push(')');
            }
        }
    }
    out
}
