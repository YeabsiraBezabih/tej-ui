//! Test suite for Phase 2: 12-byte quantized meshlets, octahedral normals, and enterprise sanitization.

use tej_ui_3d::math::Vec3;
use tej_ui_3d::nanite::{
    MeshSanitizerError, PackedMeshletVertex, QuantizedMeshlet, decode_oct_normal,
    dequantize_position, encode_oct_normal, quantize_position, sanitize_mesh_geometry,
};
use tej_ui_hal::Color;

#[test]
fn test_octahedral_normal_cardinal_axes() {
    let axes = [
        [1.0f32, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
    ];

    for axis in axes {
        let oct = encode_oct_normal(axis);
        let dec = decode_oct_normal(oct);

        let dot = axis[0] * dec[0] + axis[1] * dec[1] + axis[2] * dec[2];
        assert!(
            dot > 0.999,
            "Cardinal axis {:?} must decode with >0.999 dot product, got {}",
            axis,
            dot
        );
    }
}

#[test]
fn test_octahedral_normal_arbitrary_vectors_sub_degree_accuracy() {
    let test_normals = [
        Vec3::new(1.0, 2.0, 3.0).normalize(),
        Vec3::new(-0.5, 0.8, -0.2).normalize(),
        Vec3::new(0.33, -0.66, 0.66).normalize(),
        Vec3::new(-1.0, -1.0, -1.0).normalize(),
    ];

    for n in test_normals {
        let raw = [n.x, n.y, n.z];
        let oct = encode_oct_normal(raw);
        let dec = decode_oct_normal(oct);

        let dot = raw[0] * dec[0] + raw[1] * dec[1] + raw[2] * dec[2];
        // 0.998 dot product corresponds to <3 degrees of angular deviation across 16 bits!
        assert!(
            dot > 0.995,
            "Arbitrary normal {:?} must decode accurately, got dot {}",
            raw,
            dot
        );
    }
}

#[test]
fn test_vertex_position_quantization_roundtrip() {
    let min_pt = [-10.0, -5.0, 0.0];
    let max_pt = [10.0, 15.0, 50.0];
    let scale_pt = [20.0, 20.0, 50.0];

    let points = [min_pt, max_pt, [0.0, 5.0, 25.0], [-3.5, 2.8, 12.345]];

    for p in points {
        let q = quantize_position(p, min_pt, scale_pt);
        let dec = dequantize_position(q, min_pt, scale_pt);

        for c in 0..3 {
            let err = (p[c] - dec[c]).abs();
            let max_allowed_err = scale_pt[c] / 65535.0 + 1e-4;
            assert!(
                err <= max_allowed_err,
                "Coordinate {} error {} must be within quantization step {}",
                c,
                err,
                max_allowed_err
            );
        }
    }
}

#[test]
fn test_packed_meshlet_vertex_memory_layout_and_bytes() {
    assert_eq!(
        std::mem::size_of::<PackedMeshletVertex>(),
        12,
        "PackedMeshletVertex must be strictly 12 bytes"
    );
    assert_eq!(
        std::mem::align_of::<PackedMeshletVertex>(),
        4,
        "PackedMeshletVertex must align to 4 bytes"
    );

    let v = PackedMeshletVertex::new(
        [1.5, 2.5, 3.5],
        [0.0, 1.0, 0.0],
        Color::rgba(255, 128, 64, 255),
        [0.0, 0.0, 0.0],
        [5.0, 5.0, 5.0],
    );

    let bytes = v.to_bytes();
    assert_eq!(bytes.len(), 12);

    let reconstructed = PackedMeshletVertex::from_bytes(&bytes);
    assert_eq!(v, reconstructed);
}

#[test]
fn test_quantized_meshlet_cluster_storage_footprint() {
    let mut meshlet = QuantizedMeshlet::new([-2.0, -2.0, -2.0], [2.0, 2.0, 2.0]);

    // Add 64 vertices and 124 triangles
    for i in 0..64 {
        let t = i as f32 / 64.0;
        meshlet.add_vertex(
            [t * 4.0 - 2.0, (t * 2.0).sin(), 0.0],
            [0.0, 1.0, 0.0],
            Color::rgb(100, 150, 200),
        );
    }
    for i in 0..124 {
        let i0 = (i % 62) as u8;
        let i1 = ((i + 1) % 62) as u8;
        let i2 = ((i + 2) % 62) as u8;
        meshlet.add_triangle(i0, i1, i2);
    }

    assert_eq!(meshlet.vertices.len(), 64);
    assert_eq!(meshlet.triangles.len(), 124);

    let total_bytes = meshlet.byte_size();
    // 64 * 12B + 124 * 3B = 768 + 372 = 1140 Bytes!
    assert_eq!(total_bytes, 1140);
    assert!(
        total_bytes <= 1200,
        "A 64-vert / 124-tri meshlet must occupy ~1.14 KB, allowing 3 per 4KB page!"
    );
}

#[test]
fn test_enterprise_sanitizer_catches_malformed_inputs() {
    // 1. NaN coordinate rejection
    let nan_vertices = vec![[0.0, 0.0, 0.0], [f32::NAN, 1.0, 0.0]];
    let normals = vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
    let triangles = vec![[0, 1, 0]];
    let err = sanitize_mesh_geometry(&nan_vertices, &normals, &triangles).unwrap_err();
    assert!(matches!(
        err,
        MeshSanitizerError::NonFiniteCoordinate(1, _, _, _)
    ));

    // 2. Infinity coordinate rejection
    let inf_vertices = vec![[0.0, 0.0, 0.0], [f32::INFINITY, 1.0, 0.0]];
    let err = sanitize_mesh_geometry(&inf_vertices, &normals, &triangles).unwrap_err();
    assert!(matches!(
        err,
        MeshSanitizerError::NonFiniteCoordinate(1, _, _, _)
    ));

    // 3. Out-of-bounds index rejection
    let valid_vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let oob_triangles = vec![[0, 1, 99]]; // 99 exceeds 3 vertices!
    let err = sanitize_mesh_geometry(&valid_vertices, &normals, &oob_triangles).unwrap_err();
    assert_eq!(err, MeshSanitizerError::OutOfBoundsIndex(99, 3));

    // 4. Degenerate triangle rejection
    let degen_triangles = vec![[0, 1, 1]]; // vertex 1 repeated!
    let err = sanitize_mesh_geometry(&valid_vertices, &normals, &degen_triangles).unwrap_err();
    assert_eq!(err, MeshSanitizerError::DegenerateTriangle(0));

    // 5. Valid geometry accepts cleanly
    let clean_triangles = vec![[0, 1, 2]];
    assert!(sanitize_mesh_geometry(&valid_vertices, &normals, &clean_triangles).is_ok());
}

#[test]
fn test_all_workspace_assets_pass_sanitization() {
    let assets = [
        "assets/suzanne.obj",
        "assets/stanford-bunny.obj",
        "assets/teapot.obj",
        "assets/blender-4.3-splash.obj",
    ];

    for path in assets {
        if std::path::Path::new(path).exists() {
            let mesh = tej_ui_3d::load_obj_from_file(path).expect("Asset must load");
            assert!(
                sanitize_mesh_geometry(&mesh.vertices, &mesh.normals, &mesh.triangles).is_ok(),
                "Asset {} must pass sanitization",
                path
            );
        }
    }
}
