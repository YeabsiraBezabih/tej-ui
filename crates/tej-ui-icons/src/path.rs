//! Vector path geometry and compact SVG path data parser.

use smallvec::SmallVec;
use tej_ui_layout::{Point, Rect};

/// Vector path drawing command.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    QuadTo {
        control: Point,
        end: Point,
    },
    CubicTo {
        ctrl1: Point,
        ctrl2: Point,
        end: Point,
    },
    Close,
}

/// A compact vector path composed of sequential drawing commands.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VectorPath {
    pub commands: SmallVec<[PathCommand; 12]>,
}

impl VectorPath {
    /// Creates an empty vector path.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a move command.
    pub fn move_to(&mut self, p: Point) {
        self.commands.push(PathCommand::MoveTo(p));
    }

    /// Appends a line command.
    pub fn line_to(&mut self, p: Point) {
        self.commands.push(PathCommand::LineTo(p));
    }

    /// Appends a quadratic bezier curve command.
    pub fn quad_to(&mut self, control: Point, end: Point) {
        self.commands.push(PathCommand::QuadTo { control, end });
    }

    /// Appends a cubic bezier curve command.
    pub fn cubic_to(&mut self, ctrl1: Point, ctrl2: Point, end: Point) {
        self.commands
            .push(PathCommand::CubicTo { ctrl1, ctrl2, end });
    }

    /// Closes the current path sub-contour.
    pub fn close(&mut self) {
        self.commands.push(PathCommand::Close);
    }

    /// Computes bounding box enclosing all control and end points.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        if self.commands.is_empty() {
            return Rect::ZERO;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        let mut update = |p: Point| {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        };

        for cmd in &self.commands {
            match *cmd {
                PathCommand::MoveTo(p) | PathCommand::LineTo(p) => update(p),
                PathCommand::QuadTo { control, end } => {
                    update(control);
                    update(end);
                }
                PathCommand::CubicTo { ctrl1, ctrl2, end } => {
                    update(ctrl1);
                    update(ctrl2);
                    update(end);
                }
                PathCommand::Close => {}
            }
        }

        if min_x > max_x || min_y > max_y {
            Rect::ZERO
        } else {
            Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
        }
    }

    /// Scales the vector path to fit within target square dimensions (e.g. 24x24 -> 16x16).
    #[must_use]
    pub fn scaled(&self, target_size: f32, viewbox: f32) -> Self {
        let scale = target_size / viewbox;
        let mut scaled_cmds = SmallVec::with_capacity(self.commands.len());

        let sp = |p: Point| Point::new(p.x * scale, p.y * scale);

        for cmd in &self.commands {
            let sc = match *cmd {
                PathCommand::MoveTo(p) => PathCommand::MoveTo(sp(p)),
                PathCommand::LineTo(p) => PathCommand::LineTo(sp(p)),
                PathCommand::QuadTo { control, end } => PathCommand::QuadTo {
                    control: sp(control),
                    end: sp(end),
                },
                PathCommand::CubicTo { ctrl1, ctrl2, end } => PathCommand::CubicTo {
                    ctrl1: sp(ctrl1),
                    ctrl2: sp(ctrl2),
                    end: sp(end),
                },
                PathCommand::Close => PathCommand::Close,
            };
            scaled_cmds.push(sc);
        }

        Self {
            commands: scaled_cmds,
        }
    }

    /// Fast token parser for SVG `d="..."` path commands.
    #[must_use]
    pub fn parse_svg(d: &str) -> Self {
        let mut path = Self::new();
        let mut chars = d.chars().peekable();
        let mut current_pt = Point::ZERO;

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() || c == ',' {
                chars.next();
                continue;
            }

            match c {
                'M' | 'm' => {
                    let is_rel = c == 'm';
                    chars.next();
                    if let Some((x, y)) = parse_point(&mut chars) {
                        current_pt = if is_rel {
                            Point::new(current_pt.x + x, current_pt.y + y)
                        } else {
                            Point::new(x, y)
                        };
                        path.move_to(current_pt);
                    }
                }
                'L' | 'l' => {
                    let is_rel = c == 'l';
                    chars.next();
                    if let Some((x, y)) = parse_point(&mut chars) {
                        current_pt = if is_rel {
                            Point::new(current_pt.x + x, current_pt.y + y)
                        } else {
                            Point::new(x, y)
                        };
                        path.line_to(current_pt);
                    }
                }
                'H' | 'h' => {
                    let is_rel = c == 'h';
                    chars.next();
                    if let Some(x) = parse_float(&mut chars) {
                        current_pt.x = if is_rel { current_pt.x + x } else { x };
                        path.line_to(current_pt);
                    }
                }
                'V' | 'v' => {
                    let is_rel = c == 'v';
                    chars.next();
                    if let Some(y) = parse_float(&mut chars) {
                        current_pt.y = if is_rel { current_pt.y + y } else { y };
                        path.line_to(current_pt);
                    }
                }
                'Z' | 'z' => {
                    chars.next();
                    path.close();
                }
                _ => {
                    // Skip unsupported or extra tokens
                    chars.next();
                }
            }
        }

        path
    }
}

fn parse_float(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<f32> {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() || c == ',' {
            chars.next();
        } else {
            break;
        }
    }

    let mut buf = String::with_capacity(8);
    let mut has_digit = false;

    if let Some(&c) = chars.peek() {
        if c == '-' || c == '+' {
            buf.push(c);
            chars.next();
        }
    }

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() || c == '.' {
            buf.push(c);
            has_digit = true;
            chars.next();
        } else {
            break;
        }
    }

    if has_digit {
        buf.parse::<f32>().ok()
    } else {
        None
    }
}

fn parse_point(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<(f32, f32)> {
    let x = parse_float(chars)?;
    let y = parse_float(chars)?;
    Some((x, y))
}
