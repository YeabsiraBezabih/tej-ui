//! Interactive DataGrid column resizing, multi-column sorting, and inline cell editing coordinator.

use std::cmp::Ordering;
use std::fmt::Debug;

use crate::datagrid::column::SortDirection;

/// Individual column sorting criterion within a multi-column sort pipeline.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SortCriterion {
    pub column_id: String,
    pub direction: SortDirection,
}

/// Multi-column sorting coordinator tracking primary and secondary sort criteria.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct MultiColumnSorter {
    criteria: Vec<SortCriterion>,
}

impl MultiColumnSorter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Read active sorting criteria in priority order.
    #[must_use]
    pub fn criteria(&self) -> &[SortCriterion] {
        &self.criteria
    }

    /// Toggle or update sort for `column_id`.
    ///
    /// If `is_multi` (e.g. Shift-click) is true, appends/cycles in multi-sort hierarchy.
    /// Otherwise, clears prior criteria and sets this as the sole sort column.
    pub fn toggle_sort(&mut self, column_id: &str, is_multi: bool) {
        if !is_multi {
            let next_dir = self
                .criteria
                .iter()
                .find(|c| c.column_id == column_id)
                .map_or(SortDirection::Ascending, |c| c.direction.cycle());

            self.criteria.clear();
            if next_dir != SortDirection::None {
                self.criteria.push(SortCriterion {
                    column_id: column_id.to_string(),
                    direction: next_dir,
                });
            }
            return;
        }

        if let Some(pos) = self.criteria.iter().position(|c| c.column_id == column_id) {
            let next_dir = self.criteria[pos].direction.cycle();
            if next_dir == SortDirection::None {
                self.criteria.remove(pos);
            } else {
                self.criteria[pos].direction = next_dir;
            }
        } else {
            self.criteria.push(SortCriterion {
                column_id: column_id.to_string(),
                direction: SortDirection::Ascending,
            });
        }
    }

    /// Returns sort direction for a given column, if active.
    #[must_use]
    pub fn direction_for(&self, column_id: &str) -> SortDirection {
        self.criteria
            .iter()
            .find(|c| c.column_id == column_id)
            .map_or(SortDirection::None, |c| c.direction)
    }

    /// Compare two values across multi-column criteria with comparator callbacks.
    pub fn compare_records<T, F>(&self, a: &T, b: &T, mut cmp_fn: F) -> Ordering
    where
        F: FnMut(&str, &T, &T) -> Ordering,
    {
        for crit in &self.criteria {
            let ord = cmp_fn(&crit.column_id, a, b);
            let final_ord = match crit.direction {
                SortDirection::Ascending => ord,
                SortDirection::Descending => ord.reverse(),
                SortDirection::None => Ordering::Equal,
            };
            if final_ord != Ordering::Equal {
                return final_ord;
            }
        }
        Ordering::Equal
    }
}

/// Interactive column resizing state machine.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct ColumnResizeSession {
    pub column_index: usize,
    pub start_pointer_x: f32,
    pub initial_width: f32,
}

impl ColumnResizeSession {
    /// Calculate updated column width during an active drag, clamped to `min_width`.
    #[must_use]
    pub fn compute_new_width(&self, current_pointer_x: f32, min_width: f32) -> f32 {
        let delta = current_pointer_x - self.start_pointer_x;
        (self.initial_width + delta).max(min_width)
    }
}

/// Inline cell editor state tracking the active editing target and modified draft string.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct InlineCellEditor {
    pub row: usize,
    pub col: usize,
    pub draft_value: String,
    pub is_active: bool,
}

impl InlineCellEditor {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Open editor on targeted row and column with initial text.
    pub fn begin_edit(&mut self, row: usize, col: usize, initial_text: impl Into<String>) {
        self.row = row;
        self.col = col;
        self.draft_value = initial_text.into();
        self.is_active = true;
    }

    /// Commit the current edit draft and deactivate.
    pub fn commit(&mut self) -> Option<(usize, usize, String)> {
        if self.is_active {
            self.is_active = false;
            Some((self.row, self.col, std::mem::take(&mut self.draft_value)))
        } else {
            None
        }
    }

    /// Cancel the active edit discarding drafts.
    pub fn cancel(&mut self) {
        self.is_active = false;
        self.draft_value.clear();
    }
}
