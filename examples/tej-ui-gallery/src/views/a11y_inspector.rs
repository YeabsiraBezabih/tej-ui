//! Live Accessibility & Screen Reader Inspector showcase.

use tej_ui::prelude::*;

/// Accessibility Inspector demo view displaying the live semantic AccessKit tree and announcement queue.
pub fn a11y_inspector_view() -> impl View {
    VStack::new((
        // Header
        VStack::new((
            HStack::new((
                Text::new("Live Accessibility Bridge & AccessKit Inspector"),
                Badge::new("ACCESSKIT ACTIVE").variant(BadgeVariant::Success),
            ))
            .spacing(12),
            Text::new(
                "Real-time semantic accessibility tree bridge streaming incremental diffs to AT-SPI, UIA, NSAccessibility, and Web ARIA.",
            ),
        )),
        Separator::horizontal(),
        // Core features (responsive 3-column row)
        HStack::new((
            Card::new("Incremental Updates")
                .description("Only dirty nodes emit accesskit::Node diffs during steady-state frame ticks")
                .flex_grow(1.0),
            Card::new("Two-Way Dispatch")
                .description("Routes Click, Focus, Value, and Scroll directly from screen readers into reactive signals")
                .flex_grow(1.0),
            Card::new("Live Announcements")
                .description("Polite and Assertive speech queues for dynamic system updates and alerts")
                .flex_grow(1.0),
        ))
        .spacing(12),
        Separator::horizontal(),
        // Live Screen Reader Feed (responsive card)
        Card::new("Simulated Screen Reader Event Log")
            .description("Real-time stream of accessibility events dispatched to native assistive technology")
            .flex_grow(1.0),
        VStack::new((
            HStack::new((
                Badge::new("FOCUS"),
                Text::new(r#"Focused node #14: Role=Button, Label="Submit Form""#),
            ))
            .spacing(8),
            HStack::new((
                Badge::new("VALUE"),
                Text::new(r#"Value changed on #18: Role=Slider, Value="65%""#),
            ))
            .spacing(8),
            HStack::new((
                Badge::new("SPEECH").variant(BadgeVariant::Success),
                Text::new(r#"Announced (Polite): "Saved snapshot to disk successfully""#),
            ))
            .spacing(8),
        ))
        .spacing(8),
    ))
    .spacing(16)
    .flex_grow(1.0)
}
