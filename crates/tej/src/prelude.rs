//! Ergonomic unified prelude for `tej-ui` developers.

pub use tej_core::{
    Effect, Epoch, Memo, NodeId, ReadSignal, Signal, SignalId, WriteSignal, batch, create_effect,
};

pub use tej_view::{
    Button, Canvas, CanvasPainter, CanvasState, Checkbox, EitherState, For, HStack, If, Input,
    SearchBar, StyleProps, Styled, StyledState, Text, VStack, View, ViewExt,
};

pub use tej_layout::{
    AlignItems, Dimension, FlexDirection, Insets, JustifyContent, LayoutArena, LayoutEngine,
    LayoutStyle, Point, Rect, Size,
};

pub use tej_text::{
    BidiClassifier, BidiRun, FontCategory, FontFallbackChain, FontWeight, GlyphCluster, ShapedRun,
    SystemFont, TextDirection, TextMeasurer, TextShaper,
};

pub use tej_hal::{
    Color, DirtyRectTracker, MonitorId, MonitorInfo, MonitorManager, MouseButton, PlatformWindow,
    SurfaceCompositor, UiEvent, WindowConfig, WindowId, WindowRegistry,
};

pub use tej_platform_linux::{LinuxSurface, LinuxWindow};
pub use tej_platform_macos::{
    MacosWindow, MacosWindowAttributes, MacosWindowStyle, MetalSurfaceCompositor,
};
pub use tej_platform_windows::{
    DirectCompositionSurface, DwmAttributes, WindowBackdrop, WindowsWindow,
};

pub use tej_db_sync::{DbSyncHook, ReactiveDbQuery, VirtualList, ZeroCopyRow};

pub use tej_widgets::{
    CommandAction, CommandItem, CommandPalette, DataGrid, DataGridDimensions, GridCellCoords,
    GridColumn, MenuItem, Modal, PinState, Popover, PopoverPlacement, SortDirection, TabItem, Tabs,
    ToastItem, ToastSeverity, TreeNode, TreeView, fuzzy_match,
};

pub use tej_services::{
    Clipboard, ClipboardData, Credential, CredentialStore, DragDropEvent, DropPayload, FileDialog,
    FileFilter, Modifiers, Shortcut, ShortcutHandler, ShortcutManager, TrayEvent, TrayIcon,
};

pub use crate::app::App;
