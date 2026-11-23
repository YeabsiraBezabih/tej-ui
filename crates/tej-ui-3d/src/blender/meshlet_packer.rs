//! Packs raw 3D mesh geometry into 64-vertex / 124-triangle 4 KiB Nanite meshlet pages.

use crate::nanite::PackedMeshletVertex;
use mydb::spatial_viewport::ClusterMetadata;
use mydb_core::spatial::{Aabb3D, BoundingSphere, MeshletPageHeader};
use mydb_core::{PAGE_MAGIC, PageId};
use tej_ui_hal::Color;

/// A packed 4 KiB meshlet page ready for database storage and zero-copy GPU uploads.
#[derive(Debug, Clone)]
pub struct PackedMeshlet {
    pub metadata: ClusterMetadata,
    pub page_bytes: [u8; 4096],
}

/// Packs 3D triangles and vertices into 64-byte aligned 4 KiB Nanite meshlet pages.
#[derive(Debug, Clone, Copy, Default)]
pub struct MeshletPacker;

impl MeshletPacker {
    /// Partitions an indexed triangle mesh into standard Nanite clusters (<= 64 vertices, <= 124 triangles).
    pub fn pack_geometry(
        vertices: &[[f32; 3]],
        triangles: &[[u32; 3]],
        lod_level: u8,
        lod_error: f32,
    ) -> Vec<PackedMeshlet> {
        let mut result = Vec::new();
        let max_tris_per_meshlet = 124;

        for (cluster_id_usize, chunk) in triangles.chunks(max_tris_per_meshlet).enumerate() {
            let cluster_id = cluster_id_usize as u32;
            let mut page_bytes = [0u8; 4096];
            page_bytes[0..4].copy_from_slice(&PAGE_MAGIC);

            // 1. Calculate AABB and Bounding Sphere
            let mut min_pt = [f32::INFINITY; 3];
            let mut max_pt = [f32::NEG_INFINITY; 3];

            for tri in chunk {
                for &idx in tri {
                    if let Some(v) = vertices.get(idx as usize) {
                        min_pt[0] = min_pt[0].min(v[0]);
                        min_pt[1] = min_pt[1].min(v[1]);
                        min_pt[2] = min_pt[2].min(v[2]);

                        max_pt[0] = max_pt[0].max(v[0]);
                        max_pt[1] = max_pt[1].max(v[1]);
                        max_pt[2] = max_pt[2].max(v[2]);
                    }
                }
            }

            let center = [
                (min_pt[0] + max_pt[0]) * 0.5,
                (min_pt[1] + max_pt[1]) * 0.5,
                (min_pt[2] + max_pt[2]) * 0.5,
            ];

            let mut radius = 0.0f32;
            for tri in chunk {
                for &idx in tri {
                    if let Some(v) = vertices.get(idx as usize) {
                        let dx = v[0] - center[0];
                        let dy = v[1] - center[1];
                        let dz = v[2] - center[2];
                        let d = (dx * dx + dy * dy + dz * dz).sqrt();
                        radius = radius.max(d);
                    }
                }
            }

            let sphere = BoundingSphere::new(center, radius);
            let bounds = Aabb3D::new(min_pt, max_pt);

            // 2. Build 64-byte aligned MeshletPageHeader
            let vertex_count = (chunk.len() * 3).min(64) as u16;
            let triangle_count = chunk.len() as u16;

            let header = MeshletPageHeader::new(
                lod_level,
                0, // flags
                vertex_count,
                triangle_count,
                0, // material_id
                [center[0], center[1], center[2], radius],
                min_pt,
                max_pt,
                lod_error,
            );

            // Copy header into byte offset 64..128
            page_bytes[64..128].copy_from_slice(&header.to_bytes());

            // 3. Copy vertex coordinates into payload slice (offset 128)
            let mut payload_offset = 128;
            for tri in chunk {
                for &idx in tri {
                    if let Some(v) = vertices.get(idx as usize) {
                        if payload_offset + 12 <= 4096 {
                            page_bytes[payload_offset..payload_offset + 4]
                                .copy_from_slice(&v[0].to_le_bytes());
                            page_bytes[payload_offset + 4..payload_offset + 8]
                                .copy_from_slice(&v[1].to_le_bytes());
                            page_bytes[payload_offset + 8..payload_offset + 12]
                                .copy_from_slice(&v[2].to_le_bytes());
                            payload_offset += 12;
                        }
                    }
                }
            }

            let metadata = ClusterMetadata::new(
                cluster_id,
                bounds,
                sphere,
                lod_level,
                lod_error,
                PageId(cluster_id as u64 + 1),
            );

            result.push(PackedMeshlet {
                metadata,
                page_bytes,
            });
        }

        result
    }

    /// Partitions an indexed triangle mesh into standard Nanite clusters using 12-byte quantized vertices.
    pub fn pack_geometry_quantized(
        vertices: &[[f32; 3]],
        normals: &[[f32; 3]],
        colors: &[Color],
        triangles: &[[u32; 3]],
        lod_level: u8,
        lod_error: f32,
    ) -> Vec<PackedMeshlet> {
        let mut result = Vec::new();
        let max_tris_per_meshlet = 124;

        for (cluster_id_usize, chunk) in triangles.chunks(max_tris_per_meshlet).enumerate() {
            let cluster_id = cluster_id_usize as u32;
            let mut page_bytes = [0u8; 4096];
            page_bytes[0..4].copy_from_slice(&PAGE_MAGIC);

            let mut min_pt = [f32::INFINITY; 3];
            let mut max_pt = [f32::NEG_INFINITY; 3];

            for tri in chunk {
                for &idx in tri {
                    if let Some(v) = vertices.get(idx as usize) {
                        min_pt[0] = min_pt[0].min(v[0]);
                        min_pt[1] = min_pt[1].min(v[1]);
                        min_pt[2] = min_pt[2].min(v[2]);

                        max_pt[0] = max_pt[0].max(v[0]);
                        max_pt[1] = max_pt[1].max(v[1]);
                        max_pt[2] = max_pt[2].max(v[2]);
                    }
                }
            }

            let center = [
                (min_pt[0] + max_pt[0]) * 0.5,
                (min_pt[1] + max_pt[1]) * 0.5,
                (min_pt[2] + max_pt[2]) * 0.5,
            ];

            let mut radius = 0.0f32;
            for tri in chunk {
                for &idx in tri {
                    if let Some(v) = vertices.get(idx as usize) {
                        let dx = v[0] - center[0];
                        let dy = v[1] - center[1];
                        let dz = v[2] - center[2];
                        let d = (dx * dx + dy * dy + dz * dz).sqrt();
                        radius = radius.max(d);
                    }
                }
            }

            let sphere = BoundingSphere::new(center, radius);
            let bounds = Aabb3D::new(min_pt, max_pt);
            let scale_pt = [
                (max_pt[0] - min_pt[0]).max(1e-6),
                (max_pt[1] - min_pt[1]).max(1e-6),
                (max_pt[2] - min_pt[2]).max(1e-6),
            ];

            let vertex_count = (chunk.len() * 3).min(64) as u16;
            let triangle_count = chunk.len() as u16;

            let header = MeshletPageHeader::new(
                lod_level,
                1, // flag 1 indicates 12-byte quantized payload
                vertex_count,
                triangle_count,
                0,
                [center[0], center[1], center[2], radius],
                min_pt,
                max_pt,
                lod_error,
            );

            page_bytes[64..128].copy_from_slice(&header.to_bytes());

            // Write 12-byte packed vertices into offset 128
            let mut payload_offset = 128;
            for tri in chunk {
                for &idx in tri {
                    if let Some(v) = vertices.get(idx as usize) {
                        let n = normals
                            .get(idx as usize)
                            .copied()
                            .unwrap_or([0.0, 1.0, 0.0]);
                        let c = colors.get(idx as usize).copied().unwrap_or(Color::WHITE);
                        let pv = PackedMeshletVertex::new(*v, n, c, min_pt, scale_pt);

                        if payload_offset + 12 <= 4096 {
                            page_bytes[payload_offset..payload_offset + 12]
                                .copy_from_slice(&pv.to_bytes());
                            payload_offset += 12;
                        }
                    }
                }
            }

            let metadata = ClusterMetadata::new(
                cluster_id,
                bounds,
                sphere,
                lod_level,
                lod_error,
                PageId(cluster_id as u64 + 1),
            );

            result.push(PackedMeshlet {
                metadata,
                page_bytes,
            });
        }

        result
    }
}
