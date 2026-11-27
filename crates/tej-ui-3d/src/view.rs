//! 3D Canvas view widget and OS compositor bridge.

use crate::camera::PerspectiveCamera;
use crate::controls::OrbitControls;
use crate::renderer::Rasterizer3D;
use crate::scene::Scene;
use std::fmt;
use std::sync::Arc;
use tej_ui_hal::{Color, SurfaceCompositor, UiEvent};
use tej_ui_layout::{Dimension, Point, Rect};
use tej_ui_view::View;

/// Retained state for a mounted `Canvas3D` widget.
#[derive(Debug, Default)]
pub struct Canvas3DState {
    pub bounds: Rect,
    pub node: u32,
    pub frame_count: u64,
}

/// Reactive 3D Canvas widget that renders a Three.js-style 3D scene within the 2D UI tree.
#[derive(Clone)]
pub struct Canvas3D {
    scene: Arc<parking_lot::RwLock<Scene>>,
    camera: Arc<parking_lot::RwLock<PerspectiveCamera>>,
    controls: Arc<parking_lot::RwLock<OrbitControls>>,
    rasterizer: Arc<parking_lot::Mutex<Option<Rasterizer3D>>>,
    width: Dimension,
    height: Dimension,
}

impl fmt::Debug for Canvas3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Canvas3D")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl Canvas3D {
    /// Creates a new `Canvas3D` widget with a scene and camera.
    pub fn new(scene: Scene, camera: PerspectiveCamera) -> Self {
        let mut controls = OrbitControls::new();
        controls.sync_from_camera(&camera);
        Self {
            scene: Arc::new(parking_lot::RwLock::new(scene)),
            camera: Arc::new(parking_lot::RwLock::new(camera)),
            controls: Arc::new(parking_lot::RwLock::new(controls)),
            rasterizer: Arc::new(parking_lot::Mutex::new(None)),
            width: Dimension::Auto,
            height: Dimension::Auto,
        }
    }

    /// Returns the dimensions of the cached internal rasterizer, if initialized.
    #[must_use]
    pub fn cached_rasterizer_size(&self) -> Option<(usize, usize)> {
        self.rasterizer.lock().as_ref().map(|r| (r.width, r.height))
    }

    /// Sets explicit widget width.
    #[must_use]
    pub fn width(mut self, width: Dimension) -> Self {
        self.width = width;
        self
    }

    /// Sets explicit widget height.
    #[must_use]
    pub fn height(mut self, height: Dimension) -> Self {
        self.height = height;
        self
    }

    /// Access the underlying scene for mutations (e.g. adding meshes, moving objects).
    #[must_use]
    pub fn scene(&self) -> Arc<parking_lot::RwLock<Scene>> {
        Arc::clone(&self.scene)
    }

    /// Access the perspective camera.
    #[must_use]
    pub fn camera(&self) -> Arc<parking_lot::RwLock<PerspectiveCamera>> {
        Arc::clone(&self.camera)
    }

    /// Access the orbit controls.
    #[must_use]
    pub fn controls(&self) -> Arc<parking_lot::RwLock<OrbitControls>> {
        Arc::clone(&self.controls)
    }

    /// Dispatches UI mouse and scroll events to interactive 3D orbit controls.
    pub fn handle_event(&self, event: &UiEvent) {
        let mut cam = self.camera.write();
        let mut ctrl = self.controls.write();
        ctrl.handle_event(event, &mut cam);
    }

    /// Renders the 3D scene into an offline rasterizer.
    pub fn render_offline(&self, rasterizer: &mut Rasterizer3D) {
        let scene = self.scene.read();
        let camera = self.camera.read();
        rasterizer.render_scene(&scene, &camera);
    }

    /// Composites the rendered 3D scene onto the 2D surface compositor, reusing a cached
    /// [`Rasterizer3D`] instance and resizing only when viewport dimensions change.
    pub fn paint_direct(&self, compositor: &mut dyn SurfaceCompositor, bounds: Rect) {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let w = (bounds.size.width as usize).clamp(32, 2048);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let h = (bounds.size.height as usize).clamp(32, 2048);

        let mut guard = self.rasterizer.lock();
        let rasterizer = match guard.as_mut() {
            Some(r) => {
                r.resize(w, h);
                r
            }
            None => guard.insert(Rasterizer3D::new(w, h)),
        };

        rasterizer.clear(Color::rgb(30, 30, 36));
        self.render_offline(rasterizer);

        // Blit color buffer directly to 2D compositor
        compositor.blit_pixels(bounds.origin, w, h, &rasterizer.color_buffer);

        // Draw boundary border around 3D viewport
        let p0 = bounds.origin;
        let p1 = Point::new(bounds.origin.x + bounds.size.width, bounds.origin.y);
        let p2 = Point::new(
            bounds.origin.x + bounds.size.width,
            bounds.origin.y + bounds.size.height,
        );
        let p3 = Point::new(bounds.origin.x, bounds.origin.y + bounds.size.height);
        compositor.draw_line(p0, p1, 1.0, Color::rgb(55, 58, 70));
        compositor.draw_line(p1, p2, 1.0, Color::rgb(55, 58, 70));
        compositor.draw_line(p2, p3, 1.0, Color::rgb(55, 58, 70));
        compositor.draw_line(p3, p0, 1.0, Color::rgb(55, 58, 70));
    }

    /// Legacy paint entrypoint delegating to `paint_direct`.
    pub fn paint(&self, compositor: &mut dyn SurfaceCompositor, bounds: Rect) {
        self.paint_direct(compositor, bounds);
    }
}

impl View for Canvas3D {
    type State = Canvas3DState;

    fn build(&self) -> Self::State {
        Canvas3DState::default()
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.frame_count += 1;
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        state.frame_count += 1;
    }

    fn layout(&self, state: &mut Self::State, arena: &mut tej_ui_layout::LayoutArena) -> u32 {
        let style = tej_ui_layout::LayoutStyle {
            width: self.width,
            height: self.height,
            ..Default::default()
        };
        let node = arena.allocate_node(style);
        state.node = node;
        node
    }

    fn paint(&self, state: &Self::State, ctx: &mut tej_ui_view::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(Rect::ZERO);
        self.paint_direct(ctx.compositor, bounds);
    }

    fn register_events(&self, state: &Self::State, dispatcher: &mut tej_ui_view::EventDispatcher) {
        let cam = Arc::clone(&self.camera);
        let ctrl = Arc::clone(&self.controls);
        dispatcher.register_bubble(
            state.node,
            Arc::new(move |event, _phase, _node| {
                let mut c = cam.write();
                let mut o = ctrl.write();
                o.handle_event(event, &mut c);
                tej_ui_view::EventPropagation::Repaint
            }),
        );
    }
}
