/// Pinning alignment for a DataGrid column.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PinState {
    #[default]
    None,
    Left,
    Right,
}

/// Sort direction of a column header.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SortDirection {
    #[default]
    None,
    Ascending,
    Descending,
}

impl SortDirection {
    /// Cycle to the next sort direction: None -> Ascending -> Descending -> None.
    #[must_use]
    pub const fn cycle(self) -> Self {
        match self {
            Self::None => Self::Ascending,
            Self::Ascending => Self::Descending,
            Self::Descending => Self::None,
        }
    }
}

/// Column descriptor for the enterprise DataGrid.
#[derive(Clone, Debug)]
pub struct GridColumn {
    pub id: String,
    pub title: String,
    pub width: f32,
    pub min_width: f32,
    pub pin: PinState,
    pub sort: SortDirection,
}

impl GridColumn {
    /// Create a new column with ID and title.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            width: 150.0,
            min_width: 60.0,
            pin: PinState::None,
            sort: SortDirection::None,
        }
    }

    /// Set column width.
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(self.min_width);
        self
    }

    /// Pin column to Left or Right side of viewport.
    #[must_use]
    pub fn pin(mut self, pin: PinState) -> Self {
        self.pin = pin;
        self
    }

    /// Set initial sort direction.
    #[must_use]
    pub fn sort(mut self, sort: SortDirection) -> Self {
        self.sort = sort;
        self
    }
}
