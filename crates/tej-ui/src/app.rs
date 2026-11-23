use std::fmt::Debug;
use tej_ui_hal::{DirtyRectTracker, PlatformWindow, SurfaceCompositor, WindowConfig};
use tej_ui_layout::{LayoutArena, LayoutEngine};
use tej_ui_view::View;

#[cfg(target_os = "windows")]
use tej_ui_platform_windows::{
    DirectCompositionSurface as NativeSurface, WindowsWindow as NativeWindow,
};

#[cfg(target_os = "macos")]
use tej_ui_platform_macos::{MacosWindow as NativeWindow, MetalSurfaceCompositor as NativeSurface};

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use tej_ui_platform_linux::{LinuxSurface as NativeSurface, LinuxWindow as NativeWindow};

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

    /// Run the interactive event loop coordinating windowing, events, layout, and rendering.
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut window = NativeWindow::from_config(self.config);
        if !window.is_open() {
            return Ok(());
        }

        let mut surface = NativeSurface::from_size(window.size());
        let mut layout_arena = LayoutArena::new();
        let mut dirty_tracker = DirtyRectTracker::new();
        let mut dispatcher = tej_ui_view::EventDispatcher::new();

        let mut state = self.root_view.build();

        let mut root_node = self.root_view.layout(&mut state, &mut layout_arena);
        LayoutEngine::compute_layout(&mut layout_arena, root_node, window.size());
        self.root_view.register_events(&state, &mut dispatcher);

        dirty_tracker.mark_full_dirty();
        let mut scroll_offset_y: f32 = 0.0;

        while window.is_open() {
            let events = window.poll_events();
            let mut needs_relayout = false;
            let mut needs_repaint = false;

            for event in &events {
                match event {
                    tej_ui_hal::UiEvent::Resize { width, height } => {
                        let new_size = tej_ui_layout::Size::new(*width, *height);
                        surface.resize(new_size);
                        needs_relayout = true;
                    }
                    tej_ui_hal::UiEvent::Scroll { delta_y, .. } => {
                        scroll_offset_y = (scroll_offset_y - delta_y * 45.0).max(0.0);
                        needs_repaint = true;
                        dirty_tracker.mark_full_dirty();
                    }
                    tej_ui_hal::UiEvent::MouseMove { position } => {
                        let adj_pos = if position.x >= 220.0 && position.y >= 50.0 {
                            tej_ui_layout::Point::new(position.x, position.y + scroll_offset_y)
                        } else {
                            *position
                        };
                        let adj_event = tej_ui_hal::UiEvent::MouseMove { position: adj_pos };
                        let res = dispatcher.dispatch(&adj_event, &layout_arena, root_node);
                        if res.handled || res.needs_repaint {
                            needs_repaint = true;
                            dirty_tracker.mark_full_dirty();
                        }
                    }
                    tej_ui_hal::UiEvent::MouseDown { button, position } => {
                        let adj_pos = if position.x >= 220.0 && position.y >= 50.0 {
                            tej_ui_layout::Point::new(position.x, position.y + scroll_offset_y)
                        } else {
                            *position
                        };
                        let adj_event = tej_ui_hal::UiEvent::MouseDown {
                            button: *button,
                            position: adj_pos,
                        };
                        let res = dispatcher.dispatch(&adj_event, &layout_arena, root_node);
                        if res.handled {
                            needs_relayout = true;
                            needs_repaint = true;
                            dirty_tracker.mark_full_dirty();
                        } else if res.needs_repaint {
                            needs_repaint = true;
                            dirty_tracker.mark_full_dirty();
                        }
                    }
                    tej_ui_hal::UiEvent::MouseUp { button, position } => {
                        let adj_pos = if position.x >= 220.0 && position.y >= 50.0 {
                            tej_ui_layout::Point::new(position.x, position.y + scroll_offset_y)
                        } else {
                            *position
                        };
                        let adj_event = tej_ui_hal::UiEvent::MouseUp {
                            button: *button,
                            position: adj_pos,
                        };
                        let res = dispatcher.dispatch(&adj_event, &layout_arena, root_node);
                        if res.handled {
                            needs_relayout = true;
                            needs_repaint = true;
                            dirty_tracker.mark_full_dirty();
                        } else if res.needs_repaint {
                            needs_repaint = true;
                            dirty_tracker.mark_full_dirty();
                        }
                    }
                    _ => {
                        let res = dispatcher.dispatch(event, &layout_arena, root_node);
                        if res.handled {
                            needs_relayout = true;
                            needs_repaint = true;
                            dirty_tracker.mark_full_dirty();
                        } else if res.needs_repaint {
                            needs_repaint = true;
                            dirty_tracker.mark_full_dirty();
                        }
                    }
                }
            }

            if needs_repaint || needs_relayout || dirty_tracker.has_dirty() {
                self.root_view.patch(&mut state, 0);
            }

            if needs_relayout {
                layout_arena.clear();
                root_node = self.root_view.layout(&mut state, &mut layout_arena);
                LayoutEngine::compute_layout(&mut layout_arena, root_node, window.size());
                dispatcher.clear_handlers();
                self.root_view.register_events(&state, &mut dispatcher);
                dirty_tracker.mark_full_dirty();
                needs_repaint = true;
            }

            if needs_repaint || dirty_tracker.has_dirty() {
                surface.begin_frame(dirty_tracker.rects());
                let cur_theme = tej_ui_view::use_theme();
                surface.fill_rect(
                    tej_ui_layout::Rect::new(0.0, 0.0, window.size().width, window.size().height),
                    cur_theme.colors.background,
                );
                let mut paint_ctx = tej_ui_view::PaintContext::new(
                    &mut surface,
                    &layout_arena,
                    &cur_theme,
                    dirty_tracker.rects(),
                )
                .with_scroll(scroll_offset_y, 220.0)
                .with_interaction(
                    dispatcher.focused_node,
                    dispatcher.hovered_node,
                    dispatcher.pressed_node,
                );
                self.root_view.paint(&state, &mut paint_ctx);
                surface.end_frame();
                window.present(&surface);
                dirty_tracker.reset();
                // When actively rendering, run at standard ~120Hz tick pacing
                std::thread::sleep(std::time::Duration::from_millis(8));
            } else {
                window.present(&surface);
                // When idle with no mutations, sleep longer (16ms) to eliminate CPU wakeups
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
        }

        Ok(())
    }

    /// Run the single-frame execution or event loop.
    pub fn run_headless_frame(self) -> V::State {
        let window = NativeWindow::from_config(self.config);
        let mut surface = NativeSurface::from_size(window.size());
        let mut layout_arena = LayoutArena::new();
        let mut dirty_tracker = DirtyRectTracker::new();

        // 1. Build view tree state
        let mut state = self.root_view.build();

        // 2. Compute initial layout across view tree
        let root_node = self.root_view.layout(&mut state, &mut layout_arena);
        LayoutEngine::compute_layout(&mut layout_arena, root_node, window.size());

        // 3. Mark dirty, paint view hierarchy, and present
        let theme = tej_ui_view::use_theme();
        dirty_tracker.mark_full_dirty();
        surface.begin_frame(dirty_tracker.rects());
        surface.fill_rect(
            tej_ui_layout::Rect::new(0.0, 0.0, window.size().width, window.size().height),
            theme.colors.background,
        );

        let mut paint_ctx = tej_ui_view::PaintContext::new(
            &mut surface,
            &layout_arena,
            &theme,
            dirty_tracker.rects(),
        );
        self.root_view.paint(&state, &mut paint_ctx);

        surface.end_frame();
        dirty_tracker.reset();

        state
    }
}
