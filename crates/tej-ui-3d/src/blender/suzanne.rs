//! Procedural 3D model geometry generator representing complex Blender sculpts.

use crate::math::Vec3;
use crate::mesh::{Mesh, Vertex};
use std::f32::consts::PI;
use tej_ui_hal::Color;

/// Raw 3D mesh representation extracted from a Blender asset.
#[derive(Debug, Clone)]
pub struct BlenderMesh {
    pub name: String,
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<Color>,
    pub triangles: Vec<[u32; 3]>,
}

impl BlenderMesh {
    /// Generates a complex sculpted 3D model (Suzanne-like torus-knot topology)
    /// with hundreds of vertices and triangles suitable for Nanite meshlet clustering.
    #[must_use]
    pub fn generate_sculpt(rings: usize, tube_segments: usize) -> Self {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let colors = Vec::new();
        let mut triangles = Vec::new();

        let harm_p = 2.0f32; // Torus knot harmonic p
        let harm_q = 3.0f32; // Torus knot harmonic q
        let r_tube = 0.4f32;
        let rings_f = crate::math::count_to_f32(rings);
        let segs_f = crate::math::count_to_f32(tube_segments);

        for i in 0..rings {
            let u_angle = (crate::math::count_to_f32(i) / rings_f) * 2.0 * PI;
            let radius = 1.2 + 0.6 * (harm_q * u_angle).cos();
            let x0 = radius * (harm_p * u_angle).cos();
            let y0 = radius * (harm_p * u_angle).sin();
            let z0 = 0.6 * (harm_q * u_angle).sin();

            // Tangent approximation
            let u_next = u_angle + 0.01;
            let r_next = 1.2 + 0.6 * (harm_q * u_next).cos();
            let x_next = r_next * (harm_p * u_next).cos();
            let y_next = r_next * (harm_p * u_next).sin();
            let z_next = 0.6 * (harm_q * u_next).sin();

            let tx = x_next - x0;
            let ty = y_next - y0;
            let tz = z_next - z0;
            let t_len = (tx * tx + ty * ty + tz * tz).sqrt().max(1e-5);
            let (tx, ty, tz) = (tx / t_len, ty / t_len, tz / t_len);

            // Normal vector
            let mut nx = -ty;
            let mut ny = tx;
            let nz = 0.0f32;
            let n_len = (nx * nx + ny * ny + nz * nz).sqrt();
            if n_len > 1e-5 {
                nx /= n_len;
                ny /= n_len;
            } else {
                nx = 1.0;
                ny = 0.0;
            }

            // Binormal
            let bx = ty * nz - tz * ny;
            let by = tz * nx - tx * nz;
            let bz = tx * ny - ty * nx;

            for j in 0..tube_segments {
                let v_angle = (crate::math::count_to_f32(j) / segs_f) * 2.0 * PI;
                let cos_v = v_angle.cos();
                let sin_v = v_angle.sin();

                let px = x0 + r_tube * (cos_v * nx + sin_v * bx);
                let py = y0 + r_tube * (cos_v * ny + sin_v * by);
                let pz = z0 + r_tube * (cos_v * nz + sin_v * bz);

                vertices.push([px, py, pz]);
                normals.push([
                    cos_v * nx + sin_v * bx,
                    cos_v * ny + sin_v * by,
                    cos_v * nz + sin_v * bz,
                ]);
            }
        }

        // Generate triangles
        for i in 0..rings {
            let next_i = (i + 1) % rings;
            for j in 0..tube_segments {
                let next_j = (j + 1) % tube_segments;

                let idx0 = (i * tube_segments + j) as u32;
                let idx1 = (next_i * tube_segments + j) as u32;
                let idx2 = (next_i * tube_segments + next_j) as u32;
                let idx3 = (i * tube_segments + next_j) as u32;

                triangles.push([idx0, idx1, idx2]);
                triangles.push([idx0, idx2, idx3]);
            }
        }

        Self {
            name: "Blender_Sculpt_Mesh".to_string(),
            vertices,
            normals,
            colors,
            triangles,
        }
    }

    /// Convert BlenderMesh into an indexed 3D Mesh ready for software or hardware rasterization.
    #[must_use]
    pub fn to_mesh(&self, color: Color) -> Mesh {
        // Calculate smooth geometric vertex normals from triangle faces
        let mut accumulated_normals = vec![Vec3::ZERO; self.vertices.len()];
        for tri in &self.triangles {
            let idx0 = tri[0] as usize;
            let idx1 = tri[1] as usize;
            let idx2 = tri[2] as usize;
            if idx0 < self.vertices.len()
                && idx1 < self.vertices.len()
                && idx2 < self.vertices.len()
            {
                let p0 = Vec3::new(
                    self.vertices[idx0][0],
                    self.vertices[idx0][1],
                    self.vertices[idx0][2],
                );
                let p1 = Vec3::new(
                    self.vertices[idx1][0],
                    self.vertices[idx1][1],
                    self.vertices[idx1][2],
                );
                let p2 = Vec3::new(
                    self.vertices[idx2][0],
                    self.vertices[idx2][1],
                    self.vertices[idx2][2],
                );
                let n = (p1 - p0).cross(p2 - p0);
                accumulated_normals[idx0] = accumulated_normals[idx0] + n;
                accumulated_normals[idx1] = accumulated_normals[idx1] + n;
                accumulated_normals[idx2] = accumulated_normals[idx2] + n;
            }
        }

        let mut vertices = Vec::with_capacity(self.vertices.len());
        for (i, &pos) in self.vertices.iter().enumerate() {
            let norm = if accumulated_normals[i].length_squared() > 1e-6 {
                accumulated_normals[i].normalize()
            } else {
                Vec3::new(0.0, 1.0, 0.0)
            };
            let vert_c = self.colors.get(i).copied().unwrap_or(color);
            vertices.push(Vertex::new(
                Vec3::new(pos[0], pos[1], pos[2]),
                norm,
                [0.0, 0.0],
                vert_c,
            ));
        }
        let mut indices = Vec::with_capacity(self.triangles.len() * 3);
        for tri in &self.triangles {
            indices.push(tri[0]);
            indices.push(tri[1]);
            indices.push(tri[2]);
        }
        Mesh { vertices, indices }
    }

    /// Centers the model at the origin (0, 0, 0) and normalizes coordinates to fit within a target radius.
    pub fn normalize_and_center(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        let mut min_bound = self.vertices[0];
        let mut max_bound = self.vertices[0];
        for v in &self.vertices {
            for i in 0..3 {
                min_bound[i] = min_bound[i].min(v[i]);
                max_bound[i] = max_bound[i].max(v[i]);
            }
        }
        let center = [
            f32::midpoint(min_bound[0], max_bound[0]),
            f32::midpoint(min_bound[1], max_bound[1]),
            f32::midpoint(min_bound[2], max_bound[2]),
        ];
        let extent_x = max_bound[0] - min_bound[0];
        let extent_y = max_bound[1] - min_bound[1];
        let extent_z = max_bound[2] - min_bound[2];
        let max_extent = extent_x.max(extent_y).max(extent_z).max(1e-5);
        let scale = 2.4 / max_extent;

        for v in &mut self.vertices {
            v[0] = (v[0] - center[0]) * scale;
            v[1] = (v[1] - center[1]) * scale;
            v[2] = (v[2] - center[2]) * scale;
        }
    }
}
