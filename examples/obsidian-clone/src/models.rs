//! Data models for the Obsidian clone vault, notes, and graph nodes.

/// An individual markdown note in the Obsidian vault.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NoteItem {
    pub id: u64,
    pub title: String,
    pub folder: String,
    pub content: String,
    pub tags: Vec<String>,
    pub outgoing_links: Vec<String>,
}

impl NoteItem {
    /// Create a new note item.
    #[must_use]
    pub fn new(
        id: u64,
        title: impl Into<String>,
        folder: impl Into<String>,
        content: impl Into<String>,
        tags: Vec<String>,
        outgoing_links: Vec<String>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            folder: folder.into(),
            content: content.into(),
            tags,
            outgoing_links,
        }
    }
}
