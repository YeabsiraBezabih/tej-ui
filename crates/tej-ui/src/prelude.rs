//! Ergonomic unified prelude for `tej-ui` developers.

pub use tej_ui_core::{
    Effect, Epoch, Memo, NodeId, ReadSignal, Resource, ResourceState, Signal, SignalId, Store,
    WriteSignal, batch, clear_context_stack, create_effect, provide_context, use_context,
    with_context,
};

pub use tej_ui_view::{
    Button, ButtonSize, ButtonVariant, Canvas, CanvasPainter, CanvasState, CaretStyle, Checkbox,
    ColorTokens, ContextProvider, CssStyleSheet, CursorIcon, EitherState, ElevationTokens,
    EventDispatchResult, EventDispatcher, FillRule, FontVariantNumeric, For, Grid, HStack, If,
    Input, PaintContext, RadiiTokens, SearchBar, Shadow, SpacingTokens, StrokeLineCap,
    StrokeLineJoin, StyleProps, Styled, StyledState, SvgPaintStyle, TOTAL_CSS_PROPERTIES,
    TestHarness, TestNode, Text, TextStyle, TextUnderlineStyle, TextWrap, Theme, TypographyTokens,
    VStack, VectorEffect, View, ViewExt, provide_theme, use_theme,
};

pub use tej_ui_layout::{
    AlignItems, AnchorPositionArea, Breakpoint, ContainerQuery, Dimension, DisplayMode,
    FlexDirection, FlexWrap, GridAutoFlow, Insets, JustifyContent, JustifyItems, JustifySelf,
    LayoutArena, LayoutEngine, LayoutStyle, LogicalCornerRadii, LogicalInsets, LogicalSize,
    OverscrollBehavior, PlaceContent, Point, Rect, ResizeMode, ScrollSnapAlign, ScrollSnapStop,
    ScrollSnapType, ScrollbarGutter, ScrollbarWidth, Size,
};

pub use tej_ui_text::{
    BidiClassifier, BidiRun, FontCategory, FontFallbackChain, FontWeight, GlyphCluster, ShapedRun,
    SystemFont, TextDirection, TextMeasurer, TextShaper,
};

pub use tej_ui_hal::{
    Color, DirtyRectTracker, MonitorId, MonitorInfo, MonitorManager, MouseButton, PlatformWindow,
    SurfaceCompositor, UiEvent, WindowConfig, WindowId, WindowRegistry,
};

pub use tej_ui_platform_linux::{LinuxSurface, LinuxWindow};
pub use tej_ui_platform_macos::{
    MacosWindow, MacosWindowAttributes, MacosWindowStyle, MetalSurfaceCompositor,
};
pub use tej_ui_platform_web::{WebEventBridge, WebSurfaceCompositor, WebWindow};
pub use tej_ui_platform_windows::{
    DirectCompositionSurface, DwmAttributes, WindowBackdrop, WindowsWindow,
};

#[cfg(feature = "a11y")]
pub use tej_ui_a11y::{
    A11yAction, A11yBridge, A11yBridgeEvent, A11yId, A11yNode, A11yPreferences, A11yRole, A11yTree,
    AccessKitBridge, Announcement, AnnouncementPoliteness, AnnouncementQueue, HeadlessA11yAdapter,
    PlatformA11yAdapter, WebAriaAdapter, WebAriaShadowNode,
};

#[cfg(feature = "db-sync")]
pub use tej_ui_db_sync::{
    CowTimeTravelStack, DbRowLens, DbSyncHook, DbVirtualWindow, DirectRleFilter, FilterBitset,
    PaletteItem, PaletteSearchResult, ReactiveDbQuery, RleColumn, RleRun, SemanticPalette,
    VirtualList, ZeroCopyRow,
};

#[cfg(feature = "widgets")]
pub use tej_ui_widgets::{
    Accordion, AccordionItem, Alert, AlertSeverity, Avatar, Badge, BadgeVariant, Breadcrumb,
    BreadcrumbItem, Card, CommandAction, CommandItem, CommandPalette, CursorPos, DataGrid,
    DataGridDimensions, EditAction, FormController, FormField, GridCellCoords, GridColumn,
    ImeState, MenuItem, Modal, Pagination, PinState, Popover, PopoverPlacement, Progress,
    RadioGroup, RadioItem, RichTextEditor, RichTextEditorState, ScrollArea, Select, SelectOption,
    Separator, SeparatorOrientation, Sheet, SheetSide, Sidebar, SidebarItem, Skeleton, Slider,
    SortDirection, Switch, TabItem, Tabs, TextBuffer, TextSelection, ToastItem, ToastSeverity,
    Tooltip, TreeNode, TreeView, UndoStack, email, fuzzy_match, max_length, min_length,
    numeric_range, required, required_option,
};

#[cfg(feature = "services")]
pub use tej_ui_services::{
    Clipboard, ClipboardData, Credential, CredentialStore, DragDropEvent, DropPayload, FileDialog,
    FileFilter, Modifiers, Shortcut, ShortcutHandler, ShortcutManager, TrayEvent, TrayIcon,
};

#[cfg(feature = "devtools")]
pub use tej_ui_devtools::{
    DagNodeInfo, DagSnapshot, DirtyHudOverlay, FrameMetrics, HotReloadHost, ReactiveDagInspector,
};

#[cfg(feature = "3d")]
pub use tej_ui_3d::{
    BlenderMesh, CameraPreset, Canvas3D, Canvas3DState, GpuTextureDescriptor, GpuTextureFormat,
    Light, LightKind, Mat4, Mesh, MeshletPacker, MockGpuUploader, NaniteFrameStats,
    NaniteGizmoManager, NaniteRenderEngine, NaniteStreamer, Object3D, OrbitControls, PackedMeshlet,
    PerspectiveCamera, Rasterizer3D, Ray, Scene, StandardMaterial, TextureHandle, Transform3D,
    Vec3, Vertex, cluster_debug_color, load_obj_from_file, parse_obj,
};

#[cfg(feature = "icons")]
pub use tej_ui_icons::{
    ActionIcon, CommunicationIcon, DeviceIcon, EditorIcon, Icon, IconSource, IconState, LucideIcon,
    MediaIcon, NavigationIcon, PathCommand, SystemIcon, VectorPath,
};

#[cfg(feature = "animation")]
pub use tej_ui_animation::{
    AnimatedSpring, AnimatedValue, Lerp, SpringConfig, SpringSimulation, VsyncTicker,
};

pub use crate::app::App;
