//! Zero-copy database storage and note seeding for the Obsidian vault.

use crate::models::NoteItem;
use mydb::index::SecondaryIndex;
use mydb::{Database, MemStorage};
use mydb_core::DbError;
use std::sync::Arc;
use tej_db_sync::DbSyncHook;

/// The initialized vault database engine.
pub struct VaultDatabase {
    pub db: Arc<Database<MemStorage>>,
    #[allow(dead_code)]
    pub hook: Arc<DbSyncHook>,
    pub tag_index: SecondaryIndex,
}

impl VaultDatabase {
    /// Initialize the in-memory database and populate mock Obsidian notes.
    pub fn open_seeded() -> Result<Self, DbError> {
        let db = Arc::new(Database::open_memory()?);
        let hook = Arc::new(DbSyncHook::new());
        let tag_index = SecondaryIndex::new("note_tags");

        let notes = default_vault_notes();

        let mut tx = db.begin_write()?;
        for note in &notes {
            let note_key = format!("note:{:08}", note.id);
            tx.put(note_key.as_bytes(), note.content.as_bytes())?;

            for tag in &note.tags {
                tag_index.insert_entry(&mut tx, tag.as_bytes(), note_key.as_bytes())?;
            }
        }
        tx.commit()?;

        Ok(Self {
            db,
            hook,
            tag_index,
        })
    }

    /// Retrieve all seeded notes.
    pub fn initial_notes(&self) -> Vec<NoteItem> {
        default_vault_notes()
    }
}

/// Returns the initial suite of mock notes showcasing Markdown, LaTeX, Mermaid, and SVG.
pub fn default_vault_notes() -> Vec<NoteItem> {
    vec![
        NoteItem::new(
            1,
            "Architecture of tej-ui.md",
            "Architecture",
            "# Architecture of tej-ui\n\n\
            Modular, zero-copy, cross-platform native UI framework.\n\n\
            ```mermaid\n\
            graph TD\n\
            Core[\"tej-core (Signals & Graph)\"] --> View[\"tej-view (Static Combinators)\"]\n\
            View --> Layout[\"tej-layout (Taffy SoA)\"]\n\
            Layout --> HAL[\"tej-hal (OS Compositor)\"]\n\
            HAL --> Platform[\"Linux / Windows / macOS\"]\n\
            ```\n\n\
            ## Core Invariants\n\
            - Zero-allocation steady-state frame rendering\n\
            - DirectComposition and Wayland 2-tier compositors\n\
            - Mmap zero-copy database integration\n\n\
            Tags: #rust #architecture #tej-ui",
            vec!["rust".into(), "architecture".into(), "tej-ui".into()],
            vec!["2".into(), "3".into(), "4".into()],
        ),
        NoteItem::new(
            2,
            "Spring Physics and VSync.md",
            "Animation",
            "# Spring Physics and VSync\n\n\
            Smooth velocity-based spring transitions using symplectic Euler integration.\n\n\
            ## Mathematical Formulation\n\
            $$\\ddot{x} = \\frac{-k(x - x_t) - c v}{m}$$\n\n\
            Natural frequency $\\omega_0$ and damping ratio $\\zeta$:\n\
            $$\\omega_0 = \\sqrt{\\frac{k}{m}}$$\n\
            $$\\zeta = \\frac{c}{2\\sqrt{km}}$$\n\n\
            Critical damping condition occurs when $\\zeta = 1.0$.\n\n\
            Tags: #animation #physics #tej-animation",
            vec!["animation".into(), "physics".into(), "tej-animation".into()],
            vec!["1".into(), "2".into()],
        ),
        NoteItem::new(
            3,
            "Zero-Copy DBMS Integration.md",
            "Database",
            "# Zero-Copy DBMS Integration\n\n\
            Using mydb's copy-on-write B+ Tree and mmap page slices.\n\n\
            <svg width=\"480\" height=\"100\">\n\
              <rect x=\"10\" y=\"10\" width=\"120\" height=\"75\" rx=\"6\" fill=\"#1e293b\" stroke=\"#3b82f6\" stroke-width=\"2\"/>\n\
              <text x=\"24\" y=\"42\" fill=\"#f8fafc\" font-size=\"13\">DBMS Buffer</text>\n\
              <text x=\"32\" y=\"62\" fill=\"#94a3b8\" font-size=\"11\">Mmap Page</text>\n\
              <line x1=\"130\" y1=\"47\" x2=\"175\" y2=\"47\" stroke=\"#10b981\" stroke-width=\"2\"/>\n\
              <polygon points=\"182,47 172,42 172,52\" fill=\"#10b981\"/>\n\
              <rect x=\"185\" y=\"10\" width=\"120\" height=\"75\" rx=\"6\" fill=\"#1e293b\" stroke=\"#10b981\" stroke-width=\"2\"/>\n\
              <text x=\"197\" y=\"42\" fill=\"#f8fafc\" font-size=\"13\">ZeroCopyRow</text>\n\
              <text x=\"212\" y=\"62\" fill=\"#94a3b8\" font-size=\"11\">&[u8] Slice</text>\n\
              <line x1=\"305\" y1=\"47\" x2=\"350\" y2=\"47\" stroke=\"#6366f1\" stroke-width=\"2\"/>\n\
              <polygon points=\"357,47 347,42 347,52\" fill=\"#6366f1\"/>\n\
              <rect x=\"360\" y=\"10\" width=\"110\" height=\"75\" rx=\"6\" fill=\"#1e293b\" stroke=\"#6366f1\" stroke-width=\"2\"/>\n\
              <text x=\"384\" y=\"42\" fill=\"#f8fafc\" font-size=\"13\">UI View</text>\n\
              <text x=\"380\" y=\"62\" fill=\"#94a3b8\" font-size=\"11\">Virtual List</text>\n\
            </svg>\n\n\
            ## Key Characteristics\n\
            - Zero memory allocations on row traversal\n\
            - WAL commit hook with reactive cache invalidation\n\n\
            Tags: #mydb #zero-copy #rust",
            vec!["mydb".into(), "zero-copy".into(), "rust".into()],
            vec!["1".into(), "4".into()],
        ),
        NoteItem::new(
            4,
            "Interactive Playground.md",
            "Playground",
            "# Interactive Markdown Playground\n\n\
            You can edit this note live! Click the [Mode: Edit] button in the upper right (or press 'E') to toggle between Edit and Preview modes.\n\n\
            ## Features Checklist\n\
            - [x] Live LaTeX Math equation rendering\n\
            - [x] Native Mermaid flowchart diagram compiler\n\
            - [x] Embedded SVG vector graphics\n\
            - [ ] Try typing your own text here!\n\n\
            ```rust\n\
            fn render_note(vault: &Vault) -> Result<(), UiError> {\n\
                println!(\"Interactive markdown editing active!\");\n\
                Ok(())\n\
            }\n\
            ```\n\n\
            $$\\int_{0}^{\\infty} e^{-x^2} dx = \\frac{\\sqrt{\\pi}}{2}$$\n\n\
            Tags: #demo #playground #interactive",
            vec!["demo".into(), "playground".into(), "interactive".into()],
            vec!["1".into(), "2".into(), "3".into()],
        ),
        NoteItem::new(
            5,
            "3D Knowledge Graph Rendering.md",
            "Graphics",
            "# 3D Knowledge Graph\n\n\
            Interactive visual graph of connected notes using `tej-3d`.\n\n\
            ## Implementation\n\
            - Perspective camera & orbit controls\n\
            - Procedural sphere meshes for notes\n\
            - Blinn-Phong lighting and raycasting\n\n\
            Tags: #graphics #3d #visuals",
            vec!["graphics".into(), "3d".into(), "visuals".into()],
            vec!["1".into(), "6".into()],
        ),
        NoteItem::new(
            6,
            "Native Platform Compositing.md",
            "Platforms",
            "# Native Platform Compositing\n\n\
            Hybrid 2-tier OS compositor for Linux, Windows, and macOS.\n\n\
            - DirectComposition & Mica on Windows\n\
            - CAMetalLayer on macOS\n\
            - Wayland / X11 on Linux\n\n\
            Tags: #hal #native #desktop",
            vec!["hal".into(), "native".into(), "desktop".into()],
            vec!["1".into(), "3".into()],
        ),
        NoteItem::new(
            7,
            "Daily Note - 2026-11-13.md",
            "Daily",
            "# Daily Log: 2026-11-13\n\n\
            - [x] Integrate Lucide icon pack\n\
            - [x] Implement RichTextEditor with native IME\n\
            - [x] Add Spring physics animation engine\n\
            - [x] Native LaTeX math & Mermaid diagram rendering\n\
            - [x] In-place interactive Markdown editor\n\n\
            Tags: #daily #journal",
            vec!["daily".into(), "journal".into()],
            vec!["1".into(), "4".into()],
        ),
    ]
}
