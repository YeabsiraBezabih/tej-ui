use crate::command_palette::fuzzy::fuzzy_match;
use crate::command_palette::item::CommandItem;
use std::fmt::Debug;
use tej_ui_core::Signal;
use tej_ui_view::View;

/// Enterprise Command Palette (`Cmd + K`) overlay widget.
#[derive(Clone, Debug)]
pub struct CommandPalette {
    query: Signal<String>,
    selected_index: Signal<usize>,
    items: Vec<CommandItem>,
    max_visible: usize,
}

impl CommandPalette {
    /// Create a new command palette with reactive query and selection signals.
    pub fn new(
        query: Signal<String>,
        selected_index: Signal<usize>,
        items: Vec<CommandItem>,
    ) -> Self {
        Self {
            query,
            selected_index,
            items,
            max_visible: 10,
        }
    }

    /// Set maximum visible suggestions.
    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }

    /// Filter and rank items based on the active query string.
    #[must_use]
    pub fn filtered_items(&self) -> Vec<CommandItem> {
        let q = self.query.get();
        if q.is_empty() {
            return self.items.iter().take(self.max_visible).cloned().collect();
        }

        let mut scored: Vec<(i32, CommandItem)> = self
            .items
            .iter()
            .filter_map(|item| fuzzy_match(&q, &item.title).map(|m| (m.score, item.clone())))
            .collect();

        // Sort descending by score
        scored.sort_by_key(|a| std::cmp::Reverse(a.0));
        scored
            .into_iter()
            .take(self.max_visible)
            .map(|(_, item)| item)
            .collect()
    }
}

/// State of a mounted CommandPalette.
#[derive(Debug)]
pub struct CommandPaletteState {
    pub current_query: String,
    pub current_selection: usize,
    pub visible_items: Vec<CommandItem>,
}

impl View for CommandPalette {
    type State = CommandPaletteState;

    fn build(&self) -> Self::State {
        CommandPaletteState {
            current_query: self.query.get(),
            current_selection: self.selected_index.get(),
            visible_items: self.filtered_items(),
        }
    }

    fn rebuild(&self, state: &mut Self::State) {
        state.current_query = self.query.get();
        state.current_selection = self.selected_index.get();
        state.visible_items = self.filtered_items();
    }

    fn patch(&self, state: &mut Self::State, _dirty_bits: u64) {
        self.rebuild(state);
    }
}
