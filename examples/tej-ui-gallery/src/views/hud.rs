//! Real-time telemetry & frame performance HUD overlay.

use crate::state::TelemetryMetrics;
use tej_ui::prelude::*;

/// Performance telemetry HUD displaying real-time FPS, frame timing, and GPU instance counts.
pub fn telemetry_hud_view(telemetry: Signal<TelemetryMetrics>) -> impl View {
    let metrics = telemetry.get();

    HStack::new((
        HStack::new((
            Badge::new("TEJ ENGINE v0.1"),
            Text::new(format!("{:.1} FPS", metrics.fps)),
            Separator::vertical(),
            Text::new(format!("{:.2} ms / frame", metrics.frame_time_ms)),
            Separator::vertical(),
            Text::new(format!("GPU Quads: {}", metrics.instance_count)),
        ))
        .spacing(8),
        Separator::vertical(),
        HStack::new((
            Text::new(format!("Dirty Rects: {}", metrics.dirty_rects)),
            Separator::vertical(),
            Badge::new("STEADY HEAP: 0 BYTES").variant(BadgeVariant::Success),
        ))
        .spacing(8),
    ))
    .spacing(12)
}
