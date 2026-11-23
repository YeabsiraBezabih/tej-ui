//! 3D Hybrid Viewport showcase integrating `tej-3d` with `tej-hal`.

use tej_ui::prelude::*;

/// 3D Hybrid Viewport demo view showing 3D geometry seamlessly composited beneath 2D UI.
pub fn viewport_3d_view() -> impl View {
    VStack::new((
        // Header
        VStack::new((
            HStack::new((
                Text::new("3D Hybrid Mesh & Nanite Viewport"),
                Badge::new("REVERSED-Z 32-BIT").variant(BadgeVariant::Success),
            ))
            .spacing(12),
            Text::new(
                "Shared GPU device and queue architecture compositing 3D visibility-buffer meshes directly into the 2D UI render pass.",
            ),
        )),
        Separator::horizontal(),
        // Architecture highlights (responsive 3-column row)
        HStack::new((
            Card::new("Zero CPU Readback")
                .description("Output render target texture binds directly as sampled texture in 2D compositor pass")
                .flex_grow(1.0),
            Card::new("Reversed-Z Float")
                .description("Sub-millimeter depth precision across kilometers without z-fighting artifacts")
                .flex_grow(1.0),
            Card::new("Hierarchical LOD")
                .description("Dynamic Nanite-style cluster decimation based on screen-space error bounds")
                .flex_grow(1.0),
        ))
        .spacing(12),
        Separator::horizontal(),
        // Live Status (responsive card)
        Card::new("Active GPU 3D Pipeline Details")
            .description("Real-time GPU staging ring bound to WGPU visibility buffer")
            .flex_grow(1.0),
        HStack::new((
            Badge::new("Mesh: Stanford Bunny (69K Triangles)"),
            Badge::new("Pass: Visibility Buffer G-Buffer"),
            Badge::new("Target: 1920x1080 Rgba8UnormSrgb"),
            Badge::new("Compositor: Alpha Blended"),
        ))
        .spacing(8),
    ))
    .spacing(16)
    .flex_grow(1.0)
}
