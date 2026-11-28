//! Unit and integration tests for `GpuSurfaceBridge` and native GPU runner integration.

#![cfg(all(feature = "native-gpu", not(target_arch = "wasm32")))]

use tej_ui::prelude::*;
use tej_ui_hal::gpu::{
    AtlasFormat, BackendConfig, GpuCompositor, GpuContext, HeadlessTarget, RenderTarget, VramAtlas,
};

#[test]
fn test_gpu_surface_bridge_quad_line_text_and_clip_translation() {
    let mut bridge = GpuSurfaceBridge::new(Size::new(800.0, 600.0), 2.0);
    assert_eq!(bridge.logical_size(), Size::new(800.0, 600.0));
    assert!((bridge.scale_factor() - 2.0).abs() < 1e-6);

    bridge.begin_frame(&[Rect::new(0.0, 0.0, 800.0, 600.0)]);

    // 1. Unclipped solid rect
    bridge.fill_rect(Rect::new(10.0, 20.0, 100.0, 50.0), Color::rgb(255, 0, 0));
    assert_eq!(bridge.quads().len(), 1);
    assert_eq!(bridge.quads()[0].bounds, [10.0, 20.0, 100.0, 50.0]);
    assert_eq!(bridge.quads()[0].corner_radii, [0.0; 4]);

    // 2. Clipped rounded rect with nested clip intersection
    bridge.push_clip(Rect::new(50.0, 50.0, 200.0, 200.0), [4.0; 4]);
    bridge.push_clip(Rect::new(100.0, 100.0, 300.0, 300.0), [8.0; 4]);
    assert_eq!(bridge.clip_depth(), 2);
    // Intersection of (50..250, 50..250) and (100..400, 100..400) is (100, 100, 150, 150)
    assert_eq!(bridge.active_clip(), Some([100.0, 100.0, 150.0, 150.0]));

    bridge.fill_rounded_rect(
        Rect::new(80.0, 80.0, 120.0, 60.0),
        12.0,
        Color::rgb(0, 255, 0),
    );
    assert_eq!(bridge.quads().len(), 2);
    assert_eq!(bridge.quads()[1].corner_radii, [12.0; 4]);
    assert_eq!(bridge.quads()[1].clip_bounds, [100.0, 100.0, 150.0, 150.0]);
    assert_eq!(bridge.quads()[1].clip_radii, [8.0; 4]);

    assert!(bridge.pop_clip());
    assert!(bridge.pop_clip());
    assert!(!bridge.pop_clip());
    assert_eq!(bridge.clip_depth(), 0);

    // 3. Stroked line tessellated into VectorPipeline mesh
    bridge.draw_line(
        Point::new(10.0, 10.0),
        Point::new(150.0, 80.0),
        3.0,
        Color::rgb(0, 0, 255),
    );
    assert_eq!(bridge.line_count(), 1);
    assert!(!bridge.vector_mesh().is_empty());
    assert!(bridge.vector_mesh().indices.len() >= 6);

    // 4. Text translated into non-whitespace glyph draw commands
    bridge.draw_text("Hi!", Point::new(20.0, 40.0), 16.0, Color::WHITE);
    assert_eq!(bridge.glyphs().len(), 3);
    assert_eq!(bridge.glyphs()[0].ch, 'H');
    assert_eq!(bridge.glyphs()[1].ch, 'i');
    assert_eq!(bridge.glyphs()[2].ch, '!');

    // Transparent shapes are skipped
    bridge.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::TRANSPARENT);
    assert_eq!(bridge.quads().len(), 2);

    bridge.end_frame();
    assert_eq!(bridge.frame_count(), 1);

    // Resize updates logical dimensions
    bridge.resize(Size::new(1024.0, 768.0));
    assert_eq!(bridge.logical_size(), Size::new(1024.0, 768.0));
}

#[test]
fn test_app_headless_and_gpu_surface_bridge_integration() {
    let config = WindowConfig {
        title: String::from("GPU Bridge Test"),
        size: Size::new(320.0, 240.0),
        ..Default::default()
    };

    let app = App::new(VStack::new((
        Text::new("Native GPU Runner"),
        HStack::new((
            Button::new(Text::new("Action")),
            Checkbox::new(Signal::new(true)),
        )),
    )))
    .window(config.clone());

    let mut bridge = GpuSurfaceBridge::new(config.size, 1.0);
    let state = app.run_frame_on_surface(&mut bridge);

    assert_eq!(state.child_state.0.current_text, "Native GPU Runner");
    assert_eq!(bridge.frame_count(), 1);
    // Background + button/checkbox quads + text glyphs should be recorded
    assert!(
        !bridge.quads().is_empty(),
        "expected view tree to emit GPU quads"
    );
    assert!(
        !bridge.glyphs().is_empty(),
        "expected view tree to emit text glyphs"
    );

    // Also verify NativeWinitApp constructs cleanly without panicking
    let _winit_app = NativeWinitApp::new(
        config,
        VStack::new((Text::new("Ready"), Button::new(Text::new("Go")))),
    );

    // Flush the recorded bridge frame to a real headless GpuCompositor when an adapter is available
    let backend_cfg = BackendConfig {
        allow_software_fallback: true,
        ..Default::default()
    };
    let ctx = match GpuContext::new_headless(&backend_cfg) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping headless GPU flush in native_runner_tests: {e}");
            return;
        }
    };

    let format = backend_cfg.preferred_format;
    let Ok(headless) = HeadlessTarget::new(&ctx, 320, 240, format) else {
        return;
    };

    let mut compositor = GpuCompositor::new(&ctx, format, &backend_cfg);
    let mut atlas =
        VramAtlas::new(ctx.device(), 512, 512, AtlasFormat::Grayscale).expect("create atlas");
    let mut target = RenderTarget::Headless(headless);

    let stats = bridge
        .flush_to_compositor(&ctx, &mut compositor, &mut target, Some(&mut atlas))
        .expect("flush bridge to GpuCompositor");

    assert!(stats.instance_count > 0);
}

#[test]
fn test_draw_text_styled_baseline_alignment_and_surface_compositor_clipping() {
    let mut bridge = GpuSurfaceBridge::new(Size::new(640.0, 480.0), 2.0);
    let surface: &mut dyn SurfaceCompositor = &mut bridge;

    surface.begin_frame(&[Rect::new(0.0, 0.0, 640.0, 480.0)]);

    // Verify SurfaceCompositor::push_clip / pop_clip updates glyph clip_bounds cleanly
    let clip_rect = Rect::new(20.0, 30.0, 300.0, 120.0);
    surface.push_clip(clip_rect, [6.0; 4]);
    surface.draw_text_styled(
        "Tag.",
        Point::new(40.0, 50.0),
        24.0,
        400,
        tej_ui_hal::TextFamily::SansSerif,
        Color::WHITE,
    );
    surface.pop_clip();

    // Draw unclipped text after pop_clip to confirm clip stack was restored
    surface.draw_text_styled(
        "ok",
        Point::new(40.0, 100.0),
        16.0,
        700,
        tej_ui_hal::TextFamily::Monospace,
        Color::WHITE,
    );
    surface.end_frame();

    let glyphs = bridge.glyphs();
    assert_eq!(
        glyphs.len(),
        6,
        "expected 4 glyphs from 'Tag.' + 2 from 'ok'"
    );

    let glyph_t = glyphs.iter().find(|g| g.ch == 'T').expect("missing 'T'");
    let glyph_a = glyphs.iter().find(|g| g.ch == 'a').expect("missing 'a'");
    let glyph_g = glyphs.iter().find(|g| g.ch == 'g').expect("missing 'g'");
    let glyph_dot = glyphs.iter().find(|g| g.ch == '.').expect("missing '.'");
    let glyph_o = glyphs.iter().find(|g| g.ch == 'o').expect("missing 'o'");

    // Clipped glyphs carry the SurfaceCompositor clip rect; post-pop glyphs are unclipped
    let expected_clip = [20.0, 30.0, 300.0, 120.0];
    assert_eq!(glyph_t.clip_bounds, expected_clip);
    assert_eq!(glyph_dot.clip_bounds, expected_clip);
    assert_ne!(glyph_o.clip_bounds, expected_clip);
    assert_eq!(glyph_o.weight, 700);
    assert!(glyph_o.monospace);
    assert_eq!(glyph_t.size_px, 48); // 24.0 * 2.0 scale factor

    // Baseline alignment checks:
    // 1. Period '.' must sit strictly lower (larger top y) than capital 'T'
    assert!(
        glyph_dot.bounds[1] > glyph_t.bounds[1],
        "expected '.' y ({}) > 'T' y ({})",
        glyph_dot.bounds[1],
        glyph_t.bounds[1]
    );

    // 2. Descender 'g' must extend strictly below lowercase 'a''s bottom edge
    let a_bottom = glyph_a.bounds[1] + glyph_a.bounds[3];
    let g_bottom = glyph_g.bounds[1] + glyph_g.bounds[3];
    assert!(
        g_bottom > a_bottom,
        "expected 'g' bottom ({g_bottom}) > 'a' bottom ({a_bottom})"
    );
}
