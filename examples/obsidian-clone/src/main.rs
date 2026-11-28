//! Obsidian Clone Interactive Application for `tej-ui`.
//!
//! Run interactively as a native desktop GUI app or in terminal REPL mode.
//! Run with `--test` for batch automated validation.

mod app_view;
mod db;
mod desktop_gui;
mod graph;
mod markdown;
mod models;
mod palette;
mod repl;

use db::VaultDatabase;
use desktop_gui::run_desktop_gui;
use repl::run_interactive;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let is_test_mode = args.iter().any(|a| a == "--test" || a == "--batch");
    let is_cli_mode = args.iter().any(|a| a == "--cli" || a == "--repl");

    println!("========================================================");
    println!("   Starting Obsidian Desktop Clone (Powered by tej-ui)   ");
    println!("========================================================");

    let vault_db = VaultDatabase::open_seeded()?;
    let notes = VaultDatabase::initial_notes();

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

    if is_cli_mode {
        // Launch interactive terminal REPL
        return run_interactive(&vault_db, notes);
    }

    // Launch authentic native desktop GUI window (Wayland / X11 / Win32 / AppKit)
    run_desktop_gui(&notes);
    Ok(())
}
