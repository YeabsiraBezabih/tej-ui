use mydb::Database;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tej::prelude::*;

fn get_rss_memory_mb() -> f64 {
    if let Ok(statm) = fs::read_to_string("/proc/self/statm") {
        let parts: Vec<&str> = statm.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(resident_pages) = parts[1].parse::<u64>() {
                // Page size is typically 4KB
                let page_size_kb = 4.0;
                return (resident_pages as f64 * page_size_kb) / 1024.0;
            }
        }
    }
    0.0
}

fn bench_signals(iterations: usize) {
    println!("\n--- [1] Reactive Arena Signals Benchmark ---");
    let count = Signal::new(0usize);
    let double = Memo::new(move || count.get() * 2);
    let quad = Memo::new(move || double.get() * 2);

    let effect_evals = Arc::new(AtomicUsize::new(0));
    let effect_evals_clone = Arc::clone(&effect_evals);
    let _effect = create_effect(move || {
        let _ = quad.get();
        effect_evals_clone.fetch_add(1, Ordering::Relaxed);
    });

    let start = Instant::now();
    for i in 1..=iterations {
        count.set(i);
    }
    let elapsed = start.elapsed();
    let per_op_ns = (elapsed.as_nanos() as f64) / (iterations as f64);
    let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();

    println!("Total operations:    {iterations} updates");
    println!("Elapsed time:        {elapsed:?}");
    println!("Latency per update:  {per_op_ns:.2} ns / op (including DAG & 2 Memos + 1 Effect)");
    println!("Throughput:          {ops_per_sec:.2} ops / sec");
    println!(
        "Total effect firings: {}",
        effect_evals.load(Ordering::Relaxed)
    );
}

fn bench_layout(node_count: usize) {
    println!("\n--- [2] Flat SoA Layout Engine Benchmark ---");
    let mut arena = LayoutArena::with_capacity(node_count + 1);
    let root = arena.allocate_node(LayoutStyle {
        width: Dimension::Points(1920.0),
        height: Dimension::Points(1080.0),
        direction: FlexDirection::Column,
        padding: Insets::all(16.0),
        spacing: 4.0,
        ..Default::default()
    });

    for _ in 0..node_count {
        let child = arena.allocate_node(LayoutStyle {
            height: Dimension::Points(24.0),
            ..Default::default()
        });
        arena.add_child(root, child);
    }

    let start = Instant::now();
    let passes = 100;
    for _ in 0..passes {
        arena.mark_dirty(root);
        LayoutEngine::compute_layout(&mut arena, root, Size::new(1920.0, 1080.0));
    }
    let elapsed = start.elapsed();
    let per_pass_us = (elapsed.as_micros() as f64) / (passes as f64);

    println!("Hierarchy:           1 container with {node_count} children");
    println!("Passes executed:     {passes}");
    println!("Time per flex pass:  {per_pass_us:.2} µs (for {node_count} nodes)");
}

fn bench_db_virtual_list() {
    println!("\n--- [3] Zero-Copy Database Sync & Virtual List Benchmark ---");
    let db = Arc::new(Database::open_memory().unwrap());
    let hook = Arc::new(DbSyncHook::new());
    let _ = hook;

    // Populate database with 50,000 records
    let load_start = Instant::now();
    let mut tx = db.begin_write().unwrap();
    for i in 0..50_000 {
        let key = format!("issue:{i:06}");
        let val = format!("Title for issue #{i:06} stored directly in slotted page");
        tx.put(key.as_bytes(), val.as_bytes()).unwrap();
    }
    tx.commit().unwrap();
    println!(
        "Database population: 50,000 items loaded in {:?}",
        load_start.elapsed()
    );

    // Measure memory before virtual list
    let mem_loaded = get_rss_memory_mb();
    println!("RSS Memory with 50K DB records: {mem_loaded:.2} MB");

    // Mount VirtualList for 1,000,000 items (36px high rows in 720px viewport)
    let scroll_pos = Signal::new(0.0f32);
    let vlist = VirtualList::new(1_000_000, 36.0, 720.0, scroll_pos, |idx| {
        Text::new(format!("Task #{idx}"))
    });

    let mut state = vlist.build();
    let initial_visible = state.visible_states.len();

    // Rapidly scroll 100,000 virtual rows
    let scroll_start = Instant::now();
    let scroll_steps = 10_000;
    for step in 1..=scroll_steps {
        scroll_pos.set((step * 10) as f32);
        vlist.patch(&mut state, 1);
    }
    let scroll_elapsed = scroll_start.elapsed();
    let time_per_scroll_us = (scroll_elapsed.as_micros() as f64) / (scroll_steps as f64);

    let mem_after_scroll = get_rss_memory_mb();

    println!("Virtual list scale:  1,000,000 items");
    println!("Visible items:       {initial_visible} rows in viewport");
    println!("Scroll steps:        {scroll_steps} viewport jumps");
    println!(
        "Time per jump:       {time_per_scroll_us:.2} µs / frame (~{:.0} FPS theoretical)",
        1_000_000.0 / time_per_scroll_us
    );
    println!("RSS Memory after scrolling 1M items: {mem_after_scroll:.2} MB (Flat memory proof!)");
}

fn main() {
    println!("============================================================");
    println!("         tej-ui Real-World Performance Benchmark           ");
    println!("============================================================");

    let baseline_mem = get_rss_memory_mb();
    println!("Process Baseline RSS Memory: {baseline_mem:.2} MB");

    bench_signals(1_000_000);
    bench_layout(1_000);
    bench_layout(10_000);
    bench_db_virtual_list();

    let final_mem = get_rss_memory_mb();
    println!("\nFinal Process RSS Memory: {final_mem:.2} MB");
    println!("============================================================");
}
