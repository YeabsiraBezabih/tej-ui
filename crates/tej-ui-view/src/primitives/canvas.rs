use crate::view::View;
use std::fmt;
use std::sync::Arc;
use tej_ui_hal::SurfaceCompositor;
use tej_ui_layout::{Dimension, Rect};

/// Render callback receiving the hardware 2D surface compositor and available bounding box.
pub type CanvasPainter = Arc<dyn Fn(&mut dyn SurfaceCompositor, Rect) + Send + Sync>;

/// Hardware-accelerated custom 2D/3D canvas surface widget.
///
/// Functions identically to an HTML `<canvas>` or native GPU surface viewport,
/// allowing rich interactive graphics (Gantt charts, node graphs, 2D vector diagrams,
/// and embedded 3D scenes) inside the static view hierarchy.
#[derive(Clone)]
pub struct Canvas {
    painter: CanvasPainter,
    width: Dimension,
    height: Dimension,
}

impl fmt::Debug for Canvas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Canvas")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}

/// State retained by a mounted Canvas widget.
#[derive(Debug, Default)]
pub struct CanvasState {
    pub bounds: Rect,
    pub redraw_count: u64,
    pub node: u32,
}

impl Canvas {
    /// Create a new hardware-accelerated canvas with a painter closure.
    pub fn new<F>(painter: F) -> Self
    where
        F: Fn(&mut dyn SurfaceCompositor, Rect) + Send + Sync + 'static,
    {
        Self {
            painter: Arc::new(painter),
            width: Dimension::Auto,
            height: Dimension::Auto,
        }
    }

    /// Set explicit canvas width constraint.
    #[must_use]
    pub fn width(mut self, width: Dimension) -> Self {
        self.width = width;
        self
    }

    /// Set explicit canvas height constraint.
    #[must_use]
    pub fn height(mut self, height: Dimension) -> Self {
        self.height = height;
        self
    }

    /// Paint the canvas surface with the registered painter closure.
    pub fn paint_direct(&self, compositor: &mut dyn SurfaceCompositor, bounds: Rect) {
        (self.painter)(compositor, bounds);
    }
}

impl View for Canvas {
    type State = CanvasState;

    fn build(&self) -> Self::State {
        CanvasState::default()
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.redraw_count += 1;
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        state.redraw_count += 1;
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

    fn paint(&self, state: &Self::State, ctx: &mut crate::paint::PaintContext<'_>) {
        let bounds = ctx
            .arena
            .computed_bounds
            .get(state.node as usize)
            .copied()
            .unwrap_or(state.bounds);
        (self.painter)(ctx.compositor, bounds);
    }
}
