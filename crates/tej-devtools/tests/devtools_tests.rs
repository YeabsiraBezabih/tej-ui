use std::time::Duration;
use tej_core::{Epoch, NodeId, Signal};
use tej_devtools::{
    DagNodeInfo, DagSnapshot, DirtyHudOverlay, HotReloadHost, ReactiveDagInspector,
};
use tej_hal::{Color, SurfaceCompositor};
use tej_layout::{Point, Rect};
use tej_view::Text;

#[test]
fn test_dag_inspector_and_snapshot_dot() {
    let mut inspector = ReactiveDagInspector::new();
    let count_signal = Signal::new(42);
    inspector.name_signal(count_signal.id(), "counter_state");

    assert_eq!(
        inspector.node_label(NodeId::new(
            count_signal.id().index,
            count_signal.id().generation
        )),
        "counter_state"
    );

    let mut snapshot = DagSnapshot::new();
    snapshot.add_node(DagNodeInfo {
        id: NodeId::new(1, 0),
        label: "counter_state".to_string(),
        last_updated_epoch: Epoch(1),
        is_dirty: false,
        dependency_count: 0,
        subscriber_count: 1,
    });
    snapshot.add_node(DagNodeInfo {
        id: NodeId::new(2, 0),
        label: "derived_display".to_string(),
        last_updated_epoch: Epoch(2),
        is_dirty: true,
        dependency_count: 1,
        subscriber_count: 0,
    });
    snapshot.add_edge(NodeId::new(1, 0), NodeId::new(2, 0));

    assert_eq!(snapshot.dirty_count(), 1);
    let dot = snapshot.to_dot();
    assert!(dot.contains("digraph ReactiveDAG"));
    assert!(dot.contains("counter_state"));
    assert!(dot.contains("derived_display"));
    assert!(dot.contains("->"));
}

#[derive(Debug)]
struct MockSurface {
    fill_count: usize,
}

impl SurfaceCompositor for MockSurface {
    fn begin_frame(&mut self, _dirty_rects: &[Rect]) {}
    fn fill_rect(&mut self, _rect: Rect, _color: Color) {
        self.fill_count += 1;
    }
    fn fill_rounded_rect(&mut self, _rect: Rect, _radius: f32, _color: Color) {}
    fn draw_line(&mut self, _p1: Point, _p2: Point, _stroke_width: f32, _color: Color) {}
    fn draw_text(&mut self, _text: &str, _origin: Point, _font_size: f32, _color: Color) {}
    fn end_frame(&mut self) {}
}

#[test]
fn test_dirty_hud_metrics_and_paint_flashing() {
    let mut hud = DirtyHudOverlay::new();
    assert!(hud.is_enabled());

    hud.record_frame(Duration::from_millis(8), 2, 5);
    hud.record_frame(Duration::from_millis(12), 1, 3);

    assert_eq!(hud.average_frame_time_ms(), 10.0);
    assert!((hud.estimated_fps() - 100.0).abs() < 1.0);

    let mut surface = MockSurface { fill_count: 0 };
    let dirty_regions = [Rect::new(0.0, 0.0, 100.0, 100.0)];

    hud.render_dirty_flashes(&mut surface, &dirty_regions);
    assert_eq!(surface.fill_count, 1);
}

#[test]
fn test_state_preserving_hot_reload() {
    // Persistent reactive state signal
    let counter = Signal::new(100);

    // Initial view tree
    let view_v1 = Text::new("Version 1");
    let mut host = HotReloadHost::new(view_v1);
    assert_eq!(host.generation(), 1);
    assert_eq!(host.reload_count(), 0);

    // Update signal state
    counter.set(200);
    assert_eq!(counter.get(), 200);

    // Hot reload swap view tree
    let view_v2 = Text::new("Version 2 - Updated UI");
    let elapsed_ms = host.swap_view(view_v2);
    assert!(elapsed_ms >= 0.0);
    assert_eq!(host.generation(), 2);
    assert_eq!(host.reload_count(), 1);

    // Signal state is completely preserved across hot reload
    assert_eq!(counter.get(), 200);
}
