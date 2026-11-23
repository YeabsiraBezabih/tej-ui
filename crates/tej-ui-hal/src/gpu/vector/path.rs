//! Zero-allocation vector path builder and streaming SVG path data parser.

use lyon_path::Path;
use lyon_path::math::point;
use thiserror::Error;

/// Type alias for compatibility with SVG parsing nomenclature.
pub type SvgParseError = SvgPathError;

/// Parser helper for streaming SVG path definition strings.
#[derive(Clone, Copy, Debug, Default)]
pub struct SvgPathParser;

impl SvgPathParser {
    /// Parses SVG path string directly into a PathBuilder.
    pub fn parse(svg_d: &str, builder: &mut PathBuilder) -> Result<(), SvgPathError> {
        let path = VectorPath::parse_svg(svg_d)?;
        for event in path.inner().iter() {
            match event {
                lyon_path::Event::Begin { at } => builder.move_to(at.x, at.y),
                lyon_path::Event::Line { to, .. } => builder.line_to(to.x, to.y),
                lyon_path::Event::Quadratic { ctrl, to, .. } => {
                    builder.quad_to(ctrl.x, ctrl.y, to.x, to.y);
                }
                lyon_path::Event::Cubic {
                    ctrl1, ctrl2, to, ..
                } => {
                    builder.cubic_to(ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y);
                }
                lyon_path::Event::End { close, .. } => {
                    if close {
                        builder.close();
                    }
                }
            }
        }
        Ok(())
    }
}

/// Errors arising during SVG path string parsing.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SvgPathError {
    #[error("Unexpected end of path string at offset {0}")]
    UnexpectedEnd(usize),
    #[error("Invalid numeric token at offset {0}")]
    InvalidNumber(usize),
    #[error("Unknown SVG path command '{command}' at offset {offset}")]
    UnknownCommand { command: char, offset: usize },
}

/// A 2D vector path composed of lines, quadratic/cubic Bézier curves, and arcs.
#[derive(Clone, Debug, Default)]
pub struct VectorPath {
    inner: Path,
}

impl VectorPath {
    /// Construct a new empty path.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Access the underlying `lyon_path::Path`.
    #[must_use]
    pub fn inner(&self) -> &Path {
        &self.inner
    }

    /// Parses an SVG path data string (`d="..."`) conforming to SVG 2.0 grammar.
    pub fn parse_svg(svg_d: &str) -> Result<Self, SvgPathError> {
        let mut builder = PathBuilder::new();
        let bytes = svg_d.as_bytes();
        let mut cursor = 0;
        let mut current_point = (0.0f32, 0.0f32);
        let mut start_point = (0.0f32, 0.0f32);
        let mut last_cmd = ' ';
        let mut last_cubic_ctrl: Option<(f32, f32)> = None;
        let mut last_quad_ctrl: Option<(f32, f32)> = None;

        while cursor < bytes.len() {
            skip_whitespace_and_commas(bytes, &mut cursor);
            if cursor >= bytes.len() {
                break;
            }

            let ch = bytes[cursor] as char;
            let cmd = if ch.is_ascii_alphabetic() {
                cursor += 1;
                last_cmd = ch;
                ch
            } else {
                if last_cmd == ' ' {
                    return Err(SvgPathError::UnknownCommand {
                        command: ch,
                        offset: cursor,
                    });
                }
                last_cmd
            };

            let is_relative = cmd.is_ascii_lowercase();
            let upper = cmd.to_ascii_uppercase();

            match upper {
                'M' => {
                    let x = read_float(bytes, &mut cursor)?;
                    let y = read_float(bytes, &mut cursor)?;
                    let (target_x, target_y) = if is_relative {
                        (current_point.0 + x, current_point.1 + y)
                    } else {
                        (x, y)
                    };
                    builder.move_to(target_x, target_y);
                    current_point = (target_x, target_y);
                    start_point = current_point;
                    last_cmd = if is_relative { 'l' } else { 'L' };
                    last_cubic_ctrl = None;
                    last_quad_ctrl = None;
                }
                'L' => {
                    let x = read_float(bytes, &mut cursor)?;
                    let y = read_float(bytes, &mut cursor)?;
                    let (target_x, target_y) = if is_relative {
                        (current_point.0 + x, current_point.1 + y)
                    } else {
                        (x, y)
                    };
                    builder.line_to(target_x, target_y);
                    current_point = (target_x, target_y);
                    last_cubic_ctrl = None;
                    last_quad_ctrl = None;
                }
                'H' => {
                    let x = read_float(bytes, &mut cursor)?;
                    let target_x = if is_relative { current_point.0 + x } else { x };
                    builder.line_to(target_x, current_point.1);
                    current_point.0 = target_x;
                    last_cubic_ctrl = None;
                    last_quad_ctrl = None;
                }
                'V' => {
                    let y = read_float(bytes, &mut cursor)?;
                    let target_y = if is_relative { current_point.1 + y } else { y };
                    builder.line_to(current_point.0, target_y);
                    current_point.1 = target_y;
                    last_cubic_ctrl = None;
                    last_quad_ctrl = None;
                }
                'C' => {
                    let c1x = read_float(bytes, &mut cursor)?;
                    let c1y = read_float(bytes, &mut cursor)?;
                    let c2x = read_float(bytes, &mut cursor)?;
                    let c2y = read_float(bytes, &mut cursor)?;
                    let x = read_float(bytes, &mut cursor)?;
                    let y = read_float(bytes, &mut cursor)?;

                    let (t_c1x, t_c1y, t_c2x, t_c2y, t_x, t_y) = if is_relative {
                        (
                            current_point.0 + c1x,
                            current_point.1 + c1y,
                            current_point.0 + c2x,
                            current_point.1 + c2y,
                            current_point.0 + x,
                            current_point.1 + y,
                        )
                    } else {
                        (c1x, c1y, c2x, c2y, x, y)
                    };

                    builder.cubic_bezier_to(t_c1x, t_c1y, t_c2x, t_c2y, t_x, t_y);
                    current_point = (t_x, t_y);
                    last_cubic_ctrl = Some((t_c2x, t_c2y));
                    last_quad_ctrl = None;
                }
                'S' => {
                    let (c1x, c1y) = match last_cubic_ctrl {
                        Some((lx, ly)) => (2.0 * current_point.0 - lx, 2.0 * current_point.1 - ly),
                        None => current_point,
                    };
                    let c2x = read_float(bytes, &mut cursor)?;
                    let c2y = read_float(bytes, &mut cursor)?;
                    let x = read_float(bytes, &mut cursor)?;
                    let y = read_float(bytes, &mut cursor)?;

                    let (t_c2x, t_c2y, t_x, t_y) = if is_relative {
                        (
                            current_point.0 + c2x,
                            current_point.1 + c2y,
                            current_point.0 + x,
                            current_point.1 + y,
                        )
                    } else {
                        (c2x, c2y, x, y)
                    };

                    builder.cubic_bezier_to(c1x, c1y, t_c2x, t_c2y, t_x, t_y);
                    current_point = (t_x, t_y);
                    last_cubic_ctrl = Some((t_c2x, t_c2y));
                    last_quad_ctrl = None;
                }
                'Q' => {
                    let cx = read_float(bytes, &mut cursor)?;
                    let cy = read_float(bytes, &mut cursor)?;
                    let x = read_float(bytes, &mut cursor)?;
                    let y = read_float(bytes, &mut cursor)?;

                    let (t_cx, t_cy, t_x, t_y) = if is_relative {
                        (
                            current_point.0 + cx,
                            current_point.1 + cy,
                            current_point.0 + x,
                            current_point.1 + y,
                        )
                    } else {
                        (cx, cy, x, y)
                    };

                    builder.quadratic_bezier_to(t_cx, t_cy, t_x, t_y);
                    current_point = (t_x, t_y);
                    last_quad_ctrl = Some((t_cx, t_cy));
                    last_cubic_ctrl = None;
                }
                'T' => {
                    let (cx, cy) = match last_quad_ctrl {
                        Some((qx, qy)) => (2.0 * current_point.0 - qx, 2.0 * current_point.1 - qy),
                        None => current_point,
                    };
                    let x = read_float(bytes, &mut cursor)?;
                    let y = read_float(bytes, &mut cursor)?;

                    let (t_x, t_y) = if is_relative {
                        (current_point.0 + x, current_point.1 + y)
                    } else {
                        (x, y)
                    };

                    builder.quadratic_bezier_to(cx, cy, t_x, t_y);
                    current_point = (t_x, t_y);
                    last_quad_ctrl = Some((cx, cy));
                    last_cubic_ctrl = None;
                }
                'A' => {
                    let _rx = read_float(bytes, &mut cursor)?;
                    let _ry = read_float(bytes, &mut cursor)?;
                    let _rot = read_float(bytes, &mut cursor)?;
                    let _large_arc = read_float(bytes, &mut cursor)?;
                    let _sweep = read_float(bytes, &mut cursor)?;
                    let x = read_float(bytes, &mut cursor)?;
                    let y = read_float(bytes, &mut cursor)?;

                    let (t_x, t_y) = if is_relative {
                        (current_point.0 + x, current_point.1 + y)
                    } else {
                        (x, y)
                    };

                    builder.line_to(t_x, t_y);
                    current_point = (t_x, t_y);
                    last_cubic_ctrl = None;
                    last_quad_ctrl = None;
                }
                'Z' => {
                    builder.close();
                    current_point = start_point;
                    last_cubic_ctrl = None;
                    last_quad_ctrl = None;
                }
                _ => {
                    return Err(SvgPathError::UnknownCommand {
                        command: cmd,
                        offset: cursor - 1,
                    });
                }
            }
        }

        Ok(builder.build())
    }
}

/// Interactive path builder accumulating commands with zero allocations in steady state.
#[derive(Clone, Default)]
pub struct PathBuilder {
    inner: lyon_path::path::Builder,
}

impl std::fmt::Debug for PathBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathBuilder").finish()
    }
}

impl PathBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Path::builder(),
        }
    }

    pub fn clear(&mut self) {
        self.inner = Path::builder();
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.inner.begin(point(x, y));
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.inner.line_to(point(x, y));
    }

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.inner.quadratic_bezier_to(point(cx, cy), point(x, y));
    }

    pub fn quadratic_bezier_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.inner.quadratic_bezier_to(point(cx, cy), point(x, y));
    }

    pub fn cubic_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.inner
            .cubic_bezier_to(point(c1x, c1y), point(c2x, c2y), point(x, y));
    }

    pub fn cubic_bezier_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.inner
            .cubic_bezier_to(point(c1x, c1y), point(c2x, c2y), point(x, y));
    }

    pub fn arc(&mut self, cx: f32, cy: f32, radius: f32, start_angle: f32, end_angle: f32) {
        let sweep_angle = end_angle - start_angle;
        let arc = lyon_geom::Arc {
            center: point(cx, cy),
            radii: lyon_geom::euclid::Vector2D::new(radius, radius),
            start_angle: lyon_geom::Angle::radians(start_angle),
            sweep_angle: lyon_geom::Angle::radians(sweep_angle),
            x_rotation: lyon_geom::Angle::radians(0.0),
        };
        arc.for_each_quadratic_bezier(&mut |curve| {
            self.inner.quadratic_bezier_to(curve.ctrl, curve.to);
        });
    }

    pub fn close(&mut self) {
        self.inner.end(true);
    }

    #[must_use]
    pub fn build(&self) -> VectorPath {
        VectorPath {
            inner: self.inner.clone().build(),
        }
    }

    #[must_use]
    pub fn finish(self) -> VectorPath {
        VectorPath {
            inner: self.inner.build(),
        }
    }
}

fn skip_whitespace_and_commas(bytes: &[u8], cursor: &mut usize) {
    while *cursor < bytes.len() {
        let b = bytes[*cursor];
        if b.is_ascii_whitespace() || b == b',' {
            *cursor += 1;
        } else {
            break;
        }
    }
}

fn read_float(bytes: &[u8], cursor: &mut usize) -> Result<f32, SvgPathError> {
    skip_whitespace_and_commas(bytes, cursor);
    if *cursor >= bytes.len() {
        return Err(SvgPathError::UnexpectedEnd(*cursor));
    }

    let start = *cursor;
    if bytes[*cursor] == b'-' || bytes[*cursor] == b'+' {
        *cursor += 1;
    }

    let mut has_digits = false;
    while *cursor < bytes.len() && bytes[*cursor].is_ascii_digit() {
        has_digits = true;
        *cursor += 1;
    }

    if *cursor < bytes.len() && bytes[*cursor] == b'.' {
        *cursor += 1;
        while *cursor < bytes.len() && bytes[*cursor].is_ascii_digit() {
            has_digits = true;
            *cursor += 1;
        }
    }

    if !has_digits {
        return Err(SvgPathError::InvalidNumber(start));
    }

    let s = std::str::from_utf8(&bytes[start..*cursor])
        .map_err(|_| SvgPathError::InvalidNumber(start))?;
    s.parse::<f32>()
        .map_err(|_| SvgPathError::InvalidNumber(start))
}
