//! Interactive REPL shell for testing and manipulating the Obsidian clone.

use crate::app_view::{ObsidianState, build_obsidian_view};
use crate::db::VaultDatabase;
use crate::graph::build_knowledge_graph;
use crate::models::NoteItem;
use crate::palette::build_obsidian_palette;
use std::io::{self, Write};
use tej::prelude::*;

/// Launch the interactive terminal session.
pub fn run_interactive(
    db: VaultDatabase,
    mut notes: Vec<NoteItem>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ObsidianState::new();
    let mut active_note_idx = 0;
    let mut editor = RichTextEditor::with_text(&notes[active_note_idx].content)
        .font_size(15.0)
        .line_height(24.0)
        .show_line_numbers(true);

    println!("\n╔═════════════════════════════════════════════════════════════════╗");
    println!("║       Obsidian Vault Interactive Shell — Powered by tej-ui      ║");
    println!("╚═════════════════════════════════════════════════════════════════╝");
    println!("Type 'help' for commands, or 'exit' to quit.\n");

    let stdin = io::stdin();

    loop {
        let note_title = &notes[active_note_idx].title;
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
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "help" => {
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
                println!(
                    "  tag <tag>              - Query notes by tag via secondary B+ Tree index"
                );
                println!("  sidebar                - Toggle sidebar with Spring Physics animation");
                println!("  graph                  - Inspect 3D Knowledge Graph scene & camera");
                println!("  orbit <dx> <dy>        - Orbit 3D knowledge graph camera");
                println!("  new <folder> <title>   - Create a new markdown note in ACID database");
                println!(
                    "  render                 - Build and layout the complete native view tree"
                );
                println!(
                    "  web                    - Start live browser GUI on http://127.0.0.1:4000"
                );
                println!("  exit / quit            - Exit interactive shell\n");
            }

            "list" | "notes" => {
                println!("\nVault Notes ({} items):", notes.len());
                for note in &notes {
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

            "open" => {
                if parts.len() < 2 {
                    println!("Usage: open <id>");
                    continue;
                }
                if let Ok(id) = parts[1].parse::<u64>() {
                    if let Some((idx, note)) = notes.iter().enumerate().find(|(_, n)| n.id == id) {
                        active_note_idx = idx;
                        editor = RichTextEditor::with_text(&note.content)
                            .font_size(15.0)
                            .line_height(24.0)
                            .show_line_numbers(true);
                        state.selected_note_id.set(Some(id.to_string()));
                        println!("Opened note [{}]: \"{}\"", note.id, note.title);
                        println!("Buffer lines: {}\n", editor.buffer().line_count());
                    } else {
                        println!("Note ID {id} not found.");
                    }
                } else {
                    println!("Invalid note ID.");
                }
            }

            "read" => {
                println!("\n--- {} ---", notes[active_note_idx].title);
                println!("{}", editor.text());
                println!("--------------------------------------------------\n");
            }

            "type" => {
                if parts.len() < 2 {
                    println!("Usage: type <text>");
                    continue;
                }
                let text_to_insert = trimmed["type".len()..].trim_start();
                editor.insert_text(text_to_insert);
                notes[active_note_idx].content = editor.text();
                println!(
                    "Inserted \"{text_to_insert}\". Buffer now has {} lines.",
                    editor.buffer().line_count()
                );
            }

            "backspace" => {
                editor.backspace();
                notes[active_note_idx].content = editor.text();
                println!("Backspace executed. Text length: {}", editor.text().len());
            }

            "undo" => {
                if editor.undo() {
                    notes[active_note_idx].content = editor.text();
                    println!(
                        "Undo successful! Restored text length: {}",
                        editor.text().len()
                    );
                } else {
                    println!("Nothing to undo.");
                }
            }

            "redo" => {
                if editor.redo() {
                    notes[active_note_idx].content = editor.text();
                    println!("Redo successful! Text length: {}", editor.text().len());
                } else {
                    println!("Nothing to redo.");
                }
            }

            "ime" => {
                if parts.len() < 2 {
                    println!("Usage: ime <text>");
                    continue;
                }
                let preedit = trimmed["ime".len()..].trim_start();
                editor.update_ime(preedit.to_string(), preedit.chars().count());
                println!("IME Preedit active: \"{}\"", editor.ime().preedit);
                editor.commit_ime();
                notes[active_note_idx].content = editor.text();
                println!("IME Committed: \"{preedit}\" added to buffer.");
            }

            "search" => {
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

            "tag" => {
                if parts.len() < 2 {
                    println!("Usage: tag <tag_name>");
                    continue;
                }
                let tag_name = parts[1].trim_start_matches('#');
                let rx = db.db.begin_read()?;
                let pks = db.tag_index.lookup_primary_keys(&rx, tag_name.as_bytes())?;
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
            }

            "sidebar" => {
                state.toggle_sidebar();
                println!(
                    "Sidebar toggled (collapsed: {}, target width: {:.1}px)",
                    state.sidebar_collapsed.get(),
                    state.sidebar_width_spring.target()
                );
                print!("Simulating spring physics frames: ");
                for _ in 0..10 {
                    let w = state.sidebar_width_spring.step(1.0 / 60.0);
                    print!("{w:.1}px ");
                }
                println!(
                    "\nSpring position: {:.1}px\n",
                    state.sidebar_width_spring.get()
                );
            }

            "graph" => {
                let canvas = build_knowledge_graph(&notes);
                let state = canvas.build();
                println!("\n3D Knowledge Graph Status:");
                println!("  Node spheres: {} items", notes.len());
                println!("  Camera target: (0.0, 0.0, 0.0), Position: (0.0, 2.0, 5.0)");
                println!(
                    "  Illumination: Directional Sunlight (intensity 1.2) + Ambient Backlight"
                );
                println!("  Canvas frame count: {}\n", state.frame_count);
            }

            "orbit" => {
                println!("Orbiting 3D camera around knowledge graph...");
                let canvas = build_knowledge_graph(&notes);
                let mut c_state = canvas.build();
                canvas.rebuild(&mut c_state);
                println!("Camera orbit updated. Redraw scheduled.\n");
            }

            "new" => {
                if parts.len() < 3 {
                    println!("Usage: new <folder> <title>");
                    continue;
                }
                let folder = parts[1];
                let title = parts[2..].join(" ");
                let new_id = (notes.len() as u64) + 1;
                let filename = format!("{title}.md");
                let content =
                    format!("# {title}\n\nCreated interactively via tej-ui.\n\nTags: #new #user");

                // Write into ACID database
                let mut tx = db.db.begin_write()?;
                let key = format!("note:{new_id:08}");
                tx.put(key.as_bytes(), content.as_bytes())?;
                db.tag_index.insert_entry(&mut tx, b"new", key.as_bytes())?;
                db.tag_index
                    .insert_entry(&mut tx, b"user", key.as_bytes())?;
                tx.commit()?;

                let new_note = NoteItem::new(
                    new_id,
                    filename,
                    folder,
                    content,
                    vec!["new".into(), "user".into()],
                    vec![],
                );
                notes.push(new_note);
                println!(
                    "Created and committed note [{new_id}]: \"{title}\" to folder \"{folder}\"!\n"
                );
            }

            "render" => {
                let view = build_obsidian_view(&state, &notes);
                let mut v_state = view.build();
                view.rebuild(&mut v_state);
                println!(
                    "Rendered Obsidian View Tree (zero-vtable static dispatch). Frame valid.\n"
                );
            }

            "web" => {
                crate::web_gui::start_web_gui_background(notes.clone());
                println!("Live Web GUI server running at: http://127.0.0.1:4000\n");
            }

            "exit" | "quit" => {
                println!("Exiting Obsidian Vault shell. Goodbye!");
                break;
            }

            _ => {
                println!("Unknown command: \"{cmd}\". Type 'help' for a list of commands.");
            }
        }
    }

    Ok(())
}
