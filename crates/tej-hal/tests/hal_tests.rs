use tej_hal::{Color, DirtyRectTracker, MouseButton, UiEvent};
use tej_layout::{Point, Rect};

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
