//! Table 2: Honest Architectural Trade-off Matrix.

/// Print Table 2: Honest Architectural Trade-off Matrix.
pub fn print_architectural_tradeoff_matrix() {
    println!("\n");
    println!(
        "================================================================================================================================="
    );
    println!(
        "                               TABLE 2: HONEST ARCHITECTURAL TRADE-OFF MATRIX                                                    "
    );
    println!(
        "================================================================================================================================="
    );
    println!(
        "{:<14} | {:<24} | {:<42} | {:<38}",
        "Framework", "Core Superpower (Wins)", "Real-World Sweet Spot", "Architectural Limitation"
    );
    println!("{:-<14}-|-{:-<24}-|-{:-<42}-|-{:-<38}", "", "", "", "");

    println!(
        "{:<14} | {:<24} | {:<42} | {:<38}",
        "egui v0.32",
        "100% Dynamic Churn",
        "Debug HUDs, Game Tools, Oscilloscopes",
        "Full frame redraws; burns battery idle"
    );

    println!(
        "{:<14} | {:<24} | {:<42} | {:<38}",
        "Slint v1.18",
        "Tiny Binary Size",
        "Embedded MCUs, Micro-dashboards, Appliances",
        "DSL barrier; runtime dynamic tree limits"
    );

    println!(
        "{:<14} | {:<24} | {:<42} | {:<38}",
        "GPUI (Zed)",
        "GPU Atlas Glyph Batching",
        "High-throughput code editors, GPU terminals",
        "Heavy GPU initialization; large memory"
    );

    println!(
        "{:<14} | {:<24} | {:<42} | {:<38}",
        "Iced v0.14",
        "Elm Safety & Purity",
        "State machine UIs, cross-platform apps",
        "Rebuild allocations per message tick"
    );

    println!(
        "{:<14} | {:<24} | {:<42} | {:<38}",
        "Dioxus v0.7",
        "Universal Fullstack SSR",
        "Web + Desktop shared React-style codebases",
        "VDOM diffing overhead on large tables"
    );

    println!(
        "{:<14} | {:<24} | {:<42} | {:<38}",
        "tej-ui",
        "Fine-Grained Reactivity",
        "100M-row Enterprise Apps, Zero-Copy DBs",
        "Ahead-of-time typing; no runtime eval"
    );
    println!(
        "=================================================================================================================================\n"
    );
}
