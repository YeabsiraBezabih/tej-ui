use tej_ui_layout::{
    AlignItems, ContainerQuery, Dimension, DisplayMode, FlexDirection, FlexWrap, Insets,
    JustifyContent, LayoutArena, LayoutEngine, LayoutStyle, Size,
};

#[test]
fn test_align_items_center_and_flex_end() {
    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(300.0),
        height: Dimension::Points(100.0),
        direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        ..Default::default()
    });
    let child = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(50.0),
        height: Dimension::Points(20.0),
        ..Default::default()
    });
    arena.add_child(root, child);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 100.0));
    assert_eq!(arena.computed_bounds[child as usize].origin.y, 40.0);

    arena.styles[root as usize].align_items = AlignItems::FlexEnd;
    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 100.0));
    assert_eq!(arena.computed_bounds[child as usize].origin.y, 80.0);
}

#[test]
fn test_justify_content_center_end_space_between_and_around() {
    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(200.0),
        height: Dimension::Points(50.0),
        direction: FlexDirection::Row,
        justify_content: JustifyContent::Center,
        ..Default::default()
    });
    let c1 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(40.0),
        height: Dimension::Points(20.0),
        ..Default::default()
    });
    let c2 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(40.0),
        height: Dimension::Points(20.0),
        ..Default::default()
    });
    arena.add_child(root, c1);
    arena.add_child(root, c2);

    // Center: free = 120, start = 60
    LayoutEngine::compute_layout(&mut arena, root, Size::new(200.0, 50.0));
    assert_eq!(arena.computed_bounds[c1 as usize].origin.x, 60.0);
    assert_eq!(arena.computed_bounds[c2 as usize].origin.x, 100.0);

    // FlexEnd: free = 120, start = 120
    arena.styles[root as usize].justify_content = JustifyContent::FlexEnd;
    LayoutEngine::compute_layout(&mut arena, root, Size::new(200.0, 50.0));
    assert_eq!(arena.computed_bounds[c1 as usize].origin.x, 120.0);
    assert_eq!(arena.computed_bounds[c2 as usize].origin.x, 160.0);

    // SpaceBetween: c1 at 0, c2 at 160
    arena.styles[root as usize].justify_content = JustifyContent::SpaceBetween;
    LayoutEngine::compute_layout(&mut arena, root, Size::new(200.0, 50.0));
    assert_eq!(arena.computed_bounds[c1 as usize].origin.x, 0.0);
    assert_eq!(arena.computed_bounds[c2 as usize].origin.x, 160.0);

    // SpaceAround: per_item = 60, c1 at 30, c2 at 30 + 40 + 60 = 130
    arena.styles[root as usize].justify_content = JustifyContent::SpaceAround;
    LayoutEngine::compute_layout(&mut arena, root, Size::new(200.0, 50.0));
    assert_eq!(arena.computed_bounds[c1 as usize].origin.x, 30.0);
    assert_eq!(arena.computed_bounds[c2 as usize].origin.x, 130.0);
}

#[test]
fn test_margin_applied_in_flex_layout() {
    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(300.0),
        height: Dimension::Points(100.0),
        direction: FlexDirection::Row,
        ..Default::default()
    });
    let c1 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(50.0),
        height: Dimension::Points(20.0),
        margin: Insets::new(10.0, 15.0, 10.0, 5.0),
        ..Default::default()
    });
    let c2 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(50.0),
        height: Dimension::Points(20.0),
        margin: Insets::new(0.0, 0.0, 0.0, 10.0),
        ..Default::default()
    });
    arena.add_child(root, c1);
    arena.add_child(root, c2);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 100.0));
    assert_eq!(arena.computed_bounds[c1 as usize].origin.x, 5.0);
    assert_eq!(arena.computed_bounds[c1 as usize].origin.y, 10.0);
    // c2 starts after c1's left(5) + width(50) + right(15) + c2's left(10) = 80
    assert_eq!(arena.computed_bounds[c2 as usize].origin.x, 80.0);
}

#[test]
fn test_grid_row_span_occupancy_and_height() {
    let mut arena = LayoutArena::new();
    let grid = arena.allocate_node(LayoutStyle {
        display: DisplayMode::Grid,
        grid_columns: 2,
        grid_column_gap: 10.0,
        grid_row_gap: 10.0,
        width: Dimension::Points(210.0),
        height: Dimension::Auto,
        ..Default::default()
    });

    let tall = arena.allocate_node(LayoutStyle {
        col_span: 1,
        row_span: 2,
        height: Dimension::Auto,
        ..Default::default()
    });
    let top_right = arena.allocate_node(LayoutStyle {
        col_span: 1,
        row_span: 1,
        height: Dimension::Points(40.0),
        ..Default::default()
    });
    let bottom_right = arena.allocate_node(LayoutStyle {
        col_span: 1,
        row_span: 1,
        height: Dimension::Points(50.0),
        ..Default::default()
    });

    arena.add_child(grid, tall);
    arena.add_child(grid, top_right);
    arena.add_child(grid, bottom_right);

    LayoutEngine::compute_layout(&mut arena, grid, Size::new(210.0, 300.0));

    let b_tall = arena.computed_bounds[tall as usize];
    let b_tr = arena.computed_bounds[top_right as usize];
    let b_br = arena.computed_bounds[bottom_right as usize];

    // tall spans rows 0 and 1 in col 0: height = 40 + 10 + 50 = 100
    assert_eq!(b_tall.origin.x, 0.0);
    assert_eq!(b_tall.origin.y, 0.0);
    assert_eq!(b_tall.size.height, 100.0);

    // top_right in col 1, row 0
    assert_eq!(b_tr.origin.x, 110.0);
    assert_eq!(b_tr.origin.y, 0.0);

    // bottom_right skips occupied (row 1, col 0) and lands in (row 1, col 1)
    assert_eq!(b_br.origin.x, 110.0);
    assert_eq!(b_br.origin.y, 50.0);
}

#[test]
fn test_auto_sibling_does_not_collapse_with_flex_grow() {
    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(300.0),
        height: Dimension::Points(100.0),
        direction: FlexDirection::Row,
        ..Default::default()
    });
    let auto_child = arena.allocate_node(LayoutStyle {
        width: Dimension::Auto,
        height: Dimension::Points(50.0),
        ..Default::default()
    });
    let grow_child = arena.allocate_node(LayoutStyle {
        width: Dimension::Auto,
        height: Dimension::Points(50.0),
        flex_grow: 2.0,
        ..Default::default()
    });
    arena.add_child(root, auto_child);
    arena.add_child(root, grow_child);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 100.0));

    let auto_w = arena.computed_bounds[auto_child as usize].size.width;
    let grow_w = arena.computed_bounds[grow_child as usize].size.width;
    assert!(auto_w > 0.0, "Auto sibling must not collapse to 0 width");
    assert!((auto_w - 100.0).abs() < 0.01);
    assert!((grow_w - 200.0).abs() < 0.01);
}

#[test]
fn test_column_flex_wrap_wraps_to_second_column() {
    let mut arena = LayoutArena::new();
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(300.0),
        height: Dimension::Points(100.0),
        direction: FlexDirection::Column,
        flex_wrap: FlexWrap::Wrap,
        spacing: 10.0,
        ..Default::default()
    });

    let c1 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(60.0),
        height: Dimension::Points(45.0),
        ..Default::default()
    });
    let c2 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(60.0),
        height: Dimension::Points(45.0),
        ..Default::default()
    });
    let c3 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(60.0),
        height: Dimension::Points(45.0),
        ..Default::default()
    });

    arena.add_child(root, c1);
    arena.add_child(root, c2);
    arena.add_child(root, c3);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 100.0));

    let b1 = arena.computed_bounds[c1 as usize];
    let b2 = arena.computed_bounds[c2 as usize];
    let b3 = arena.computed_bounds[c3 as usize];

    assert_eq!(b1.origin.x, 0.0);
    assert_eq!(b1.origin.y, 0.0);
    assert_eq!(b2.origin.x, 0.0);
    assert_eq!(b2.origin.y, 55.0);
    // c3 overflows height 100 -> wraps to column 2 at x = 60 + 10 = 70, y = 0
    assert_eq!(b3.origin.x, 70.0);
    assert_eq!(b3.origin.y, 0.0);
}

#[test]
fn test_storage_cycle_detection_and_reparenting() {
    let mut arena = LayoutArena::new();
    let a = arena.allocate_node(LayoutStyle::default());
    let b = arena.allocate_node(LayoutStyle::default());
    let c = arena.allocate_node(LayoutStyle::default());

    arena.add_child(a, b);
    arena.add_child(b, c);

    // Self-cycle rejected
    arena.add_child(a, a);
    assert_eq!(arena.parents[a as usize], None);

    // Ancestor cycle rejected (c -> a would form a -> b -> c -> a)
    arena.add_child(c, a);
    assert_eq!(arena.parents[a as usize], None);
    assert!(!arena.children[c as usize].contains(&a));

    // Reparenting c from b to a detaches c from b
    arena.add_child(a, c);
    assert_eq!(arena.parents[c as usize], Some(a));
    assert!(!arena.children[b as usize].contains(&c));
    assert!(arena.children[a as usize].contains(&c));

    // mark_dirty and path_from_root terminate cleanly
    arena.mark_dirty(c);
    let path = arena.path_from_root(c);
    assert_eq!(path.as_slice(), &[a, c]);
}

#[test]
fn test_container_query_no_dead_zone_or_overlap() {
    let compact = ContainerQuery::compact();
    let medium = ContainerQuery::medium();
    let expanded = ContainerQuery::expanded();

    // Check previously broken dead-zone / overlap widths
    for &w in &[0.0, 479.9, 479.95, 479.999] {
        assert!(compact.matches(w, 100.0), "width {w} should match compact");
        assert!(!medium.matches(w, 100.0), "width {w} must not match medium");
        assert!(
            !expanded.matches(w, 100.0),
            "width {w} must not match expanded"
        );
    }

    for &w in &[480.0, 600.0, 839.9, 839.95, 839.999] {
        assert!(
            !compact.matches(w, 100.0),
            "width {w} must not match compact"
        );
        assert!(medium.matches(w, 100.0), "width {w} should match medium");
        assert!(
            !expanded.matches(w, 100.0),
            "width {w} must not match expanded"
        );
    }

    for &w in &[840.0, 1280.0] {
        assert!(
            !compact.matches(w, 100.0),
            "width {w} must not match compact"
        );
        assert!(!medium.matches(w, 100.0), "width {w} must not match medium");
        assert!(
            expanded.matches(w, 100.0),
            "width {w} should match expanded"
        );
    }
}
