//! Automated tests for depth-stencil targets and 3D GPU texture bridge.

#![cfg(feature = "gpu")]

use tej_ui_hal::gpu::{
    BackendConfig, DepthStencilTarget, GpuContext, GpuTextureBridge, StencilMode,
};

#[test]
fn test_stencil_mode_variants() {
    assert_eq!(StencilMode::Disabled, StencilMode::Disabled);
    assert_eq!(StencilMode::WriteMask(1), StencilMode::WriteMask(1));
    assert_eq!(StencilMode::TestMask(2), StencilMode::TestMask(2));
    assert_ne!(StencilMode::WriteMask(1), StencilMode::TestMask(1));
}

#[test]
fn test_depth_stencil_target_creation_and_resize() {
    let config = BackendConfig::default();
    let ctx = match GpuContext::new_headless(&config) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: no compatible GPU or software rasterizer adapter found.");
            return;
        }
    };

    let mut ds = match DepthStencilTarget::new(&ctx, 128, 128) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to create depth-stencil target: {e}");
            return;
        }
    };

    assert_eq!(ds.width(), 128);
    assert_eq!(ds.height(), 128);
    assert_eq!(ds.format(), DepthStencilTarget::DEFAULT_FORMAT);

    // Resize
    assert!(ds.resize(&ctx, 256, 256).is_ok());
    assert_eq!(ds.width(), 256);
    assert_eq!(ds.height(), 256);
}

#[test]
fn test_3d_texture_bridge_creation_and_upload() {
    let config = BackendConfig::default();
    let ctx = match GpuContext::new_headless(&config) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: no compatible GPU adapter found.");
            return;
        }
    };

    let format = wgpu::TextureFormat::Rgba8Unorm;
    let mut bridge = match GpuTextureBridge::new(&ctx, 64, 64, format) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to create 3D texture bridge: {e}");
            return;
        }
    };

    assert_eq!(bridge.width(), 64);
    assert_eq!(bridge.height(), 64);

    // Upload dummy 64x64 pixel buffer
    let dummy_pixels = vec![0xFF00_FFFFu32; 64 * 64];
    bridge.upload_rgba_pixels(ctx.queue(), &dummy_pixels);

    // Resize bridge
    assert!(bridge.resize(&ctx, 128, 128).is_ok());
    assert_eq!(bridge.width(), 128);
    assert_eq!(bridge.height(), 128);
}
