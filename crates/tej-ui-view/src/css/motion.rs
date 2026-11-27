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
#[derive(Clone, PartialEq, Debug)]
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

impl Default for MotionProps {
    fn default() -> Self {
        Self {
            transition_duration_ms: 0,
            transition_delay_ms: 0,
            transition_timing: TimingFunction::default(),
            transform_translate_x: 0.0,
            transform_translate_y: 0.0,
            transform_scale: 1.0,
            transform_rotate_deg: 0.0,
            transform_origin_x: 0.0,
            transform_origin_y: 0.0,
            animation_name: None,
            animation_duration_ms: 0,
            animation_delay_ms: 0,
            animation_timing: TimingFunction::default(),
            animation_iterations: 0,
            animation_direction: AnimationDirection::default(),
            animation_fill_mode: AnimationFillMode::default(),
            animation_play_state: AnimationPlayState::default(),
            perspective: None,
            transform_style: TransformStyle::default(),
            backface_visible: false,
            scroll_driven_progress: false,
        }
    }
}
