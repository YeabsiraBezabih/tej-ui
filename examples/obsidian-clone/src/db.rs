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

        let notes = vec![
            NoteItem::new(
                1,
                "Architecture of tej-ui.md",
                "Architecture",
                "# Architecture of tej-ui\n\nModular, zero-copy, cross-platform native UI framework.\n\n## References\n- [[Zero-Copy DBMS Integration]]\n- [[3D Knowledge Graph Rendering]]\n- [[Spring Physics and VSync]]\n\nTags: #rust #architecture #tej-ui",
                vec!["rust".into(), "architecture".into(), "tej-ui".into()],
                vec!["2".into(), "3".into(), "4".into()],
            ),
            NoteItem::new(
                2,
                "Zero-Copy DBMS Integration.md",
                "Database",
                "# Zero-Copy DBMS Integration\n\nUsing mydb's copy-on-write B+ Tree and mmap page slices.\n\n## Features\n- Zero-copy row slicing with `ZeroCopyRow`\n- WAL commit notifications via `DbSyncHook`\n- $O(\\log N)$ prefix-bounded scans\n\nTags: #mydb #zero-copy #rust",
                vec!["mydb".into(), "zero-copy".into(), "rust".into()],
                vec!["1".into(), "4".into()],
            ),
            NoteItem::new(
                3,
                "3D Knowledge Graph Rendering.md",
                "Graphics",
                "# 3D Knowledge Graph\n\nInteractive visual graph of connected notes using `tej-3d`.\n\n## Implementation\n- Perspective cameras & orbit controls\n- Procedural sphere meshes for notes\n- Blinn-Phong lighting and raycasting\n\nTags: #graphics #3d #visuals",
                vec!["graphics".into(), "3d".into(), "visuals".into()],
                vec!["1".into(), "5".into()],
            ),
            NoteItem::new(
                4,
                "Spring Physics and VSync.md",
                "Animation",
                "# Spring Physics and VSync\n\nSmooth velocity-based spring transitions using symplectic Euler integration.\n\n## Mathematical Model\n$$\\ddot{x} = \\frac{-k(x - x_t) - c v}{m}$$\n\nTags: #animation #physics #tej-animation",
                vec!["animation".into(), "physics".into(), "tej-animation".into()],
                vec!["1".into(), "2".into()],
            ),
            NoteItem::new(
                5,
                "Native Platform Compositing.md",
                "Platforms",
                "# Native Platform Compositing\n\nHybrid 2-tier OS compositor for Linux, Windows, and macOS.\n\n- DirectComposition & Mica on Windows\n- CAMetalLayer on macOS\n- Wayland / X11 on Linux\n\nTags: #hal #native #desktop",
                vec!["hal".into(), "native".into(), "desktop".into()],
                vec!["1".into(), "3".into()],
            ),
            NoteItem::new(
                6,
                "Daily Note - 2026-11-13.md",
                "Daily",
                "# Daily Log: 2026-11-13\n\n- [x] Integrate Lucide icon pack\n- [x] Implement RichTextEditor with native IME\n- [x] Add Spring physics animation engine\n- [x] Build Obsidian clone demo\n\nTags: #daily #journal",
                vec!["daily".into(), "journal".into()],
                vec!["1".into(), "4".into()],
            ),
        ];

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
        vec![
            NoteItem::new(
                1,
                "Architecture of tej-ui.md",
                "Architecture",
                "# Architecture of tej-ui\n\nModular, zero-copy, cross-platform native UI framework.\n\n## References\n- [[Zero-Copy DBMS Integration]]\n- [[3D Knowledge Graph Rendering]]\n- [[Spring Physics and VSync]]\n\nTags: #rust #architecture #tej-ui",
                vec!["rust".into(), "architecture".into(), "tej-ui".into()],
                vec!["2".into(), "3".into(), "4".into()],
            ),
            NoteItem::new(
                2,
                "Zero-Copy DBMS Integration.md",
                "Database",
                "# Zero-Copy DBMS Integration\n\nUsing mydb's copy-on-write B+ Tree and mmap page slices.\n\n## Features\n- Zero-copy row slicing with `ZeroCopyRow`\n- WAL commit notifications via `DbSyncHook`\n- $O(\\log N)$ prefix-bounded scans\n\nTags: #mydb #zero-copy #rust",
                vec!["mydb".into(), "zero-copy".into(), "rust".into()],
                vec!["1".into(), "4".into()],
            ),
            NoteItem::new(
                3,
                "3D Knowledge Graph Rendering.md",
                "Graphics",
                "# 3D Knowledge Graph\n\nInteractive visual graph of connected notes using `tej-3d`.\n\n## Implementation\n- Perspective cameras & orbit controls\n- Procedural sphere meshes for notes\n- Blinn-Phong lighting and raycasting\n\nTags: #graphics #3d #visuals",
                vec!["graphics".into(), "3d".into(), "visuals".into()],
                vec!["1".into(), "5".into()],
            ),
            NoteItem::new(
                4,
                "Spring Physics and VSync.md",
                "Animation",
                "# Spring Physics and VSync\n\nSmooth velocity-based spring transitions using symplectic Euler integration.\n\n## Mathematical Model\n$$\\ddot{x} = \\frac{-k(x - x_t) - c v}{m}$$\n\nTags: #animation #physics #tej-animation",
                vec!["animation".into(), "physics".into(), "tej-animation".into()],
                vec!["1".into(), "2".into()],
            ),
            NoteItem::new(
                5,
                "Native Platform Compositing.md",
                "Platforms",
                "# Native Platform Compositing\n\nHybrid 2-tier OS compositor for Linux, Windows, and macOS.\n\n- DirectComposition & Mica on Windows\n- CAMetalLayer on macOS\n- Wayland / X11 on Linux\n\nTags: #hal #native #desktop",
                vec!["hal".into(), "native".into(), "desktop".into()],
                vec!["1".into(), "3".into()],
            ),
            NoteItem::new(
                6,
                "Daily Note - 2026-11-13.md",
                "Daily",
                "# Daily Log: 2026-11-13\n\n- [x] Integrate Lucide icon pack\n- [x] Implement RichTextEditor with native IME\n- [x] Add Spring physics animation engine\n- [x] Build Obsidian clone demo\n\nTags: #daily #journal",
                vec!["daily".into(), "journal".into()],
                vec!["1".into(), "4".into()],
            ),
        ]
    }
}
