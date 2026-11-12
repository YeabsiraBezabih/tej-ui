use tej_hal::{
    Color, DirtyRectTracker, MonitorId, MonitorInfo, MonitorManager, MouseButton, PlatformWindow,
    UiEvent, WindowId, WindowRegistry,
};
use tej_layout::{Point, Rect, Size};

#[test]
fn test_dirty_rect_aggregation() {
    let mut tracker = DirtyRectTracker::new();
    assert!(!tracker.has_dirty());

    tracker.add_rect(Rect::new(10.0, 10.0, 50.0, 50.0));
    assert!(tracker.has_dirty());
    assert_eq!(tracker.rects().len(), 1);

    // Overlapping rect should merge
    tracker.add_rect(Rect::new(30.0, 30.0, 50.0, 50.0));
    assert_eq!(tracker.rects().len(), 1);
    let merged = tracker.rects()[0];
    assert_eq!(merged.origin.x, 10.0);
    assert_eq!(merged.origin.y, 10.0);
    assert_eq!(merged.size.width, 70.0);
    assert_eq!(merged.size.height, 70.0);

    tracker.reset();
    assert!(!tracker.has_dirty());
}

#[test]
fn test_ui_events_model() {
    let evt = UiEvent::MouseDown {
        button: MouseButton::Left,
        position: Point::new(100.0, 200.0),
    };

    if let UiEvent::MouseDown { button, position } = evt {
        assert_eq!(button, MouseButton::Left);
        assert_eq!(position.x, 100.0);
        assert_eq!(position.y, 200.0);
    } else {
        panic!("Event pattern mismatch");
    }
}

#[test]
fn test_color_constructors() {
    let c = Color::rgb(10, 20, 30);
    assert_eq!(c.r, 10);
    assert_eq!(c.g, 20);
    assert_eq!(c.b, 30);
    assert_eq!(c.a, 255);
}

#[derive(Debug)]
struct MockWindow {
    id: WindowId,
    title: String,
    size: Size,
}

impl PlatformWindow for MockWindow {
    fn id(&self) -> WindowId {
        self.id
    }
    fn size(&self) -> Size {
        self.size
    }
    fn request_redraw(&self) {}
    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }
    fn poll_events(&mut self) -> Vec<UiEvent> {
        Vec::new()
    }
}

#[test]
fn test_multi_monitor_and_window_registry() {
    // 1. Multi-monitor management
    let mut manager = MonitorManager::default();
    assert_eq!(manager.monitors().len(), 1);

    // Add secondary 4K monitor at 2.0x DPI scale factor
    manager.add_monitor(MonitorInfo::new(
        MonitorId(2),
        "Dell 4K External",
        Rect::from_origin_size(Point::new(1920.0, 0.0), Size::new(3840.0, 2160.0)),
        2.0,
        false,
    ));

    assert_eq!(manager.monitors().len(), 2);
    let secondary = manager.monitor_at_point(Point::new(2000.0, 500.0)).unwrap();
    assert_eq!(secondary.id, MonitorId(2));
    assert_eq!(secondary.scale_factor, 2.0);
    assert_eq!(secondary.physical_size(), Size::new(7680.0, 4320.0));

    // 2. Window Registry
    let mut registry = WindowRegistry::new();
    assert_eq!(registry.count(), 0);

    let win1 = Box::new(MockWindow {
        id: WindowId(1),
        title: "Main".to_string(),
        size: Size::new(800.0, 600.0),
    });
    let win2 = Box::new(MockWindow {
        id: WindowId(2),
        title: "Inspector".to_string(),
        size: Size::new(400.0, 600.0),
    });

    registry.register(win1);
    registry.register(win2);
    assert_eq!(registry.count(), 2);
    assert_eq!(registry.focused_window_id(), Some(WindowId(1)));

    registry.set_focused_window(WindowId(2));
    assert_eq!(registry.focused_window_id(), Some(WindowId(2)));

    let removed = registry.unregister(WindowId(1));
    assert!(removed.is_some());
    assert_eq!(registry.count(), 1);
}
