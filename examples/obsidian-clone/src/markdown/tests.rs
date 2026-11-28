//! Unit test suite for LaTeX, Mermaid, and SVG markdown parsing and compilation.

use super::latex::{MathNode, parse_latex};
use super::mermaid::parse_mermaid;
use super::svg::extract_tags;

#[test]
fn test_latex_parser_fractions_and_sqrt() {
    let nodes = parse_latex(r"\frac{-k(x - x_t) - c v}{m}");
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        MathNode::Fraction(num, den) => {
            assert!(!num.is_empty());
            assert_eq!(den.len(), 1);
        }
        _ => panic!("Expected fraction math node"),
    }

    let sqrt_nodes = parse_latex(r"\sqrt{\frac{k}{m}}");
    assert_eq!(sqrt_nodes.len(), 1);
    match &sqrt_nodes[0] {
        MathNode::Sqrt(inner) => {
            assert_eq!(inner.len(), 1);
        }
        _ => panic!("Expected sqrt math node"),
    }
}

#[test]
fn test_latex_parser_subscripts_and_superscripts() {
    let nodes = parse_latex(r"\omega_0 = x^2");
    assert!(!nodes.is_empty());
    let mut has_subscript = false;
    let mut has_superscript = false;
    for n in &nodes {
        if let MathNode::Subscript(base, sub) = n {
            assert_eq!(base, "ω");
            assert_eq!(sub, "0");
            has_subscript = true;
        }
        if let MathNode::Superscript(base, sup) = n {
            assert_eq!(base, "x");
            assert_eq!(sup, "2");
            has_superscript = true;
        }
    }
    assert!(has_subscript, "Subscript should be parsed");
    assert!(has_superscript, "Superscript should be parsed");
}

#[test]
fn test_mermaid_parser_flowchart() {
    let input = r#"
    graph TD
    A["tej-core"] --> B["tej-view"]
    B --> C["tej-layout"]
    "#;
    let graph = parse_mermaid(input);
    assert!(!graph.is_horizontal);
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);
    assert!(graph.nodes.contains_key("A"));
    assert!(graph.nodes.contains_key("B"));
    assert!(graph.nodes.contains_key("C"));
}

#[test]
fn test_mermaid_parser_with_edge_labels() {
    let input = r"
    graph LR
    Start -->|Valid| Process
    Process -->|Done| End
    ";
    let graph = parse_mermaid(input);
    assert!(graph.is_horizontal);
    assert_eq!(graph.edges.len(), 2);
    assert_eq!(graph.edges[0].label.as_deref(), Some("Valid"));
    assert_eq!(graph.edges[1].label.as_deref(), Some("Done"));
}

#[test]
fn test_svg_tag_parser() {
    let svg_content = r##"
    <svg width="400" height="100">
        <rect x="10" y="20" width="80" height="40" fill="#3b82f6"/>
        <line x1="90" y1="40" x2="150" y2="40" stroke="#10b981"/>
        <text x="25" y="45" fill="#ffffff">Buffer</text>
    </svg>
    "##;
    let tags = extract_tags(svg_content);
    assert_eq!(tags.len(), 4);
    assert!(tags[0].starts_with("<svg"));
    assert!(tags[1].starts_with("<rect"));
    assert!(tags[2].starts_with("<line"));
    assert!(tags[3].starts_with("<text"));
}

#[test]
fn test_editor_text_operations() {
    use crate::desktop_gui::{delete_before_cursor, insert_char_at, insert_newline_at};

    let mut doc = String::from("Hello World");
    let (line, col) = insert_char_at(&mut doc, 0, 5, ", Beautiful");
    assert_eq!(doc, "Hello, Beautiful World");
    assert_eq!((line, col), (0, 16));

    let (line, col) = insert_newline_at(&mut doc, 0, 16);
    assert_eq!(doc, "Hello, Beautiful\n World");
    assert_eq!((line, col), (1, 0));

    let (line, col) = delete_before_cursor(&mut doc, 1, 0);
    assert_eq!(doc, "Hello, Beautiful World");
    assert_eq!((line, col), (0, 16));
}
