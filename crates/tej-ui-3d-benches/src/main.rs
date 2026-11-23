//! Reproducible Benchmark Suite for `tej-3d` Nanite & Out-of-Core Rendering.
//!
//! Orchestrates the 8 evaluation phases across the 4 workload scale tiers.

mod phases_1_4;
mod phases_5_8;
mod reporter;
mod tiers;

use mydb::Database;
use mydb_core::spatial::Aabb3D;
use tempfile::NamedTempFile;

use phases_1_4::{
    run_phase_1_cold_start, run_phase_2_culling_throughput, run_phase_3_differential_streaming,
    run_phase_4_rasterization_throughput,
};
use phases_5_8::{
    run_phase_5_picking_and_navigation, run_phase_6_acid_gizmo, run_phase_7_visual_fidelity,
    run_phase_8_security_fuzzing,
};
use reporter::{FullBenchmarkSuiteResults, print_comparative_table};
use tiers::{
    TierDescriptor, build_tier_3_studio_scene, build_tier_4_out_of_core_disk, load_tier_1_mesh,
    load_tier_2_mesh,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing 4-Tier benchmark workloads...");

    // 1. Build Tier 4 out-of-core file directly on disk
    let temp_file = NamedTempFile::new()?;
    let t4 = build_tier_4_out_of_core_disk(temp_file.path())?;

    println!("Running Phase 1: Cold-Start TTFF on 10M-triangle out-of-core disk asset...");
    let p1 = run_phase_1_cold_start(temp_file.path())?;

    // 2. Load Tier 1 & Tier 2 metadata
    let (m1, _s1, meshlets1) = load_tier_1_mesh();
    let t1 = TierDescriptor {
        name: "Tier 1: Micro / UI Widget (Suzanne)",
        tri_count: m1.triangles.len(),
        vert_count: m1.vertices.len(),
        meshlet_count: meshlets1.len(),
    };

    let (m2, _s2, meshlets2) = load_tier_2_mesh();
    let t2 = TierDescriptor {
        name: "Tier 2: Standard Asset (Stanford Bunny + Teapot)",
        tri_count: m2.triangles.len(),
        vert_count: m2.vertices.len(),
        meshlet_count: meshlets2.len(),
    };

    // 3. Build Tier 3: Complex Studio Scene (1,000,000 Triangles in out-of-core memory DB)
    let db3 = Database::open_memory_with_capacity(512)?;
    let domain3 = Aabb3D::new([-1000.0; 3], [1000.0; 3]);
    let vp3 = db3.create_spatial_viewport("tier3_vp", domain3)?;
    let mut engine3 = tej_ui_3d::NaniteRenderEngine::new(vp3, 2048);
    let t3 = build_tier_3_studio_scene(&mut engine3)?;

    println!("Running remaining evaluation phases (Phases 2 through 8)...");

    // Phase 2: SIMD Frustum, LOD & Hi-Z Culling throughput on 1M-triangle viewport
    let p2 = run_phase_2_culling_throughput(engine3.viewport_mut());

    // Phase 3: Out-of-Core differential streaming & cache churn
    let p3 = run_phase_3_differential_streaming(&mut engine3);

    // Phase 4: Rasterization throughput and overdraw resistance (1080p)
    let p4 = run_phase_4_rasterization_throughput();

    // Phase 5: Raycast picking across 1M triangles and 6-DOF navigation
    let p5 = run_phase_5_picking_and_navigation(engine3.viewport_mut());

    // Phase 6: ACID Gizmo transform mutations & COW undo snapshots
    let p6 = run_phase_6_acid_gizmo(&db3, engine3.viewport_mut())?;

    // Phase 7: Visual fidelity & deterministic verification
    let p7 = run_phase_7_visual_fidelity();

    // Phase 8: Security fuzzing suite (10,000 malformed inputs)
    let p8 = run_phase_8_security_fuzzing();

    let suite_results = FullBenchmarkSuiteResults {
        tiers: vec![t1, t2, t3, t4],
        p1,
        p2,
        p3,
        p4,
        p5,
        p6,
        p7,
        p8,
    };

    // Print full comparative report
    print_comparative_table(&suite_results);

    // Quality gate assertions
    assert!(
        suite_results.p1.peak_rss_ram_mb < 64.0,
        "Peak RSS exceeded 64 MB cap: {:.1} MB",
        suite_results.p1.peak_rss_ram_mb
    );
    assert!(
        suite_results.p1.cold_open_ttff_ms < 50.0,
        "Cold open TTFF exceeded budget: {:.2} ms",
        suite_results.p1.cold_open_ttff_ms
    );
    assert!(
        suite_results.p8.zero_panics,
        "Panic occurred during fuzz testing!"
    );

    Ok(())
}
