//! Shared element transitions, layout morphing, and keyframe coordination.

use crate::interpolator::Lerp;
use crate::spring::{SpringConfig, SpringSimulation};
use tej_ui_layout::Rect;

/// A shared element layout transition between a source bounding box and a target bounding box.
#[derive(Debug, Clone)]
pub struct SharedElementTransition {
    id: String,
    source_rect: Rect,
    target_rect: Rect,
    spring: SpringSimulation,
    is_active: bool,
}

impl SharedElementTransition {
    /// Creates a shared element transition with given id, starting from source rect to target rect.
    pub fn new(
        id: impl Into<String>,
        source_rect: Rect,
        target_rect: Rect,
        config: SpringConfig,
    ) -> Self {
        let mut spring = SpringSimulation::new(0.0, config);
        spring.set_target(1.0);
        Self {
            id: id.into(),
            source_rect,
            target_rect,
            spring,
            is_active: true,
        }
    }

    /// Identifier of the shared transition element.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Advances the transition by delta time in seconds.
    pub fn step(&mut self, dt_seconds: f32) {
        if self.is_active {
            self.spring.step(dt_seconds);
            if self.spring.is_at_rest() {
                self.is_active = false;
            }
        }
    }

    /// Current progress from 0.0 (source) to 1.0 (target).
    pub fn progress(&self) -> f32 {
        self.spring.current
    }

    /// Computes the current interpolated morphed bounding box.
    pub fn current_rect(&self) -> Rect {
        let t = self.progress().clamp(0.0, 1.0);
        Rect::new(
            self.source_rect
                .origin
                .x
                .lerp(&self.target_rect.origin.x, t),
            self.source_rect
                .origin
                .y
                .lerp(&self.target_rect.origin.y, t),
            self.source_rect
                .size
                .width
                .lerp(&self.target_rect.size.width, t),
            self.source_rect
                .size
                .height
                .lerp(&self.target_rect.size.height, t),
        )
    }

    /// True if the animation has settled at rest.
    pub fn is_completed(&self) -> bool {
        !self.is_active
    }
}

/// Keyframe transition coordinator supporting discrete easing curves.
#[derive(Debug, Clone)]
pub struct KeyframeSequence<T: Lerp + Clone> {
    keyframes: Vec<(f32, T)>,
}

impl<T: Lerp + Clone> KeyframeSequence<T> {
    /// Creates an empty keyframe sequence.
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
        }
    }

    /// Inserts a keyframe at time `t` (0.0 to 1.0).
    pub fn add_keyframe(mut self, t: f32, value: T) -> Self {
        self.keyframes.push((t.clamp(0.0, 1.0), value));
        self.keyframes
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        self
    }

    /// Evaluates the keyframe sequence at normalized progress `t` (0.0 to 1.0).
    pub fn evaluate(&self, t: f32) -> Option<T> {
        if self.keyframes.is_empty() {
            return None;
        }
        let t = t.clamp(0.0, 1.0);

        if t <= self.keyframes[0].0 {
            return Some(self.keyframes[0].1.clone());
        }
        if t >= self.keyframes.last().map(|k| k.0).unwrap_or(1.0) {
            return self.keyframes.last().map(|k| k.1.clone());
        }

        for window in self.keyframes.windows(2) {
            let (t0, v0) = &window[0];
            let (t1, v1) = &window[1];
            if t >= *t0 && t <= *t1 {
                let range = t1 - t0;
                let local_t = if range.abs() < 1e-6 {
                    0.0
                } else {
                    (t - t0) / range
                };
                return Some(v0.lerp(v1, local_t));
            }
        }

        self.keyframes.last().map(|k| k.1.clone())
    }
}

impl<T: Lerp + Clone> Default for KeyframeSequence<T> {
    fn default() -> Self {
        Self::new()
    }
}
