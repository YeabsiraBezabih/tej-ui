use std::f32::consts::FRAC_PI_2;
use tej_ui_3d::{GLB_MAGIC, GLB_VERSION_2, GlbError, GlbPackage, NO_PARENT, SceneGraphSoA, Vec3};

#[test]
fn test_soa_scene_graph_hierarchy_and_dirty_propagation() {
    let mut graph = SceneGraphSoA::new();
    assert!(graph.is_empty());

    // Root node at (10, 0, 0)
    let root = graph.add_node(
        "RootCar",
        None,
        Vec3::new(10.0, 0.0, 0.0),
        [0.0, 0.0, 0.0, 1.0], // Identity quat
        Vec3::ONE,
    );
    assert_eq!(graph.parent_ids[root as usize], NO_PARENT);

    // Child node attached to root at local (0, 5, 0)
    let wheel = graph.add_node(
        "WheelFrontLeft",
        Some(root),
        Vec3::new(0.0, 5.0, 0.0),
        [0.0, 0.0, 0.0, 1.0],
        Vec3::ONE,
    );
    assert_eq!(graph.parent_ids[wheel as usize], root);

    // Initial world transforms update
    graph.update_world_transforms();

    let root_world = graph.world_matrix(root).unwrap();
    let root_pos = root_world.transform_point(Vec3::ZERO);
    assert_eq!(root_pos, Vec3::new(10.0, 0.0, 0.0));

    let wheel_world = graph.world_matrix(wheel).unwrap();
    let wheel_pos = wheel_world.transform_point(Vec3::ZERO);
    assert_eq!(wheel_pos, Vec3::new(10.0, 5.0, 0.0));

    // Move root to (20, 0, 0) -> wheel must automatically update to (20, 5, 0)
    graph.set_local_transform(
        root,
        Vec3::new(20.0, 0.0, 0.0),
        [0.0, 0.0, 0.0, 1.0],
        Vec3::ONE,
    );
    graph.update_world_transforms();

    let updated_wheel = graph
        .world_matrix(wheel)
        .unwrap()
        .transform_point(Vec3::ZERO);
    assert_eq!(updated_wheel, Vec3::new(20.0, 5.0, 0.0));
}

#[test]
fn test_soa_quaternion_rotation() {
    let mut graph = SceneGraphSoA::new();

    // 90-degree rotation around Y axis: q = [0, sin(45°), 0, cos(45°)]
    let sin45 = (FRAC_PI_2 * 0.5).sin();
    let cos45 = (FRAC_PI_2 * 0.5).cos();
    let q_rot_y = [0.0, sin45, 0.0, cos45];

    let node = graph.add_node("RotatedNode", None, Vec3::ZERO, q_rot_y, Vec3::ONE);
    graph.update_world_transforms();

    let mat = graph.world_matrix(node).unwrap();
    // Rotating (1, 0, 0) by +90 deg around Y gives (0, 0, -1) in right-handed coords
    let p_trans = mat.transform_point(Vec3::new(1.0, 0.0, 0.0));
    assert!((p_trans.x).abs() < 1e-5);
    assert!((p_trans.y).abs() < 1e-5);
    assert!((p_trans.z - (-1.0)).abs() < 1e-5);
}

#[test]
fn test_glb_package_builder_and_zero_copy_parsing() {
    let json_text = r#"{"asset":{"version":"2.0"},"scene":0}"#;
    let mut bin_data = Vec::new();

    // 3 Vertices: (0,0,0), (1,0,0), (0,1,0)
    for p in &[
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    ] {
        bin_data.extend_from_slice(&p.x.to_le_bytes());
        bin_data.extend_from_slice(&p.y.to_le_bytes());
        bin_data.extend_from_slice(&p.z.to_le_bytes());
    }

    // 3 Indices (u16): 0, 1, 2
    for &idx in &[0_u16, 1, 2] {
        bin_data.extend_from_slice(&idx.to_le_bytes());
    }

    let glb_bytes = GlbPackage::build_glb(json_text, &bin_data);
    assert!(glb_bytes.len() >= 12);

    let pkg = GlbPackage::parse(&glb_bytes).expect("Valid GLB must parse cleanly");
    assert_eq!(pkg.header.magic, GLB_MAGIC);
    assert_eq!(pkg.header.version, GLB_VERSION_2);
    assert_eq!(pkg.json_text().unwrap().trim(), json_text);

    // Extract positions
    let positions = pkg.extract_positions(0, 3).expect("Positions must extract");
    assert_eq!(positions.len(), 3);
    assert_eq!(positions[1], Vec3::new(1.0, 0.0, 0.0));

    // Extract indices (offset 36 bytes: 3 * 12)
    let indices = pkg
        .extract_indices_u16(36, 3)
        .expect("Indices must extract");
    assert_eq!(indices, vec![0, 1, 2]);
}

#[test]
fn test_glb_sanitization_and_error_handling() {
    // 1. Buffer too short
    let short = [0u8; 8];
    assert_eq!(
        GlbPackage::parse(&short).unwrap_err(),
        GlbError::BufferTooShort
    );

    // 2. Corrupt magic
    let bad_magic = [0u8; 16];
    assert!(matches!(
        GlbPackage::parse(&bad_magic).unwrap_err(),
        GlbError::InvalidMagic(_)
    ));

    // 3. Buffer overrun
    let mut fake_glb = vec![0u8; 24];
    fake_glb[..4].copy_from_slice(&GLB_MAGIC.to_le_bytes());
    fake_glb[4..8].copy_from_slice(&GLB_VERSION_2.to_le_bytes());
    fake_glb[8..12].copy_from_slice(&1000_u32.to_le_bytes()); // Claims 1000 bytes
    assert!(matches!(
        GlbPackage::parse(&fake_glb).unwrap_err(),
        GlbError::BufferOverrun { .. }
    ));
}
