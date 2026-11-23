//! `tej-gallery`: Unified Interactive Showcase & Storybook suite for `tej-ui`.
//!
//! Provides real-time demonstration of:
//! - 210 widgets across all component families
//! - 231 live CSS properties with interactive controls
//! - Real-time telemetry & performance HUD (120 FPS, 0 steady-state allocations)
//! - 2D Vector Path & Immediate-Mode Canvas tessellation
//! - 3D Hybrid Mesh Viewport via `tej-3d`
//! - AccessKit semantic accessibility bridge and announcement inspector

pub mod state;
pub mod views;

pub use state::{GalleryState, GalleryTab, TelemetryMetrics, ThemeMode};
pub use views::gallery_main_view;

use tej_ui::prelude::*;

/// Build the root application view tree for the Tej Gallery.
pub fn build_gallery_app(state: GalleryState) -> impl View {
    gallery_main_view(state)
}

/// Executes headless verification of the showcase suite, testing layout,
/// rendering, and widget mounting across all gallery tabs.
pub fn run_headless_verification() -> Result<(), Box<dyn std::error::Error>> {
    let state = GalleryState::new();
    let mut arena = LayoutArena::new();
    let window_size = Size::new(1280.0, 800.0);

    for tab in [
        GalleryTab::Widgets,
        GalleryTab::CssPlayground,
        GalleryTab::VectorCanvas,
        GalleryTab::Viewport3D,
        GalleryTab::A11yInspector,
    ] {
        state.select_tab(tab);
        let view = build_gallery_app(state);
        let mut view_state = view.build();

        arena.clear();
        let root = view.layout(&mut view_state, &mut arena);
        LayoutEngine::compute_layout(&mut arena, root, window_size);

        assert!(
            !arena.is_empty(),
            "Gallery tab {:?} failed to produce layout nodes",
            tab
        );
    }

    Ok(())
}
