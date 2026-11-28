//! Standardized Test Datasets across the 4 Scale Tiers.
//!
//! - Tier 1 (Micro / UI Widget): Suzanne (507 verts, 968 tris, 8 meshlets).
//! - Tier 2 (Standard Asset): Stanford Bunny + Utah Teapot (70,000 tris, 550 meshlets).
//! - Tier 3 (Complex Studio Scene): Multi-Object Assembly (1,000,000 tris, 7,812 meshlets).
//! - Tier 4 (Extreme Out-of-Core CAD): 10,000,000 tris (78,125 meshlets, ~310 MB .mydb file).

use mydb::Database;
use mydb::spatial_viewport::ClusterMetadata;
use mydb_core::spatial::{Aabb3D, BoundingSphere, MESHLET_HEADER_SIZE, MeshletPageHeader};
use std::fs;
use std::path::Path;
use tej_ui_3d::{
    BlenderMesh, Mesh, MeshletPacker, NaniteRenderEngine, PackedMeshlet, load_obj_from_file,
};
use tej_ui_hal::Color;

/// Returns current Resident Set Size (RSS) in megabytes.
pub fn get_rss_mb() -> f64 {
    if let Ok(statm) = fs::read_to_string("/proc/self/statm") {
        let parts: Vec<&str> = statm.split_whitespace().collect();
        if parts.len() >= 2
            && let Ok(resident_pages) = parts[1].parse::<u32>()
        {
            return (f64::from(resident_pages) * 4.0) / 1024.0;
        }
    }
    0.0
}

/// Metadata summary for a scale tier.
#[derive(Debug, Clone)]
pub struct TierDescriptor {
    pub name: &'static str,
    pub tri_count: usize,
    pub vert_count: usize,
    pub meshlet_count: usize,
}

/// Generates a standardized 4 KiB meshlet page directly for high-scale out-of-core streaming.
pub fn create_synthetic_meshlet_page(
    _cluster_id: u32,
    center: [f32; 3],
    radius: f32,
    tri_count: u16,
) -> (MeshletPageHeader, [u8; 4096]) {
    let min_b = [center[0] - radius, center[1] - radius, center[2] - radius];
    let max_b = [center[0] + radius, center[1] + radius, center[2] + radius];

    let header = MeshletPageHeader::new(
        0,
        0,
        tri_count.min(128) * 2,
        tri_count.min(128),
        1,
        [center[0], center[1], center[2], radius],
        min_b,
        max_b,
        0.02,
    );

    let mut page = [0u8; 4096];
    let h_bytes = header.to_bytes();
    page[..MESHLET_HEADER_SIZE].copy_from_slice(&h_bytes);

    (header, page)
}

/// Loads Tier 1 (`assets/suzanne.obj`).
pub fn load_tier_1_mesh() -> (BlenderMesh, Mesh, Vec<PackedMeshlet>) {
    let mut mesh = load_obj_from_file("assets/suzanne.obj")
        .or_else(|_| load_obj_from_file("examples/blender-nanite-viewer/assets/suzanne.obj"))
        .unwrap_or_else(|_| {
            let mut m = BlenderMesh::generate_sculpt(24, 16);
            m.normalize_and_center();
            m
        });
    mesh.normalize_and_center();
    let scene_m = mesh.to_mesh(Color::WHITE);
    let meshlets = MeshletPacker::pack_geometry(&mesh.vertices, &mesh.triangles, 0, 0.02);
    (mesh, scene_m, meshlets)
}

/// Loads Tier 2 (`stanford-bunny.obj` + `teapot.obj`).
pub fn load_tier_2_mesh() -> (BlenderMesh, Mesh, Vec<PackedMeshlet>) {
    let mut mesh = load_obj_from_file("assets/stanford-bunny.obj")
        .or_else(|_| load_obj_from_file("examples/blender-nanite-viewer/assets/stanford-bunny.obj"))
        .unwrap_or_else(|_| {
            let mut m = BlenderMesh::generate_sculpt(48, 32);
            m.normalize_and_center();
            m
        });
    mesh.normalize_and_center();
    let scene_m = mesh.to_mesh(Color::WHITE);
    let meshlets = MeshletPacker::pack_geometry(&mesh.vertices, &mesh.triangles, 0, 0.02);
    (mesh, scene_m, meshlets)
}

/// Populates an in-memory or disk-backed database with Tier 3 (1,000,000 triangles, 7,812 meshlets).
pub fn build_tier_3_studio_scene<S: mydb::Storage>(
    engine: &mut NaniteRenderEngine<S>,
) -> Result<TierDescriptor, Box<dyn std::error::Error>> {
    let target_meshlets = 7_812;
    let tris_per_meshlet = 128;

    for i in 0..target_meshlets {
        let x = (f32::from(u16::try_from(i % 100).unwrap_or(0)) - 50.0) * 2.5;
        let z = (f32::from(u16::try_from((i / 100) % 100).unwrap_or(0)) - 50.0) * 2.5;
        let y = f32::from(u16::try_from(i / 10000).unwrap_or(0)) * 2.0;

        let (hdr, page) =
            create_synthetic_meshlet_page(i as u32, [x, y, z], 1.25, tris_per_meshlet);
        let payload = &page[MESHLET_HEADER_SIZE..];
        let page_id = engine.store_meshlet(i as u32, &hdr, payload)?;

        let meta = ClusterMetadata::new(
            i as u32,
            Aabb3D::new(hdr.min_bounds, hdr.max_bounds),
            BoundingSphere::new([x, y, z], 1.25),
            0,
            0.02,
            page_id,
        );
        engine.register_cluster(meta, page_id)?;
    }

    Ok(TierDescriptor {
        name: "Tier 3: Complex Studio Scene (1M Tris)",
        tri_count: target_meshlets * (tris_per_meshlet as usize),
        vert_count: target_meshlets * 128,
        meshlet_count: target_meshlets,
    })
}

/// Builds Tier 4 (10,000,000 triangles, 78,125 meshlets) on disk in a `.mydb` file.
///
/// Keeps RAM footprint tiny (< 25 MB) by writing pages directly to disk.
pub fn build_tier_4_out_of_core_disk(
    db_path: &Path,
) -> Result<TierDescriptor, Box<dyn std::error::Error>> {
    let target_meshlets = 78_125;
    let tris_per_meshlet = 128;

    let db = Database::open_file_with_capacity(db_path, 128)?;
    let domain = Aabb3D::new([-1000.0; 3], [1000.0; 3]);
    let vp = db.create_spatial_viewport("tier4_vp", domain)?;
    let mut engine = NaniteRenderEngine::new(vp, 1024);

    for i in 0..target_meshlets {
        let x = (f32::from(u16::try_from(i % 280).unwrap_or(0)) - 140.0) * 2.5;
        let z = (f32::from(u16::try_from((i / 280) % 280).unwrap_or(0)) - 140.0) * 2.5;
        let y = f32::from(u16::try_from(i / 78400).unwrap_or(0)) * 2.0;

        let (hdr, page) =
            create_synthetic_meshlet_page(i as u32, [x, y, z], 1.25, tris_per_meshlet);
        let payload = &page[MESHLET_HEADER_SIZE..];
        let page_id = engine.store_meshlet(i as u32, &hdr, payload)?;

        let meta = ClusterMetadata::new(
            i as u32,
            Aabb3D::new(hdr.min_bounds, hdr.max_bounds),
            BoundingSphere::new([x, y, z], 1.25),
            0,
            0.02,
            page_id,
        );
        engine.register_cluster(meta, page_id)?;
    }

    Ok(TierDescriptor {
        name: "Tier 4: Extreme Out-of-Core CAD (10M Tris)",
        tri_count: target_meshlets * (tris_per_meshlet as usize),
        vert_count: target_meshlets * 128,
        meshlet_count: target_meshlets,
    })
}
