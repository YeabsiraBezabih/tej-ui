//! Buttons, Badges, and Avatars showcase view.

use tej_ui::prelude::*;

/// Category view showcasing button variants, sizing, badges, and user avatars.
pub fn buttons_badges_view() -> impl View {
    VStack::new((
        // Section 1: Button Variants
        Card::new("Button Style Variants")
            .description("Comprehensive button variants conforming to Tier-1 design tokens")
            .flex_grow(1.0)
            .content(
                VStack::new((
                    HStack::new((
                        Button::new(Text::new("Primary Action")).variant(ButtonVariant::Default),
                        Button::new(Text::new("Secondary")).variant(ButtonVariant::Secondary),
                        Button::new(Text::new("Destructive")).variant(ButtonVariant::Destructive),
                        Button::new(Text::new("Outline")).variant(ButtonVariant::Outline),
                        Button::new(Text::new("Ghost")).variant(ButtonVariant::Ghost),
                        Button::new(Text::new("Link Style")).variant(ButtonVariant::Link),
                    ))
                    .spacing(8),
                    HStack::new((
                        Button::new(Text::new("Small Size")).size(ButtonSize::Sm),
                        Button::new(Text::new("Default Sizing")).size(ButtonSize::Default),
                        Button::new(Text::new("Large CTA")).size(ButtonSize::Lg),
                        Button::new(Text::new("Disabled Action"))
                            .variant(ButtonVariant::Secondary)
                            .disabled(true),
                    ))
                    .spacing(8),
                ))
                .spacing(8),
            ),
        // Section 2: Badges & Tags
        Card::new("Badges & Semantic Tags")
            .description("Micro-indicators communicating status, counts, and category labels")
            .flex_grow(1.0)
            .content(
                HStack::new((
                    Badge::new("DEFAULT").variant(BadgeVariant::Default),
                    Badge::new("SECONDARY").variant(BadgeVariant::Secondary),
                    Badge::new("OUTLINE").variant(BadgeVariant::Outline),
                    Badge::new("ACTIVE / SUCCESS").variant(BadgeVariant::Success),
                    Badge::new("WARNING").variant(BadgeVariant::Warning),
                    Badge::new("DESTRUCTIVE").variant(BadgeVariant::Destructive),
                ))
                .spacing(8),
            ),
        // Section 3: User Avatars
        Card::new("User Avatars & Initials")
            .description("Circular profile representations with fallback initials calculation")
            .flex_grow(1.0)
            .content(
                HStack::new((
                    HStack::new((
                        Avatar::new("Alice Smith").diameter(40.0),
                        VStack::new((Text::new("Alice Smith"), Text::new("alice@tej.dev"))),
                    ))
                    .spacing(8),
                    HStack::new((
                        Avatar::new("Bob Jones").diameter(40.0),
                        VStack::new((Text::new("Bob Jones"), Text::new("bob@tej.dev"))),
                    ))
                    .spacing(8),
                    HStack::new((
                        Avatar::new("Rust Core").diameter(40.0),
                        VStack::new((Text::new("Rust Core"), Text::new("core@tej.dev"))),
                    ))
                    .spacing(8),
                ))
                .spacing(16),
            ),
    ))
    .spacing(16)
}
