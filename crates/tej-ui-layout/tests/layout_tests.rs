use tej_ui_layout::{
    Dimension, FlexDirection, Insets, LayoutArena, LayoutEngine, LayoutStyle, Size,
};

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
fn test_nested_auto_height_intrinsic_measurement() {
    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(800.0),
        height: Dimension::Points(600.0),
        direction: FlexDirection::Column,
        spacing: 10.0,
        ..Default::default()
    });

    // Child container with Auto height containing 2 fixed children
    let child_stack = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(800.0),
        height: Dimension::Auto,
        direction: FlexDirection::Column,
        spacing: 5.0,
        padding: Insets::all(10.0),
        ..Default::default()
    });

    let item1 = arena.allocate_node(LayoutStyle {
        height: Dimension::Points(50.0),
        ..Default::default()
    });
    let item2 = arena.allocate_node(LayoutStyle {
        height: Dimension::Points(70.0),
        ..Default::default()
    });

    arena.add_child(child_stack, item1);
    arena.add_child(child_stack, item2);

    let second_child = arena.allocate_node(LayoutStyle {
        height: Dimension::Points(100.0),
        ..Default::default()
    });

    arena.add_child(root, child_stack);
    arena.add_child(root, second_child);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(800.0, 600.0));

    // child_stack intrinsic height = 10 padding top + 50 item1 + 5 spacing + 70 item2 + 10 padding btm = 145.0
    let child_bounds = arena.computed_bounds[child_stack as usize];
    assert_eq!(child_bounds.origin.y, 0.0);
    assert_eq!(child_bounds.size.height, 145.0);

    // second_child should start after child_stack + root spacing (145 + 10 = 155)
    let second_bounds = arena.computed_bounds[second_child as usize];
    assert_eq!(second_bounds.origin.y, 155.0);
    assert_eq!(second_bounds.size.height, 100.0);
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

#[test]
fn test_responsive_container_query_adaptation() {
    use tej_ui_layout::{Breakpoint, ContainerQuery};

    assert_eq!(Breakpoint::from_width(320.0), Breakpoint::Compact);
    assert_eq!(Breakpoint::from_width(600.0), Breakpoint::Medium);
    assert_eq!(Breakpoint::from_width(1200.0), Breakpoint::Expanded);

    let mut arena = LayoutArena::new();

    // Container with adaptive container query: Row when wide, switches to Column when narrow (< 400pt)
    let container = arena.allocate_node(LayoutStyle {
        width: Dimension::Percent(1.0),
        height: Dimension::Points(300.0),
        direction: FlexDirection::Row,
        container_query: Some(
            ContainerQuery::new()
                .max_width(400.0)
                .direction(FlexDirection::Column)
                .spacing(8.0),
        ),
        ..Default::default()
    });

    let child1 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(80.0),
        height: Dimension::Points(40.0),
        ..Default::default()
    });
    let child2 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(80.0),
        height: Dimension::Points(40.0),
        ..Default::default()
    });

    arena.add_child(container, child1);
    arena.add_child(container, child2);

    // 1. Layout with wide parent (800pt) -> should remain Row
    LayoutEngine::compute_layout(&mut arena, container, Size::new(800.0, 600.0));
    let c1_bounds = arena.computed_bounds[child1 as usize];
    let c2_bounds = arena.computed_bounds[child2 as usize];
    assert_eq!(c1_bounds.origin.y, 0.0);
    assert_eq!(c2_bounds.origin.y, 0.0);
    assert_eq!(c2_bounds.origin.x, 80.0); // side-by-side in row

    // 2. Layout with narrow parent (350pt) -> container query triggers Column layout
    LayoutEngine::compute_layout(&mut arena, container, Size::new(350.0, 600.0));
    let c1_narrow = arena.computed_bounds[child1 as usize];
    let c2_narrow = arena.computed_bounds[child2 as usize];
    assert_eq!(c1_narrow.origin.x, 0.0);
    assert_eq!(c2_narrow.origin.x, 0.0);
    assert_eq!(c2_narrow.origin.y, 48.0); // 40 height + 8 spacing stacked vertically
}

#[test]
fn test_rtl_row_layout_mirroring() {
    use tej_ui_layout::LayoutDirection;

    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(500.0),
        height: Dimension::Points(100.0),
        direction: FlexDirection::Row,
        layout_direction: LayoutDirection::Rtl, // Right-to-Left Arabic/Hebrew flow
        padding: Insets::all(10.0),
        spacing: 20.0,
        ..Default::default()
    });

    let first = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(100.0),
        ..Default::default()
    });
    let second = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(150.0),
        ..Default::default()
    });

    arena.add_child(root, first);
    arena.add_child(root, second);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(500.0, 100.0));

    // Inner width = 500 - 20 (padding) = 480. Inner origin.x = 10.
    // In RTL, the first child is placed at the rightmost edge of the inner box:
    // first.x = 10 + 480 - 0 - 100 = 390.0!
    let first_bounds = arena.computed_bounds[first as usize];
    assert_eq!(first_bounds.origin.x, 390.0);
    assert_eq!(first_bounds.size.width, 100.0);

    // Second child is placed to the left of the first (separated by spacing 20.0):
    // second.x = 10 + 480 - (100 + 20) - 150 = 220.0!
    let second_bounds = arena.computed_bounds[second as usize];
    assert_eq!(second_bounds.origin.x, 220.0);
    assert_eq!(second_bounds.size.width, 150.0);
}

#[test]
fn test_flex_wrap_multi_row_layout() {
    use tej_ui_layout::FlexWrap;

    let mut arena = LayoutArena::new();
    // Container width 200pt, 3 children of 80pt each. With FlexWrap::Wrap, the 3rd child must wrap to line 2.
    let container = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(200.0),
        height: Dimension::Points(100.0),
        direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        spacing: 10.0,
        ..Default::default()
    });

    let child1 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(80.0),
        height: Dimension::Points(30.0),
        ..Default::default()
    });
    let child2 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(80.0),
        height: Dimension::Points(30.0),
        ..Default::default()
    });
    let child3 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(80.0),
        height: Dimension::Points(30.0),
        ..Default::default()
    });

    arena.add_child(container, child1);
    arena.add_child(container, child2);
    arena.add_child(container, child3);

    LayoutEngine::compute_layout(&mut arena, container, Size::new(200.0, 100.0));

    let c1 = arena.computed_bounds[child1 as usize];
    let c2 = arena.computed_bounds[child2 as usize];
    let c3 = arena.computed_bounds[child3 as usize];

    assert_eq!(c1.origin.x, 0.0);
    assert_eq!(c1.origin.y, 0.0);
    assert_eq!(c2.origin.x, 90.0); // 80 + 10 spacing
    assert_eq!(c2.origin.y, 0.0);

    // Child 3 doesn't fit on row 1 (90 + 80 + 10 = 180 > 200 - 80? line_x 90 + 80 = 170. Next would be 170 + 10 = 180 + 80 = 260 > 200)
    // So Child 3 wraps to row 2!
    assert_eq!(c3.origin.x, 0.0);
    assert_eq!(c3.origin.y, 40.0); // 30 height + 10 spacing
}

#[test]
fn test_css_grid_12_columns_layout() {
    use tej_ui_layout::DisplayMode;

    let mut arena = LayoutArena::new();
    let grid = arena.allocate_node(LayoutStyle {
        display: DisplayMode::Grid,
        grid_columns: 12,
        grid_column_gap: 10.0,
        grid_row_gap: 15.0,
        width: Dimension::Points(1210.0),
        height: Dimension::Auto,
        ..Default::default()
    });

    let c1 = arena.allocate_node(LayoutStyle {
        col_span: 6,
        height: Dimension::Points(40.0),
        ..Default::default()
    });
    let c2 = arena.allocate_node(LayoutStyle {
        col_span: 6,
        height: Dimension::Points(40.0),
        ..Default::default()
    });
    let c3 = arena.allocate_node(LayoutStyle {
        col_span: 12,
        height: Dimension::Points(50.0),
        ..Default::default()
    });

    arena.add_child(grid, c1);
    arena.add_child(grid, c2);
    arena.add_child(grid, c3);

    LayoutEngine::compute_layout(&mut arena, grid, Size::new(1210.0, 500.0));

    let b1 = arena.computed_bounds[c1 as usize];
    let b2 = arena.computed_bounds[c2 as usize];
    let b3 = arena.computed_bounds[c3 as usize];

    assert_eq!(b1.origin.x, 0.0);
    assert_eq!(b1.origin.y, 0.0);
    assert!(b2.origin.x > b1.origin.x);
    assert_eq!(b2.origin.y, 0.0);

    // Row 2 wrap
    assert_eq!(b3.origin.x, 0.0);
    assert_eq!(b3.origin.y, 55.0); // 40.0 + 15.0 row_gap
    assert_eq!(b3.size.width, 1210.0);
}
