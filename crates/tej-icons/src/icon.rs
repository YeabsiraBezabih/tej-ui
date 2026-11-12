//! Vector Icon view widget and rasterizer.

use crate::lucide::LucideIcon;
use crate::path::{PathCommand, VectorPath};
use tej_hal::{Color, SurfaceCompositor};
use tej_layout::{Point, Rect, Size};
use tej_view::View;

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
        for cmd in &path.commands {
            match *cmd {
                PathCommand::MoveTo(p) => {
                    current_pt = Point::new(origin.x + p.x, origin.y + p.y);
                }
                PathCommand::LineTo(p) => {
                    let next_pt = Point::new(origin.x + p.x, origin.y + p.y);
                    compositor.draw_line(current_pt, next_pt, self.stroke_width, self.color);
                    current_pt = next_pt;
                }
                PathCommand::QuadTo { end, .. } | PathCommand::CubicTo { end, .. } => {
                    let next_pt = Point::new(origin.x + end.x, origin.y + end.y);
                    compositor.draw_line(current_pt, next_pt, self.stroke_width, self.color);
                    current_pt = next_pt;
                }
                PathCommand::Close => {}
            }
        }
    }
}

impl View for Icon {
    type State = IconState;

    fn build(&self) -> Self::State {
        IconState {
            bounds: Rect::from_origin_size(Point::ZERO, Size::new(self.size, self.size)),
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
}
