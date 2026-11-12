//! Ergonomic unified prelude for `tej-ui` developers.

pub use tej_core::{
    Effect, Epoch, Memo, NodeId, ReadSignal, Signal, SignalId, WriteSignal, batch, create_effect,
};

pub use tej_view::{
    Button, Canvas, CanvasPainter, CanvasState, Checkbox, EitherState, For, HStack, If, Input,
    SearchBar, Text, VStack, View,
};

pub use tej_layout::{
    AlignItems, Dimension, FlexDirection, Insets, JustifyContent, LayoutArena, LayoutEngine,
    LayoutStyle, Point, Rect, Size,
};

pub use tej_text::{FontWeight, SystemFont, TextMeasurer};

pub use tej_hal::{
    Color, DirtyRectTracker, MouseButton, PlatformWindow, SurfaceCompositor, UiEvent, WindowConfig,
    WindowId,
};

pub use tej_db_sync::{DbSyncHook, ReactiveDbQuery, VirtualList, ZeroCopyRow};

pub use crate::app::App;
