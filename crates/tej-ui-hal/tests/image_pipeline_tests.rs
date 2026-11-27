//! Unit and headless GPU integration tests for `ImagePipeline` and `ImageInstance`.

#![cfg(feature = "gpu")]

use tej_ui_hal::gpu::{
    BackendConfig, GpuCompositor, GpuContext, GpuTextureBridge, HeadlessTarget, IMAGE_SHADER_WGSL,
    ImageInstance, ImagePipeline, PremulColor, RenderTarget, ViewportUniform,
};

#[test]
fn test_image_instance_layout_and_size() {
    assert_eq!(core::mem::size_of::<ImageInstance>(), 64);
    assert_eq!(core::mem::align_of::<ImageInstance>(), 4);

    let inst = ImageInstance::new([10.0, 20.0, 200.0, 100.0], [0.0, 0.0, 0.5, 0.5], 0.75)
        .with_clip([0.0, 0.0, 800.0, 600.0])
        .with_corner_radius(8.0);

    assert_eq!(inst.bounds, [10.0, 20.0, 200.0, 100.0]);
    assert_eq!(inst.uv_bounds, [0.0, 0.0, 0.5, 0.5]);
    assert_eq!(inst.clip_bounds, [0.0, 0.0, 800.0, 600.0]);
    assert!((inst.opacity() - 0.75).abs() < 1e-6);
    assert!((inst.corner_radius() - 8.0).abs() < 1e-6);

    let bytes = bytemuck::bytes_of(&inst);
    assert_eq!(bytes.len(), 64);

    // Opacity clamps to [0.0, 1.0]
    let clamped_hi = ImageInstance::new([0.0; 4], [0.0, 0.0, 1.0, 1.0], 2.5);
    assert!((clamped_hi.opacity() - 1.0).abs() < 1e-6);
    let clamped_lo = ImageInstance::new([0.0; 4], [0.0, 0.0, 1.0, 1.0], -1.0);
    assert!((clamped_lo.opacity() - 0.0).abs() < 1e-6);
}

#[test]
fn test_image_shader_wgsl_structure() {
    assert!(IMAGE_SHADER_WGSL.contains("struct ViewportUniform"));
    assert!(IMAGE_SHADER_WGSL.contains("struct ImageInstance"));
    assert!(IMAGE_SHADER_WGSL.contains("fn vs_main"));
    assert!(IMAGE_SHADER_WGSL.contains("fn fs_main"));
    assert!(IMAGE_SHADER_WGSL.contains("viewport.scale_factor"));
    assert!(IMAGE_SHADER_WGSL.contains("textureSample(image_texture, image_sampler, in.uv)"));
}

#[test]
fn test_image_pipeline_construction_and_compositor_render_pass() {
    let config = BackendConfig {
        allow_software_fallback: true,
        ..Default::default()
    };
    let ctx = match GpuContext::new_headless(&config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping GPU image pipeline test: {e}");
            return;
        }
    };

    let format = wgpu::TextureFormat::Rgba8Unorm;
    let uniform_buffer = ctx.device().create_buffer(&wgpu::BufferDescriptor {
        label: Some("test_image_viewport_uniform"),
        size: core::mem::size_of::<ViewportUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Standalone pipeline construction validates WGSL shader compilation on the device.
    let pipeline = ImagePipeline::new(&ctx, format, &uniform_buffer, 32);
    assert_eq!(pipeline.target_format(), format);
    assert_eq!(pipeline.image_count(), 0);

    // Create a 16x16 GpuTextureBridge filled with solid cyan-ish RGBA (R=0, G=255, B=128, A=255).
    // Little-endian u32 layout for RGBA8Unorm: 0xAA_BB_GG_RR = 0xFF_80_FF_00.
    let bridge = GpuTextureBridge::new(&ctx, 16, 16, format).expect("create bridge");
    let tex_pixels = vec![0xFF80_FF00u32; 16 * 16];
    bridge.upload_rgba_pixels(ctx.queue(), &tex_pixels);

    let _bg = pipeline.create_bridge_bind_group(ctx.device(), &bridge);

    // Test end-to-end rendering through GpuCompositor::draw_textured_quad + render_image_pass.
    let headless = match HeadlessTarget::new(&ctx, 64, 64, format) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Skipping headless target render: {e}");
            return;
        }
    };

    let mut compositor = GpuCompositor::new(&ctx, format, &config);
    compositor.begin_frame(64, 64, 1.0, PremulColor::BLACK);

    let mut target = RenderTarget::Headless(headless);
    compositor
        .submit(&ctx, &mut target)
        .expect("submit base clear frame");

    compositor.draw_textured_quad([16.0, 16.0, 32.0, 32.0], [0.0, 0.0, 1.0, 1.0], 1.0);
    assert_eq!(compositor.image_pipeline().image_count(), 1);

    if let RenderTarget::Headless(ref target_headless) = target {
        compositor.render_image_pass(
            &ctx,
            target_headless.view(),
            bridge.view(),
            Some(bridge.sampler()),
        );

        let mut pixels = Vec::new();
        target_headless
            .readback_rgba(&ctx, &mut pixels)
            .expect("readback");
        assert_eq!(pixels.len(), 64 * 64 * 4);

        // Outside the image quad at (4, 4) should remain black
        let bg = (4 * 64 + 4) * 4;
        assert_eq!(pixels[bg], 0);
        assert_eq!(pixels[bg + 1], 0);
        assert_eq!(pixels[bg + 2], 0);

        // Inside the image quad at (32, 32) should match uploaded RGBA (R≈0, G≈255, B≈128)
        let fg = (32 * 64 + 32) * 4;
        assert!(pixels[fg] < 10, "expected R≈0, got {}", pixels[fg]);
        assert!(
            pixels[fg + 1] > 240,
            "expected G≈255, got {}",
            pixels[fg + 1]
        );
        assert!(
            (i32::from(pixels[fg + 2]) - 128).abs() < 16,
            "expected B≈128, got {}",
            pixels[fg + 2]
        );
        assert!(
            pixels[fg + 3] > 240,
            "expected A≈255, got {}",
            pixels[fg + 3]
        );
    }
}
