//! Obsidian Clone Interactive Application for `tej-ui`.
//!
//! Run interactively via terminal or open the live Web GUI in the browser.
//! Run with `--test` for batch automated validation.

mod app_view;
mod db;
mod graph;
mod models;
mod palette;
mod repl;
mod web_gui;

use db::VaultDatabase;
use repl::run_interactive;
use std::env;
use std::error::Error;
use web_gui::start_web_gui_background;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let is_test_mode = args.iter().any(|a| a == "--test" || a == "--batch");

    println!("========================================================");
    println!("   Starting Obsidian Desktop Clone (Powered by tej-ui)   ");
    println!("========================================================");

    let vault_db = VaultDatabase::open_seeded()?;
    let notes = vault_db.initial_notes();

    // Start background web server for live visual GUI testing
    start_web_gui_background(notes.clone());
    println!("  → Live Web GUI Server active at: http://127.0.0.1:4000");

    if is_test_mode {
        println!("\n[Test Mode] Executing automated verification...");
        let rx = vault_db.db.begin_read()?;
        let matching_pks = vault_db.tag_index.lookup_primary_keys(&rx, b"rust")?;
        assert!(!matching_pks.is_empty(), "Tag index lookup failed");
        println!(
            "  ✓ Automated test passed with {} notes tagged #rust",
            matching_pks.len()
        );
        return Ok(());
    }

    // Launch interactive REPL
    run_interactive(vault_db, notes)
}
