//! `tej-animation`: Velocity-based spring physics and reactive transition engine.
//!
//! Provides physically modeled spring configurations, numerical symplectic Euler integrators,
//! `Lerp` interpolation traits, VSync tickers, and reactive `AnimatedSpring` signals.

pub mod animated;
pub mod interpolator;
pub mod morph;
pub mod spring;
pub mod ticker;

pub use animated::{AnimatedSpring, AnimatedValue};
pub use interpolator::Lerp;
pub use morph::{KeyframeSequence, SharedElementTransition};
pub use spring::{SpringConfig, SpringSimulation};
pub use ticker::VsyncTicker;
