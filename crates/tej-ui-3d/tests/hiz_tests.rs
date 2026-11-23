use tej_ui_3d::{HIZ_CELLS0, HIZ_CELLS1, HIZ_CELLS2, HIZ_W0, HiZBuffer, Mat4, Vec3};

#[test]
fn test_hiz_initialization_and_clear() {
    let mut hiz = HiZBuffer::new();
    assert_eq!(hiz.level0.len(), HIZ_CELLS0);
    assert_eq!(hiz.level1.len(), HIZ_CELLS1);
    assert_eq!(hiz.level2.len(), HIZ_CELLS2);

    for &d in &hiz.level0 {
        assert_eq!(d, 0.0);
    }

    hiz.level0[0] = 0.9;
    hiz.clear();
    assert_eq!(hiz.level0[0], 0.0);
}

#[test]
fn test_hiz_update_from_depth_buffer_and_mip_pyramid() {
    let width = 128;
    let height = 72;
    let mut depths = vec![0.0_f32; width * height];

    // Paint a solid wall covering the top-left quadrant at depth 0.85
    for y in 0..36 {
        for x in 0..64 {
            depths[y * width + x] = 0.85;
        }
    }

    let mut hiz = HiZBuffer::new();
    hiz.update_from_depth_buffer(&depths, width, height);

    // Top-left cell in Level 0 (0, 0) should have conservative depth 0.85
    assert!((hiz.level0[0] - 0.85).abs() < 1e-5);
    // Bottom-right cell (HIZ_W0 - 1, HIZ_H0 - 1) has empty background 0.0
    assert_eq!(hiz.level0[HIZ_CELLS0 - 1], 0.0);

    // Mip 1 (32x18) top-left should also be 0.85
    assert!((hiz.level1[0] - 0.85).abs() < 1e-5);
    // Mip 2 (16x9) top-left should also be 0.85
    assert!((hiz.level2[0] - 0.85).abs() < 1e-5);
}

#[test]
fn test_hiz_is_rect_occluded_reversed_z() {
    let mut hiz = HiZBuffer::new();
    // Fill top-left region with solid wall at depth 0.90
    for cy in 0..10 {
        for cx in 0..20 {
            hiz.level0[cy * HIZ_W0 + cx] = 0.90;
        }
    }

    let screen_w = 640.0;
    let screen_h = 360.0;

    // 1. Background cluster inside the wall footprint at depth 0.40 (farther in Reversed-Z)
    // Should be reported as OCCLUDED
    let occluded = hiz.is_rect_occluded(
        [50.0, 50.0, 100.0, 80.0],
        0.40, // Closer than far plane, but farther than wall 0.90
        (screen_w, screen_h),
    );
    assert!(
        occluded,
        "Background geometry behind wall must be reported occluded"
    );

    // 2. Foreground cluster inside the wall footprint at depth 0.95 (closer in Reversed-Z)
    // Should NOT be occluded
    let visible_fg = hiz.is_rect_occluded([50.0, 50.0, 100.0, 80.0], 0.95, (screen_w, screen_h));
    assert!(
        !visible_fg,
        "Foreground geometry closer than wall must NOT be occluded"
    );

    // 3. Cluster that extends outside the wall into empty background (depth 0.0)
    // Should NOT be occluded
    let visible_edge = hiz.is_rect_occluded([150.0, 50.0, 300.0, 80.0], 0.40, (screen_w, screen_h));
    assert!(
        !visible_edge,
        "Geometry spanning into unoccluded space must NOT be occluded"
    );
}

#[test]
fn test_hiz_sphere_occlusion_query() {
    let mut hiz = HiZBuffer::new();
    // Fill entire screen with an opaque wall at Reversed-Z depth 0.05 (distance ~2.0 meters)
    hiz.level0.fill(0.05);

    let screen_w = 800.0;
    let screen_h = 600.0;
    let view = Mat4::look_at(
        Vec3::new(0.0, 0.0, 10.0),
        Vec3::ZERO,
        Vec3::new(0.0, 1.0, 0.0),
    );
    let proj = Mat4::perspective_reversed_z(60.0_f32.to_radians(), screen_w / screen_h, 0.1, 100.0);
    let vp = proj.mul_mat(&view);

    // A small sphere placed far behind the wall (at Z = -20.0, dist = 30.0m, depth ~0.003)
    let occluded_sphere =
        hiz.is_sphere_occluded(Vec3::new(0.0, 0.0, -20.0), 1.0, &vp, screen_w, screen_h);
    assert!(
        occluded_sphere,
        "Distant sphere behind opaque wall must be occluded"
    );

    // A sphere placed right in front of the camera (at Z = 9.6, dist = 0.4m, depth ~0.25 > 0.05)
    let foreground_sphere =
        hiz.is_sphere_occluded(Vec3::new(0.0, 0.0, 9.6), 0.2, &vp, screen_w, screen_h);
    assert!(
        !foreground_sphere,
        "Sphere in front of wall must NOT be occluded"
    );
}
