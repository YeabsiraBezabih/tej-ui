//! 12-Byte cache-aligned quantized vertex representation and local index compression.
//!
//! Compresses vertex positions into 16-bit local AABB integers, normals into 16-bit
//! octahedral vectors, and triangle indices into local 8-bit offsets.

use super::octahedral::{decode_oct_normal, encode_oct_normal};
use crate::math::Vec3;
use tej_ui_hal::Color;

/// 12-Byte cache-aligned quantized vertex structure.
///
/// Reduces vertex storage footprint from 28 bytes (`Vec3` pos + `Vec3` norm + `Color`)
/// down to 12 bytes with sub-millimeter geometric accuracy within the meshlet's AABB.
#[repr(C, align(4))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PackedMeshletVertex {
    /// 16-bit normalized integer coordinates relative to the meshlet's local AABB.
    pub pos_quantized: [u16; 3], // 6 Bytes
    /// 16-bit octahedral unit normal vector.
    pub oct_normal: [i8; 2], // 2 Bytes
    /// RGBA vertex color or packed UV coordinates.
    pub color_rgba: [u8; 4], // 4 Bytes
}

impl PackedMeshletVertex {
    /// Size of the packed vertex structure in bytes.
    pub const BYTE_SIZE: usize = std::mem::size_of::<Self>();

    /// Creates and quantizes a new packed vertex relative to local AABB bounds.
    #[must_use]
    pub fn new(
        world_pos: [f32; 3],
        normal: [f32; 3],
        color: Color,
        min_pt: [f32; 3],
        scale_pt: [f32; 3],
    ) -> Self {
        let pos_quantized = quantize_position(world_pos, min_pt, scale_pt);
        let oct_normal = encode_oct_normal(normal);
        let color_rgba = [color.r, color.g, color.b, color.a];

        Self {
            pos_quantized,
            oct_normal,
            color_rgba,
        }
    }

    /// Dequantizes the vertex back into world-space position, normal vector, and Color.
    #[must_use]
    pub fn dequantize(&self, min_pt: [f32; 3], scale_pt: [f32; 3]) -> (Vec3, Vec3, Color) {
        let p = dequantize_position(self.pos_quantized, min_pt, scale_pt);
        let n = decode_oct_normal(self.oct_normal);
        let c = Color::rgba(
            self.color_rgba[0],
            self.color_rgba[1],
            self.color_rgba[2],
            self.color_rgba[3],
        );
        (Vec3::new(p[0], p[1], p[2]), Vec3::new(n[0], n[1], n[2]), c)
    }

    /// Serializes this vertex into a 12-byte buffer.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; 12] {
        let mut buf = [0u8; 12];
        buf[0..2].copy_from_slice(&self.pos_quantized[0].to_le_bytes());
        buf[2..4].copy_from_slice(&self.pos_quantized[1].to_le_bytes());
        buf[4..6].copy_from_slice(&self.pos_quantized[2].to_le_bytes());
        buf[6] = self.oct_normal[0] as u8;
        buf[7] = self.oct_normal[1] as u8;
        buf[8..12].copy_from_slice(&self.color_rgba);
        buf
    }

    /// Deserializes a packed vertex from a 12-byte slice.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let q0 = u16::from_le_bytes([bytes[0], bytes[1]]);
        let q1 = u16::from_le_bytes([bytes[2], bytes[3]]);
        let q2 = u16::from_le_bytes([bytes[4], bytes[5]]);
        let n0 = bytes[6] as i8;
        let n1 = bytes[7] as i8;
        let color = [bytes[8], bytes[9], bytes[10], bytes[11]];

        Self {
            pos_quantized: [q0, q1, q2],
            oct_normal: [n0, n1],
            color_rgba: color,
        }
    }
}

/// Quantizes a 3D coordinate into local 16-bit unsigned integers `[0..65535]`.
#[inline]
#[must_use]
pub fn quantize_position(p: [f32; 3], min_pt: [f32; 3], scale_pt: [f32; 3]) -> [u16; 3] {
    let sx = if scale_pt[0] > 1e-6 { scale_pt[0] } else { 1.0 };
    let sy = if scale_pt[1] > 1e-6 { scale_pt[1] } else { 1.0 };
    let sz = if scale_pt[2] > 1e-6 { scale_pt[2] } else { 1.0 };

    let nx = ((p[0] - min_pt[0]) / sx).clamp(0.0, 1.0);
    let ny = ((p[1] - min_pt[1]) / sy).clamp(0.0, 1.0);
    let nz = ((p[2] - min_pt[2]) / sz).clamp(0.0, 1.0);

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    [
        (nx * 65535.0).round() as u16,
        (ny * 65535.0).round() as u16,
        (nz * 65535.0).round() as u16,
    ]
}

/// Dequantizes a 16-bit integer position back into full 32-bit floating point coordinates.
#[inline]
#[must_use]
pub fn dequantize_position(q: [u16; 3], min_pt: [f32; 3], scale_pt: [f32; 3]) -> [f32; 3] {
    let inv = 1.0 / 65535.0;
    [
        min_pt[0] + f32::from(q[0]) * inv * scale_pt[0],
        min_pt[1] + f32::from(q[1]) * inv * scale_pt[1],
        min_pt[2] + f32::from(q[2]) * inv * scale_pt[2],
    ]
}

/// A compact meshlet cluster holding up to 128 local vertices and 128 triangles.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantizedMeshlet {
    pub min_pt: [f32; 3],
    pub scale_pt: [f32; 3],
    pub vertices: Vec<PackedMeshletVertex>,
    pub triangles: Vec<[u8; 3]>,
}

impl QuantizedMeshlet {
    /// Creates an empty quantized meshlet with bounding box extents.
    #[must_use]
    pub fn new(min_pt: [f32; 3], max_pt: [f32; 3]) -> Self {
        let scale_pt = [
            (max_pt[0] - min_pt[0]).max(1e-6),
            (max_pt[1] - min_pt[1]).max(1e-6),
            (max_pt[2] - min_pt[2]).max(1e-6),
        ];
        Self {
            min_pt,
            scale_pt,
            vertices: Vec::new(),
            triangles: Vec::new(),
        }
    }

    /// Adds a vertex and returns its local 8-bit index.
    pub fn add_vertex(&mut self, pos: [f32; 3], normal: [f32; 3], color: Color) -> Option<u8> {
        if self.vertices.len() >= 255 {
            return None;
        }
        let idx = self.vertices.len() as u8;
        self.vertices.push(PackedMeshletVertex::new(
            pos,
            normal,
            color,
            self.min_pt,
            self.scale_pt,
        ));
        Some(idx)
    }

    /// Adds a local triangle defined by local 8-bit vertex indices.
    pub fn add_triangle(&mut self, i0: u8, i1: u8, i2: u8) {
        self.triangles.push([i0, i1, i2]);
    }

    /// Total payload size of this quantized meshlet in bytes.
    #[must_use]
    pub fn byte_size(&self) -> usize {
        self.vertices.len() * PackedMeshletVertex::BYTE_SIZE + self.triangles.len() * 3
    }
}
