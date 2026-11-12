use std::fmt::Debug;
use tej_hal::{DirtyRectTracker, PlatformWindow, SurfaceCompositor, WindowConfig};
use tej_layout::{LayoutArena, LayoutEngine};
use tej_platform_linux::{LinuxSurface, LinuxWindow};
use tej_view::View;

/// High-level application runner coordinating windowing, reactive updates, layout, and presentation.
pub struct App<V: View> {
    config: WindowConfig,
    root_view: V,
}

impl<V: View> Debug for App<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("config", &self.config)
            .field("root_view", &self.root_view)
            .finish()
    }
}

impl<V: View> App<V> {
    /// Create an application with a root view tree.
    pub fn new(root_view: V) -> Self {
        Self {
            config: WindowConfig::default(),
            root_view,
        }
    }

    /// Set window configuration.
    pub fn window(mut self, config: WindowConfig) -> Self {
        self.config = config;
        self
    }

    /// Run the single-frame execution or event loop.
    pub fn run_headless_frame(self) -> V::State {
        let window = LinuxWindow::new(self.config);
        let mut surface = LinuxSurface::new(window.size());
        let mut layout_arena = LayoutArena::new();
        let mut dirty_tracker = DirtyRectTracker::new();

        // 1. Build view tree state
        let state = self.root_view.build();

        // 2. Compute initial layout
        let root_node = layout_arena.allocate_node(tej_layout::LayoutStyle::default());
        LayoutEngine::compute_layout(&mut layout_arena, root_node, window.size());

        // 3. Mark dirty and present
        dirty_tracker.mark_full_dirty();
        surface.begin_frame(dirty_tracker.rects());
        surface.end_frame();
        dirty_tracker.reset();

        state
    }
}
