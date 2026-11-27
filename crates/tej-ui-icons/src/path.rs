//! Vector path geometry and compact SVG path data parser.

use smallvec::SmallVec;
use std::f64::consts::{FRAC_PI_2, TAU};
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

    /// Full SVG 1.1 `d="..."` path parser supporting `M/m`, `L/l`, `H/h`, `V/v`,
    /// `C/c`, `S/s`, `Q/q`, `T/t`, `A/a` (converted to cubic Béziers), `Z/z`,
    /// and implicit repeated coordinate tuples.
    #[must_use]
    pub fn parse_svg(d: &str) -> Self {
        let mut path = Self::new();
        let mut chars = d.chars().peekable();
        let mut current_pt = Point::ZERO;
        let mut subpath_start = Point::ZERO;
        let mut last_cubic_ctrl: Option<Point> = None;
        let mut last_quad_ctrl: Option<Point> = None;

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() || c == ',' {
                chars.next();
                continue;
            }

            if !c.is_ascii_alphabetic() {
                // Unexpected stray token outside a command
                chars.next();
                continue;
            }

            chars.next();
            match c {
                'M' | 'm' => {
                    let is_rel = c == 'm';
                    let mut first = true;
                    while has_number(&mut chars) {
                        let Some((x, y)) = parse_point(&mut chars) else {
                            break;
                        };
                        current_pt = if is_rel {
                            Point::new(current_pt.x + x, current_pt.y + y)
                        } else {
                            Point::new(x, y)
                        };
                        if first {
                            subpath_start = current_pt;
                            path.move_to(current_pt);
                            first = false;
                        } else {
                            path.line_to(current_pt);
                        }
                    }
                    last_cubic_ctrl = None;
                    last_quad_ctrl = None;
                }
                'L' | 'l' => {
                    let is_rel = c == 'l';
                    while has_number(&mut chars) {
                        let Some((x, y)) = parse_point(&mut chars) else {
                            break;
                        };
                        current_pt = if is_rel {
                            Point::new(current_pt.x + x, current_pt.y + y)
                        } else {
                            Point::new(x, y)
                        };
                        path.line_to(current_pt);
                    }
                    last_cubic_ctrl = None;
                    last_quad_ctrl = None;
                }
                'H' | 'h' => {
                    let is_rel = c == 'h';
                    while has_number(&mut chars) {
                        let Some(x) = parse_float(&mut chars) else {
                            break;
                        };
                        current_pt.x = if is_rel { current_pt.x + x } else { x };
                        path.line_to(current_pt);
                    }
                    last_cubic_ctrl = None;
                    last_quad_ctrl = None;
                }
                'V' | 'v' => {
                    let is_rel = c == 'v';
                    while has_number(&mut chars) {
                        let Some(y) = parse_float(&mut chars) else {
                            break;
                        };
                        current_pt.y = if is_rel { current_pt.y + y } else { y };
                        path.line_to(current_pt);
                    }
                    last_cubic_ctrl = None;
                    last_quad_ctrl = None;
                }
                'C' | 'c' => {
                    let is_rel = c == 'c';
                    while has_number(&mut chars) {
                        let Some((x1, y1)) = parse_point(&mut chars) else {
                            break;
                        };
                        let Some((x2, y2)) = parse_point(&mut chars) else {
                            break;
                        };
                        let Some((x, y)) = parse_point(&mut chars) else {
                            break;
                        };
                        let (ctrl1, ctrl2, end) = if is_rel {
                            (
                                Point::new(current_pt.x + x1, current_pt.y + y1),
                                Point::new(current_pt.x + x2, current_pt.y + y2),
                                Point::new(current_pt.x + x, current_pt.y + y),
                            )
                        } else {
                            (Point::new(x1, y1), Point::new(x2, y2), Point::new(x, y))
                        };
                        path.cubic_to(ctrl1, ctrl2, end);
                        last_cubic_ctrl = Some(ctrl2);
                        last_quad_ctrl = None;
                        current_pt = end;
                    }
                }
                'S' | 's' => {
                    let is_rel = c == 's';
                    while has_number(&mut chars) {
                        let Some((x2, y2)) = parse_point(&mut chars) else {
                            break;
                        };
                        let Some((x, y)) = parse_point(&mut chars) else {
                            break;
                        };
                        let ctrl1 = match last_cubic_ctrl {
                            Some(prev) => {
                                Point::new(2.0 * current_pt.x - prev.x, 2.0 * current_pt.y - prev.y)
                            }
                            None => current_pt,
                        };
                        let (ctrl2, end) = if is_rel {
                            (
                                Point::new(current_pt.x + x2, current_pt.y + y2),
                                Point::new(current_pt.x + x, current_pt.y + y),
                            )
                        } else {
                            (Point::new(x2, y2), Point::new(x, y))
                        };
                        path.cubic_to(ctrl1, ctrl2, end);
                        last_cubic_ctrl = Some(ctrl2);
                        last_quad_ctrl = None;
                        current_pt = end;
                    }
                }
                'Q' | 'q' => {
                    let is_rel = c == 'q';
                    while has_number(&mut chars) {
                        let Some((x1, y1)) = parse_point(&mut chars) else {
                            break;
                        };
                        let Some((x, y)) = parse_point(&mut chars) else {
                            break;
                        };
                        let (control, end) = if is_rel {
                            (
                                Point::new(current_pt.x + x1, current_pt.y + y1),
                                Point::new(current_pt.x + x, current_pt.y + y),
                            )
                        } else {
                            (Point::new(x1, y1), Point::new(x, y))
                        };
                        path.quad_to(control, end);
                        last_quad_ctrl = Some(control);
                        last_cubic_ctrl = None;
                        current_pt = end;
                    }
                }
                'T' | 't' => {
                    let is_rel = c == 't';
                    while has_number(&mut chars) {
                        let Some((x, y)) = parse_point(&mut chars) else {
                            break;
                        };
                        let control = match last_quad_ctrl {
                            Some(prev) => {
                                Point::new(2.0 * current_pt.x - prev.x, 2.0 * current_pt.y - prev.y)
                            }
                            None => current_pt,
                        };
                        let end = if is_rel {
                            Point::new(current_pt.x + x, current_pt.y + y)
                        } else {
                            Point::new(x, y)
                        };
                        path.quad_to(control, end);
                        last_quad_ctrl = Some(control);
                        last_cubic_ctrl = None;
                        current_pt = end;
                    }
                }
                'A' | 'a' => {
                    let is_rel = c == 'a';
                    while has_number(&mut chars) {
                        let Some(rx) = parse_float(&mut chars) else {
                            break;
                        };
                        let Some(ry) = parse_float(&mut chars) else {
                            break;
                        };
                        let Some(x_rot) = parse_float(&mut chars) else {
                            break;
                        };
                        let Some(large_arc) = parse_flag(&mut chars) else {
                            break;
                        };
                        let Some(sweep) = parse_flag(&mut chars) else {
                            break;
                        };
                        let Some((x, y)) = parse_point(&mut chars) else {
                            break;
                        };
                        let end = if is_rel {
                            Point::new(current_pt.x + x, current_pt.y + y)
                        } else {
                            Point::new(x, y)
                        };
                        arc_to_cubics(&mut path, current_pt, end, rx, ry, x_rot, large_arc, sweep);
                        current_pt = end;
                        last_cubic_ctrl = None;
                        last_quad_ctrl = None;
                    }
                }
                'Z' | 'z' => {
                    path.close();
                    current_pt = subpath_start;
                    last_cubic_ctrl = None;
                    last_quad_ctrl = None;
                }
                _ => {
                    last_cubic_ctrl = None;
                    last_quad_ctrl = None;
                }
            }
        }

        path
    }
}

/// Converts an SVG elliptical arc from `start` to `end` into 1..=4 cubic Bézier segments
/// following the W3C SVG 1.1 Implementation Notes (Section F.6.5 & F.6.6).
#[allow(clippy::too_many_arguments)]
fn arc_to_cubics(
    path: &mut VectorPath,
    start: Point,
    end: Point,
    rx_in: f32,
    ry_in: f32,
    x_axis_rotation_deg: f32,
    large_arc: bool,
    sweep: bool,
) {
    let x1 = f64::from(start.x);
    let y1 = f64::from(start.y);
    let x2 = f64::from(end.x);
    let y2 = f64::from(end.y);

    if (x1 - x2).abs() < 1e-6 && (y1 - y2).abs() < 1e-6 {
        return;
    }

    let mut rx = f64::from(rx_in).abs();
    let mut ry = f64::from(ry_in).abs();
    if rx < 1e-6 || ry < 1e-6 {
        path.line_to(end);
        return;
    }

    let phi = f64::from(x_axis_rotation_deg).to_radians();
    let (sin_phi, cos_phi) = phi.sin_cos();

    let dx2 = (x1 - x2) * 0.5;
    let dy2 = (y1 - y2) * 0.5;
    let x1p = cos_phi * dx2 + sin_phi * dy2;
    let y1p = -sin_phi * dx2 + cos_phi * dy2;

    let x1p2 = x1p * x1p;
    let y1p2 = y1p * y1p;
    let lambda = x1p2 / (rx * rx) + y1p2 / (ry * ry);
    if lambda > 1.0 {
        let scale = lambda.sqrt();
        rx *= scale;
        ry *= scale;
    }

    let rx2 = rx * rx;
    let ry2 = ry * ry;
    let num = (rx2 * ry2 - rx2 * y1p2 - ry2 * x1p2).max(0.0);
    let den = rx2 * y1p2 + ry2 * x1p2;
    let coef_mag = if den > 1e-12 { (num / den).sqrt() } else { 0.0 };
    let coef = if large_arc == sweep {
        -coef_mag
    } else {
        coef_mag
    };

    let cxp = coef * (rx * y1p / ry);
    let cyp = coef * (-ry * x1p / rx);

    let cx = cos_phi * cxp - sin_phi * cyp + (x1 + x2) * 0.5;
    let cy = sin_phi * cxp + cos_phi * cyp + (y1 + y2) * 0.5;

    let ux = (x1p - cxp) / rx;
    let uy = (y1p - cyp) / ry;
    let vx = (-x1p - cxp) / rx;
    let vy = (-y1p - cyp) / ry;

    let theta1 = vec_angle(1.0, 0.0, ux, uy);
    let mut dtheta = vec_angle(ux, uy, vx, vy);
    if !sweep && dtheta > 0.0 {
        dtheta -= TAU;
    } else if sweep && dtheta < 0.0 {
        dtheta += TAU;
    }

    let segments = (dtheta.abs() / FRAC_PI_2).ceil().max(1.0) as usize;
    let delta = dtheta / (segments as f64);
    let alpha = (4.0 / 3.0) * (delta * 0.25).tan();

    let map_pt = |px: f64, py: f64| -> Point {
        let x = cos_phi * (rx * px) - sin_phi * (ry * py) + cx;
        let y = sin_phi * (rx * px) + cos_phi * (ry * py) + cy;
        Point::new(x as f32, y as f32)
    };

    for i in 0..segments {
        let t1 = theta1 + (i as f64) * delta;
        let t2 = t1 + delta;
        let (sin1, cos1) = t1.sin_cos();
        let (sin2, cos2) = t2.sin_cos();

        let ctrl1 = map_pt(cos1 - alpha * sin1, sin1 + alpha * cos1);
        let ctrl2 = map_pt(cos2 + alpha * sin2, sin2 - alpha * cos2);
        let seg_end = if i + 1 == segments {
            end
        } else {
            map_pt(cos2, sin2)
        };
        path.cubic_to(ctrl1, ctrl2, seg_end);
    }
}

#[inline]
fn vec_angle(ux: f64, uy: f64, vx: f64, vy: f64) -> f64 {
    let dot = ux * vx + uy * vy;
    let cross = ux * vy - uy * vx;
    cross.atan2(dot)
}

fn skip_separators(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() || c == ',' {
            chars.next();
        } else {
            break;
        }
    }
}

fn has_number(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    skip_separators(chars);
    matches!(chars.peek(), Some('0'..='9' | '-' | '+' | '.'))
}

fn parse_flag(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<bool> {
    skip_separators(chars);
    match chars.peek() {
        Some('0') => {
            chars.next();
            Some(false)
        }
        Some('1') => {
            chars.next();
            Some(true)
        }
        _ => None,
    }
}

fn parse_float(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<f32> {
    skip_separators(chars);

    let mut buf = String::with_capacity(12);
    let mut has_digit = false;
    let mut has_dot = false;

    if let Some(&c) = chars.peek() {
        if c == '-' || c == '+' {
            buf.push(c);
            chars.next();
        }
    }

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            buf.push(c);
            has_digit = true;
            chars.next();
        } else if c == '.' && !has_dot {
            buf.push(c);
            has_dot = true;
            chars.next();
        } else {
            break;
        }
    }

    // Optional scientific notation exponent (e.g., 1e-3, 2.5E+2)
    if has_digit && matches!(chars.peek(), Some('e' | 'E')) {
        buf.push(chars.next().unwrap());
        if matches!(chars.peek(), Some('+' | '-')) {
            buf.push(chars.next().unwrap());
        }
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                buf.push(c);
                chars.next();
            } else {
                break;
            }
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
