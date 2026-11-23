use tej_ui_3d::{
    Canvas3D, Light, Mat4, Mesh, Object3D, OrbitControls, PerspectiveCamera, Rasterizer3D, Scene,
    StandardMaterial, Vec3,
};
use tej_ui_hal::{Color, MouseButton, UiEvent};
use tej_ui_layout::{Point, Size};
use tej_ui_view::View;

#[test]
fn test_math_mat4_transform_and_projection() {
    let trans = Mat4::translation(Vec3::new(10.0, 20.0, 30.0));
    let pt = trans.transform_point(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(pt, Vec3::new(11.0, 22.0, 33.0));

    let proj = Mat4::perspective(45.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
    let center = proj.transform_point(Vec3::new(0.0, 0.0, -10.0));
    assert!((center.x).abs() < 0.001);
    assert!((center.y).abs() < 0.001);
}

#[test]
fn test_mesh_generation_cube_and_sphere() {
    let cube = Mesh::cube(2.0, 2.0, 2.0, Color::rgb(255, 0, 0));
    assert_eq!(cube.vertices.len(), 24);
    assert_eq!(cube.indices.len(), 36);

    let sphere = Mesh::sphere(1.0, 8, 8, Color::rgb(0, 0, 255));
    assert!(!sphere.vertices.is_empty());
    assert!(!sphere.indices.is_empty());
}

#[test]
fn test_camera_raycasting_and_sphere_intersection() {
    let camera = PerspectiveCamera {
        position: Vec3::new(0.0, 0.0, 5.0),
        target: Vec3::ZERO,
        ..Default::default()
    };

    // Center screen ray should hit a sphere at the origin
    let ray = camera.screen_to_ray(Point::new(400.0, 300.0), 800.0, 600.0);
    let hit = ray.intersect_sphere(Vec3::ZERO, 1.0);
    assert!(hit.is_some());
    let dist = hit.unwrap();
    assert!((dist - 4.0).abs() < 0.1); // 5.0 - 1.0 radius = 4.0

    // Ray far off to the side should miss
    let miss_ray = camera.screen_to_ray(Point::new(0.0, 0.0), 800.0, 600.0);
    let miss = miss_ray.intersect_sphere(Vec3::ZERO, 0.2);
    assert!(miss.is_none());
}

#[test]
fn test_orbit_controls_interaction() {
    let mut controls = OrbitControls::new();
    let mut camera = PerspectiveCamera::default();

    // Mouse drag rotation
    controls.handle_event(
        &UiEvent::MouseDown {
            button: MouseButton::Left,
            position: Point::new(100.0, 100.0),
        },
        &mut camera,
    );
    controls.handle_event(
        &UiEvent::MouseMove {
            position: Point::new(150.0, 120.0),
        },
        &mut camera,
    );
    controls.handle_event(
        &UiEvent::MouseUp {
            button: MouseButton::Left,
            position: Point::new(150.0, 120.0),
        },
        &mut camera,
    );

    // Zoom event
    let old_radius = controls.radius;
    controls.handle_event(
        &UiEvent::Scroll {
            delta_x: 0.0,
            delta_y: 2.0,
            position: Point::new(150.0, 120.0),
        },
        &mut camera,
    );
    assert!(controls.radius < old_radius);
}

#[test]
fn test_dynamic_camera_panning_and_presets() {
    let mut controls = OrbitControls::new();
    let mut camera = PerspectiveCamera::default();

    // Right-click drag should pan camera target dynamically
    controls.handle_event(
        &UiEvent::MouseDown {
            button: MouseButton::Right,
            position: Point::new(200.0, 200.0),
        },
        &mut camera,
    );
    controls.handle_event(
        &UiEvent::MouseMove {
            position: Point::new(240.0, 260.0),
        },
        &mut camera,
    );
    controls.handle_event(
        &UiEvent::MouseUp {
            button: MouseButton::Right,
            position: Point::new(240.0, 260.0),
        },
        &mut camera,
    );

    // Target must have dynamically moved from origin
    assert!(camera.target.length_squared() > 1e-4);

    // Test camera view presets
    controls.set_preset(tej_ui_3d::CameraPreset::Top, &mut camera);
    assert!(camera.position.y > 0.0);

    controls.set_preset(tej_ui_3d::CameraPreset::Reset, &mut camera);
    assert_eq!(camera.target, Vec3::ZERO);

    // Test focus point
    let focus_pt = Vec3::new(12.0, 4.0, -8.0);
    controls.focus_point(focus_pt, 5.0, &mut camera);
    assert_eq!(camera.target, focus_pt);
    assert!((controls.radius - 5.0).abs() < 1e-4);
}

#[test]
fn test_rasterizer3d_and_canvas3d_view() {
    let mut scene = Scene::new();
    let cube = Mesh::cube(1.0, 1.0, 1.0, Color::rgb(200, 50, 50));
    let mat = StandardMaterial::new(Color::rgb(200, 50, 50));
    scene.add_object(Object3D::new(1, "MainCube", cube, mat));
    scene.add_light(Light::directional(
        Vec3::new(1.0, 2.0, 1.0),
        Color::rgb(255, 255, 255),
        1.0,
    ));

    let camera = PerspectiveCamera {
        position: Vec3::new(0.0, 1.0, 3.0),
        target: Vec3::ZERO,
        ..Default::default()
    };

    // Render offline in 3D rasterizer
    let mut rasterizer = Rasterizer3D::new(64, 64);
    rasterizer.clear(Color::rgb(10, 10, 10));
    rasterizer.render_scene(&scene, &camera);
    assert_eq!(rasterizer.size(), Size::new(64.0, 64.0));

    // Test Canvas3D View widget integration
    let canvas_widget = Canvas3D::new(scene, camera);
    let mut state = canvas_widget.build();
    assert_eq!(state.frame_count, 0);

    canvas_widget.rebuild(&mut state);
    assert_eq!(state.frame_count, 1);

    canvas_widget.patch(&mut state, 1);
    assert_eq!(state.frame_count, 2);
}

#[test]
fn test_nanite_meshlet_packer_and_header_serialization() {
    use mydb_core::spatial::MeshletPageHeader;
    use tej_ui_3d::{BlenderMesh, MeshletPacker};

    let mesh = BlenderMesh::generate_sculpt(8, 8);
    assert!(!mesh.vertices.is_empty());
    assert!(!mesh.triangles.is_empty());

    let packed = MeshletPacker::pack_geometry(&mesh.vertices, &mesh.triangles, 0, 0.01);
    assert!(!packed.is_empty());

    // Verify 64-byte aligned header deserialization from 4 KiB page
    for item in &packed {
        let header = MeshletPageHeader::from_page_bytes(&item.page_bytes).unwrap();
        assert_eq!(header.magic, 0x4D53_484C);
        assert!(header.vertex_count > 0);
        assert!(header.triangle_count > 0);

        let payload = MeshletPageHeader::payload_slice(&item.page_bytes);
        assert_eq!(payload.len(), 4096 - 128);
    }
}

#[test]
fn test_nanite_viewport_frame_loop_and_differential_streaming() {
    use mydb::Database;
    use mydb_core::spatial::{Aabb3D, MeshletPageHeader};
    use tej_ui_3d::{BlenderMesh, MeshletPacker, MockGpuUploader, NaniteRenderEngine};

    let db = Database::open_memory().unwrap();
    let viewport = db
        .create_spatial_viewport("scene_geo", Aabb3D::new([-10.0; 3], [10.0; 3]))
        .unwrap();

    let mut engine = NaniteRenderEngine::new(viewport, 128);

    // Pack geometry and store meshlets in database page cache
    let mesh = BlenderMesh::generate_sculpt(6, 6);
    let packed = MeshletPacker::pack_geometry(&mesh.vertices, &mesh.triangles, 0, 0.02);

    for item in &packed {
        let header = MeshletPageHeader::from_page_bytes(&item.page_bytes).unwrap();
        let payload = MeshletPageHeader::payload_slice(&item.page_bytes);
        engine
            .store_meshlet(item.metadata.cluster_id, &header, payload)
            .unwrap();
    }

    let mut uploader = MockGpuUploader::default();

    // 1. Frame 1: Frustum enclosing the geometry at origin
    let eye = [0.0f32, 0.0, 5.0];
    let mut planes = [[0.0f32; 4]; 6];
    planes[4] = [0.0, 0.0, -1.0, 1.0]; // Near plane facing -Z
    planes[5] = [0.0, 0.0, 1.0, 50.0]; // Far plane at Z = -50
    planes[0] = [1.0, 0.0, 0.0, 20.0]; // Left plane
    planes[1] = [-1.0, 0.0, 0.0, 20.0]; // Right plane
    planes[2] = [0.0, 1.0, 0.0, 20.0]; // Bottom plane
    planes[3] = [0.0, -1.0, 0.0, 20.0]; // Top plane
    let frustum = mydb::FrustumPlanes::new(planes);

    let stats = engine.render_frame_with_frustum(&frustum, eye, 0.1, &mut uploader);
    assert!(stats.clusters_visible > 0);
    assert!(stats.clusters_uploaded > 0);
    assert_eq!(uploader.uploaded_slots.len(), stats.clusters_uploaded);

    // 2. Frame 2: Camera unchanged -> 0 differential uploads (Zero-alloc steady state)
    let stats_idle = engine.render_frame_with_frustum(&frustum, eye, 0.1, &mut uploader);
    assert_eq!(stats_idle.clusters_uploaded, 0);
    assert_eq!(stats_idle.clusters_evicted, 0);

    // 3. Frame 3: Turn camera 180 degrees (looking along +Z, pointing away from geometry)
    let eye_back = [0.0f32, 0.0, 5.0];
    let mut planes_behind = [[0.0f32; 4]; 6];
    planes_behind[4] = [0.0, 0.0, 1.0, -10.0]; // Near plane far away
    planes_behind[5] = [0.0, 0.0, -1.0, 50.0];
    planes_behind[0] = [1.0, 0.0, 0.0, 20.0];
    planes_behind[1] = [-1.0, 0.0, 0.0, 20.0];
    planes_behind[2] = [0.0, 1.0, 0.0, 20.0];
    planes_behind[3] = [0.0, -1.0, 0.0, 20.0];
    let frustum_behind = mydb::FrustumPlanes::new(planes_behind);

    let stats_moved =
        engine.render_frame_with_frustum(&frustum_behind, eye_back, 0.1, &mut uploader);
    assert!(stats_moved.clusters_evicted > 0 || stats_moved.clusters_visible == 0);
}

#[test]
fn test_nanite_raycast_picking_and_acid_gizmo_undo_redo() {
    use mydb::Database;
    use mydb_core::spatial::Aabb3D;
    use tej_ui_3d::{BlenderMesh, MeshletPacker, NaniteRenderEngine};

    let db = Database::open_memory().unwrap();
    let viewport = db
        .create_spatial_viewport("scene_geo", Aabb3D::new([-10.0; 3], [10.0; 3]))
        .unwrap();

    let mut engine = NaniteRenderEngine::new(viewport, 64);
    let mesh = BlenderMesh::generate_sculpt(4, 4);
    let packed = MeshletPacker::pack_geometry(&mesh.vertices, &mesh.triangles, 0, 0.05);

    for item in &packed {
        engine
            .register_cluster(item.metadata, item.metadata.page_id)
            .unwrap();
    }

    // Unproject center screen ray
    let inv_vp = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let eye = [0.0f32, 0.0, 5.0];

    // Pick center screen
    let hit_cluster = engine.handle_pointer_click(400.0, 300.0, 800.0, 600.0, &inv_vp, eye);
    if let Some(id) = hit_cluster {
        assert_eq!(engine.gizmo().selected(), Some(id));

        // Drag translation
        assert!(engine.on_gizmo_drag([1.0, 0.0, 0.0]).is_ok());

        // Instant ACID undo
        assert!(engine.on_user_undo());

        // Instant ACID redo
        assert!(engine.on_user_redo());
    }
}

#[test]
fn test_suzanne_mesh_rasterization() {
    let mut mesh = tej_ui_3d::load_obj_from_file("assets/suzanne.obj")
        .or_else(|_| tej_ui_3d::load_obj_from_file("../../assets/suzanne.obj"))
        .expect("Load suzanne");
    mesh.normalize_and_center();
    assert_eq!(mesh.vertices.len(), 507);
    assert_eq!(mesh.triangles.len(), 968);

    let mut scene = Scene::new();
    let obj = Object3D::new(
        1,
        "suzanne",
        mesh.to_mesh(Color::rgb(220, 150, 50)),
        StandardMaterial::new(Color::rgb(220, 150, 50)),
    );
    scene.add_object(obj);
    scene.add_light(Light::directional(
        Vec3::new(0.5, 0.8, 1.0),
        Color::WHITE,
        1.0,
    ));

    let camera = PerspectiveCamera {
        position: Vec3::new(0.0, 0.2, 3.2),
        target: Vec3::ZERO,
        ..Default::default()
    };

    let mut rasterizer = Rasterizer3D::new(256, 256);
    rasterizer.render_scene(&scene, &camera);

    let non_bg = rasterizer
        .color_buffer
        .iter()
        .filter(|&&c| c != 0xFF1E1E24)
        .count();
    // Verify Suzanne covers a substantial portion of the screen (thousands of pixels)
    assert!(
        non_bg > 5000,
        "Suzanne must cover significant screen pixels, got {}",
        non_bg
    );

    // Verify rich 3D shading variation (directional diffuse + specular highlights)
    let unique_colors: std::collections::HashSet<u32> = rasterizer
        .color_buffer
        .iter()
        .copied()
        .filter(|&c| c != 0xFF1E1E24)
        .collect();
    assert!(
        unique_colors.len() > 100,
        "Must have rich 3D shading gradients, got {}",
        unique_colors.len()
    );
}

#[test]
fn test_game_entity_wasd_navigation() {
    let mut controls = OrbitControls::new();
    let mut camera = PerspectiveCamera {
        position: Vec3::new(0.0, 0.0, 5.0),
        target: Vec3::ZERO,
        ..Default::default()
    };

    let initial_pos = camera.position;

    // Press 'W' to move forward
    controls.handle_event(&UiEvent::KeyDown { key_code: 87 }, &mut camera);
    assert!(
        camera.position.z < initial_pos.z,
        "Forward must move towards target"
    );

    // Press 'D' to strafe right
    let pos_before_strafe = camera.position;
    controls.handle_event(&UiEvent::KeyDown { key_code: 68 }, &mut camera);
    assert!(
        camera.position.x > pos_before_strafe.x,
        "Strafe right must increase X"
    );

    // Press Space (32) to ascend
    let pos_before_ascend = camera.position;
    controls.handle_event(&UiEvent::KeyDown { key_code: 32 }, &mut camera);
    assert!(
        camera.position.y > pos_before_ascend.y,
        "Ascend must increase Y"
    );

    // TextInput 's' moves backward
    let pos_before_back = camera.position;
    controls.handle_event(
        &UiEvent::TextInput {
            text: "s".to_string(),
        },
        &mut camera,
    );
    assert!(
        camera.position.z > pos_before_back.z,
        "Backward must move along +Z"
    );
}

#[test]
fn test_splash_screen_multi_color_rendering() {
    let mut mesh = tej_ui_3d::load_obj_from_file("assets/blender-4.3-splash.obj")
        .or_else(|_| tej_ui_3d::load_obj_from_file("../../assets/blender-4.3-splash.obj"))
        .expect("Load splash");
    mesh.normalize_and_center();
    assert!(
        !mesh.colors.is_empty(),
        "Must contain embedded vertex colors"
    );

    let mut scene = Scene::new();
    let obj = Object3D::new(
        1,
        "splash",
        mesh.to_mesh(Color::WHITE),
        StandardMaterial::new(Color::WHITE),
    );
    scene.add_object(obj);
    scene.add_light(Light::directional(
        Vec3::new(0.5, 0.8, 1.0),
        Color::WHITE,
        1.0,
    ));

    let camera = PerspectiveCamera {
        position: Vec3::new(0.0, 0.0, 3.5),
        target: Vec3::ZERO,
        ..Default::default()
    };

    let mut rasterizer = Rasterizer3D::new(256, 256);
    rasterizer.render_scene(&scene, &camera);

    let unique_colors: std::collections::HashSet<u32> = rasterizer
        .color_buffer
        .iter()
        .copied()
        .filter(|&c| c != 0xFF1E1E24)
        .collect();
    assert!(
        unique_colors.len() > 150,
        "Splash scene must render in full multi-color, got {} unique colors",
        unique_colors.len()
    );
}
