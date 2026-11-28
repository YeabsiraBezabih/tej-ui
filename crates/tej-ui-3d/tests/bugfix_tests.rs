//! Regression and unit tests for 3D engine math, rasterizer, meshlet packer, Canvas3D, and picking.

use mydb_core::spatial::MeshletPageHeader;
use std::f32::consts::FRAC_PI_2;
use tej_ui_3d::{
    Canvas3D, Mat4, Mesh, MeshletPacker, Object3D, PerspectiveCamera, Rasterizer3D, Scene,
    StandardMaterial, Transform3D, Vec3, screen_point_to_ray, screen_point_to_ray_for_camera,
    screen_point_to_ray_with_depth,
};
use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{Point, Rect};

#[test]
fn test_rotation_z_and_transform3d_matrix() {
    let rz = Mat4::rotation_z(FRAC_PI_2);
    let p_x = rz.transform_point(Vec3::new(1.0, 0.0, 0.0));
    assert!(p_x.x.abs() < 1e-5);
    assert!((p_x.y - 1.0).abs() < 1e-5);
    assert!(p_x.z.abs() < 1e-5);

    let p_y = rz.transform_point(Vec3::new(0.0, 1.0, 0.0));
    assert!((p_y.x + 1.0).abs() < 1e-5);
    assert!(p_y.y.abs() < 1e-5);

    // Transform3D::matrix() must apply rotation.z
    let tf = Transform3D {
        position: Vec3::new(10.0, 0.0, 0.0),
        rotation: Vec3::new(0.0, 0.0, FRAC_PI_2),
        scale: Vec3::ONE,
    };
    let out = tf.matrix().transform_point(Vec3::new(1.0, 0.0, 0.0));
    assert!((out.x - 10.0).abs() < 1e-5);
    assert!((out.y - 1.0).abs() < 1e-5);
    assert!(out.z.abs() < 1e-5);
}

#[test]
fn test_orthographic_right_handed_negative_z_convention() {
    let eye = Vec3::new(0.0, 0.0, 10.0);
    let target = Vec3::ZERO;
    let view = Mat4::look_at(eye, target, Vec3::UP);
    let proj = Mat4::orthographic(-5.0, 5.0, -5.0, 5.0, 0.0, 20.0);
    let vp = proj.mul_mat(&view);

    // Point at near plane (z_world = 10.0 => z_view = 0.0) -> ndc_z = 0.0
    let p_near = vp.transform_point(Vec3::new(0.0, 0.0, 10.0));
    assert!(
        p_near.z.abs() < 1e-5,
        "expected near depth 0.0, got {}",
        p_near.z
    );

    // Point at target (z_world = 0.0 => z_view = -10.0, halfway to far=20) -> ndc_z = 0.5
    let p_mid = vp.transform_point(Vec3::ZERO);
    assert!(
        (p_mid.z - 0.5).abs() < 1e-5,
        "expected midpoint depth 0.5, got {}",
        p_mid.z
    );

    // Point at far plane (z_world = -10.0 => z_view = -20.0) -> ndc_z = 1.0
    let p_far = vp.transform_point(Vec3::new(0.0, 0.0, -10.0));
    assert!(
        (p_far.z - 1.0).abs() < 1e-5,
        "expected far depth 1.0, got {}",
        p_far.z
    );
}

#[test]
fn test_rasterizer_no_false_early_z_culling_and_honors_reversed_z() {
    // Small cube in front at origin + large cube behind it centered at origin.
    // The broken single-pixel Vec3::ZERO Early-Z test previously discarded the entire larger cube.
    let mut scene = Scene::new();

    let mut small_front = Object3D::new(
        1,
        "SmallFront",
        Mesh::cube(0.5, 0.5, 0.5, Color::rgb(255, 0, 0)),
        StandardMaterial::default(),
    );
    small_front.transform.position = Vec3::new(0.0, 0.0, 1.0);
    scene.add_object(small_front);

    let mut large_back = Object3D::new(
        2,
        "LargeBack",
        Mesh::cube(3.0, 3.0, 3.0, Color::rgb(0, 255, 0)),
        StandardMaterial::default(),
    );
    large_back.transform.position = Vec3::new(0.0, 0.0, -1.0);
    scene.add_object(large_back);

    // Test both Reversed-Z (true) and Standard-Z (false)
    for reversed_z in [true, false] {
        let mut cam = PerspectiveCamera::new(45.0, 1.0, 0.1, 100.0);
        cam.position = Vec3::new(0.0, 0.0, 6.0);
        cam.target = Vec3::ZERO;
        cam.reversed_z = reversed_z;

        let mut rasterizer = Rasterizer3D::new(128, 128);
        rasterizer.clear(Color::rgb(0, 0, 0));
        rasterizer.render_scene(&scene, &cam);

        // Center pixel (64, 64) should be covered by the red front cube
        let center_px = rasterizer.color_buffer[64 * 128 + 64];
        let c_r = (center_px >> 16) & 0xFF;
        let c_g = (center_px >> 8) & 0xFF;
        assert!(
            c_r > c_g && c_r > 20,
            "reversed_z={reversed_z}: center pixel should be red front cube, got 0x{center_px:08X}"
        );

        // Off-center pixel (40, 64) should still be covered by the green back cube (not culled!)
        let side_px = rasterizer.color_buffer[64 * 128 + 40];
        let s_r = (side_px >> 16) & 0xFF;
        let s_g = (side_px >> 8) & 0xFF;
        assert!(
            s_g > s_r && s_g > 20,
            "reversed_z={reversed_z}: side pixel should be green back cube, got 0x{side_px:08X}"
        );
    }
}

#[test]
fn test_meshlet_packer_preserves_all_triangles_without_truncation() {
    // 250 triangles (> 124 and > 110) must span multiple 4 KiB pages with zero dropped triangles
    let num_tris = 250usize;
    let mut vertices = Vec::with_capacity(num_tris * 3);
    let mut normals = Vec::with_capacity(num_tris * 3);
    let mut colors = Vec::with_capacity(num_tris * 3);
    let mut triangles = Vec::with_capacity(num_tris);

    for i in 0..num_tris {
        let base = (i * 3) as u32;
        let x = f32::from(u16::try_from(i).unwrap_or(u16::MAX)) * 0.1;
        vertices.push([x, 0.0, 0.0]);
        vertices.push([x + 0.05, 1.0, 0.0]);
        vertices.push([x + 0.1, 0.0, 0.5]);
        normals.extend_from_slice(&[[0.0, 0.0, 1.0]; 3]);
        colors.extend_from_slice(&[Color::WHITE; 3]);
        triangles.push([base, base + 1, base + 2]);
    }

    let packed = MeshletPacker::pack_geometry(&vertices, &triangles, 0, 0.01);
    assert!(packed.len() >= 3);

    let mut total_tris = 0usize;
    for page in &packed {
        let header = MeshletPageHeader::from_page_bytes(&page.page_bytes).unwrap();
        assert_eq!(header.vertex_count, header.triangle_count * 3);
        assert!(usize::from(header.vertex_count) * 12 <= 4096 - 128);
        total_tris += usize::from(header.triangle_count);
    }
    assert_eq!(total_tris, num_tris);

    // Quantized packer must also preserve all triangles and exact vertex counts
    let packed_q =
        MeshletPacker::pack_geometry_quantized(&vertices, &normals, &colors, &triangles, 0, 0.01);
    let mut total_tris_q = 0usize;
    for page in &packed_q {
        let header = MeshletPageHeader::from_page_bytes(&page.page_bytes).unwrap();
        assert_eq!(header.vertex_count, header.triangle_count * 3);
        assert!(usize::from(header.vertex_count) * 12 <= 4096 - 128);
        total_tris_q += usize::from(header.triangle_count);
    }
    assert_eq!(total_tris_q, num_tris);
}

#[derive(Debug, Default)]
struct DummyCompositor {
    blit_count: usize,
}

impl SurfaceCompositor for DummyCompositor {
    fn begin_frame(&mut self, _dirty_rects: &[Rect]) {}
    fn fill_rect(&mut self, _rect: Rect, _color: Color) {}
    fn fill_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: Color) {}
    fn draw_line(&mut self, _p1: Point, _p2: Point, _stroke_width: f32, _color: Color) {}
    fn draw_text(&mut self, _text: &str, _origin: Point, _font_size: f32, _color: Color) {}
    fn blit_pixels(&mut self, _origin: Point, _width: usize, _height: usize, _pixels: &[u32]) {
        self.blit_count += 1;
    }
    fn end_frame(&mut self) {}
}

#[test]
fn test_canvas3d_caches_and_resizes_rasterizer() {
    let scene = Scene::new();
    let cam = PerspectiveCamera::default();
    let canvas = Canvas3D::new(scene, cam);

    assert_eq!(canvas.cached_rasterizer_size(), None);

    let mut comp = DummyCompositor::default();
    canvas.paint_direct(&mut comp, Rect::new(0.0, 0.0, 120.0, 80.0));
    assert_eq!(canvas.cached_rasterizer_size(), Some((120, 80)));
    assert_eq!(comp.blit_count, 1);

    // Same dimensions reuse cached rasterizer
    canvas.paint_direct(&mut comp, Rect::new(0.0, 0.0, 120.0, 80.0));
    assert_eq!(canvas.cached_rasterizer_size(), Some((120, 80)));
    assert_eq!(comp.blit_count, 2);

    // Changed dimensions resize the cached rasterizer
    canvas.paint_direct(&mut comp, Rect::new(0.0, 0.0, 200.0, 150.0));
    assert_eq!(canvas.cached_rasterizer_size(), Some((200, 150)));
    assert_eq!(comp.blit_count, 3);
}

/// Simple 4x4 column-major matrix inversion for picking test verification.
fn invert_mat4(m: &[f32; 16]) -> [f32; 16] {
    let mut inv = [0.0f64; 16];
    let a: [f64; 16] = m.map(f64::from);

    inv[0] = a[5] * a[10] * a[15] - a[5] * a[11] * a[14] - a[9] * a[6] * a[15]
        + a[9] * a[7] * a[14]
        + a[13] * a[6] * a[11]
        - a[13] * a[7] * a[10];
    inv[4] = -a[4] * a[10] * a[15] + a[4] * a[11] * a[14] + a[8] * a[6] * a[15]
        - a[8] * a[7] * a[14]
        - a[12] * a[6] * a[11]
        + a[12] * a[7] * a[10];
    inv[8] = a[4] * a[9] * a[15] - a[4] * a[11] * a[13] - a[8] * a[5] * a[15]
        + a[8] * a[7] * a[13]
        + a[12] * a[5] * a[11]
        - a[12] * a[7] * a[9];
    inv[12] = -a[4] * a[9] * a[14] + a[4] * a[10] * a[13] + a[8] * a[5] * a[14]
        - a[8] * a[6] * a[13]
        - a[12] * a[5] * a[10]
        + a[12] * a[6] * a[9];

    inv[1] = -a[1] * a[10] * a[15] + a[1] * a[11] * a[14] + a[9] * a[2] * a[15]
        - a[9] * a[3] * a[14]
        - a[13] * a[2] * a[11]
        + a[13] * a[3] * a[10];
    inv[5] = a[0] * a[10] * a[15] - a[0] * a[11] * a[14] - a[8] * a[2] * a[15]
        + a[8] * a[3] * a[14]
        + a[12] * a[2] * a[11]
        - a[12] * a[3] * a[10];
    inv[9] = -a[0] * a[9] * a[15] + a[0] * a[11] * a[13] + a[8] * a[1] * a[15]
        - a[8] * a[3] * a[13]
        - a[12] * a[1] * a[11]
        + a[12] * a[3] * a[9];
    inv[13] = a[0] * a[9] * a[14] - a[0] * a[10] * a[13] - a[8] * a[1] * a[14]
        + a[8] * a[2] * a[13]
        + a[12] * a[1] * a[10]
        - a[12] * a[2] * a[9];

    inv[2] = a[1] * a[6] * a[15] - a[1] * a[7] * a[14] - a[5] * a[2] * a[15]
        + a[5] * a[3] * a[14]
        + a[13] * a[2] * a[7]
        - a[13] * a[3] * a[6];
    inv[6] = -a[0] * a[6] * a[15] + a[0] * a[7] * a[14] + a[4] * a[2] * a[15]
        - a[4] * a[3] * a[14]
        - a[12] * a[2] * a[7]
        + a[12] * a[3] * a[6];
    inv[10] = a[0] * a[5] * a[15] - a[0] * a[7] * a[13] - a[4] * a[1] * a[15]
        + a[4] * a[3] * a[13]
        + a[12] * a[1] * a[7]
        - a[12] * a[3] * a[5];
    inv[14] = -a[0] * a[5] * a[14] + a[0] * a[6] * a[13] + a[4] * a[1] * a[14]
        - a[4] * a[2] * a[13]
        - a[12] * a[1] * a[6]
        + a[12] * a[2] * a[5];

    inv[3] = -a[1] * a[6] * a[11] + a[1] * a[7] * a[10] + a[5] * a[2] * a[11]
        - a[5] * a[3] * a[10]
        - a[9] * a[2] * a[7]
        + a[9] * a[3] * a[6];
    inv[7] = a[0] * a[6] * a[11] - a[0] * a[7] * a[10] - a[4] * a[2] * a[11]
        + a[4] * a[3] * a[10]
        + a[8] * a[2] * a[7]
        - a[8] * a[3] * a[6];
    inv[11] = -a[0] * a[5] * a[11] + a[0] * a[7] * a[9] + a[4] * a[1] * a[11]
        - a[4] * a[3] * a[9]
        - a[8] * a[1] * a[7]
        + a[8] * a[3] * a[5];
    inv[15] = a[0] * a[5] * a[10] - a[0] * a[6] * a[9] - a[4] * a[1] * a[10]
        + a[4] * a[2] * a[9]
        + a[8] * a[1] * a[6]
        - a[8] * a[2] * a[5];

    let det = a[0] * inv[0] + a[1] * inv[4] + a[2] * inv[8] + a[3] * inv[12];
    let inv_det = 1.0 / det;
    let mut out = [0.0f32; 16];
    for i in 0..16 {
        out[i] = (inv[i] * inv_det) as f32;
    }
    out
}

#[test]
fn test_screen_point_to_ray_supports_reversed_z() {
    let mut cam_rev = PerspectiveCamera::new(45.0, 16.0 / 9.0, 0.1, 1000.0);
    cam_rev.position = Vec3::new(0.0, 0.0, 5.0);
    cam_rev.target = Vec3::ZERO;
    cam_rev.reversed_z = true;

    let mut cam_std = cam_rev.clone();
    cam_std.reversed_z = false;

    let inv_rev = invert_mat4(&cam_rev.view_projection_matrix().to_cols_array());
    let inv_std = invert_mat4(&cam_std.view_projection_matrix().to_cols_array());

    // Center of viewport should point straight along -Z: (0, 0, -1)
    let ray_rev = screen_point_to_ray_for_camera(400.0, 225.0, 800.0, 450.0, &inv_rev, &cam_rev);
    let ray_std = screen_point_to_ray_for_camera(400.0, 225.0, 800.0, 450.0, &inv_std, &cam_std);
    let ray_auto = screen_point_to_ray(400.0, 225.0, 800.0, 450.0, &inv_rev, [0.0, 0.0, 5.0]);
    let ray_explicit = screen_point_to_ray_with_depth(
        400.0,
        225.0,
        800.0,
        450.0,
        &inv_rev,
        [0.0, 0.0, 5.0],
        tej_ui_3d::nanite::picking::DepthRange::ReversedZ,
    );

    for (label, ray) in [
        ("rev", ray_rev),
        ("std", ray_std),
        ("auto", ray_auto),
        ("explicit", ray_explicit),
    ] {
        assert!(
            ray.dir[0].abs() < 1e-4 && ray.dir[1].abs() < 1e-4 && (ray.dir[2] + 1.0).abs() < 1e-4,
            "{label} center ray should be (0, 0, -1), got {:?}",
            ray.dir
        );
    }

    // Off-center screen point must agree between Reversed-Z and Standard-Z
    let off_rev = screen_point_to_ray_for_camera(600.0, 100.0, 800.0, 450.0, &inv_rev, &cam_rev);
    let off_std = screen_point_to_ray_for_camera(600.0, 100.0, 800.0, 450.0, &inv_std, &cam_std);
    for i in 0..3 {
        assert!(
            (off_rev.dir[i] - off_std.dir[i]).abs() < 1e-3,
            "off-center ray axis {i} mismatch: rev={:?} vs std={:?}",
            off_rev.dir,
            off_std.dir
        );
    }
}
