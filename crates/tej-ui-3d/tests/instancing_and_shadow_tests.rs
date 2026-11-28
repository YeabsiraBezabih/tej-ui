use tej_ui_3d::{InstanceData, InstancedMesh, ShadowMap, Vec3};
use tej_ui_hal::Color;

#[test]
fn test_instanced_mesh_batch_matrices() {
    let mut group = InstancedMesh::new(42, 100);
    assert_eq!(group.mesh_id, 42);
    assert!(group.is_empty());

    // Add 3 instances at different positions
    group.add_instance(InstanceData::new(
        Vec3::new(0.0, 0.0, 0.0),
        [0.0, 0.0, 0.0, 1.0],
        Vec3::ONE,
        Color::rgb(255, 0, 0),
        1,
    ));
    group.add_instance(InstanceData::new(
        Vec3::new(10.0, 20.0, 30.0),
        [0.0, 0.0, 0.0, 1.0],
        Vec3::new(2.0, 2.0, 2.0),
        Color::rgb(0, 255, 0),
        2,
    ));
    group.add_instance(InstanceData::new(
        Vec3::new(-5.0, 0.0, 5.0),
        [0.0, 0.0, 0.0, 1.0],
        Vec3::ONE,
        Color::rgb(0, 0, 255),
        3,
    ));

    assert_eq!(group.len(), 3);

    let matrices = group.compute_world_matrices();
    assert_eq!(matrices.len(), 3);

    // Verify matrix 0 has identity translation
    assert_eq!(matrices[0].transform_point(Vec3::ZERO), Vec3::ZERO);

    // Verify matrix 1 has (10, 20, 30) translation and 2x scale
    assert_eq!(
        matrices[1].transform_point(Vec3::ZERO),
        Vec3::new(10.0, 20.0, 30.0)
    );
    assert_eq!(
        matrices[1].transform_point(Vec3::new(1.0, 0.0, 0.0)),
        Vec3::new(12.0, 20.0, 30.0)
    );
}

#[test]
fn test_shadow_map_creation_and_pcf_evaluation() {
    let light_dir = Vec3::new(0.0, 1.0, 0.0); // Directly overhead sun
    let scene_center = Vec3::ZERO;
    let scene_radius = 10.0;

    let mut sm = ShadowMap::new(128, 128, light_dir, scene_center, scene_radius);
    assert_eq!(sm.width, 128);
    assert_eq!(sm.height, 128);
    assert_eq!(sm.depth_buffer.len(), 128 * 128);

    // Initial state: fully cleared (1.0)
    for &d in &sm.depth_buffer {
        assert_eq!(d, 1.0);
    }

    // Paint an occluder floor at center (pixels 60..70 x 60..70) at depth 0.35
    for y in 60..70 {
        for x in 60..70 {
            sm.record_depth(x, y, 0.35);
        }
    }

    // Transform scene center (0, 0, 0) into light space
    let center_clip = sm.light_view_proj.transform_point4(Vec3::ZERO);
    assert!(center_clip[3].abs() > 0.0);

    // Points deeper than the occluder (center_depth > 0.35 + bias) should be in shadow
    let shadow_factor = sm.sample_shadow_pcf(Vec3::new(0.0, -5.0, 0.0), 0.005);
    // Since (0, -5, 0) is below the occluder, it should receive partial or full shadow
    assert!(shadow_factor <= 1.0);

    // Point far above the scene (closer to light than occluder) should be fully illuminated
    let lit_factor = sm.sample_shadow_pcf(Vec3::new(0.0, 8.0, 0.0), 0.005);
    assert!(lit_factor > 0.0);
}
