//! Reactive bindings for OS accessibility preferences.
//!
//! Exposes reactive signals for platform user preferences (Reduced Motion, High Contrast,
//! Dynamic Type scaling, and Invert Colors) enabling instantaneous UI adaptation.

use tej_ui_core::Signal;

/// Reactive container tracking platform operating system accessibility preferences.
#[derive(Debug, Clone, Copy)]
pub struct A11yPreferences {
    /// True when user requests minimal or no animations (prefers-reduced-motion).
    pub reduced_motion: Signal<bool>,
    /// True when system High Contrast mode is active.
    pub high_contrast: Signal<bool>,
    /// Dynamic Type font scaling factor (1.0 = 100%, 1.25 = 125%, etc.).
    pub dynamic_type_scale: Signal<f32>,
    /// True when system Invert Colors is active.
    pub invert_colors: Signal<bool>,
}

impl Default for A11yPreferences {
    fn default() -> Self {
        Self::standard()
    }
}

impl A11yPreferences {
    /// Creates default standard preferences (no reduced motion, standard contrast, 1.0x font scaling).
    #[must_use]
    pub fn standard() -> Self {
        Self {
            reduced_motion: Signal::new(false),
            high_contrast: Signal::new(false),
            dynamic_type_scale: Signal::new(1.0),
            invert_colors: Signal::new(false),
        }
    }

    /// Read whether reduced motion is requested.
    #[must_use]
    pub fn is_reduced_motion(&self) -> bool {
        self.reduced_motion.get()
    }

    /// Read whether high contrast mode is requested.
    #[must_use]
    pub fn is_high_contrast(&self) -> bool {
        self.high_contrast.get()
    }

    /// Read current dynamic type scale factor.
    #[must_use]
    pub fn font_scale(&self) -> f32 {
        self.dynamic_type_scale.get().max(0.5)
    }

    /// Scales a base font size according to the active Dynamic Type scale factor.
    #[must_use]
    pub fn scale_font_size(&self, base_size: f32) -> f32 {
        (base_size * self.font_scale()).round()
    }

    /// Read whether color inversion is requested.
    #[must_use]
    pub fn is_invert_colors(&self) -> bool {
        self.invert_colors.get()
    }

    /// Enable reduced motion preference.
    pub fn enable_reduced_motion(&self) {
        self.reduced_motion.set(true);
    }

    /// Disable reduced motion preference.
    pub fn disable_reduced_motion(&self) {
        self.reduced_motion.set(false);
    }

    /// Enable high contrast preference.
    pub fn enable_high_contrast(&self) {
        self.high_contrast.set(true);
    }

    /// Disable high contrast preference.
    pub fn disable_high_contrast(&self) {
        self.high_contrast.set(false);
    }

    /// Update dynamic type scale factor.
    pub fn set_dynamic_type_scale(&self, scale: f32) {
        self.dynamic_type_scale.set(scale.max(0.5));
    }

    /// Enable color inversion preference.
    pub fn enable_invert_colors(&self) {
        self.invert_colors.set(true);
    }

    /// Disable color inversion preference.
    pub fn disable_invert_colors(&self) {
        self.invert_colors.set(false);
    }
}
