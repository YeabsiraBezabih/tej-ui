use tej_layout::{Dimension, FlexDirection, Insets, LayoutArena, LayoutEngine, LayoutStyle, Size};

#[test]
fn test_layout_allocation_and_hierarchy() {
    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(800.0),
        height: Dimension::Points(600.0),
        direction: FlexDirection::Column,
        ..Default::default()
    });

    let child1 = arena.allocate_node(LayoutStyle {
        height: Dimension::Points(100.0),
        ..Default::default()
    });
    let child2 = arena.allocate_node(LayoutStyle {
        height: Dimension::Points(200.0),
        ..Default::default()
    });

    arena.add_child(root, child1);
    arena.add_child(root, child2);

    assert_eq!(arena.len(), 3);
    assert_eq!(arena.children[root as usize].len(), 2);
}

#[test]
fn test_column_layout_computation() {
    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(400.0),
        height: Dimension::Points(600.0),
        direction: FlexDirection::Column,
        padding: Insets::all(20.0),
        spacing: 10.0,
        ..Default::default()
    });

    let header = arena.allocate_node(LayoutStyle {
        height: Dimension::Points(50.0),
        ..Default::default()
    });
    let body = arena.allocate_node(LayoutStyle {
        height: Dimension::Points(200.0),
        ..Default::default()
    });

    arena.add_child(root, header);
    arena.add_child(root, body);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(400.0, 600.0));

    let root_bounds = arena.computed_bounds[root as usize];
    assert_eq!(root_bounds.origin.x, 0.0);
    assert_eq!(root_bounds.origin.y, 0.0);
    assert_eq!(root_bounds.size.width, 400.0);
    assert_eq!(root_bounds.size.height, 600.0);

    let header_bounds = arena.computed_bounds[header as usize];
    assert_eq!(header_bounds.origin.x, 20.0);
    assert_eq!(header_bounds.origin.y, 20.0);
    assert_eq!(header_bounds.size.width, 360.0); // 400 - 40 padding
    assert_eq!(header_bounds.size.height, 50.0);

    let body_bounds = arena.computed_bounds[body as usize];
    assert_eq!(body_bounds.origin.x, 20.0);
    assert_eq!(body_bounds.origin.y, 80.0); // 20 padding + 50 header + 10 spacing
    assert_eq!(body_bounds.size.width, 360.0);
    assert_eq!(body_bounds.size.height, 200.0);
}

#[test]
fn test_row_layout_computation() {
    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(500.0),
        height: Dimension::Points(100.0),
        direction: FlexDirection::Row,
        padding: Insets::all(10.0),
        spacing: 20.0,
        ..Default::default()
    });

    let left = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(100.0),
        ..Default::default()
    });
    let right = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(200.0),
        ..Default::default()
    });

    arena.add_child(root, left);
    arena.add_child(root, right);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(500.0, 100.0));

    let left_bounds = arena.computed_bounds[left as usize];
    assert_eq!(left_bounds.origin.x, 10.0);
    assert_eq!(left_bounds.size.width, 100.0);

    let right_bounds = arena.computed_bounds[right as usize];
    assert_eq!(right_bounds.origin.x, 130.0); // 10 padding + 100 left + 20 spacing
    assert_eq!(right_bounds.size.width, 200.0);
}
