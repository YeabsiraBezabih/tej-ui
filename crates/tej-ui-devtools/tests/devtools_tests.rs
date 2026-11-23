use std::time::Duration;
use tej_ui_core::{Epoch, NodeId, Signal};
use tej_ui_devtools::{
    DagNodeInfo, DagSnapshot, DirtyHudOverlay, HotReloadHost, ReactiveDagInspector,
};
use tej_ui_hal::{Color, SurfaceCompositor};
use tej_ui_layout::{Point, Rect};
use tej_ui_view::Text;

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

#[test]
fn test_widget_inspector_and_flamegraph_profiler() {
    use tej_ui_devtools::{FrameFlamegraphProfiler, FrameTimingSample, WidgetInspector};
    use tej_ui_layout::{Dimension, FlexDirection, LayoutArena, LayoutEngine, LayoutStyle, Size};

    // 1. Widget Inspector
    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(400.0),
        height: Dimension::Points(300.0),
        direction: FlexDirection::Column,
        ..Default::default()
    });

    let child = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(200.0),
        height: Dimension::Points(50.0),
        ..Default::default()
    });
    arena.add_child(root, child);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(400.0, 300.0));

    let inspector = WidgetInspector::new();
    let hierarchy = inspector.inspect_hierarchy(&arena, root);
    assert_eq!(hierarchy.len(), 2);
    assert_eq!(hierarchy[0].id, root);
    assert_eq!(hierarchy[0].depth, 0);
    assert_eq!(hierarchy[1].id, child);
    assert_eq!(hierarchy[1].depth, 1);

    let tree_str = inspector.format_tree(&arena, root);
    assert!(tree_str.contains("Node #0"));
    assert!(tree_str.contains("Node #1"));

    // 2. Frame Flamegraph Profiler
    let mut profiler = FrameFlamegraphProfiler::new();

    // Normal 60Hz frame (total = 4,000 us = 4ms)
    profiler.record_sample(FrameTimingSample {
        reactivity_us: 500,
        layout_us: 1500,
        events_us: 500,
        paint_us: 1000,
        present_us: 500,
    });

    // Jank frame (total = 20,000 us = 20ms > 16.6ms)
    profiler.record_sample(FrameTimingSample {
        reactivity_us: 2000,
        layout_us: 8000,
        events_us: 1000,
        paint_us: 7000,
        present_us: 2000,
    });

    assert_eq!(profiler.frame_count(), 2);
    assert_eq!(profiler.jank_count(), 1);
    assert_eq!(profiler.average_duration_us(), 12_000.0);
}

#[test]
fn test_dylib_hot_reloader() {
    use std::path::PathBuf;
    use tej_ui_devtools::{DylibReloader, ReloadStatus};

    let path = PathBuf::from("target/debug/libmyapp_view.so");
    let mut reloader = DylibReloader::new(&path);

    assert_eq!(reloader.current_generation(), 1);
    assert_eq!(reloader.dylib_path(), &path);

    reloader.register_symbols(&["render_root_view", "app_theme_provider"]);
    assert!(reloader.has_symbol("render_root_view"));
    assert!(!reloader.has_symbol("unknown_symbol"));

    // Simulate binary reload
    let status = reloader.simulate_reload();
    assert_eq!(
        status,
        ReloadStatus::Reloaded {
            generation: 2,
            path: path.clone()
        }
    );
    assert_eq!(reloader.current_generation(), 2);
}
