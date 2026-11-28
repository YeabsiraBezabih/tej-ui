use tej_ui_layout::{
    AlignItems, AlignSelf, Dimension, DisplayMode, FlexDirection, Insets, LayoutArena,
    LayoutEngine, LayoutStyle, Overflow, Rect, Size,
};

#[test]
fn test_nested_styled_vstack_cards_no_collapse_and_zero_overlap() {
    // Outer Column -> two Row padded cards -> each containing a Column with flex_grow: 1.0 and children.
    // Previously, measure_intrinsic_extent returned None for the inner Column because of flex_grow > 0.0,
    // collapsing the cards to ~24-32px and causing severe vertical overlap.
    let mut arena = LayoutArena::new();

    let outer = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(600.0),
        height: Dimension::Points(800.0),
        direction: FlexDirection::Column,
        spacing: 16.0,
        padding: Insets::all(20.0),
        ..Default::default()
    });

    let build_card = |arena: &mut LayoutArena| -> u32 {
        let card = arena.allocate_node(LayoutStyle {
            width: Dimension::Auto,
            height: Dimension::Auto,
            direction: FlexDirection::Row,
            padding: Insets::all(16.0),
            border: Insets::all(1.0),
            ..Default::default()
        });

        let inner_col = arena.allocate_node(LayoutStyle {
            width: Dimension::Auto,
            height: Dimension::Auto,
            direction: FlexDirection::Column,
            flex_grow: 1.0,
            spacing: 8.0,
            ..Default::default()
        });

        let title = arena.allocate_node(LayoutStyle {
            height: Dimension::Points(28.0),
            ..Default::default()
        });
        let subtitle = arena.allocate_node(LayoutStyle {
            height: Dimension::Points(24.0),
            ..Default::default()
        });
        let body = arena.allocate_node(LayoutStyle {
            height: Dimension::Points(40.0),
            ..Default::default()
        });

        arena.add_child(inner_col, title);
        arena.add_child(inner_col, subtitle);
        arena.add_child(inner_col, body);
        arena.add_child(card, inner_col);
        card
    };

    let card1 = build_card(&mut arena);
    let card2 = build_card(&mut arena);
    arena.add_child(outer, card1);
    arena.add_child(outer, card2);

    LayoutEngine::compute_layout(&mut arena, outer, Size::new(600.0, 800.0));

    let b1 = arena.computed_bounds[card1 as usize];
    let b2 = arena.computed_bounds[card2 as usize];

    // Expected card height: 16*2 padding + 1*2 border + (28 + 8 + 24 + 8 + 40) = 34 + 108 = 142.0
    assert!(
        b1.size.height > 80.0,
        "card1 collapsed to {}px; expected full content height",
        b1.size.height
    );
    assert!(
        b2.size.height > 80.0,
        "card2 collapsed to {}px; expected full content height",
        b2.size.height
    );
    assert!((b1.size.height - 142.0).abs() < 0.01);
    assert!((b2.size.height - 142.0).abs() < 0.01);
    assert!(
        b2.origin.y >= b1.origin.y + b1.size.height,
        "card2 (y={}) overlapped card1 (bottom={})",
        b2.origin.y,
        b1.origin.y + b1.size.height
    );
    assert!((b2.origin.y - (b1.origin.y + b1.size.height + 16.0)).abs() < 0.01);
}

#[test]
fn test_hstack_two_columns_flex_shrink_prevents_right_edge_overflow() {
    // 2-column HStack in 1360px width where columns have large intrinsic widths (900px + 900px = 1800px).
    let mut arena = LayoutArena::new();

    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(1360.0),
        height: Dimension::Points(720.0),
        direction: FlexDirection::Row,
        spacing: 20.0,
        ..Default::default()
    });

    let col1 = arena.allocate_node(LayoutStyle {
        width: Dimension::Auto,
        height: Dimension::Auto,
        direction: FlexDirection::Column,
        ..Default::default()
    });
    let col1_content = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(900.0),
        height: Dimension::Points(200.0),
        ..Default::default()
    });
    arena.add_child(col1, col1_content);

    let col2 = arena.allocate_node(LayoutStyle {
        width: Dimension::Auto,
        height: Dimension::Auto,
        direction: FlexDirection::Column,
        ..Default::default()
    });
    let col2_content = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(900.0),
        height: Dimension::Points(200.0),
        ..Default::default()
    });
    arena.add_child(col2, col2_content);

    arena.add_child(root, col1);
    arena.add_child(root, col2);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(1360.0, 720.0));

    let b1 = arena.computed_bounds[col1 as usize];
    let b2 = arena.computed_bounds[col2 as usize];

    // Available width after 20px spacing = 1340px -> each equal column shrinks to 670px
    assert!(b1.origin.x >= 0.0);
    assert!(b2.origin.x >= b1.max_x());
    assert!(
        b1.max_x() <= 1360.0 + 0.01,
        "col1 overflowed right edge: {}",
        b1.max_x()
    );
    assert!(
        b2.max_x() <= 1360.0 + 0.01,
        "col2 overflowed right edge: {}",
        b2.max_x()
    );
    assert!((b1.size.width - 670.0).abs() < 0.01);
    assert!((b2.size.width - 670.0).abs() < 0.01);
    assert!((b2.max_x() - 1360.0).abs() < 0.01);
}

#[test]
fn test_flex_shrink_respects_min_width_and_zero_shrink() {
    let mut arena = LayoutArena::new();

    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(500.0),
        height: Dimension::Points(100.0),
        direction: FlexDirection::Row,
        ..Default::default()
    });

    // Fixed non-shrinking sidebar (flex_shrink: 0.0)
    let sidebar = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(200.0),
        height: Dimension::Points(100.0),
        flex_shrink: 0.0,
        ..Default::default()
    });
    // Panel with min_width: 250.0 (wants 400.0, shrinks down to floor 250.0)
    let main_panel = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(400.0),
        min_width: Dimension::Points(250.0),
        height: Dimension::Points(100.0),
        ..Default::default()
    });
    // Flexible panel (wants 200.0, absorbs remaining overflow -> 50.0)
    let side_panel = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(200.0),
        height: Dimension::Points(100.0),
        ..Default::default()
    });

    arena.add_child(root, sidebar);
    arena.add_child(root, main_panel);
    arena.add_child(root, side_panel);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(500.0, 100.0));

    let bs = arena.computed_bounds[sidebar as usize];
    let bm = arena.computed_bounds[main_panel as usize];
    let bp = arena.computed_bounds[side_panel as usize];

    assert!((bs.size.width - 200.0).abs() < 0.01);
    assert!((bm.size.width - 250.0).abs() < 0.01);
    assert!((bp.size.width - 50.0).abs() < 0.01);
    assert!((bp.max_x() - 500.0).abs() < 0.01);
}

#[test]
fn test_display_none_steals_zero_space_and_zero_spacing() {
    let mut arena = LayoutArena::new();

    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(400.0),
        height: Dimension::Points(100.0),
        direction: FlexDirection::Row,
        spacing: 20.0,
        ..Default::default()
    });

    let c1 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(100.0),
        height: Dimension::Points(40.0),
        ..Default::default()
    });
    let hidden = arena.allocate_node(LayoutStyle {
        display: DisplayMode::None,
        width: Dimension::Points(150.0),
        height: Dimension::Points(40.0),
        ..Default::default()
    });
    let hidden_child = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(80.0),
        height: Dimension::Points(20.0),
        ..Default::default()
    });
    arena.add_child(hidden, hidden_child);

    let c2 = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(100.0),
        height: Dimension::Points(40.0),
        ..Default::default()
    });

    arena.add_child(root, c1);
    arena.add_child(root, hidden);
    arena.add_child(root, c2);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(400.0, 100.0));

    let b1 = arena.computed_bounds[c1 as usize];
    let bh = arena.computed_bounds[hidden as usize];
    let bhc = arena.computed_bounds[hidden_child as usize];
    let b2 = arena.computed_bounds[c2 as usize];

    assert_eq!(bh, Rect::ZERO);
    assert_eq!(bhc, Rect::ZERO);
    assert_eq!(b1.origin.x, 0.0);
    // Only ONE spacing gap (20px) between c1 and c2, not two!
    assert_eq!(b2.origin.x, 120.0);
    assert_eq!(b2.size.width, 100.0);
}

#[test]
fn test_min_max_clamping_and_dimension_fill() {
    let mut arena = LayoutArena::new();

    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(500.0),
        height: Dimension::Points(200.0),
        direction: FlexDirection::Row,
        padding: Insets::all(10.0),
        border: Insets::all(5.0),
        spacing: 10.0,
        ..Default::default()
    });
    // inner_width = 500 - 20 - 10 = 470, inner_height = 200 - 20 - 10 = 170

    // Clamped to max_width = 120.0
    let clamped_max = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(250.0),
        max_width: Dimension::Points(120.0),
        height: Dimension::Fill,
        ..Default::default()
    });
    // Clamped to min_width = 80.0
    let clamped_min = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(30.0),
        min_width: Dimension::Points(80.0),
        height: Dimension::Points(50.0),
        ..Default::default()
    });
    // Dimension::Fill takes remaining main width: 470 - 120 - 80 - 20 (two gaps) = 250.0
    let filler = arena.allocate_node(LayoutStyle {
        width: Dimension::Fill,
        height: Dimension::Fill,
        overflow_x: Overflow::Hidden,
        overflow_y: Overflow::Scroll,
        ..Default::default()
    });

    arena.add_child(root, clamped_max);
    arena.add_child(root, clamped_min);
    arena.add_child(root, filler);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(500.0, 200.0));

    let b_max = arena.computed_bounds[clamped_max as usize];
    let b_min = arena.computed_bounds[clamped_min as usize];
    let b_fill = arena.computed_bounds[filler as usize];

    assert_eq!(b_max.origin.x, 15.0);
    assert_eq!(b_max.origin.y, 15.0);
    assert!((b_max.size.width - 120.0).abs() < 0.01);
    assert!((b_max.size.height - 170.0).abs() < 0.01);

    assert!((b_min.size.width - 80.0).abs() < 0.01);
    assert!((b_min.size.height - 50.0).abs() < 0.01);

    assert!((b_fill.size.width - 250.0).abs() < 0.01);
    assert!((b_fill.size.height - 170.0).abs() < 0.01);
}

#[test]
fn test_nested_percent_and_border_box_sizing() {
    let mut arena = LayoutArena::new();

    // Outer 420x220 with padding=6, border=4 => inner 400x200
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(420.0),
        height: Dimension::Points(220.0),
        padding: Insets::all(6.0),
        border: Insets::all(4.0),
        direction: FlexDirection::Row,
        ..Default::default()
    });

    // Child is 50% of inner (200x100) — must NOT double-multiply percent!
    let child = arena.allocate_node(LayoutStyle {
        width: Dimension::Percent(0.5),
        height: Dimension::Percent(0.5),
        padding: Insets::all(10.0),
        direction: FlexDirection::Row,
        ..Default::default()
    });

    // Grandchild is 50% of child's inner (200 - 20 = 180 => 90, 100 - 20 = 80 => 40)
    let grandchild = arena.allocate_node(LayoutStyle {
        width: Dimension::Percent(0.5),
        height: Dimension::Percent(0.5),
        ..Default::default()
    });

    arena.add_child(child, grandchild);
    arena.add_child(root, child);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(420.0, 220.0));

    let bc = arena.computed_bounds[child as usize];
    let bg = arena.computed_bounds[grandchild as usize];

    assert!((bc.size.width - 200.0).abs() < 0.01);
    assert!((bc.size.height - 100.0).abs() < 0.01);
    assert!((bg.size.width - 90.0).abs() < 0.01);
    assert!((bg.size.height - 40.0).abs() < 0.01);
}

#[test]
fn test_cross_axis_center_and_align_self_measure_intrinsic_size() {
    let mut arena = LayoutArena::new();

    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(300.0),
        height: Dimension::Points(120.0),
        direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        ..Default::default()
    });

    // Auto-height container with 30px tall child should center at y = (120 - 30) / 2 = 45.0
    let centered_box = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(80.0),
        height: Dimension::Auto,
        direction: FlexDirection::Column,
        ..Default::default()
    });
    let centered_inner = arena.allocate_node(LayoutStyle {
        height: Dimension::Points(30.0),
        ..Default::default()
    });
    arena.add_child(centered_box, centered_inner);

    // AlignSelf::FlexEnd overrides AlignItems::Center: 20px tall child sits at y = 120 - 20 = 100.0
    let end_box = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(80.0),
        height: Dimension::Auto,
        align_self: AlignSelf::FlexEnd,
        direction: FlexDirection::Column,
        ..Default::default()
    });
    let end_inner = arena.allocate_node(LayoutStyle {
        height: Dimension::Points(20.0),
        ..Default::default()
    });
    arena.add_child(end_box, end_inner);

    arena.add_child(root, centered_box);
    arena.add_child(root, end_box);

    LayoutEngine::compute_layout(&mut arena, root, Size::new(300.0, 120.0));

    let bc = arena.computed_bounds[centered_box as usize];
    let be = arena.computed_bounds[end_box as usize];

    assert!((bc.size.height - 30.0).abs() < 0.01);
    assert!((bc.origin.y - 45.0).abs() < 0.01);

    assert!((be.size.height - 20.0).abs() < 0.01);
    assert!((be.origin.y - 100.0).abs() < 0.01);
}
