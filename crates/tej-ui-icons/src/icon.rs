//! Vector Icon view widget and rasterizer.

use crate::catalog::LucideIcon;
use crate::path::{PathCommand, VectorPath};
use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{Point, Rect, Size};
use tej_ui_view::View;

/// Source representation of an icon.
#[derive(Debug, Clone, PartialEq)]
pub enum IconSource {
    Lucide(LucideIcon),
    Custom(Box<VectorPath>),
}

/// Retained state for a mounted Icon widget.
#[derive(Debug, Default)]
pub struct IconState {
    pub bounds: Rect,
    pub node: u32,
    pub redraw_count: u64,
}

/// Zero-allocation vector icon component with configurable size, color, and stroke width.
#[derive(Debug, Clone, PartialEq)]
pub struct Icon {
    source: IconSource,
    size: f32,
    stroke_width: f32,
    color: Color,
}

impl Icon {
    /// Creates an icon from a standard Lucide icon enum.
    #[must_use]
    pub const fn new(icon: LucideIcon) -> Self {
        Self {
            source: IconSource::Lucide(icon),
            size: 20.0,
            stroke_width: 2.0,
            color: Color::rgb(30, 41, 59), // Slate 800
        }
    }

    /// Creates an icon from custom vector path geometry.
    #[must_use]
    pub fn from_path(path: VectorPath) -> Self {
        Self {
            source: IconSource::Custom(Box::new(path)),
            size: 20.0,
            stroke_width: 2.0,
            color: Color::rgb(30, 41, 59),
        }
    }

    /// Creates an icon by parsing an SVG path `d="..."` string.
    #[must_use]
    pub fn from_svg(d: &str) -> Self {
        Self::from_path(VectorPath::parse_svg(d))
    }

    /// Sets square display size (width and height).
    #[must_use]
    pub const fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets outline stroke width.
    #[must_use]
    pub const fn stroke_width(mut self, stroke: f32) -> Self {
        self.stroke_width = stroke;
        self
    }

    /// Sets stroke color.
    #[must_use]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Resolves the vector path geometry.
    #[must_use]
    pub fn vector_path(&self) -> VectorPath {
        match &self.source {
            IconSource::Lucide(lucide) => lucide.to_vector_path(),
            IconSource::Custom(path) => (**path).clone(),
        }
    }

    /// Paints the icon onto the 2D surface compositor.
    pub fn paint(&self, compositor: &mut dyn SurfaceCompositor, bounds: Rect) {
        let path = self.vector_path().scaled(self.size, 24.0);
        let origin = bounds.origin;

        let mut current_pt = origin;
        let mut subpath_start = origin;

        for cmd in &path.commands {
            match *cmd {
                PathCommand::MoveTo(p) => {
                    current_pt = Point::new(origin.x + p.x, origin.y + p.y);
                    subpath_start = current_pt;
                }
                PathCommand::LineTo(p) => {
                    let next_pt = Point::new(origin.x + p.x, origin.y + p.y);
                    compositor.draw_line(current_pt, next_pt, self.stroke_width, self.color);
                    current_pt = next_pt;
                }
                PathCommand::QuadTo { control, end } => {
                    let p0 = current_pt;
                    let p1 = Point::new(origin.x + control.x, origin.y + control.y);
                    let p2 = Point::new(origin.x + end.x, origin.y + end.y);
                    let poly_len = dist(p0, p1) + dist(p1, p2);
                    let steps = ((poly_len * 0.5).ceil() as usize).clamp(4, 24);
                    let mut prev = p0;
                    for i in 1..=steps {
                        let t = (i as f32) / (steps as f32);
                        let mt = 1.0 - t;
                        let pt = if i == steps {
                            p2
                        } else {
                            Point::new(
                                mt * mt * p0.x + 2.0 * mt * t * p1.x + t * t * p2.x,
                                mt * mt * p0.y + 2.0 * mt * t * p1.y + t * t * p2.y,
                            )
                        };
                        compositor.draw_line(prev, pt, self.stroke_width, self.color);
                        prev = pt;
                    }
                    current_pt = p2;
                }
                PathCommand::CubicTo { ctrl1, ctrl2, end } => {
                    let p0 = current_pt;
                    let p1 = Point::new(origin.x + ctrl1.x, origin.y + ctrl1.y);
                    let p2 = Point::new(origin.x + ctrl2.x, origin.y + ctrl2.y);
                    let p3 = Point::new(origin.x + end.x, origin.y + end.y);
                    let poly_len = dist(p0, p1) + dist(p1, p2) + dist(p2, p3);
                    let steps = ((poly_len * 0.5).ceil() as usize).clamp(6, 32);
                    let mut prev = p0;
                    for i in 1..=steps {
                        let t = (i as f32) / (steps as f32);
                        let mt = 1.0 - t;
                        let mt2 = mt * mt;
                        let t2 = t * t;
                        let pt = if i == steps {
                            p3
                        } else {
                            Point::new(
                                mt2 * mt * p0.x
                                    + 3.0 * mt2 * t * p1.x
                                    + 3.0 * mt * t2 * p2.x
                                    + t2 * t * p3.x,
                                mt2 * mt * p0.y
                                    + 3.0 * mt2 * t * p1.y
                                    + 3.0 * mt * t2 * p2.y
                                    + t2 * t * p3.y,
                            )
                        };
                        compositor.draw_line(prev, pt, self.stroke_width, self.color);
                        prev = pt;
                    }
                    current_pt = p3;
                }
                PathCommand::Close => {
                    if (current_pt.x - subpath_start.x).abs() > 1e-4
                        || (current_pt.y - subpath_start.y).abs() > 1e-4
                    {
                        compositor.draw_line(
                            current_pt,
                            subpath_start,
                            self.stroke_width,
                            self.color,
                        );
                    }
                    current_pt = subpath_start;
                }
            }
        }
    }
}

#[inline]
fn dist(a: Point, b: Point) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    (dx * dx + dy * dy).sqrt()
}

impl View for Icon {
    type State = IconState;

    fn build(&self) -> Self::State {
        IconState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(self.size, self.size)),
            node: 0,
            redraw_count: 0,
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.bounds.size = Size::new(self.size, self.size);
        state.redraw_count += 1;
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let style = tej_ui_layout::LayoutStyle {
            width: tej_ui_layout::Dimension::Points(self.size),
            height: tej_ui_layout::Dimension::Points(self.size),
            ..Default::default()
        };
        let node = arena.allocate_node(style);
        state.node = node;
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(state.bounds);
        self.paint(ctx.compositor, bounds);
    }
}
