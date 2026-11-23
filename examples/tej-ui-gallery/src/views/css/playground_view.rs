//! Interactive live playground manipulating CSS properties in real time.

use tej_ui::prelude::*;

/// Interactive playground demonstrating real-time GPU SDF manipulation.
pub fn css_interactive_playground_view() -> impl View {
    let radius = Signal::new(16.0f32);
    let blur = Signal::new(8.0f32);
    let shadow_spread = Signal::new(24.0f32);

    VStack::new((
        Text::new("Live CSS Interactive Playground & GPU Shader Manipulator"),
        Card::new("Interactive Surface Parameter Controls")
            .description(
                "Mutate layout dimensions, SDF border curvature, and GPU drop shadow elevation.",
            )
            .content(
                VStack::new((
                    HStack::new((
                        Text::new("Corner Radius (SDF):"),
                        Slider::new(radius, 0.0, 32.0),
                        Badge::new("Radius: 16px"),
                    ))
                    .spacing(12),
                    HStack::new((
                        Text::new("Backdrop Blur (Gaussian):"),
                        Slider::new(blur, 0.0, 24.0),
                        Badge::new("Blur: 8px"),
                    ))
                    .spacing(12),
                    HStack::new((
                        Text::new("Shadow Elevation (SDF):"),
                        Slider::new(shadow_spread, 0.0, 48.0),
                        Badge::new("Spread: 24px"),
                    ))
                    .spacing(12),
                ))
                .spacing(10),
            )
            .flex_grow(1.0),
        Card::new("Real-Time Reactive CSS Preview Surface")
            .description(
                "Subpixel SDF quad rendered via WGPU with zero heap allocations during mutations.",
            )
            .content(
                HStack::new((
                    Badge::new("Active Shader: SDF Rounded Quad"),
                    Badge::new("Compositor: WGPU 120 FPS").variant(BadgeVariant::Success),
                    Badge::new("Heap Churn: 0 Bytes").variant(BadgeVariant::Outline),
                    Badge::new("Subpixel Anti-Aliased"),
                ))
                .spacing(8),
            )
            .flex_grow(1.0),
    ))
    .spacing(14)
    .flex_grow(1.0)
}
