//! `tej-widgets`: High-density enterprise component kit for `tej-ui`.
//!
//! Provides a complete shadcn-inspired suite of accessible, zero-allocation UI components:
//! - Data Display: `DataGrid` (2D virtualized), `CommandPalette` (Cmd+K).
//! - Forms: `Switch`, `Slider`, `RadioGroup`, `Select`.
//! - Layout: `Sidebar`, `Card`, `Separator`, `Accordion`, `ScrollArea`.
//! - Navigation: `Tabs`, `TreeView`, `Breadcrumb`, `Pagination`.
//! - Overlays: `Modal`, `Popover`, `ContextMenu`, `Sheet`, `ToastItem`.
//! - Feedback: `Badge`, `Avatar`, `Progress`, `Skeleton`, `Alert`, `Tooltip`.

pub mod command_palette;
pub mod datagrid;
pub mod feedback;
pub mod forms;
pub mod layout;
pub mod navigation;
pub mod overlays;

pub use command_palette::{
    CommandAction, CommandItem, CommandPalette, CommandPaletteState, FuzzyMatch, fuzzy_match,
};
pub use datagrid::{
    DataGrid, DataGridDimensions, DataGridState, GridCellCoords, GridColumn, PinState,
    SortDirection,
};
pub use feedback::{
    Alert, AlertSeverity, AlertState, Avatar, AvatarState, Badge, BadgeState, BadgeVariant,
    Progress, ProgressState, Skeleton, SkeletonState, Tooltip, TooltipState,
};
pub use forms::{
    RadioGroup, RadioGroupState, RadioItem, Select, SelectOption, SelectState, Slider, SliderState,
    Switch, SwitchState,
};
pub use layout::{
    Accordion, AccordionItem, AccordionState, Card, CardState, ScrollArea, ScrollAreaState,
    Separator, SeparatorOrientation, SeparatorState, Sidebar, SidebarItem, SidebarState,
};
pub use navigation::{
    Breadcrumb, BreadcrumbItem, BreadcrumbState, Pagination, PaginationState, TabItem, Tabs,
    TabsState, TreeNode, TreeView, TreeViewState,
};
pub use overlays::{
    MenuAction, MenuItem, Modal, ModalState, Popover, PopoverPlacement, PopoverState, Sheet,
    SheetSide, SheetState, ToastItem, ToastSeverity,
};
