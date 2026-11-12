use tej_hal::{Color, SurfaceCompositor};
use tej_icons::{Icon, LucideIcon, PathCommand, VectorPath};
use tej_layout::{Point, Rect, Size};
use tej_view::View;

#[test]
fn test_svg_path_parsing() {
    let d = "M10 20 L30 40 Z";
    let path = VectorPath::parse_svg(d);
    assert_eq!(path.commands.len(), 3);
    assert_eq!(
        path.commands[0],
        PathCommand::MoveTo(Point::new(10.0, 20.0))
    );
    assert_eq!(
        path.commands[1],
        PathCommand::LineTo(Point::new(30.0, 40.0))
    );
    assert_eq!(path.commands[2], PathCommand::Close);

    let bounds = path.bounds();
    assert_eq!(bounds.origin.x, 10.0);
    assert_eq!(bounds.origin.y, 20.0);
    assert_eq!(bounds.size.width, 20.0);
    assert_eq!(bounds.size.height, 20.0);
}

#[test]
fn test_lucide_icon_vector_paths() {
    let search = LucideIcon::Search;
    let path = search.to_vector_path();
    assert!(!path.commands.is_empty());

    let plus = LucideIcon::Plus;
    let p_path = plus.to_vector_path();
    assert!(!p_path.commands.is_empty());
}

#[derive(Debug)]
struct MockCompositor {
    draw_line_count: usize,
}

impl SurfaceCompositor for MockCompositor {
    fn begin_frame(&mut self, _dirty_rects: &[Rect]) {}
    fn fill_rect(&mut self, _rect: Rect, _color: Color) {}
    fn fill_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: Color) {}
    fn draw_line(&mut self, _p1: Point, _p2: Point, _stroke_width: f32, _color: Color) {
        self.draw_line_count += 1;
    }
    fn draw_text(&mut self, _text: &str, _origin: Point, _font_size: f32, _color: Color) {}
    fn end_frame(&mut self) {}
}

#[test]
fn test_icon_view_widget_and_paint() {
    let icon = Icon::new(LucideIcon::ChevronDown)
        .size(16.0)
        .stroke_width(1.5)
        .color(Color::rgb(100, 100, 100));

    assert!(!icon.vector_path().commands.is_empty());

    let mut state = icon.build();
    assert_eq!(state.bounds.size, Size::new(16.0, 16.0));

    icon.rebuild(&mut state);
    assert_eq!(state.redraw_count, 1);

    let mut comp = MockCompositor { draw_line_count: 0 };
    icon.paint(&mut comp, Rect::new(10.0, 10.0, 16.0, 16.0));
    assert!(comp.draw_line_count > 0);
}

#[test]
fn test_custom_svg_icon() {
    let custom = Icon::from_svg("M0 0 L10 10").size(24.0);
    assert!(matches!(
        custom.vector_path().commands[1],
        PathCommand::LineTo(_)
    ));
}
