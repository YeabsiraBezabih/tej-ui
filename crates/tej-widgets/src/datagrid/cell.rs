/// Coordinates identifying a cell location inside the 2D DataGrid.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct GridCellCoords {
    pub row: usize,
    pub col: usize,
}

impl GridCellCoords {
    /// Create cell coordinates for row and column index.
    #[inline]
    #[must_use]
    pub const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}
