//! Scrolling mechanics, pointer cursors, gestures, and user interaction.

use crate::events::CursorIcon;
use tej_ui_hal::Color;
use tej_ui_layout::{
    OverscrollBehavior, ResizeMode, ScrollSnapAlign, ScrollSnapStop, ScrollSnapType,
    ScrollbarGutter, ScrollbarWidth,
};

/// Overflow clipping and scrolling mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Clip,
    Scroll,
    Auto,
}

/// Scroll animation behavior on programmatic jumps or anchor navigation.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ScrollBehavior {
    #[default]
    Auto,
    Smooth,
}

/// Pointer event hit testing pass-through policy.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PointerEvents {
    #[default]
    Auto,
    None,
}

/// Scrolling, pointer, touch, and viewport interaction properties.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct InteractionProps {
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub scroll_behavior: ScrollBehavior,
    pub scroll_margin_top: f32,
    pub scroll_padding_top: f32,
    pub scroll_snap_type: ScrollSnapType,
    pub scroll_snap_align: ScrollSnapAlign,
    pub scroll_snap_stop: ScrollSnapStop,
    pub overscroll_behavior: OverscrollBehavior,
    pub scrollbar_width: ScrollbarWidth,
    pub scrollbar_gutter: ScrollbarGutter,
    pub scrollbar_thumb_color: Option<Color>,
    pub cursor: CursorIcon,
    pub pointer_events: PointerEvents,
    pub user_select: bool,
    pub touch_pan_x: bool,
    pub touch_pan_y: bool,
    pub resize: ResizeMode,
}
