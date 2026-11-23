//! 2D Vector path tessellation and immediate-mode canvas showcase.

use tej_ui::prelude::*;

/// 2D Vector Path & Immediate-Mode Canvas demo view.
pub fn vector_demo_view() -> impl View {
    VStack::new((
        // Header
        VStack::new((
            HStack::new((
                Text::new("2D Vector Graphics & Immediate-Mode Canvas"),
                Badge::new("GPU TESSELLATED").variant(BadgeVariant::Success),
            ))
            .spacing(12),
            Text::new(
                "Hardware-accelerated Lyon tessellator with Wang's adaptive Bézier flattening, SVG path parser, and HTML5 Canvas API.",
            ),
        )),
        Separator::horizontal(),
        // Feature Cards (responsive 3-column row)
        HStack::new((
            Card::new("Wang's Adaptive")
                .description("Curvature-driven sub-pixel chord error tolerance (0.15px precision)")
                .flex_grow(1.0),
            Card::new("SVG 2.0 Streaming")
                .description("Zero-alloc parsing of M, L, H, V, C, S, Q, T, A, Z with control point reflection")
                .flex_grow(1.0),
            Card::new("Immediate Canvas2D")
                .description("beginPath, moveTo, bezierCurveTo, fill, stroke, save, restore, transform")
                .flex_grow(1.0),
        ))
        .spacing(12),
        Separator::horizontal(),
        // Live Canvas Demo Box (responsive card)
        Card::new("Canvas Rendering Pipeline")
            .description("Direct GPU vertex staging ring with MSAA antialiasing and Wang's flattening")
            .flex_grow(1.0),
        HStack::new((
            Badge::new("Path: Heart Shape SVG"),
            Badge::new("Winding: Non-Zero"),
            Badge::new("Cap: Round"),
            Badge::new("Join: Round"),
            Badge::new("Vertices: 384"),
        ))
        .spacing(8),
    ))
    .spacing(16)
    .flex_grow(1.0)
}
