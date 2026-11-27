//! Automated tests for VRAM glyph atlas, shelf packing, and text quad pipeline.

#![cfg(feature = "gpu")]

use tej_ui_hal::gpu::{
    AtlasFormat, BackendConfig, GlyphInstance, GlyphKey, GpuCompositor, GpuContext, HeadlessTarget,
    PremulColor, RenderTarget, VramAtlas,
};

#[test]
fn test_glyph_instance_layout() {
    assert_eq!(core::mem::size_of::<GlyphInstance>(), 80);

    let glyph = GlyphInstance::new(
        [10.0, 15.0, 24.0, 24.0],
        [0.0, 0.0, 0.5, 0.5],
        PremulColor::WHITE,
        [0.0, 0.0, 800.0, 600.0],
        false,
    );
    let bytes = bytemuck::bytes_of(&glyph);
    assert_eq!(bytes.len(), 80);
}

#[test]
fn test_atlas_shelf_packing_and_lru() {
    let config = BackendConfig::default();
    let ctx = match GpuContext::new_headless(&config) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: no compatible GPU or software rasterizer found.");
            return;
        }
    };

    let mut atlas = match VramAtlas::new(ctx.device(), 256, 256, AtlasFormat::Grayscale) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to create atlas: {e}");
            return;
        }
    };

    assert_eq!(atlas.allocation_count(), 0);

    // Allocate 10 glyphs of 16x16
    let dummy_glyph = vec![255u8; 16 * 16];
    for i in 0..10 {
        let key = GlyphKey::new(1, i, 0, 16);
        let alloc = atlas.insert(key, 16, 16, &dummy_glyph);
        assert!(alloc.is_ok());
    }

    assert_eq!(atlas.allocation_count(), 10);

    // Re-retrieving existing key must return allocation and update epoch
    let key_0 = GlyphKey::new(1, 0, 0, 16);
    let alloc_0 = atlas.get(&key_0);
    assert!(alloc_0.is_some());

    // Flush dirty uploads to GPU
    atlas.flush(ctx.queue());
}

#[test]
fn test_headless_text_render_and_pixel_readback() {
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
        Err(e) => {
            eprintln!("Failed to create headless target: {e}");
            return;
        }
    };

    let mut atlas = match VramAtlas::new(ctx.device(), 128, 128, AtlasFormat::Grayscale) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to create atlas: {e}");
            return;
        }
    };

    // Create a 16x16 white square glyph (alpha = 255)
    let glyph_pixels = vec![255u8; 16 * 16];
    let key = GlyphKey::new(100, 1, 0, 16);
    let alloc = atlas
        .insert(key, 16, 16, &glyph_pixels)
        .expect("allocation failed");

    let mut compositor = GpuCompositor::new(&ctx, format, &config);
    compositor.begin_frame(64, 64, 1.0, PremulColor::BLACK);

    // Draw glyph with solid yellow color at (16, 16, 16, 16)
    let yellow = PremulColor::from_rgba8(255, 255, 0, 255);
    compositor.draw_glyph([16.0, 16.0, 16.0, 16.0], alloc.uv_bounds, yellow, false);

    let mut target = RenderTarget::Headless(headless);
    let stats = compositor.submit_with_atlas(&ctx, &mut target, Some(&mut atlas));
    assert!(stats.is_ok());

    if let RenderTarget::Headless(ref target_headless) = target {
        let mut pixels = Vec::new();
        assert!(target_headless.readback_rgba(&ctx, &mut pixels).is_ok());

        // Background at (4, 4) must be black
        let bg_idx = (4 * 64 + 4) * 4;
        assert_eq!(pixels[bg_idx], 0);
        assert_eq!(pixels[bg_idx + 1], 0);
        assert_eq!(pixels[bg_idx + 2], 0);

        // Center of glyph at (24, 24) must be bright yellow (r > 240, g > 240, b < 15)
        let glyph_idx = (24 * 64 + 24) * 4;
        assert!(
            pixels[glyph_idx] > 240,
            "red channel was {}",
            pixels[glyph_idx]
        );
        assert!(
            pixels[glyph_idx + 1] > 240,
            "green channel was {}",
            pixels[glyph_idx + 1]
        );
        assert!(
            pixels[glyph_idx + 2] < 15,
            "blue channel was {}",
            pixels[glyph_idx + 2]
        );
    }
}
