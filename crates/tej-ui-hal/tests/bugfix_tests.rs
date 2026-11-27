//! Regression and unit tests for HAL GPU compositor, shaders, sRGB decoding, and VRAM atlas.

#![cfg(feature = "gpu")]

use tej_ui_hal::gpu::{
    AtlasFormat, BackendConfig, GlyphKey, GpuCompositor, GpuContext, GpuError, HeadlessTarget,
    PremulColor, QuadInstance, RenderTarget, TessellatedMesh, VectorVertex, VramAtlas,
};

#[test]
fn test_premul_color_srgb_to_linear_decoding() {
    // Endpoints remain exact
    let black = PremulColor::from_rgba8(0, 0, 0, 255);
    assert_eq!(black.channels, [0.0, 0.0, 0.0, 1.0]);

    let white = PremulColor::from_rgba8(255, 255, 255, 255);
    assert!((white.channels[0] - 1.0).abs() < 1e-5);
    assert!((white.channels[1] - 1.0).abs() < 1e-5);
    assert!((white.channels[2] - 1.0).abs() < 1e-5);
    assert!((white.channels[3] - 1.0).abs() < 1e-5);

    // Mid-gray (sRGB 128/255 ≈ 0.50196) must decode to ~0.21586 linear, NOT 0.50196
    let mid = PremulColor::from_rgba8(128, 128, 128, 255);
    assert!(
        (mid.channels[0] - 0.21586).abs() < 1e-3,
        "expected ~0.21586 linear for sRGB 128, got {}",
        mid.channels[0]
    );

    // Semi-transparent premultiplication applies linear alpha after sRGB decoding
    let semi = PremulColor::from_rgba8(128, 0, 255, 128);
    let alpha = 128.0 / 255.0;
    assert!((semi.channels[3] - alpha).abs() < 1e-5);
    assert!((semi.channels[0] - mid.channels[0] * alpha).abs() < 1e-4);
    assert_eq!(semi.channels[1], 0.0);
    assert!((semi.channels[2] - alpha).abs() < 1e-4);
}

#[test]
fn test_vram_atlas_insert_validates_buffer_length() {
    let config = BackendConfig::default();
    let ctx = match GpuContext::new_headless(&config) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: no compatible GPU or software rasterizer found.");
            return;
        }
    };

    let mut gray_atlas =
        VramAtlas::new(ctx.device(), 128, 128, AtlasFormat::Grayscale).expect("create gray atlas");
    let key = GlyphKey::new(1, 42, 0, 16);

    // Too few bytes must return Err(InvalidBufferSize) instead of panicking
    let short_buf = vec![255u8; 10];
    let err = gray_atlas.insert(key, 16, 16, &short_buf);
    assert!(matches!(
        err,
        Err(GpuError::InvalidBufferSize {
            expected: 256,
            actual: 10
        })
    ));

    // Zero dimensions must return Err(InvalidDimensions)
    let zero_err = gray_atlas.insert(key, 0, 16, &[]);
    assert!(matches!(
        zero_err,
        Err(GpuError::InvalidDimensions {
            width: 0,
            height: 16
        })
    ));

    // Color atlas (4 bpp) must expect width * height * 4
    let mut color_atlas =
        VramAtlas::new(ctx.device(), 128, 128, AtlasFormat::Color).expect("create color atlas");
    let wrong_color = vec![255u8; 16 * 16]; // 1 bpp instead of 4 bpp
    assert!(matches!(
        color_atlas.insert(key, 16, 16, &wrong_color),
        Err(GpuError::InvalidBufferSize {
            expected: 1024,
            actual: 256
        })
    ));

    let valid_color = vec![255u8; 16 * 16 * 4];
    assert!(color_atlas.insert(key, 16, 16, &valid_color).is_ok());
}

#[test]
fn test_compositor_scissor_reset_for_vectors_and_text_batch_count() {
    let config = BackendConfig {
        allow_software_fallback: true,
        ..Default::default()
    };
    let ctx = match GpuContext::new_headless(&config) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: no compatible GPU adapter found.");
            return;
        }
    };

    let format = wgpu::TextureFormat::Rgba8Unorm;
    let headless = match HeadlessTarget::new(&ctx, 64, 64, format) {
        Ok(t) => t,
        Err(_) => return,
    };

    let mut atlas =
        VramAtlas::new(ctx.device(), 128, 128, AtlasFormat::Grayscale).expect("create atlas");
    let glyph_pixels = vec![255u8; 8 * 8];
    let alloc = atlas
        .insert(GlyphKey::new(1, 1, 0, 8), 8, 8, &glyph_pixels)
        .expect("insert glyph");

    let mut compositor = GpuCompositor::new(&ctx, format, &config);
    compositor.begin_frame(64, 64, 1.0, PremulColor::BLACK);

    // Draw a clipped quad in the top-left corner (0..16, 0..16)
    compositor.push_clip([0.0, 0.0, 16.0, 16.0], [0.0; 4]);
    compositor.draw_rect(
        [0.0, 0.0, 16.0, 16.0],
        PremulColor::from_rgba8(255, 0, 0, 255),
    );
    // Also enqueue a text glyph under this clip (batch 1)
    compositor.draw_glyph(
        [2.0, 2.0, 8.0, 8.0],
        alloc.uv_bounds,
        PremulColor::WHITE,
        false,
    );
    compositor.pop_clip().expect("pop clip");

    // Enqueue a second text glyph under a different clip (batch 2)
    compositor.push_clip([32.0, 32.0, 16.0, 16.0], [0.0; 4]);
    compositor.draw_glyph(
        [34.0, 34.0, 8.0, 8.0],
        alloc.uv_bounds,
        PremulColor::WHITE,
        false,
    );
    compositor.pop_clip().expect("pop clip");

    // Vector triangle outside the quad's scissor (in bottom-right 40..60, 40..60)
    let green = PremulColor::from_rgba8(0, 255, 0, 255).channels;
    let mesh = TessellatedMesh {
        vertices: vec![
            VectorVertex {
                position: [40.0, 40.0],
                color: green,
            },
            VectorVertex {
                position: [60.0, 40.0],
                color: green,
            },
            VectorVertex {
                position: [50.0, 60.0],
                color: green,
            },
        ],
        indices: vec![0, 1, 2],
    };

    let mut target = RenderTarget::Headless(headless);
    let stats = compositor
        .submit_frame(&ctx, &mut target, Some(&mut atlas), Some(&mesh))
        .expect("submit_frame");

    // 1 quad batch + 2 distinct text batches = 3 total batches
    assert_eq!(stats.batch_count, 3);

    // Verify vector triangle was NOT clipped by the earlier quad scissor
    if let RenderTarget::Headless(ref target_headless) = target {
        let mut pixels = Vec::new();
        if target_headless.readback_rgba(&ctx, &mut pixels).is_ok() {
            let idx = (46 * 64 + 50) * 4;
            assert!(
                pixels[idx + 1] > 200,
                "vector green channel at (50, 46) should be visible, got {}",
                pixels[idx + 1]
            );
        }
    }
}

#[test]
fn test_quad_instance_per_side_borders_and_shadow_spread_fields() {
    let bordered = QuadInstance::bordered_rect(
        [10.0, 10.0, 100.0, 50.0],
        [4.0, 4.0, 4.0, 4.0],
        PremulColor::WHITE,
        PremulColor::BLACK,
        [1.0, 2.0, 3.0, 4.0],
    );
    assert_eq!(bordered.border_widths, [1.0, 2.0, 3.0, 4.0]);

    let shadow = QuadInstance::drop_shadow(
        [10.0, 10.0, 100.0, 50.0],
        [4.0; 4],
        PremulColor::BLACK,
        8.0,
        6.0,
    );
    assert_eq!(shadow.border_widths, [8.0, 6.0, 0.0, 0.0]);
}
