//! Verification test suite for Milestone 1, Step 1 GPU subsystem.

#![cfg(feature = "gpu")]

use tej_ui_hal::gpu::{
    BackendConfig, ClipStack, GpuCompositor, GpuContext, GpuError, HeadlessTarget,
    InstanceBufferPool, PremulColor, QuadInstance, RenderTarget, ScissorRect, ViewportUniform,
};

#[test]
fn test_gpu_struct_layouts() {
    assert_eq!(core::mem::size_of::<ViewportUniform>(), 16);
    assert_eq!(core::mem::size_of::<QuadInstance>(), 128);

    let uniform = ViewportUniform {
        viewport_size: [800.0, 600.0],
        scale_factor: 2.0,
        _pad: 0.0,
    };
    let bytes = bytemuck::bytes_of(&uniform);
    assert_eq!(bytes.len(), 16);

    let quad = QuadInstance::solid_rect([10.0, 20.0, 100.0, 50.0], PremulColor::WHITE);
    let quad_bytes = bytemuck::bytes_of(&quad);
    assert_eq!(quad_bytes.len(), 128);
}

#[test]
fn test_clip_stack_intersection_and_underflow() {
    let mut stack = ClipStack::new();
    stack.reset(800, 600, 1.0);

    assert_eq!(stack.depth(), 1);
    assert!(!stack.is_culled());
    assert_eq!(stack.current().scissor, ScissorRect::new(0, 0, 800, 600));

    // Push child clip (100, 100, 200, 200)
    stack.push([100.0, 100.0, 200.0, 200.0], [0.0; 4], 1.0);
    assert_eq!(stack.depth(), 2);
    assert_eq!(
        stack.current().scissor,
        ScissorRect::new(100, 100, 200, 200)
    );

    // Push non-overlapping clip (500, 500, 100, 100) -> intersection has 0 area
    stack.push([500.0, 500.0, 100.0, 100.0], [0.0; 4], 1.0);
    assert_eq!(stack.depth(), 3);
    assert!(stack.is_culled());
    assert!(stack.current().scissor.is_empty());

    // Pop the non-overlapping clip
    assert!(stack.pop().is_ok());
    assert_eq!(stack.depth(), 2);
    assert!(!stack.is_culled());

    // Pop child clip
    assert!(stack.pop().is_ok());
    assert_eq!(stack.depth(), 1);

    // Attempting to pop root viewport clip must return underflow error
    assert!(matches!(stack.pop(), Err(GpuError::ClipStackUnderflow)));
    assert_eq!(stack.depth(), 1);
}

#[test]
fn test_batch_coalescing_and_zero_steady_state_alloc() {
    let config = BackendConfig {
        allow_software_fallback: true,
        ..Default::default()
    };
    let ctx = match GpuContext::new_headless(&config) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: no compatible GPU or software rasterizer adapter found.");
            return;
        }
    };

    let mut pool = InstanceBufferPool::new(ctx.device(), 512);

    let scissors = [
        ScissorRect::new(0, 0, 100, 100),
        ScissorRect::new(100, 0, 100, 100),
        ScissorRect::new(0, 100, 100, 100),
        ScissorRect::new(100, 100, 100, 100),
    ];

    let mut initial_cpu_cap = 0;
    let mut initial_gpu_cap = 0;

    // Simulate 100 frames
    for frame in 0..100 {
        pool.begin_frame();

        // Push 512 quads, evenly distributed across 4 scissor regions
        for i in 0..512 {
            let scissor_idx = (i / 128) as usize;
            let scissor = scissors[scissor_idx];
            let quad = QuadInstance::solid_rect([0.0, 0.0, 10.0, 10.0], PremulColor::WHITE);
            pool.push_instance(quad, scissor);
        }

        assert_eq!(pool.batches().len(), 4);
        assert_eq!(pool.instance_count(), 512);

        let reallocated = pool.upload(ctx.device(), ctx.queue());

        if frame == 0 {
            initial_cpu_cap = pool.cpu_capacity();
            initial_gpu_cap = pool.gpu_capacity();
        } else {
            // Steady state must never reallocate
            assert!(!reallocated);
            assert_eq!(pool.cpu_capacity(), initial_cpu_cap);
            assert_eq!(pool.gpu_capacity(), initial_gpu_cap);
        }
    }
}

#[test]
fn test_headless_render_and_pixel_readback() {
    let config = BackendConfig {
        allow_software_fallback: true,
        ..Default::default()
    };
    let ctx = match GpuContext::new_headless(&config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping headless render test: {e}");
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

    let mut compositor = GpuCompositor::new(&ctx, format, &config);
    compositor.begin_frame(64, 64, 1.0, PremulColor::BLACK);

    // 1. Draw solid red 32x32 rect at (8, 8)
    let red = PremulColor::from_rgba8(255, 0, 0, 255);
    compositor.draw_rect([8.0, 8.0, 32.0, 32.0], red);

    // 2. Push clip (8, 8, 16, 16) and draw green rect extending outside (8, 8, 32, 32)
    compositor.push_clip([8.0, 8.0, 16.0, 16.0], [0.0; 4]);
    let green = PremulColor::from_rgba8(0, 255, 0, 255);
    compositor.draw_rect([8.0, 8.0, 32.0, 32.0], green);
    assert!(compositor.pop_clip().is_ok());

    // 3. Draw rounded blue rect at (40, 8) with radius 8.0
    let blue = PremulColor::from_rgba8(0, 0, 255, 255);
    compositor.draw_rounded_rect([40.0, 8.0, 16.0, 16.0], [8.0; 4], blue);

    let mut target = RenderTarget::Headless(headless);
    let stats = compositor.submit(&ctx, &mut target);
    assert!(stats.is_ok());

    if let RenderTarget::Headless(ref target_headless) = target {
        let mut pixels = Vec::new();
        assert!(target_headless.readback_rgba(&ctx, &mut pixels).is_ok());

        assert_eq!(pixels.len(), 64 * 64 * 4);

        // Check background at (2, 2) is black
        let bg_idx = (2 * 64 + 2) * 4;
        assert_eq!(pixels[bg_idx], 0);
        assert_eq!(pixels[bg_idx + 1], 0);
        assert_eq!(pixels[bg_idx + 2], 0);
        assert_eq!(pixels[bg_idx + 3], 255);

        // Check inside clipped green region at (12, 12) has green > 240
        let green_idx = (12 * 64 + 12) * 4;
        assert!(pixels[green_idx + 1] > 240);

        // Check outside clipped green region at (30, 12) remains red (r > 240, g < 15)
        let red_idx = (12 * 64 + 30) * 4;
        assert!(pixels[red_idx] > 240);
        assert!(pixels[red_idx + 1] < 15);

        // Check center of blue rounded rect at (48, 16) has blue > 240
        let blue_idx = (16 * 64 + 48) * 4;
        assert!(pixels[blue_idx + 2] > 240);
    }
}
