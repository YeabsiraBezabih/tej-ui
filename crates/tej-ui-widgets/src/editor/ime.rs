//! Native Input Method Editor (IME) composition state.

/// Tracks native OS Input Method Editor (IME) composition session state.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ImeState {
    /// True if an active IME composition session is underway.
    pub is_composing: bool,
    /// Uncommitted intermediate composition string displayed inline or in popup.
    pub preedit: String,
    /// Character offset of the cursor within the preedit string.
    pub cursor_in_preedit: usize,
    /// Candidate suggestions provided by native input engine.
    pub candidates: Vec<String>,
    /// Index of currently highlighted candidate.
    pub selected_candidate: Option<usize>,
}

impl ImeState {
    /// Create a new inactive IME state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Begin a new IME composition session.
    pub fn start(&mut self) {
        self.is_composing = true;
        self.preedit.clear();
        self.cursor_in_preedit = 0;
        self.candidates.clear();
        self.selected_candidate = None;
    }

    /// Update preedit text and cursor offset from native OS IME event.
    pub fn update(&mut self, preedit: String, cursor: usize) {
        self.is_composing = true;
        self.cursor_in_preedit = cursor.min(preedit.chars().count());
        self.preedit = preedit;
    }

    /// Update candidate suggestions.
    pub fn set_candidates(&mut self, candidates: Vec<String>, selected: Option<usize>) {
        self.candidates = candidates;
        self.selected_candidate = selected;
    }

    /// Commit the current composition string or selected candidate, returning the text to insert.
    pub fn commit(&mut self) -> String {
        self.is_composing = false;
        let text = if let Some(idx) = self.selected_candidate {
            if idx < self.candidates.len() {
                self.candidates[idx].clone()
            } else {
                std::mem::take(&mut self.preedit)
            }
        } else {
            std::mem::take(&mut self.preedit)
        };

        self.cursor_in_preedit = 0;
        self.candidates.clear();
        self.selected_candidate = None;
        text
    }

    /// Abort active composition session, discarding any uncommitted preedit text.
    pub fn cancel(&mut self) {
        self.is_composing = false;
        self.preedit.clear();
        self.cursor_in_preedit = 0;
        self.candidates.clear();
        self.selected_candidate = None;
    }
}
