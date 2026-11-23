//! Obsidian command palette (`Cmd + K`) actions and fuzzy search.

use tej_ui_core::Signal;
use tej_ui_widgets::{CommandItem, CommandPalette};

/// Build the Obsidian command palette populated with vault navigation and action shortcuts.
#[must_use]
pub fn build_obsidian_palette(
    query: Signal<String>,
    selected_index: Signal<usize>,
) -> CommandPalette {
    let items = vec![
        CommandItem::new("open_graph", "View: Open 3D Knowledge Graph", "Navigation")
            .shortcut("Cmd+G"),
        CommandItem::new("new_note", "Notes: Create New Markdown Note", "Editor").shortcut("Cmd+N"),
        CommandItem::new("toggle_sidebar", "View: Toggle Vault Sidebar", "Navigation")
            .shortcut("Cmd+B"),
        CommandItem::new("search_vault", "Search: Find in Notes & Tags", "Search")
            .shortcut("Cmd+Shift+F"),
        CommandItem::new("db_stats", "Vault: Show ACID Database Statistics", "System")
            .shortcut("Cmd+D"),
        CommandItem::new("export_markdown", "File: Export Markdown to Disk", "File")
            .shortcut("Cmd+E"),
        CommandItem::new(
            "toggle_theme",
            "Settings: Toggle Dark / Light Theme",
            "Settings",
        )
        .shortcut("Cmd+T"),
        CommandItem::new(
            "spring_bounce",
            "Animation: Trigger Bouncy Spring Pulse",
            "Animation",
        )
        .shortcut("Cmd+P"),
    ];

    CommandPalette::new(query, selected_index, items)
}
