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
    Accordion, AccordionItem, Alert, AlertSeverity, Avatar, Badge, BadgeVariant, Breadcrumb,
    BreadcrumbItem, Card, CommandAction, CommandItem, CommandPalette, DataGrid, DataGridDimensions,
    GridCellCoords, GridColumn, MenuItem, Modal, Pagination, PinState, Popover, PopoverPlacement,
    Progress, RadioGroup, RadioItem, ScrollArea, Select, SelectOption, Separator,
    SeparatorOrientation, Sheet, SheetSide, Sidebar, SidebarItem, Skeleton, Slider, SortDirection,
    Switch, TabItem, Tabs, ToastItem, ToastSeverity, Tooltip, TreeNode, TreeView, fuzzy_match,
};

pub use tej_services::{
    Clipboard, ClipboardData, Credential, CredentialStore, DragDropEvent, DropPayload, FileDialog,
    FileFilter, Modifiers, Shortcut, ShortcutHandler, ShortcutManager, TrayEvent, TrayIcon,
};

pub use tej_devtools::{
    DagNodeInfo, DagSnapshot, DirtyHudOverlay, FrameMetrics, HotReloadHost, ReactiveDagInspector,
};

pub use tej_3d::{
    Canvas3D, Canvas3DState, GpuTextureDescriptor, GpuTextureFormat, Light, LightKind, Mat4, Mesh,
    Object3D, OrbitControls, PerspectiveCamera, Rasterizer3D, Ray, Scene, StandardMaterial,
    TextureHandle, Transform3D, Vec3, Vertex,
};

pub use crate::app::App;
