use tej_ui_core::clear_context_stack;
use tej_ui_hal::Color;
use tej_ui_view::{Theme, View, ViewExt, provide_theme, use_theme};

#[test]
fn test_theme_presets_and_semantic_tokens() {
    let dark = Theme::dark();
    assert!(dark.is_dark);
    assert_eq!(dark.colors.background, Color::from_rgb_u32(0x09090B));
    assert_eq!(dark.colors.foreground, Color::from_rgb_u32(0xFAFAFA));
    assert_eq!(dark.radii.md, 6.0);
    assert_eq!(dark.spacing.md, 16.0);

    let light = Theme::light();
    assert!(!light.is_dark);
    assert_eq!(light.colors.background, Color::WHITE);
    assert_eq!(light.colors.foreground, Color::from_rgb_u32(0x09090B));

    let hc = Theme::high_contrast();
    assert!(hc.is_dark);
    assert_eq!(hc.colors.background, Color::BLACK);
    assert_eq!(hc.colors.foreground, Color::WHITE);
}

#[test]
fn test_theme_customization() {
    let custom = Theme::dark()
        .with_primary(Color::rgb(16, 185, 129)) // Emerald 500
        .with_name("Emerald Dark");

    assert_eq!(custom.name, "Emerald Dark");
    assert_eq!(custom.colors.primary, Color::rgb(16, 185, 129));
    assert_eq!(custom.colors.ring, Color::rgb(16, 185, 129));
}

#[derive(Debug)]
struct ThemedWidget;

impl View for ThemedWidget {
    type State = Color;

    fn build(&self) -> Self::State {
        use_theme().colors.primary
    }

    fn rebuild(&self, state: &mut Self::State) {
        *state = use_theme().colors.primary;
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}

#[test]
fn test_theme_context_propagation_and_subtree_override() {
    clear_context_stack();

    // Default fallback when nothing provided
    assert_eq!(use_theme().name, "Zinc Dark");

    // Provide root light theme
    provide_theme(Theme::light());
    assert_eq!(use_theme().name, "Zinc Light");

    // Subtree with custom dark theme
    let custom_dark = Theme::dark().with_primary(Color::rgb(249, 115, 22)); // Orange 500
    let tree = ThemedWidget.theme(custom_dark);

    let state = tree.build();
    assert_eq!(state, Color::rgb(249, 115, 22));

    // Outside the subtree, the ambient root theme is still intact
    assert_eq!(use_theme().name, "Zinc Light");
}
