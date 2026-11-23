//! Comprehensive verification for 2D vector path parsing, Bézier tessellation,
//! immediate-mode canvas context, and headless GPU pixel rasterization.

#![cfg(feature = "gpu")]

use tej_ui_hal::gpu::context::GpuContext;
use tej_ui_hal::gpu::surface::RenderTarget;
use tej_ui_hal::gpu::vector::{
    Canvas2DContext, FillRule, PathBuilder, PathTessellator, SvgPathParser, TessellatedMesh,
    Transform2D, VectorPath,
};
use tej_ui_hal::gpu::{BackendConfig, GpuCompositor, HeadlessTarget, PremulColor};

#[test]
fn test_transform2d_math() {
    let t = Transform2D::translation(10.0, 20.0);
    let (x, y) = t.transform_point(5.0, 5.0);
    assert!((x - 15.0).abs() < 1e-4);
    assert!((y - 25.0).abs() < 1e-4);

    let s = Transform2D::scale(2.0, 3.0);
    let (sx, sy) = s.transform_point(10.0, 10.0);
    assert!((sx - 20.0).abs() < 1e-4);
    assert!((sy - 30.0).abs() < 1e-4);

    let r = Transform2D::rotation(core::f32::consts::FRAC_PI_2);
    let (rx, ry) = r.transform_point(1.0, 0.0);
    assert!((rx - 0.0).abs() < 1e-4);
    assert!((ry - 1.0).abs() < 1e-4);

    let combined = t.multiply(&s);
    let (cx, cy) = combined.transform_point(5.0, 5.0);
    assert!((cx - 20.0).abs() < 1e-4);
    assert!((cy - 35.0).abs() < 1e-4);
}

#[test]
fn test_path_builder_and_svg_parser() {
    let mut builder = PathBuilder::new();
    builder.move_to(0.0, 0.0);
    builder.line_to(100.0, 0.0);
    builder.quad_to(150.0, 50.0, 100.0, 100.0);
    builder.cubic_to(80.0, 120.0, 20.0, 120.0, 0.0, 100.0);
    builder.close();
    let path = builder.build();
    assert!(path.inner().iter().count() > 0);

    let svg_d = "M 10 10 L 90 10 L 90 90 Z";
    let parsed = VectorPath::parse_svg(svg_d).expect("valid SVG triangle");
    assert!(parsed.inner().iter().count() > 0);

    let complex_svg = "M 10 80 Q 52.5 10, 95 80 T 180 80 C 180 120, 100 120, 10 80 Z";
    let parsed_complex = VectorPath::parse_svg(complex_svg).expect("valid complex SVG");
    assert!(parsed_complex.inner().iter().count() > 0);

    let mut dest_builder = PathBuilder::new();
    SvgPathParser::parse(svg_d, &mut dest_builder).expect("parse into builder");
    let built_dest = dest_builder.build();
    assert!(built_dest.inner().iter().count() > 0);
}

#[test]
fn test_tessellator_fill_and_stroke() {
    let parsed = VectorPath::parse_svg("M 0 0 L 100 0 L 100 100 Z").expect("valid triangle");
    let mut tess = PathTessellator::new();
    let mut mesh = TessellatedMesh::new();

    tess.tessellate_fill(&parsed, PremulColor::WHITE, FillRule::NonZero, &mut mesh);
    assert!(!mesh.vertices.is_empty());
    assert!(!mesh.indices.is_empty());
    assert_eq!(mesh.indices.len() % 3, 0);

    let vertex_count_before = mesh.vertices.len();
    tess.tessellate_stroke(&parsed, PremulColor::BLACK, 2.0, &mut mesh);
    assert!(mesh.vertices.len() > vertex_count_before);
}

#[test]
fn test_canvas_2d_context_state_and_draw() {
    let mut canvas = Canvas2DContext::new();
    canvas.set_fill_color(PremulColor::from_rgba8(255, 0, 0, 255));
    canvas.fill_rect(10.0, 10.0, 50.0, 50.0);

    assert_eq!(canvas.mesh().vertices.len(), 4);
    assert_eq!(canvas.mesh().indices.len(), 6);

    canvas.save();
    canvas.translate(20.0, 30.0);
    canvas.scale(2.0, 2.0);
    canvas.begin_path();
    canvas.move_to(0.0, 0.0);
    canvas.line_to(10.0, 0.0);
    canvas.line_to(10.0, 10.0);
    canvas.close_path();
    canvas.fill();
    canvas.restore();

    assert!(canvas.mesh().vertices.len() > 4);

    canvas.clear();
    assert!(canvas.mesh().is_empty());
}

#[test]
fn test_headless_vector_render_and_pixel_readback() {
    let config = BackendConfig::default();
    let ctx = match GpuContext::new_headless(&config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping headless vector test (no GPU adapter): {e}");
            return;
        }
    };

    let width = 64;
    let height = 64;
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let headless = match HeadlessTarget::new(&ctx, width, height, format) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to create headless target: {e}");
            return;
        }
    };

    let mut compositor = GpuCompositor::new(&ctx, format, &config);

    let mut canvas = Canvas2DContext::new();
    let green = PremulColor::from_rgba8(0, 255, 0, 255);
    canvas.set_fill_color(green);
    canvas.fill_rect(0.0, 0.0, 64.0, 64.0);

    compositor.begin_frame(width, height, 1.0, PremulColor::BLACK);

    let mut target = RenderTarget::Headless(headless);
    let stats = compositor
        .submit_frame(&ctx, &mut target, None, Some(canvas.mesh()))
        .expect("submit vector frame");

    assert!(stats.instance_count > 0);

    if let RenderTarget::Headless(ref target_headless) = target {
        let mut pixels = Vec::new();
        assert!(target_headless.readback_rgba(&ctx, &mut pixels).is_ok());
        assert_eq!(pixels.len(), (width * height * 4) as usize);

        // Center pixel must be green
        let center_idx = ((32 * width + 32) * 4) as usize;
        assert_eq!(pixels[center_idx], 0); // R
        assert!(pixels[center_idx + 1] > 200); // G
        assert_eq!(pixels[center_idx + 2], 0); // B
        assert_eq!(pixels[center_idx + 3], 255); // A
    }
}
