//! `tej-widgets`: High-density enterprise component kit for `tej-ui`.
//!
//! Provides virtualized 2D `DataGrid`, `CommandPalette` (Cmd+K), `Modal`, `Popover`,
//! `ContextMenu`, `ToastItem`, `Tabs`, and `TreeView`.

pub mod command_palette;
pub mod datagrid;
pub mod navigation;
pub mod overlays;

pub use command_palette::{
    CommandAction, CommandItem, CommandPalette, CommandPaletteState, FuzzyMatch, fuzzy_match,
};
pub use datagrid::{
    DataGrid, DataGridDimensions, DataGridState, GridCellCoords, GridColumn, PinState,
    SortDirection,
};
pub use navigation::{TabItem, Tabs, TabsState, TreeNode, TreeView, TreeViewState};
pub use overlays::{
    MenuAction, MenuItem, Modal, ModalState, Popover, PopoverPlacement, PopoverState, ToastItem,
    ToastSeverity,
};
