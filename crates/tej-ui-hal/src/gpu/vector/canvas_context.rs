//! Immediate-mode 2D canvas drawing context with affine transformation stack.

use crate::gpu::types::PremulColor;
use crate::gpu::vector::path::{PathBuilder, SvgPathParser, VectorPath};
use crate::gpu::vector::tessellator::{PathTessellator, TessellatedMesh, VectorVertex};
pub use lyon_tessellation::FillRule;

/// 2D affine transformation matrix `[a, b, c, d, e, f]` representing:
/// `x' = a * x + c * y + e`
/// `y' = b * x + d * y + f`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform2D {
    #[must_use]
    pub const fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    #[must_use]
    pub const fn translation(tx: f32, ty: f32) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: tx,
            f: ty,
        }
    }

    #[must_use]
    pub const fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        }
    }

    #[must_use]
    pub fn rotation(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }

    #[must_use]
    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    #[must_use]
    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }
}

#[derive(Clone, Debug)]
struct CanvasState {
    transform: Transform2D,
    fill_color: PremulColor,
    stroke_color: PremulColor,
    line_width: f32,
    fill_rule: FillRule,
}

/// Immediate-mode 2D canvas context for zero-allocation vector drawing.
#[derive(Debug)]
pub struct Canvas2DContext {
    state: CanvasState,
    state_stack: Vec<CanvasState>,
    builder: PathBuilder,
    tessellator: PathTessellator,
    mesh: TessellatedMesh,
}

impl Default for Canvas2DContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas2DContext {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: CanvasState {
                transform: Transform2D::identity(),
                fill_color: PremulColor::BLACK,
                stroke_color: PremulColor::BLACK,
                line_width: 1.0,
                fill_rule: FillRule::NonZero,
            },
            state_stack: Vec::with_capacity(16),
            builder: PathBuilder::new(),
            tessellator: PathTessellator::new(),
            mesh: TessellatedMesh::new(),
        }
    }

    /// Saves the current transformation and style state onto the stack.
    pub fn save(&mut self) {
        self.state_stack.push(self.state.clone());
    }

    /// Restores the most recently saved state from the stack.
    pub fn restore(&mut self) {
        if let Some(saved) = self.state_stack.pop() {
            self.state = saved;
        }
    }

    pub fn translate(&mut self, tx: f32, ty: f32) {
        let t = Transform2D::translation(tx, ty);
        self.state.transform = self.state.transform.multiply(&t);
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        let s = Transform2D::scale(sx, sy);
        self.state.transform = self.state.transform.multiply(&s);
    }

    pub fn rotate(&mut self, radians: f32) {
        let r = Transform2D::rotation(radians);
        self.state.transform = self.state.transform.multiply(&r);
    }

    pub fn set_fill_color(&mut self, color: PremulColor) {
        self.state.fill_color = color;
    }

    pub fn set_stroke_color(&mut self, color: PremulColor) {
        self.state.stroke_color = color;
    }

    pub fn set_line_width(&mut self, width: f32) {
        self.state.line_width = width.max(0.1);
    }

    pub fn set_fill_rule(&mut self, rule: FillRule) {
        self.state.fill_rule = rule;
    }

    pub fn begin_path(&mut self) {
        self.builder.clear();
    }

    pub fn close_path(&mut self) {
        self.builder.close();
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        let (tx, ty) = self.state.transform.transform_point(x, y);
        self.builder.move_to(tx, ty);
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        let (tx, ty) = self.state.transform.transform_point(x, y);
        self.builder.line_to(tx, ty);
    }

    pub fn quadratic_curve_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
        let (tcpx, tcpy) = self.state.transform.transform_point(cpx, cpy);
        let (tx, ty) = self.state.transform.transform_point(x, y);
        self.builder.quad_to(tcpx, tcpy, tx, ty);
    }

    pub fn bezier_curve_to(&mut self, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32) {
        let (tcp1x, tcp1y) = self.state.transform.transform_point(cp1x, cp1y);
        let (tcp2x, tcp2y) = self.state.transform.transform_point(cp2x, cp2y);
        let (tx, ty) = self.state.transform.transform_point(x, y);
        self.builder.cubic_to(tcp1x, tcp1y, tcp2x, tcp2y, tx, ty);
    }

    pub fn arc(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
    ) {
        let (cx, cy) = self.state.transform.transform_point(center_x, center_y);
        self.builder.arc(cx, cy, radius, start_angle, end_angle);
    }

    /// Loads and parses an SVG path string data (d attribute).
    pub fn parse_svg_path(&mut self, svg_d: &str) {
        let mut temp_builder = PathBuilder::new();
        if SvgPathParser::parse(svg_d, &mut temp_builder).is_ok() {
            let temp_path = temp_builder.build();
            // Transform and append into current path
            for event in temp_path.inner() {
                match event {
                    lyon_path::Event::Begin { at } => {
                        let (tx, ty) = self.state.transform.transform_point(at.x, at.y);
                        self.builder.move_to(tx, ty);
                    }
                    lyon_path::Event::Line { to, .. } => {
                        let (tx, ty) = self.state.transform.transform_point(to.x, to.y);
                        self.builder.line_to(tx, ty);
                    }
                    lyon_path::Event::Quadratic { ctrl, to, .. } => {
                        let (cx, cy) = self.state.transform.transform_point(ctrl.x, ctrl.y);
                        let (tx, ty) = self.state.transform.transform_point(to.x, to.y);
                        self.builder.quad_to(cx, cy, tx, ty);
                    }
                    lyon_path::Event::Cubic {
                        ctrl1, ctrl2, to, ..
                    } => {
                        let (c1x, c1y) = self.state.transform.transform_point(ctrl1.x, ctrl1.y);
                        let (c2x, c2y) = self.state.transform.transform_point(ctrl2.x, ctrl2.y);
                        let (tx, ty) = self.state.transform.transform_point(to.x, to.y);
                        self.builder.cubic_to(c1x, c1y, c2x, c2y, tx, ty);
                    }
                    lyon_path::Event::End { close, .. } => {
                        if close {
                            self.builder.close();
                        }
                    }
                }
            }
        }
    }

    /// Tessellates the current path into the mesh using the active fill color and rule.
    pub fn fill(&mut self) {
        let path = self.builder.build();
        self.tessellator.tessellate_fill(
            &path,
            self.state.fill_color,
            self.state.fill_rule,
            &mut self.mesh,
        );
    }

    /// Tessellates the current path into the mesh using the active stroke color and line width.
    pub fn stroke(&mut self) {
        let path = self.builder.build();
        self.tessellator.tessellate_stroke(
            &path,
            self.state.stroke_color,
            self.state.line_width,
            &mut self.mesh,
        );
    }

    /// Draws a solid rectangle directly into the mesh without tessellator overhead.
    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let base_idx = self.mesh.vertices.len() as u32;
        let c = self.state.fill_color.channels;

        let (p0_x, p0_y) = self.state.transform.transform_point(x, y);
        let (p1_x, p1_y) = self.state.transform.transform_point(x + width, y);
        let (p2_x, p2_y) = self.state.transform.transform_point(x + width, y + height);
        let (p3_x, p3_y) = self.state.transform.transform_point(x, y + height);

        self.mesh.vertices.push(VectorVertex {
            position: [p0_x, p0_y],
            color: c,
        });
        self.mesh.vertices.push(VectorVertex {
            position: [p1_x, p1_y],
            color: c,
        });
        self.mesh.vertices.push(VectorVertex {
            position: [p2_x, p2_y],
            color: c,
        });
        self.mesh.vertices.push(VectorVertex {
            position: [p3_x, p3_y],
            color: c,
        });

        self.mesh.indices.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);
    }

    /// Strokes a rectangle using path builder.
    pub fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.begin_path();
        self.move_to(x, y);
        self.line_to(x + width, y);
        self.line_to(x + width, y + height);
        self.line_to(x, y + height);
        self.close_path();
        self.stroke();
    }

    /// Tessellates an arbitrary pre-built path directly.
    pub fn draw_path_fill(&mut self, path: &VectorPath) {
        self.tessellator.tessellate_fill(
            path,
            self.state.fill_color,
            self.state.fill_rule,
            &mut self.mesh,
        );
    }

    /// Tessellates an arbitrary pre-built path with stroke styling.
    pub fn draw_path_stroke(&mut self, path: &VectorPath) {
        self.tessellator.tessellate_stroke(
            path,
            self.state.stroke_color,
            self.state.line_width,
            &mut self.mesh,
        );
    }

    /// Returns a reference to the accumulated GPU mesh.
    #[must_use]
    pub const fn mesh(&self) -> &TessellatedMesh {
        &self.mesh
    }

    /// Clears canvas mesh and resets path without deallocating capacities.
    pub fn clear(&mut self) {
        self.mesh.clear();
        self.builder.clear();
        self.state_stack.clear();
        self.state = CanvasState {
            transform: Transform2D::identity(),
            fill_color: PremulColor::BLACK,
            stroke_color: PremulColor::BLACK,
            line_width: 1.0,
            fill_rule: FillRule::NonZero,
        };
    }
}
