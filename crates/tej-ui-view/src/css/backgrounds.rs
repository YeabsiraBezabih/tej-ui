//! Background fills, image tiling, gradients, clipping, and alpha masking.

use tej_ui_hal::Color;

/// Scaling mode for background images.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BackgroundSize {
    #[default]
    Auto,
    Cover,
    Contain,
}

/// Repetition tiling mode for background images.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BackgroundRepeat {
    #[default]
    Repeat,
    NoRepeat,
    RepeatX,
    RepeatY,
}

/// Boundary box for background painting or clipping.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BoxClip {
    #[default]
    BorderBox,
    PaddingBox,
    ContentBox,
    Text,
}

/// Background fills, image sizing, and masking properties.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct BackgroundProps {
    pub background_color: Option<Color>,
    pub background_size: BackgroundSize,
    pub background_repeat: BackgroundRepeat,
    pub background_clip: BoxClip,
    pub background_origin: BoxClip,
    pub mask_image_gradient: bool,
    pub mask_size: BackgroundSize,
    pub mask_repeat: BackgroundRepeat,
    pub clip_path_inset: Option<[f32; 4]>,
}
