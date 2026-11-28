//! 3D Meshes, vertex attributes, and procedural geometry generators.

use crate::math::Vec3;
use tej_ui_hal::Color;

/// Single vertex with position, surface normal, UV texture coordinates, and tint color.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: [f32; 2],
    pub color: Color,
}

impl Vertex {
    #[must_use]
    pub const fn new(position: Vec3, normal: Vec3, uv: [f32; 2], color: Color) -> Self {
        Self {
            position,
            normal,
            uv,
            color,
        }
    }
}

/// 3D Indexed triangle mesh ready for GPU vertex buffer uploading or software rasterization.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    /// Creates an empty mesh.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Procedurally generates a 3D Box/Cube mesh with dimensions width, height, depth.
    #[must_use]
    pub fn cube(width: f32, height: f32, depth: f32, color: Color) -> Self {
        let hx = width * 0.5;
        let hy = height * 0.5;
        let hz = depth * 0.5;

        // 6 faces * 4 vertices = 24 vertices
        let mut vertices = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        // Helper macro/lambda to add a quad face
        let mut add_face = |p1: Vec3, p2: Vec3, p3: Vec3, p4: Vec3, n: Vec3| {
            let base = vertices.len() as u32;
            vertices.push(Vertex::new(p1, n, [0.0, 0.0], color));
            vertices.push(Vertex::new(p2, n, [1.0, 0.0], color));
            vertices.push(Vertex::new(p3, n, [1.0, 1.0], color));
            vertices.push(Vertex::new(p4, n, [0.0, 1.0], color));

            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        };

        // Front
        add_face(
            Vec3::new(-hx, -hy, hz),
            Vec3::new(hx, -hy, hz),
            Vec3::new(hx, hy, hz),
            Vec3::new(-hx, hy, hz),
            Vec3::new(0.0, 0.0, 1.0),
        );
        // Back
        add_face(
            Vec3::new(hx, -hy, -hz),
            Vec3::new(-hx, -hy, -hz),
            Vec3::new(-hx, hy, -hz),
            Vec3::new(hx, hy, -hz),
            Vec3::new(0.0, 0.0, -1.0),
        );
        // Top
        add_face(
            Vec3::new(-hx, hy, hz),
            Vec3::new(hx, hy, hz),
            Vec3::new(hx, hy, -hz),
            Vec3::new(-hx, hy, -hz),
            Vec3::new(0.0, 1.0, 0.0),
        );
        // Bottom
        add_face(
            Vec3::new(-hx, -hy, -hz),
            Vec3::new(hx, -hy, -hz),
            Vec3::new(hx, -hy, hz),
            Vec3::new(-hx, -hy, hz),
            Vec3::new(0.0, -1.0, 0.0),
        );
        // Right
        add_face(
            Vec3::new(hx, -hy, hz),
            Vec3::new(hx, -hy, -hz),
            Vec3::new(hx, hy, -hz),
            Vec3::new(hx, hy, hz),
            Vec3::new(1.0, 0.0, 0.0),
        );
        // Left
        add_face(
            Vec3::new(-hx, -hy, -hz),
            Vec3::new(-hx, -hy, hz),
            Vec3::new(-hx, hy, hz),
            Vec3::new(-hx, hy, -hz),
            Vec3::new(-1.0, 0.0, 0.0),
        );

        Self { vertices, indices }
    }

    /// Procedurally generates a UV Sphere mesh.
    #[must_use]
    pub fn sphere(radius: f32, segments: usize, rings: usize, color: Color) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let rings_f = crate::math::count_to_f32(rings);
        let segs_f = crate::math::count_to_f32(segments);
        for ring_idx in 0..=rings {
            let v_coord = crate::math::count_to_f32(ring_idx) / rings_f;
            let phi = v_coord * std::f32::consts::PI;

            for seg_idx in 0..=segments {
                let u_coord = crate::math::count_to_f32(seg_idx) / segs_f;
                let theta = u_coord * std::f32::consts::PI * 2.0;

                let nx = theta.cos() * phi.sin();
                let ny = phi.cos();
                let nz = theta.sin() * phi.sin();

                let normal = Vec3::new(nx, ny, nz);
                let position = normal * radius;

                vertices.push(Vertex::new(position, normal, [u_coord, v_coord], color));
            }
        }

        let stride = (segments + 1) as u32;
        for r in 0..rings {
            let r_idx = r as u32;
            for s in 0..segments {
                let s_idx = s as u32;
                let i0 = r_idx * stride + s_idx;
                let i1 = (r_idx + 1) * stride + s_idx;
                let i2 = (r_idx + 1) * stride + (s_idx + 1);
                let i3 = r_idx * stride + (s_idx + 1);

                indices.extend_from_slice(&[i0, i1, i2, i0, i2, i3]);
            }
        }

        Self { vertices, indices }
    }
}
