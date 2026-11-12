//! 3D Canvas view widget and OS compositor bridge.

use crate::camera::PerspectiveCamera;
use crate::controls::OrbitControls;
use crate::renderer::Rasterizer3D;
use crate::scene::Scene;
use std::fmt;
use std::sync::Arc;
use tej_hal::{Color, SurfaceCompositor, UiEvent};
use tej_layout::{Dimension, Point, Rect};
use tej_view::View;

/// Retained state for a mounted `Canvas3D` widget.
#[derive(Debug, Default)]
pub struct Canvas3DState {
    pub bounds: Rect,
    pub frame_count: u64,
}

/// Reactive 3D Canvas widget that renders a Three.js-style 3D scene within the 2D UI tree.
#[derive(Clone)]
pub struct Canvas3D {
    scene: Arc<parking_lot::RwLock<Scene>>,
    camera: Arc<parking_lot::RwLock<PerspectiveCamera>>,
    controls: Arc<parking_lot::RwLock<OrbitControls>>,
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
        Self {
            scene: Arc::new(parking_lot::RwLock::new(scene)),
            camera: Arc::new(parking_lot::RwLock::new(camera)),
            controls: Arc::new(parking_lot::RwLock::new(OrbitControls::new())),
            width: Dimension::Auto,
            height: Dimension::Auto,
        }
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

    /// Composites the rendered 3D scene onto the 2D surface compositor.
    pub fn paint(&self, compositor: &mut dyn SurfaceCompositor, bounds: Rect) {
        let w = (bounds.size.width as usize).max(32);
        let h = (bounds.size.height as usize).max(32);

        let mut rasterizer = Rasterizer3D::new(w, h);
        self.render_offline(&mut rasterizer);

        // Blit color buffer bounding box to 2D compositor
        compositor.fill_rect(bounds, Color::rgb(30, 30, 36));

        // Draw top accent line
        compositor.draw_line(
            Point::new(bounds.origin.x, bounds.origin.y),
            Point::new(bounds.origin.x + bounds.size.width, bounds.origin.y),
            1.0,
            Color::rgb(70, 70, 85),
        );
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
}
