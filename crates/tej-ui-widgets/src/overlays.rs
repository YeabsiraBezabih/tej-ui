//! Overlay components: Modal, Popover, Toast, ContextMenu, Sheet, and Collision-aware engine.

pub mod collision;
pub mod context_menu;
pub mod modal;
pub mod popover;
pub mod sheet;
pub mod toast;

pub use collision::{
    CollisionPlacementOptions, OverlayPlacementResult, PopoverAlignment,
    compute_collision_placement,
};
pub use context_menu::{MenuAction, MenuItem};
pub use modal::{Modal, ModalState};
pub use popover::{Popover, PopoverPlacement, PopoverState};
pub use sheet::{Sheet, SheetSide, SheetState};
pub use toast::{ToastItem, ToastSeverity};
