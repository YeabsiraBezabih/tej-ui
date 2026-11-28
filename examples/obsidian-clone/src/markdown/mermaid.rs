//! Native Mermaid diagram parser and vector flowchart rasterizer.
//!
//! Parses `graph TD` and `graph LR` flowchart definitions and renders node cards,
//! connecting edges, and arrowheads directly onto the Linux surface.

use std::collections::HashMap;
use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{Point, Rect};
use tej_ui_platform_linux::LinuxSurface;

const COLOR_MERMAID_BG: Color = Color::rgb(24, 25, 38);
const COLOR_MERMAID_BORDER: Color = Color::rgb(49, 50, 68);
const COLOR_NODE_BG: Color = Color::rgb(36, 37, 52);
const COLOR_NODE_BORDER: Color = Color::rgb(137, 180, 250);
const COLOR_TEXT: Color = Color::rgb(205, 214, 244);
const COLOR_EDGE: Color = Color::rgb(166, 173, 200);
const COLOR_EDGE_LABEL: Color = Color::rgb(166, 227, 161);

#[derive(Clone, Debug)]
pub struct MermaidNode {
    pub id: String,
    pub label: String,
    pub level: usize,
    pub order: usize,
}

#[derive(Clone, Debug)]
pub struct MermaidEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MermaidGraph {
    pub is_horizontal: bool,
    pub nodes: HashMap<String, MermaidNode>,
    pub edges: Vec<MermaidEdge>,
}

/// Parse a Mermaid flowchart definition into structured nodes and edges.
#[must_use]
pub fn parse_mermaid(input: &str) -> MermaidGraph {
    let mut is_horizontal = false;
    let mut nodes: HashMap<String, MermaidNode> = HashMap::new();
    let mut edges = Vec::new();

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("```") {
            continue;
        }

        if line.starts_with("graph") || line.starts_with("flowchart") {
            if line.contains("LR") {
                is_horizontal = true;
            }
            continue;
        }

        // Parse Edges: e.g. A[Start] --> B[End] or A -->|Yes| B
        if line.contains("-->") || line.contains("---") {
            let separator = if line.contains("-->") { "-->" } else { "---" };
            let parts: Vec<&str> = line.split(separator).collect();
            if parts.len() == 2 {
                let (from_id, from_label) = parse_node_token(parts[0].trim());
                let right_side = parts[1].trim();
                let (edge_label, to_side) = if let Some(stripped) = right_side.strip_prefix('|') {
                    if let Some(end_idx) = stripped.find('|') {
                        let l = &stripped[..end_idx];
                        let rest = &stripped[end_idx + 1..];
                        (Some(l.to_string()), rest.trim())
                    } else {
                        (None, right_side)
                    }
                } else {
                    (None, right_side)
                };

                let (to_id, to_label) = parse_node_token(to_side);

                if !from_id.is_empty() {
                    nodes.entry(from_id.clone()).or_insert_with(|| MermaidNode {
                        id: from_id.clone(),
                        label: from_label.unwrap_or_else(|| from_id.clone()),
                        level: 0,
                        order: 0,
                    });
                }

                if !to_id.is_empty() {
                    nodes.entry(to_id.clone()).or_insert_with(|| MermaidNode {
                        id: to_id.clone(),
                        label: to_label.unwrap_or_else(|| to_id.clone()),
                        level: 1,
                        order: 0,
                    });
                }

                edges.push(MermaidEdge {
                    from: from_id,
                    to: to_id,
                    label: edge_label,
                });
            }
        } else {
            // Standalone node
            let (id, label) = parse_node_token(line);
            if !id.is_empty() {
                nodes.entry(id.clone()).or_insert_with(|| MermaidNode {
                    id: id.clone(),
                    label: label.unwrap_or(id),
                    level: 0,
                    order: 0,
                });
            }
        }
    }

    // Compute topological node levels
    for edge in &edges {
        let from_level = nodes.get(&edge.from).map_or(0, |n| n.level);
        if let Some(to_node) = nodes.get_mut(&edge.to) {
            to_node.level = to_node.level.max(from_level + 1);
        }
    }

    // Assign order per level
    let mut level_counts: HashMap<usize, usize> = HashMap::new();
    for node in nodes.values_mut() {
        let count = level_counts.entry(node.level).or_insert(0);
        node.order = *count;
        *count += 1;
    }

    MermaidGraph {
        is_horizontal,
        nodes,
        edges,
    }
}

fn parse_node_token(token: &str) -> (String, Option<String>) {
    let t = token.trim();
    if let Some(open_idx) = t.find(['[', '(', '{']) {
        let id = t[..open_idx].trim().to_string();
        let close_char = match t.chars().nth(open_idx) {
            Some('(') => ')',
            Some('{') => '}',
            _ => ']',
        };
        if let Some(close_idx) = t[open_idx + 1..].find(close_char) {
            let label = t[open_idx + 1..open_idx + 1 + close_idx].to_string();
            return (id, Some(label));
        }
        (id, None)
    } else {
        (t.to_string(), None)
    }
}

/// Render a complete parsed Mermaid diagram into the Linux surface.
pub fn render_mermaid_diagram(
    surface: &mut LinuxSurface,
    x: f32,
    y: f32,
    max_w: f32,
    mermaid_code: &str,
) -> f32 {
    let graph = parse_mermaid(mermaid_code);
    if graph.nodes.is_empty() {
        return 0.0;
    }

    let max_level = graph.nodes.values().map(|n| n.level).max().unwrap_or(0);
    let level_count = f32::from(u16::try_from(max_level + 1).unwrap_or(u16::MAX));
    let card_w = max_w.min(760.0);
    let card_h = (level_count * 68.0 + 30.0).max(120.0);

    // Background Card
    surface.fill_rect(Rect::new(x, y, card_w, card_h), COLOR_MERMAID_BG);
    surface.fill_rect(Rect::new(x, y, card_w, 1.0), COLOR_MERMAID_BORDER);
    surface.fill_rect(
        Rect::new(x, y + card_h - 1.0, card_w, 1.0),
        COLOR_MERMAID_BORDER,
    );
    surface.fill_rect(Rect::new(x, y, 3.0, card_h), COLOR_NODE_BORDER);

    // Title pill
    surface.draw_text(
        "Mermaid Flowchart",
        Point::new(x + 16.0, y + 10.0),
        10.0,
        COLOR_NODE_BORDER,
    );

    // Compute bounding box coordinates for each node
    let mut node_positions: HashMap<String, Rect> = HashMap::new();
    let node_w = 140.0;
    let node_h = 32.0;

    for node in graph.nodes.values() {
        let lvl = f32::from(u16::try_from(node.level).unwrap_or(u16::MAX));
        let ord = f32::from(u16::try_from(node.order).unwrap_or(u16::MAX));
        let (nx, ny) = if graph.is_horizontal {
            let step_x = (card_w - 60.0) / level_count.max(1.0);
            (x + 30.0 + lvl * step_x, y + 42.0 + ord * 44.0)
        } else {
            let step_y = (card_h - 60.0) / level_count.max(1.0);
            (x + 40.0 + ord * 160.0, y + 36.0 + lvl * step_y)
        };
        node_positions.insert(node.id.clone(), Rect::new(nx, ny, node_w, node_h));
    }

    // Draw connecting edges & arrowheads
    for edge in &graph.edges {
        if let (Some(r1), Some(r2)) = (node_positions.get(&edge.from), node_positions.get(&edge.to))
        {
            let p1 = Point::new(
                r1.origin.x + r1.size.width / 2.0,
                r1.origin.y + r1.size.height,
            );
            let p2 = Point::new(r2.origin.x + r2.size.width / 2.0, r2.origin.y);

            surface.draw_line(p1, p2, 1.5, COLOR_EDGE);

            // Draw arrowhead
            surface.draw_line(Point::new(p2.x - 4.0, p2.y - 6.0), p2, 1.5, COLOR_EDGE);
            surface.draw_line(Point::new(p2.x + 4.0, p2.y - 6.0), p2, 1.5, COLOR_EDGE);

            // Optional edge label
            if let Some(ref l) = edge.label {
                let mid_x = f32::midpoint(p1.x, p2.x) + 6.0;
                let mid_y = f32::midpoint(p1.y, p2.y) - 4.0;
                surface.draw_text(l, Point::new(mid_x, mid_y), 9.0, COLOR_EDGE_LABEL);
            }
        }
    }

    // Draw node cards
    for (id, rect) in &node_positions {
        let node = &graph.nodes[id];
        surface.fill_rect(*rect, COLOR_NODE_BG);
        surface.fill_rect(
            Rect::new(rect.origin.x, rect.origin.y, rect.size.width, 1.0),
            COLOR_NODE_BORDER,
        );
        surface.fill_rect(
            Rect::new(
                rect.origin.x,
                rect.origin.y + rect.size.height - 1.0,
                rect.size.width,
                1.0,
            ),
            COLOR_NODE_BORDER,
        );
        surface.fill_rect(
            Rect::new(rect.origin.x, rect.origin.y, 1.0, rect.size.height),
            COLOR_NODE_BORDER,
        );
        surface.fill_rect(
            Rect::new(
                rect.origin.x + rect.size.width - 1.0,
                rect.origin.y,
                1.0,
                rect.size.height,
            ),
            COLOR_NODE_BORDER,
        );

        let label_trim = if node.label.len() > 16 {
            format!("{}...", &node.label[..13])
        } else {
            node.label.clone()
        };
        surface.draw_text(
            &label_trim,
            Point::new(rect.origin.x + 10.0, rect.origin.y + 9.0),
            11.0,
            COLOR_TEXT,
        );
    }

    card_h + 16.0
}
