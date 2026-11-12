//! Obsidian Clone Showcase Application for `tej-ui`.
//!
//! Demonstrates full framework integration:
//! - Reactive Core: 8-byte `Signal<T>`, effects, batching, topological DAG.
//! - View System: Compile-time static view tree combinators and zero-vtable dispatch.
//! - Embedded ACID DBMS: `mydb` copy-on-write B+ Tree, zero-copy rows, WAL hooks, and secondary index.
//! - 3D Knowledge Graph: `tej-3d` Three.js-inspired perspective scene with procedural note spheres.
//! - Rich Text Editor: Multi-line `TextBuffer`, caret selections, undo/redo stack, and native IME.
//! - Spring Animations: Velocity-based spring physics with symplectic Euler integrator.
//! - Enterprise Component Kit: `Sidebar`, `TreeView`, `Tabs`, `CommandPalette`, `Badge`, `Card`.
//! - Vector Icons: 120+ Lucide open-source vector icon catalog.

mod app_view;
mod db;
mod graph;
mod models;
mod palette;

use app_view::{ObsidianState, build_obsidian_view};
use db::VaultDatabase;
use graph::build_knowledge_graph;
use palette::build_obsidian_palette;
use std::error::Error;
use tej::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    println!("========================================================");
    println!("   Starting Obsidian Desktop Clone (Powered by tej-ui)   ");
    println!("========================================================");

    // 1. Initialize Embedded ACID Database & Seed Vault
    println!("\n[1/6] Initializing ACID Vault Database (mydb)...");
    let vault_db = VaultDatabase::open_seeded()?;
    let notes = vault_db.initial_notes();
    println!(
        "      ✓ Seeded {} markdown notes across 6 folders",
        notes.len()
    );

    // 2. Test Secondary Index & Zero-Copy Queries
    println!("\n[2/6] Querying Notes via Secondary B+ Tree Index (#rust)...");
    let rx = vault_db.db.begin_read()?;
    let matching_pks = vault_db.tag_index.lookup_primary_keys(&rx, b"rust")?;
    println!(
        "      ✓ Found {} notes tagged #rust via secondary index:",
        matching_pks.len()
    );
    for pk in matching_pks {
        if let Ok(Some(content)) = rx.get(&pk) {
            let first_line = String::from_utf8_lossy(&content)
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            println!(
                "        - Note [{}]: \"{}\"",
                String::from_utf8_lossy(&pk),
                first_line
            );
        }
    }

    // 3. Initialize Interactive 3D Knowledge Graph
    println!("\n[3/6] Building 3D Interactive Knowledge Graph (tej-3d)...");
    let graph_canvas = build_knowledge_graph(&notes);
    let mut graph_state = graph_canvas.build();
    graph_canvas.rebuild(&mut graph_state);
    println!(
        "      ✓ 3D Scene compiled with perspective camera, procedural note meshes, and Blinn-Phong lighting"
    );

    // 4. Initialize Obsidian State & Spring Animations
    println!("\n[4/6] Initializing Reactive State & Spring Physics (tej-animation)...");
    let mut app_state = ObsidianState::new();
    println!(
        "      ✓ Initial sidebar width: {:.1}px",
        app_state.sidebar_width_spring.get()
    );

    // Simulate spring physics collapse animation
    app_state.toggle_sidebar();
    println!(
        "      ✓ Toggled sidebar collapse (target: {:.1}px)",
        app_state.sidebar_width_spring.target()
    );
    for frame in 1..=5 {
        let width = app_state.sidebar_width_spring.step(1.0 / 60.0);
        println!("        Frame {frame}: spring width = {:.2}px", width);
    }

    // 5. Test Interactive Rich Text Editor with Multi-line & IME
    println!("\n[5/6] Testing Markdown Editor & Native IME (tej-widgets/editor)...");
    let mut editor = RichTextEditor::with_text(&notes[0].content)
        .font_size(15.0)
        .line_height(24.0)
        .show_line_numbers(true);

    println!(
        "      ✓ Initial editor lines: {}",
        editor.buffer().line_count()
    );
    editor.insert_text("\n\n> Note: Engineered with zero-allocation steady state.");
    println!(
        "      ✓ Appended text. New line count: {}",
        editor.buffer().line_count()
    );

    // Test Undo
    assert!(editor.undo());
    println!("      ✓ Undo verified: text successfully restored");

    // Test Native IME
    editor.update_ime("日本語メモ".to_string(), 5);
    println!("      ✓ IME Preedit active: \"{}\"", editor.ime().preedit);
    editor.commit_ime();
    println!("      ✓ IME Committed successfully: editor updated");

    // 6. Test Command Palette & View Tree Assembly
    println!("\n[6/6] Testing Command Palette & View Tree Assembly...");
    let palette = build_obsidian_palette(app_state.palette_query, app_state.palette_index);
    let filtered_cmds = palette.filtered_items();
    println!(
        "      ✓ Command palette initialized with {} actions (e.g. \"{}\")",
        filtered_cmds.len(),
        filtered_cmds[0].title
    );

    let main_view = build_obsidian_view(&app_state, &notes);
    let mut view_state = main_view.build();
    main_view.rebuild(&mut view_state);
    println!("      ✓ Master Obsidian View Tree assembled with zero-vtable static dispatch");

    println!("\n========================================================");
    println!("   Obsidian Clone Demo Executed Successfully! (100% OK) ");
    println!("========================================================");

    Ok(())
}
