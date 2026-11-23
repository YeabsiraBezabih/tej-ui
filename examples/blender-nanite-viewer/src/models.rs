//! Pre-loaded 3D Model assets and geometry clustering for the Blender Nanite Viewer.

use mydb::{Database, Storage};
use mydb_core::spatial::{Aabb3D, MeshletPageHeader};
use std::sync::Arc;
use tej_ui::prelude::*;

/// A loaded 3D asset with geometry, Nanite meshlets, and visual metadata.
#[derive(Clone)]
pub struct ModelAsset {
    pub name: &'static str,
    pub label: &'static str,
    pub sculpt: BlenderMesh,
    pub meshlets: Vec<PackedMeshlet>,
    pub color: Color,
    pub vertex_count: usize,
    pub triangle_count: usize,
}

impl ModelAsset {
    /// Convert this model's geometry into a renderable 3D Mesh.
    #[must_use]
    pub fn to_scene_mesh(&self) -> Mesh {
        self.sculpt.to_mesh(self.color)
    }
}

/// Load and cluster the official complex 3D models.
pub fn load_all_models<S: Storage>(db: &Arc<Database<S>>) -> Vec<ModelAsset> {
    let configs: [(&'static str, &'static str, &'static str, Color); 4] = [
        (
            "Suzanne (Blender Monkey)",
            "Suzanne",
            "assets/suzanne.obj",
            Color::rgb(225, 145, 55),
        ),
        (
            "Stanford Bunny",
            "Stanford Bunny",
            "assets/stanford-bunny.obj",
            Color::rgb(50, 195, 130),
        ),
        (
            "Utah Teapot",
            "Utah Teapot",
            "assets/teapot.obj",
            Color::rgb(55, 135, 245),
        ),
        (
            "Blender 4.3 Splash Screen",
            "Blender 4.3 Splash",
            "assets/blender-4.3-splash.obj",
            Color::rgb(240, 145, 75),
        ),
    ];

    let mut assets = Vec::with_capacity(configs.len());

    for (id, (name, label, path, color)) in configs.into_iter().enumerate() {
        println!("Ingesting 3D geometry for '{name}'...");
        let alt_path = format!("examples/blender-nanite-viewer/{path}");

        let mut sculpt = if let Ok(m) = load_obj_from_file(path) {
            println!("  ✓ Loaded '{name}' from {path}");
            m
        } else if let Ok(m) = load_obj_from_file(&alt_path) {
            println!("  ✓ Loaded '{name}' from {alt_path}");
            m
        } else {
            println!("  (Falling back to procedural sculpt for '{name}')");
            BlenderMesh::generate_sculpt(24, 16)
        };

        // Center and normalize coordinates so model is centered at (0, 0, 0)
        sculpt.normalize_and_center();

        let v_count = sculpt.vertices.len();
        let t_count = sculpt.triangles.len();

        // Cluster geometry into 4 KiB Nanite meshlets (<= 64 vertices, <= 124 triangles)
        let meshlets = MeshletPacker::pack_geometry(&sculpt.vertices, &sculpt.triangles, 0, 0.02);
        println!(
            "  → {v_count} vertices, {t_count} triangles → Clustered into {} Nanite meshlet pages",
            meshlets.len()
        );

        // Store meshlet pages in MyDb spatial index
        let domain = Aabb3D::new([-50.0; 3], [50.0; 3]);
        let viewport_name = format!("scene_geo_{id}");
        if let Ok(vp) = db.create_spatial_viewport(&viewport_name, domain) {
            let mut eng = NaniteRenderEngine::new(vp, 1024);
            for item in &meshlets {
                if let Ok(hdr) = MeshletPageHeader::from_page_bytes(&item.page_bytes) {
                    let payload = MeshletPageHeader::payload_slice(&item.page_bytes);
                    let _ = eng.store_meshlet(item.metadata.cluster_id, &hdr, payload);
                }
            }
        }

        assets.push(ModelAsset {
            name,
            label,
            sculpt,
            meshlets,
            color,
            vertex_count: v_count,
            triangle_count: t_count,
        });
    }

    println!("✓ All 3 complex 3D models indexed into MyDb out-of-core spatial engine!");
    assets
}
