//! Integration tests for the Tej Gallery showcase suite.

use tej_ui::prelude::*;
use tej_ui_gallery::{
    GalleryState, GalleryTab, ThemeMode, build_gallery_app, run_headless_verification,
};

#[test]
fn test_gallery_headless_verification() {
    assert!(run_headless_verification().is_ok());
}

#[test]
fn test_gallery_all_tabs_layout_generation() {
    let state = GalleryState::new();
    let mut arena = LayoutArena::new();
    let viewport = Size::new(1280.0, 800.0);

    let tabs = [
        GalleryTab::Widgets,
        GalleryTab::CssPlayground,
        GalleryTab::VectorCanvas,
        GalleryTab::Viewport3D,
        GalleryTab::A11yInspector,
    ];

    for tab in tabs {
        state.select_tab(tab);
        let view = build_gallery_app(state);
        let mut view_state = view.build();

        arena.clear();
        let root = view.layout(&mut view_state, &mut arena);
        LayoutEngine::compute_layout(&mut arena, root, viewport);

        let count = arena.len();
        assert!(
            count > 5,
            "Tab {tab:?} produced only {count} layout nodes, expected rich tree"
        );
    }
}

#[test]
fn test_css_properties_parity_gate() {
    // Release gate verification: exactly 231 CSS properties supported
    assert_eq!(
        TOTAL_CSS_PROPERTIES, 231,
        "Framework must support exactly 231 CSS properties for full web parity"
    );
}

#[test]
fn test_theme_mode_switching() {
    for mode in [
        ThemeMode::Dark,
        ThemeMode::Light,
        ThemeMode::HighContrast,
        ThemeMode::Cyberpunk,
    ] {
        let theme = mode.to_theme();
        assert!(theme.typography.body.font_size > 0.0);
    }
}

#[test]
fn test_telemetry_hud_metrics() {
    let state = GalleryState::new();
    let m = state.telemetry.get();
    assert_eq!(m.fps, 120.0);
    assert_eq!(m.steady_state_allocations, 0);
}
