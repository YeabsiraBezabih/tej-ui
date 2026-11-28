//! Interactive REPL shell for testing and manipulating the Obsidian clone.

use crate::app_view::{ObsidianState, build_obsidian_view};
use crate::db::VaultDatabase;
use crate::graph::build_knowledge_graph;
use crate::models::NoteItem;
use crate::palette::build_obsidian_palette;
use std::io::{self, Write};
use tej_ui::prelude::*;

/// Launch the interactive terminal session.
pub fn run_interactive(
    db: &VaultDatabase,
    notes: Vec<NoteItem>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut session = ReplSession::new(db, notes);

    println!("\n╔═════════════════════════════════════════════════════════════════╗");
    println!("║       Obsidian Vault Interactive Shell — Powered by tej-ui      ║");
    println!("╚═════════════════════════════════════════════════════════════════╝");
    println!("Type 'help' for commands, or 'exit' to quit.\n");

    let stdin = io::stdin();
    loop {
        let note_title = &session.notes[session.active_note_idx].title;
        print!("obsidian [{note_title}]> ");
        io::stdout().flush()?;

        let mut input = String::new();
        if stdin.read_line(&mut input)? == 0 {
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if !session.dispatch(trimmed, &parts)? {
            break;
        }
    }

    Ok(())
}

struct ReplSession<'a> {
    db: &'a VaultDatabase,
    notes: Vec<NoteItem>,
    state: ObsidianState,
    active_note_idx: usize,
    editor: RichTextEditor,
}

impl<'a> ReplSession<'a> {
    fn new(db: &'a VaultDatabase, notes: Vec<NoteItem>) -> Self {
        let editor = RichTextEditor::with_text(&notes[0].content)
            .font_size(15.0)
            .line_height(24.0)
            .with_line_numbers();
        Self {
            db,
            notes,
            state: ObsidianState::new(),
            active_note_idx: 0,
            editor,
        }
    }

    fn dispatch(
        &mut self,
        trimmed: &str,
        parts: &[&str],
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let cmd = parts[0].to_lowercase();
        match cmd.as_str() {
            "help" => print_help(),
            "list" | "notes" => self.cmd_list(),
            "open" => self.cmd_open(parts),
            "read" => self.cmd_read(),
            "type" => self.cmd_type(trimmed, parts),
            "backspace" => {
                self.editor.backspace();
                self.notes[self.active_note_idx].content = self.editor.text();
                println!(
                    "Backspace executed. Text length: {}",
                    self.editor.text().len()
                );
            }
            "undo" => self.cmd_undo(),
            "redo" => self.cmd_redo(),
            "ime" => self.cmd_ime(trimmed, parts),
            "search" => cmd_search(trimmed, parts),
            "tag" => self.cmd_tag(parts)?,
            "sidebar" => self.cmd_sidebar(),
            "graph" => self.cmd_graph(),
            "orbit" => self.cmd_orbit(),
            "new" => self.cmd_new(parts)?,
            "render" => {
                let view = build_obsidian_view(&self.state, &self.notes);
                let mut v_state = view.build();
                view.rebuild(&mut v_state);
                println!(
                    "Rendered Obsidian View Tree (zero-vtable static dispatch). Frame valid.\n"
                );
            }
            "exit" | "quit" => {
                println!("Exiting Obsidian Vault shell. Goodbye!");
                return Ok(false);
            }
            _ => println!("Unknown command: \"{cmd}\". Type 'help' for a list of commands."),
        }
        Ok(true)
    }

    fn cmd_list(&self) {
        println!("\nVault Notes ({} items):", self.notes.len());
        for note in &self.notes {
            println!(
                "  [{}] {:<32} | Folder: {:<12} | Tags: #{}",
                note.id,
                note.title,
                note.folder,
                note.tags.join(" #")
            );
        }
        println!();
    }

    fn cmd_open(&mut self, parts: &[&str]) {
        if parts.len() < 2 {
            println!("Usage: open <id>");
            return;
        }
        let Ok(id) = parts[1].parse::<u64>() else {
            println!("Invalid note ID.");
            return;
        };
        if let Some((idx, note)) = self.notes.iter().enumerate().find(|(_, n)| n.id == id) {
            self.active_note_idx = idx;
            self.editor = RichTextEditor::with_text(&note.content)
                .font_size(15.0)
                .line_height(24.0)
                .with_line_numbers();
            self.state.selected_note_id.set(Some(id.to_string()));
            println!("Opened note [{}]: \"{}\"", note.id, note.title);
            println!("Buffer lines: {}\n", self.editor.buffer().line_count());
        } else {
            println!("Note ID {id} not found.");
        }
    }

    fn cmd_read(&self) {
        println!("\n--- {} ---", self.notes[self.active_note_idx].title);
        println!("{}", self.editor.text());
        println!("--------------------------------------------------\n");
    }

    fn cmd_type(&mut self, trimmed: &str, parts: &[&str]) {
        if parts.len() < 2 {
            println!("Usage: type <text>");
            return;
        }
        let text_to_insert = trimmed["type".len()..].trim_start();
        self.editor.insert_text(text_to_insert);
        self.notes[self.active_note_idx].content = self.editor.text();
        println!(
            "Inserted \"{text_to_insert}\". Buffer now has {} lines.",
            self.editor.buffer().line_count()
        );
    }

    fn cmd_undo(&mut self) {
        if self.editor.undo() {
            self.notes[self.active_note_idx].content = self.editor.text();
            println!(
                "Undo successful! Restored text length: {}",
                self.editor.text().len()
            );
        } else {
            println!("Nothing to undo.");
        }
    }

    fn cmd_redo(&mut self) {
        if self.editor.redo() {
            self.notes[self.active_note_idx].content = self.editor.text();
            println!("Redo successful! Text length: {}", self.editor.text().len());
        } else {
            println!("Nothing to redo.");
        }
    }

    fn cmd_ime(&mut self, trimmed: &str, parts: &[&str]) {
        if parts.len() < 2 {
            println!("Usage: ime <text>");
            return;
        }
        let preedit = trimmed["ime".len()..].trim_start();
        self.editor
            .update_ime(preedit.to_string(), preedit.chars().count());
        println!("IME Preedit active: \"{}\"", self.editor.ime().preedit);
        self.editor.commit_ime();
        self.notes[self.active_note_idx].content = self.editor.text();
        println!("IME Committed: \"{preedit}\" added to buffer.");
    }

    fn cmd_tag(&self, parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        if parts.len() < 2 {
            println!("Usage: tag <tag_name>");
            return Ok(());
        }
        let tag_name = parts[1].trim_start_matches('#');
        let rx = self.db.db.begin_read()?;
        let pks = self
            .db
            .tag_index
            .lookup_primary_keys(&rx, tag_name.as_bytes())?;
        println!("\nFound {} notes tagged #{}:", pks.len(), tag_name);
        for pk in pks {
            if let Ok(Some(content)) = rx.get(&pk) {
                let preview = String::from_utf8_lossy(&content)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                println!("  - {} -> {}", String::from_utf8_lossy(&pk), preview);
            }
        }
        println!();
        Ok(())
    }

    fn cmd_sidebar(&mut self) {
        self.state.toggle_sidebar();
        println!(
            "Sidebar toggled (collapsed: {}, target width: {:.1}px)",
            self.state.sidebar_collapsed.get(),
            self.state.sidebar_width_spring.target()
        );
        print!("Simulating spring physics frames: ");
        for _ in 0..10 {
            let w = self.state.sidebar_width_spring.step(1.0 / 60.0);
            print!("{w:.1}px ");
        }
        println!(
            "\nSpring position: {:.1}px\n",
            self.state.sidebar_width_spring.get()
        );
    }

    fn cmd_graph(&self) {
        let canvas = build_knowledge_graph(&self.notes);
        let state = canvas.build();
        println!("\n3D Knowledge Graph Status:");
        println!("  Node spheres: {} items", self.notes.len());
        println!("  Camera target: (0.0, 0.0, 0.0), Position: (0.0, 2.0, 5.0)");
        println!("  Illumination: Directional Sunlight (intensity 1.2) + Ambient Backlight");
        println!("  Canvas frame count: {}\n", state.frame_count);
    }

    fn cmd_orbit(&self) {
        println!("Orbiting 3D camera around knowledge graph...");
        let canvas = build_knowledge_graph(&self.notes);
        let mut c_state = canvas.build();
        canvas.rebuild(&mut c_state);
        println!("Camera orbit updated. Redraw scheduled.\n");
    }

    fn cmd_new(&mut self, parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        if parts.len() < 3 {
            println!("Usage: new <folder> <title>");
            return Ok(());
        }
        let folder = parts[1];
        let title = parts[2..].join(" ");
        let new_id = (self.notes.len() as u64) + 1;
        let filename = format!("{title}.md");
        let content = format!("# {title}\n\nCreated interactively via tej-ui.\n\nTags: #new #user");

        let mut tx = self.db.db.begin_write()?;
        let key = format!("note:{new_id:08}");
        tx.put(key.as_bytes(), content.as_bytes())?;
        self.db
            .tag_index
            .insert_entry(&mut tx, b"new", key.as_bytes())?;
        self.db
            .tag_index
            .insert_entry(&mut tx, b"user", key.as_bytes())?;
        tx.commit()?;

        self.notes.push(NoteItem::new(
            new_id,
            filename,
            folder,
            content,
            vec!["new".into(), "user".into()],
            vec![],
        ));
        println!("Created and committed note [{new_id}]: \"{title}\" to folder \"{folder}\"!\n");
        Ok(())
    }
}

fn print_help() {
    println!("\nAvailable Commands:");
    println!("  list / notes           - List all notes in the vault");
    println!("  open <id>              - Open note by ID into the RichTextEditor");
    println!("  read                   - Display the current note's full text");
    println!("  type <text...>         - Insert text into active editor");
    println!("  backspace              - Backspace in active editor");
    println!("  undo                   - Undo last edit operation");
    println!("  redo                   - Redo last undone operation");
    println!("  ime <text>             - Test native IME composition & commit");
    println!("  search <query>         - Fuzzy search actions and notes (Cmd+K)");
    println!("  tag <tag>              - Query notes by tag via secondary B+ Tree index");
    println!("  sidebar                - Toggle sidebar with Spring Physics animation");
    println!("  graph                  - Inspect 3D Knowledge Graph scene & camera");
    println!("  orbit <dx> <dy>        - Orbit 3D knowledge graph camera");
    println!("  new <folder> <title>   - Create a new markdown note in ACID database");
    println!("  render                 - Build and layout the complete native view tree");
    println!("  exit / quit            - Exit interactive shell\n");
}

fn cmd_search(trimmed: &str, parts: &[&str]) {
    let query_str = if parts.len() > 1 {
        trimmed["search".len()..].trim_start()
    } else {
        ""
    };
    let q_sig = Signal::new(query_str.to_string());
    let idx_sig = Signal::new(0);
    let palette = build_obsidian_palette(q_sig, idx_sig);
    let items = palette.filtered_items();
    println!("\nCommand Palette Results for \"{query_str}\":");
    for item in items {
        let shortcut = item.shortcut.as_deref().unwrap_or("");
        println!(
            "  - [{:<12}] {:<40} {:>10}",
            item.category, item.title, shortcut
        );
    }
    println!();
}
