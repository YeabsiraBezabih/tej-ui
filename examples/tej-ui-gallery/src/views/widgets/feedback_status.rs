//! Feedback, Status, and Notification widgets showcase view.

use tej_ui::prelude::*;

/// Category view showcasing alerts, progress bars, and placeholder skeletons.
pub fn feedback_status_view() -> impl View {
    let progress_val = Signal::new(0.72f32);

    VStack::new((
        // Section 1: Callout Alerts
        Card::new("Callout Alerts")
            .description("System messages and status notifications with semantic color coding")
            .flex_grow(1.0)
            .content(
                VStack::new((
                    HStack::new((
                        Alert::new("Informational Update")
                            .description("Hardware GPU compositor running smoothly at 120 FPS")
                            .severity(AlertSeverity::Info),
                        Alert::new("Success Confirmation")
                            .description("Zero-copy database snapshot committed and memory-mapped")
                            .severity(AlertSeverity::Success),
                    ))
                    .spacing(8),
                    HStack::new((
                        Alert::new("Attention Warning")
                            .description("High VRAM allocation threshold reached on GPU device")
                            .severity(AlertSeverity::Warning),
                        Alert::new("Critical Failure")
                            .description("Connection timed out waiting for remote peer handshake")
                            .severity(AlertSeverity::Destructive),
                    ))
                    .spacing(8),
                ))
                .spacing(8),
            ),
        // Section 2: Progress Indicators
        Card::new("Progress & Loading Bars")
            .description("Deterministic and indeterminate task completion indicators")
            .flex_grow(1.0)
            .content(
                VStack::new((
                    HStack::new((
                        Text::new("Asset Download (72%)"),
                        Badge::new("IN PROGRESS").variant(BadgeVariant::Success),
                    ))
                    .spacing(12),
                    Progress::new(progress_val),
                ))
                .spacing(8),
            ),
        // Section 3: Shimmering Skeletons
        Card::new("Placeholder Shimmer Skeletons")
            .description("Zero-allocation placeholder boxes maintaining layout during async loads")
            .flex_grow(1.0)
            .content(
                HStack::new((
                    Skeleton::circle(44.0),
                    VStack::new((
                        Skeleton::new(Dimension::Points(180.0), Dimension::Points(16.0)),
                        Skeleton::new(Dimension::Points(120.0), Dimension::Points(14.0)),
                    ))
                    .spacing(6),
                    Skeleton::new(Dimension::Points(240.0), Dimension::Points(44.0)),
                ))
                .spacing(16),
            ),
    ))
    .spacing(16)
}
