//! `tej-widgets`: High-density enterprise component kit for `tej-ui`.
//!
//! Provides a complete shadcn-inspired suite of accessible, zero-allocation UI components:
//! - Data Display: `DataGrid` (2D virtualized), `CommandPalette` (Cmd+K).
//! - Forms: `Switch`, `Slider`, `RadioGroup`, `Select`.
//! - Layout: `Sidebar`, `Card`, `Separator`, `Accordion`, `ScrollArea`.
//! - Navigation: `Tabs`, `TreeView`, `Breadcrumb`, `Pagination`.
//! - Overlays: `Modal`, `Popover`, `ContextMenu`, `Sheet`, `ToastItem`.
//! - Feedback: `Badge`, `Avatar`, `Progress`, `Skeleton`, `Alert`, `Tooltip`.
//! - Text Editing: `RichTextEditor`, multi-line `TextBuffer`, undo/redo, and native `ImeState`.

pub mod actions;
pub mod ai;
pub mod command_palette;
pub mod creative;
pub mod datagrid;
pub mod devtools;
pub mod editor;
pub mod feedback;
pub mod forms;
pub mod layout;
pub mod media;
pub mod navigation;
pub mod overlays;
pub mod pickers;
pub mod viz;
pub mod workspace;

pub use actions::{
    ConfirmButtonState, EmojiReaction, OtpInput, OtpInputState, ReactionBar, ReactionBarState,
    ScrubFieldState, SpeedDialAction, SpeedDialState, SplitButton,
};
pub use ai::{
    AgentPlanCard, AgentPlanState, ComposerState, ContextMeter, ContextMeterState, ModelChoice,
    PlanStep, PromptComposer, ReasoningDrawer, ReasoningDrawerState, SourceCitation, StepStatus,
    ToolCallState, ToolCallTray, ToolExecutionStatus,
};
pub use creative::{
    AspectRatioPicker, AspectRatioPreset, ColorStop, GradientEditorState, HistogramSlider,
    HistogramSliderState, KnobState, RotaryKnob,
};
pub use devtools::{
    DiagnosticProblem, DiagnosticSeverity, FlameGraph, FlameGraphState, FlameNode, GitCommit,
    GitGraph, GitGraphState, KeyValueEntry, PipelineGraph, PipelineStage, PipelineState,
    PipelineStatus, RegexMatch, RegexTester, RegexTesterState, TimingPhase, TraceSpan,
    TraceWaterfall, TraceWaterfallState, UptimeBar, UptimeDay,
};
pub use media::{
    AudioWaveform, AudioWaveformState, CarouselState, Dropzone, DropzoneState, ImageCompare,
    ImageCompareState, MetricCardState, MetricTrend, SignaturePadState, SignatureStroke,
    SpotlightCard, StrokePoint, VoiceOrbState,
};
pub use viz::{
    ActivityRingsState, CalendarHeatmap, CalendarHeatmapState, CandlestickChart, CandlestickPoint,
    CandlestickState, DonutSlice, DotMatrixDisplay, FunnelStage, HeatmapCell, RadialGauge,
    RingMetric, SankeyDiagram, SankeyLink, SankeyNode, SankeyState, Treemap, TreemapNode,
    TreemapState,
};
pub use workspace::{
    DockManager, DockManagerState, DockNode, DockSplitOrientation, DockTab, GanttChart, GanttState,
    GanttTask, GraphNode, KanbanBoard, KanbanCard, KanbanColumn, KanbanState, Keyframe,
    NodeConnection, NodeEditor, NodeEditorState, NodePin, TimelineEditor, TimelineState,
    TimelineTrack,
};

pub use command_palette::{
    CommandAction, CommandItem, CommandPalette, CommandPaletteState, FuzzyMatch, fuzzy_match,
};
pub use datagrid::{
    ColumnResizeSession, DataGrid, DataGridDimensions, DataGridState, GridCellCoords, GridColumn,
    InlineCellEditor, MultiColumnSorter, PinState, SortCriterion, SortDirection,
};

pub use editor::{
    CursorPos, EditAction, ImeState, MultiCursorSelection, RichTextEditor, RichTextEditorState,
    TextBuffer, TextSelection, UndoStack,
};

pub use feedback::{
    Alert, AlertSeverity, AlertState, Avatar, AvatarState, Badge, BadgeState, BadgeVariant,
    Progress, ProgressState, Skeleton, SkeletonState, Tooltip, TooltipState,
};
pub use forms::{
    FormController, FormField, InputMask, MaskToken, MaskedFormField, RadioGroup, RadioGroupState,
    RadioItem, Select, SelectOption, SelectState, Slider, SliderState, Switch, SwitchState, email,
    max_length, min_length, numeric_range, required, required_option,
};

pub use layout::{
    Accordion, AccordionItem, AccordionState, Card, CardState, ScrollArea, ScrollAreaState,
    Separator, SeparatorOrientation, SeparatorState, Sidebar, SidebarItem, SidebarState,
    SplitDividerDrag, SplitOrientation, SplitPaneController,
};
pub use navigation::{
    Breadcrumb, BreadcrumbItem, BreadcrumbState, FlatTreeItem, GuardResult, Pagination,
    PaginationState, RouteDefinition, RouteGuard, RouteMatch, Router, TabItem, Tabs, TabsState,
    TreeMultiSelection, TreeNode, TreeView, TreeViewState, VirtualizedTreeIndex,
};

pub use overlays::{
    MenuAction, MenuItem, Modal, ModalState, Popover, PopoverPlacement, PopoverState, Sheet,
    SheetSide, SheetState, ToastItem, ToastSeverity,
};
pub use pickers::{CalendarDate, CalendarPickerState, ColorHsv, DateRange};
