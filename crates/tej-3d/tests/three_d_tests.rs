use tej_3d::{
    Canvas3D, Light, Mat4, Mesh, Object3D, OrbitControls, PerspectiveCamera, Rasterizer3D, Scene,
    StandardMaterial, Vec3,
};
use tej_hal::{Color, MouseButton, UiEvent};
use tej_layout::{Point, Size};
use tej_view::View;

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
