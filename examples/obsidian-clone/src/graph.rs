//! 3D interactive knowledge graph visualization for Obsidian notes.

use crate::models::NoteItem;
use tej_3d::{Canvas3D, Light, Mesh, Object3D, PerspectiveCamera, Scene, StandardMaterial, Vec3};
use tej_hal::Color;

/// Construct the 3D knowledge graph scene representing interconnected notes and tags.
#[must_use]
pub fn build_knowledge_graph(notes: &[NoteItem]) -> Canvas3D {
    let mut scene = Scene::new();

    // 1. Add directional and ambient lighting for 3D depth
    scene.add_light(Light::directional(
        Vec3::new(2.0, 4.0, 3.0),
        Color::rgb(255, 255, 255),
        1.2,
    ));
    scene.add_light(Light::ambient(Color::rgb(180, 200, 255), 0.8));

    // 2. Position note spheres in 3D orbit around origin
    let count = notes.len().max(1);
    for (i, note) in notes.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let radius = 2.2;
        let x = angle.cos() * radius;
        let y = (i as f32 * 0.4) - 1.0;
        let z = angle.sin() * radius;

        let note_color = match note.folder.as_str() {
            "Architecture" => Color::rgb(147, 51, 234), // Purple
            "Database" => Color::rgb(59, 130, 246),     // Blue
            "Graphics" => Color::rgb(236, 72, 153),     // Pink
            "Animation" => Color::rgb(16, 185, 129),    // Emerald
            "Platforms" => Color::rgb(245, 158, 11),    // Amber
            _ => Color::rgb(107, 114, 128),             // Gray
        };

        let sphere = Mesh::sphere(0.35, 10, 10, note_color);
        let mat = StandardMaterial::new(note_color);

        let mut obj = Object3D::new(note.id, &note.title, sphere, mat);
        obj.transform.position = Vec3::new(x, y, z);
        scene.add_object(obj);
    }

    // 3. Central Vault hub cube
    let hub = Mesh::cube(0.5, 0.5, 0.5, Color::rgb(243, 244, 246));
    let hub_mat = StandardMaterial::new(Color::rgb(243, 244, 246));
    let mut hub_obj = Object3D::new(999, "Vault Hub", hub, hub_mat);
    hub_obj.transform.position = Vec3::ZERO;
    scene.add_object(hub_obj);

    // 4. Configure perspective camera
    let camera = PerspectiveCamera {
        position: Vec3::new(0.0, 2.0, 5.0),
        target: Vec3::ZERO,
        fov_y: 50.0_f32.to_radians(),
        aspect: 16.0 / 9.0,
        near: 0.1,
        far: 100.0,
        up: Vec3::new(0.0, 1.0, 0.0),
    };

    Canvas3D::new(scene, camera)
}
