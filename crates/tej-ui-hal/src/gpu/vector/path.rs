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
        for event in path.inner() {
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
        let mut state = SvgParseState::new(svg_d.as_bytes());
        while state.cursor < state.bytes.len() {
            skip_whitespace_and_commas(state.bytes, &mut state.cursor);
            if state.cursor >= state.bytes.len() {
                break;
            }

            let ch = state.bytes[state.cursor] as char;
            let cmd = if ch.is_ascii_alphabetic() {
                state.cursor += 1;
                state.last_cmd = ch;
                ch
            } else {
                if state.last_cmd == ' ' {
                    return Err(SvgPathError::UnknownCommand {
                        command: ch,
                        offset: state.cursor,
                    });
                }
                state.last_cmd
            };

            state.apply_command(cmd)?;
        }

        Ok(state.builder.build())
    }
}

struct SvgParseState<'a> {
    bytes: &'a [u8],
    cursor: usize,
    builder: PathBuilder,
    current_point: (f32, f32),
    start_point: (f32, f32),
    last_cmd: char,
    last_cubic_ctrl: Option<(f32, f32)>,
    last_quad_ctrl: Option<(f32, f32)>,
}

#[derive(Clone, Copy)]
enum CoordMode {
    Absolute,
    Relative,
}

impl<'a> SvgParseState<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            cursor: 0,
            builder: PathBuilder::new(),
            current_point: (0.0, 0.0),
            start_point: (0.0, 0.0),
            last_cmd: ' ',
            last_cubic_ctrl: None,
            last_quad_ctrl: None,
        }
    }

    fn resolve_point(&self, mode: CoordMode, x: f32, y: f32) -> (f32, f32) {
        match mode {
            CoordMode::Relative => (self.current_point.0 + x, self.current_point.1 + y),
            CoordMode::Absolute => (x, y),
        }
    }

    fn reset_controls(&mut self) {
        self.last_cubic_ctrl = None;
        self.last_quad_ctrl = None;
    }

    fn read_next_float(&mut self) -> Result<f32, SvgPathError> {
        read_float(self.bytes, &mut self.cursor)
    }

    fn read_next_point(&mut self, mode: CoordMode) -> Result<(f32, f32), SvgPathError> {
        let x = self.read_next_float()?;
        let y = self.read_next_float()?;
        Ok(self.resolve_point(mode, x, y))
    }

    fn apply_command(&mut self, cmd: char) -> Result<(), SvgPathError> {
        let mode = if cmd.is_ascii_lowercase() {
            CoordMode::Relative
        } else {
            CoordMode::Absolute
        };
        match cmd.to_ascii_uppercase() {
            'M' => {
                let target = self.read_next_point(mode)?;
                self.builder.move_to(target.0, target.1);
                self.current_point = target;
                self.start_point = target;
                self.last_cmd = match mode {
                    CoordMode::Relative => 'l',
                    CoordMode::Absolute => 'L',
                };
                self.reset_controls();
            }
            'L' => {
                let target = self.read_next_point(mode)?;
                self.builder.line_to(target.0, target.1);
                self.current_point = target;
                self.reset_controls();
            }
            'H' => {
                let x = self.read_next_float()?;
                let target_x = self.resolve_point(mode, x, 0.0).0;
                self.builder.line_to(target_x, self.current_point.1);
                self.current_point.0 = target_x;
                self.reset_controls();
            }
            'V' => {
                let y = self.read_next_float()?;
                let target_y = self.resolve_point(mode, 0.0, y).1;
                self.builder.line_to(self.current_point.0, target_y);
                self.current_point.1 = target_y;
                self.reset_controls();
            }
            'C' | 'S' | 'Q' | 'T' => self.apply_curve_command(cmd.to_ascii_uppercase(), mode)?,
            'A' => {
                for _ in 0..5 {
                    let _ = self.read_next_float()?;
                }
                let target = self.read_next_point(mode)?;
                self.builder.line_to(target.0, target.1);
                self.current_point = target;
                self.reset_controls();
            }
            'Z' => {
                self.builder.close();
                self.current_point = self.start_point;
                self.reset_controls();
            }
            _ => {
                return Err(SvgPathError::UnknownCommand {
                    command: cmd,
                    offset: self.cursor - 1,
                });
            }
        }
        Ok(())
    }

    fn apply_curve_command(&mut self, upper: char, mode: CoordMode) -> Result<(), SvgPathError> {
        match upper {
            'C' => {
                let c1 = self.read_next_point(mode)?;
                let c2 = self.read_next_point(mode)?;
                let target = self.read_next_point(mode)?;
                self.builder
                    .cubic_bezier_to(c1.0, c1.1, c2.0, c2.1, target.0, target.1);
                self.current_point = target;
                self.last_cubic_ctrl = Some(c2);
                self.last_quad_ctrl = None;
            }
            'S' => {
                let c1 = match self.last_cubic_ctrl {
                    Some((lx, ly)) => (
                        2.0 * self.current_point.0 - lx,
                        2.0 * self.current_point.1 - ly,
                    ),
                    None => self.current_point,
                };
                let c2 = self.read_next_point(mode)?;
                let target = self.read_next_point(mode)?;
                self.builder
                    .cubic_bezier_to(c1.0, c1.1, c2.0, c2.1, target.0, target.1);
                self.current_point = target;
                self.last_cubic_ctrl = Some(c2);
                self.last_quad_ctrl = None;
            }
            'Q' => {
                let ctrl = self.read_next_point(mode)?;
                let target = self.read_next_point(mode)?;
                self.builder
                    .quadratic_bezier_to(ctrl.0, ctrl.1, target.0, target.1);
                self.current_point = target;
                self.last_quad_ctrl = Some(ctrl);
                self.last_cubic_ctrl = None;
            }
            'T' => {
                let ctrl = match self.last_quad_ctrl {
                    Some((qx, qy)) => (
                        2.0 * self.current_point.0 - qx,
                        2.0 * self.current_point.1 - qy,
                    ),
                    None => self.current_point,
                };
                let target = self.read_next_point(mode)?;
                self.builder
                    .quadratic_bezier_to(ctrl.0, ctrl.1, target.0, target.1);
                self.current_point = target;
                self.last_quad_ctrl = Some(ctrl);
                self.last_cubic_ctrl = None;
            }
            _ => {}
        }
        Ok(())
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

    /// Ends the current subpath without closing it.
    pub fn end_open(&mut self) {
        self.inner.end(false);
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
