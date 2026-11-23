//! Comprehensive test suite for Phase 1: Near-plane clipping, perspective 1/W interpolation, and Reversed-Z.

use tej_ui_3d::math::{Mat4, Vec3};
use tej_ui_3d::rasterizer::{ClipVertex, PerspectiveBarycentrics, clip_triangle_near_plane};
use tej_ui_3d::{Mesh, Object3D, PerspectiveCamera, Rasterizer3D, Scene, StandardMaterial};
use tej_ui_hal::Color;

#[test]
fn test_sutherland_hodgman_all_inside() {
    let v0 = ClipVertex {
        clip_pos: [-0.5, -0.5, 0.5, 1.0],
        normal: Vec3::new(0.0, 1.0, 0.0),
        color: Color::rgb(255, 0, 0),
    };
    let v1 = ClipVertex {
        clip_pos: [0.5, -0.5, 0.5, 1.0],
        normal: Vec3::new(0.0, 1.0, 0.0),
        color: Color::rgb(0, 255, 0),
    };
    let v2 = ClipVertex {
        clip_pos: [0.0, 0.5, 0.5, 1.0],
        normal: Vec3::new(0.0, 1.0, 0.0),
        color: Color::rgb(0, 0, 255),
    };

    let clipped = clip_triangle_near_plane(v0, v1, v2, 0.1);
    assert_eq!(
        clipped.count, 1,
        "Triangle completely in front of near plane must yield 1 triangle"
    );
    assert_eq!(clipped.triangles[0][0].clip_pos, v0.clip_pos);
    assert_eq!(clipped.triangles[0][1].clip_pos, v1.clip_pos);
    assert_eq!(clipped.triangles[0][2].clip_pos, v2.clip_pos);
}

#[test]
fn test_sutherland_hodgman_all_outside() {
    let v0 = ClipVertex {
        clip_pos: [-0.5, -0.5, 0.0, 0.05],
        normal: Vec3::UP,
        color: Color::WHITE,
    };
    let v1 = ClipVertex {
        clip_pos: [0.5, -0.5, 0.0, 0.05],
        normal: Vec3::UP,
        color: Color::WHITE,
    };
    let v2 = ClipVertex {
        clip_pos: [0.0, 0.5, 0.0, 0.05],
        normal: Vec3::UP,
        color: Color::WHITE,
    };

    let clipped = clip_triangle_near_plane(v0, v1, v2, 0.1);
    assert_eq!(
        clipped.count, 0,
        "Triangle completely behind near plane must be discarded (0 triangles)"
    );
}

#[test]
fn test_sutherland_hodgman_one_inside_two_outside() {
    // v0 is inside (w = 1.0 >= 0.5), v1 and v2 are outside (w = 0.0 < 0.5)
    let v0 = ClipVertex {
        clip_pos: [0.0, 1.0, 0.0, 1.0],
        normal: Vec3::UP,
        color: Color::rgb(200, 0, 0),
    };
    let v1 = ClipVertex {
        clip_pos: [-1.0, -1.0, 0.0, 0.0],
        normal: Vec3::UP,
        color: Color::rgb(0, 200, 0),
    };
    let v2 = ClipVertex {
        clip_pos: [1.0, -1.0, 0.0, 0.0],
        normal: Vec3::UP,
        color: Color::rgb(0, 0, 200),
    };

    let clipped = clip_triangle_near_plane(v0, v1, v2, 0.5);
    assert_eq!(
        clipped.count, 1,
        "1 inside, 2 outside must produce 1 sub-triangle"
    );

    let sub_tri = clipped.triangles[0];
    // All 3 vertices of the sub-triangle must be at or in front of near_w (w >= 0.499)
    for v in &sub_tri {
        assert!(
            v.clip_pos[3] >= 0.499,
            "Clipped vertex w must be >= near plane"
        );
    }
}

#[test]
fn test_sutherland_hodgman_two_inside_one_outside() {
    // v0 and v1 are inside (w = 1.0), v2 is outside (w = 0.0)
    let v0 = ClipVertex {
        clip_pos: [-1.0, 0.0, 0.0, 1.0],
        normal: Vec3::UP,
        color: Color::rgb(255, 0, 0),
    };
    let v1 = ClipVertex {
        clip_pos: [1.0, 0.0, 0.0, 1.0],
        normal: Vec3::UP,
        color: Color::rgb(0, 255, 0),
    };
    let v2 = ClipVertex {
        clip_pos: [0.0, -1.0, 0.0, 0.0],
        normal: Vec3::UP,
        color: Color::rgb(0, 0, 255),
    };

    let clipped = clip_triangle_near_plane(v0, v1, v2, 0.5);
    assert_eq!(
        clipped.count, 2,
        "2 inside, 1 outside must split quad into 2 sub-triangles"
    );

    // All emitted vertices must satisfy near plane condition
    for tri in &clipped.triangles[..clipped.count] {
        for v in tri {
            assert!(
                v.clip_pos[3] >= 0.499,
                "Clipped vertices must be in front of near plane"
            );
        }
    }
}

#[test]
fn test_perspective_correct_barycentric_interpolation() {
    // Vertex 0 is near the camera (w = 1.0, inv_w = 1.0)
    // Vertex 1 is far from the camera (w = 10.0, inv_w = 0.1)
    // Vertex 2 is at intermediate depth (w = 2.0, inv_w = 0.5)
    let inv_w0 = 1.0 / 1.0;
    let inv_w1 = 1.0 / 10.0;
    let inv_w2 = 1.0 / 2.0;

    // Affine midpoint of edge 0-1 (w0 = 0.5, w1 = 0.5, w2 = 0.0)
    let bary = PerspectiveBarycentrics::compute(0.5, 0.5, 0.0, inv_w0, inv_w1, inv_w2)
        .expect("Barycentrics must compute successfully");

    // Sum of perspective barycentrics must equal 1.0
    assert!((bary.k0 + bary.k1 + bary.k2 - 1.0).abs() < 1e-5);

    // In perspective projection, closer vertex (w0) has vastly greater weight on screen than farther vertex (w1):
    // k0 = 0.5 * 1.0 / (0.5 * 1.0 + 0.5 * 0.1) = 0.5 / 0.55 = 0.909!
    // k1 = 0.5 * 0.1 / (0.5 * 1.0 + 0.5 * 0.1) = 0.05 / 0.55 = 0.091!
    assert!(
        bary.k0 > 0.90,
        "Closer vertex must have higher perspective weight on screen"
    );
    assert!(
        bary.k1 < 0.10,
        "Farther vertex must have smaller perspective weight on screen"
    );
}

#[test]
fn test_reversed_z_projection_matrix_precision() {
    let near = 0.1_f32;
    let far = 1000.0_f32;
    let fov = 45.0_f32.to_radians();
    let aspect = 1.0_f32;

    let proj = Mat4::perspective_reversed_z(fov, aspect, near, far);

    // Test a point exactly on the near plane: z_view = -0.1
    let p_near = Vec3::new(0.0, 0.0, -near);
    let clip_near = proj.transform_point4(p_near);
    let ndc_near_z = clip_near[2] / clip_near[3];
    assert!(
        (ndc_near_z - 1.0).abs() < 1e-4,
        "Reversed-Z near plane must map to 1.0"
    );

    // Test a point on the far plane: z_view = -1000.0
    let p_far = Vec3::new(0.0, 0.0, -far);
    let clip_far = proj.transform_point4(p_far);
    let ndc_far_z = clip_far[2] / clip_far[3];
    assert!(
        (ndc_far_z - 0.0).abs() < 1e-4,
        "Reversed-Z far plane must map to 0.0"
    );

    // Intermediate point: z_view = -1.0
    let p_mid = Vec3::new(0.0, 0.0, -1.0);
    let clip_mid = proj.transform_point4(p_mid);
    let ndc_mid_z = clip_mid[2] / clip_mid[3];
    assert!(
        ndc_mid_z > 0.0 && ndc_mid_z < 1.0,
        "Intermediate depth must be in (0.0, 1.0)"
    );
    assert!(
        ndc_mid_z < ndc_near_z,
        "Depth must monotonically decrease from near to far"
    );
}

#[test]
fn test_rasterizer_near_plane_walkthrough_continuity() {
    // Create a 3D scene with a large wall quad
    let mut scene = Scene::new();
    let wall = Mesh::cube(4.0, 4.0, 0.1, Color::rgb(0, 150, 255));
    let mut obj = Object3D::new(
        1,
        "Wall",
        wall,
        StandardMaterial::new(Color::rgb(0, 150, 255)),
    );
    obj.transform.position = Vec3::new(0.0, 0.0, 0.0);
    scene.add_object(obj);

    let mut rasterizer = Rasterizer3D::new(128, 128);

    // Camera positioned so the near plane cuts right through the wall geometry
    let camera = PerspectiveCamera {
        position: Vec3::new(0.0, 0.0, 0.15), // Camera is only 0.15 units away from wall at z = 0.0
        target: Vec3::new(0.0, 0.0, -1.0),
        near: 0.1,
        reversed_z: true,
        ..Default::default()
    };

    rasterizer.render_scene(&scene, &camera);

    // Ensure pixels were successfully drawn without panic or division by zero
    let drawn_pixels = rasterizer
        .color_buffer
        .iter()
        .filter(|&&p| p != 0xFF1E1E24)
        .count();
    assert!(
        drawn_pixels > 0,
        "Near-plane clipped wall must render pixels without disappearing"
    );
}
