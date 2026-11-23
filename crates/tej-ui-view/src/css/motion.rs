//! Transitions, keyframe animations, 2D/3D GPU transforms, and scroll timelines.

use super::types::TimingFunction;
use std::borrow::Cow;

/// Play direction of keyframe animation cycles.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AnimationDirection {
    #[default]
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

/// How keyframe styles apply before and after execution.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AnimationFillMode {
    #[default]
    None,
    Forwards,
    Backwards,
    Both,
}

/// Active execution state of an animation.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AnimationPlayState {
    #[default]
    Running,
    Paused,
}

/// 3D transform projection style.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TransformStyle {
    #[default]
    Flat,
    Preserve3D,
}

/// Motion, transition, transform, and keyframe animation properties.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct MotionProps {
    pub transition_duration_ms: u32,
    pub transition_delay_ms: u32,
    pub transition_timing: TimingFunction,
    pub transform_translate_x: f32,
    pub transform_translate_y: f32,
    pub transform_scale: f32,
    pub transform_rotate_deg: f32,
    pub transform_origin_x: f32,
    pub transform_origin_y: f32,
    pub animation_name: Option<Cow<'static, str>>,
    pub animation_duration_ms: u32,
    pub animation_delay_ms: u32,
    pub animation_timing: TimingFunction,
    pub animation_iterations: u32,
    pub animation_direction: AnimationDirection,
    pub animation_fill_mode: AnimationFillMode,
    pub animation_play_state: AnimationPlayState,
    pub perspective: Option<f32>,
    pub transform_style: TransformStyle,
    pub backface_visible: bool,
    pub scroll_driven_progress: bool,
}
