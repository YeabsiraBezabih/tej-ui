//! Headless deterministic test runner and semantic query selector harness.

use crate::events::EventDispatcher;
use crate::view::View;
use std::fmt::Debug;
use tej_ui_hal::{MouseButton, UiEvent};
use tej_ui_layout::{LayoutArena, LayoutEngine, Point, Rect, Size};

/// Semantic descriptor of an inspected view node within the test harness.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TestNode {
    /// Arena node identifier.
    pub id: u32,
    /// Computed absolute layout bounds.
    pub bounds: Rect,
}

impl TestNode {
    /// Center point coordinates of this node.
    #[must_use]
    pub fn center(&self) -> Point {
        Point::new(
            self.bounds.origin.x + self.bounds.size.width * 0.5,
            self.bounds.origin.y + self.bounds.size.height * 0.5,
        )
    }
}

/// Headless deterministic harness for testing reactive view trees in CI without a display server.
pub struct TestHarness<V: View> {
    root_view: V,
    state: V::State,
    layout_arena: LayoutArena,
    root_node: u32,
    dispatcher: EventDispatcher,
    viewport_size: Size,
    mock_clock_millis: u64,
    frame_count: u64,
}

impl<V: View> Debug for TestHarness<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestHarness")
            .field("viewport_size", &self.viewport_size)
            .field("mock_clock_millis", &self.mock_clock_millis)
            .field("frame_count", &self.frame_count)
            .field("root_node", &self.root_node)
            .finish()
    }
}

impl<V: View> TestHarness<V> {
    /// Create a new test harness mounting `root_view` in a default 800x600 viewport.
    #[must_use]
    pub fn new(root_view: V) -> Self {
        Self::with_size(root_view, Size::new(800.0, 600.0))
    }

    /// Create a test harness with a custom viewport size.
    #[must_use]
    pub fn with_size(root_view: V, viewport_size: Size) -> Self {
        let mut state = root_view.build();
        let mut layout_arena = LayoutArena::new();
        let root_node = root_view.layout(&mut state, &mut layout_arena);
        LayoutEngine::compute_layout(&mut layout_arena, root_node, viewport_size);

        let mut dispatcher = EventDispatcher::new();
        root_view.register_events(&state, &mut dispatcher);

        Self {
            root_view,
            state,
            layout_arena,
            root_node,
            dispatcher,
            viewport_size,
            mock_clock_millis: 0,
            frame_count: 1,
        }
    }

    /// Read mutable reference to view state.
    pub fn state_mut(&mut self) -> &mut V::State {
        &mut self.state
    }

    /// Read view state.
    #[must_use]
    pub fn state(&self) -> &V::State {
        &self.state
    }

    /// Number of executed frames.
    #[must_use]
    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Current mock clock time in milliseconds.
    #[must_use]
    pub const fn mock_time_millis(&self) -> u64 {
        self.mock_clock_millis
    }

    /// Advance the deterministic mock clock by `delta_millis` and step a frame.
    pub fn advance_time(&mut self, delta_millis: u64) {
        self.mock_clock_millis += delta_millis;
        self.step_frame();
    }

    /// Run a reactive layout and reconciliation step for the next frame.
    pub fn step_frame(&mut self) {
        self.root_view.patch(&mut self.state, 0);

        self.layout_arena = LayoutArena::new();
        self.root_node = self
            .root_view
            .layout(&mut self.state, &mut self.layout_arena);
        LayoutEngine::compute_layout(&mut self.layout_arena, self.root_node, self.viewport_size);

        self.dispatcher.clear_handlers();
        self.root_view
            .register_events(&self.state, &mut self.dispatcher);

        self.frame_count += 1;
    }

    /// Dispatch an arbitrary raw UI event into the harness.
    pub fn dispatch_event(&mut self, event: &UiEvent) {
        let _ = self
            .dispatcher
            .dispatch(event, &self.layout_arena, self.root_node);
        self.step_frame();
    }

    /// Simulate a mouse click (MouseDown followed by MouseUp) at specified coordinate.
    pub fn click_at(&mut self, point: Point) {
        let down = UiEvent::MouseDown {
            button: MouseButton::Left,
            position: point,
        };
        let up = UiEvent::MouseUp {
            button: MouseButton::Left,
            position: point,
        };

        let _ = self
            .dispatcher
            .dispatch(&down, &self.layout_arena, self.root_node);
        let _ = self
            .dispatcher
            .dispatch(&up, &self.layout_arena, self.root_node);
        self.step_frame();
    }

    /// Simulate a mouse click targeting the center of a specific node.
    pub fn click_node(&mut self, node: TestNode) {
        self.click_at(node.center());
    }

    /// Query node information by its arena node ID.
    #[must_use]
    pub fn get_node(&self, node_id: u32) -> Option<TestNode> {
        if (node_id as usize) < self.layout_arena.computed_bounds.len() {
            let bounds = self.layout_arena.computed_bounds[node_id as usize];
            Some(TestNode {
                id: node_id,
                bounds,
            })
        } else {
            None
        }
    }

    /// Root node of the view tree.
    #[must_use]
    pub fn root_node(&self) -> TestNode {
        let bounds = self.layout_arena.computed_bounds[self.root_node as usize];
        TestNode {
            id: self.root_node,
            bounds,
        }
    }

    /// Total count of active layout nodes in the arena.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.layout_arena.styles.len()
    }
}
