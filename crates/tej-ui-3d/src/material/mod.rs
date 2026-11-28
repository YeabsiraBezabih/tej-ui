//! 3D Lighting and Material properties.

pub mod pbr;

use crate::math::Vec3;
pub use pbr::{PbrMaterialParams, SphericalHarmonics9, pbr_shade_fragment};
use tej_ui_hal::Color;

/// Light type in the 3D scene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightKind {
    /// Ambient uniform light in all directions.
    Ambient,
    /// Directional sunlight.
    Directional { direction: Vec3 },
    /// Point omni-light with position and falloff range.
    Point { position: Vec3, range: f32 },
}

/// Light source in the 3D scene.
#[derive(Debug, Clone, PartialEq)]
pub struct Light {
    pub kind: LightKind,
    pub color: Color,
    pub intensity: f32,
}

impl Light {
    /// Creates an ambient light.
    #[must_use]
    pub fn ambient(color: Color, intensity: f32) -> Self {
        Self {
            kind: LightKind::Ambient,
            color,
            intensity,
        }
    }

    /// Creates a directional sunlight.
    #[must_use]
    pub fn directional(direction: Vec3, color: Color, intensity: f32) -> Self {
        Self {
            kind: LightKind::Directional {
                direction: direction.normalize(),
            },
            color,
            intensity,
        }
    }
}

/// Physically-inspired standard material for shaded 3D meshes.
#[derive(Debug, Clone, PartialEq)]
pub struct StandardMaterial {
    /// Base albedo tint color.
    pub color: Color,
    /// Surface roughness [0.0 = glossy mirror, 1.0 = completely diffuse].
    pub roughness: f32,
    /// Surface metalness [0.0 = dielectric / plastic / wood, 1.0 = pure metal].
    pub metalness: f32,
    /// Whether to render in wireframe mode.
    pub wireframe: bool,
}

impl Default for StandardMaterial {
    fn default() -> Self {
        Self {
            color: Color::rgb(220, 220, 230),
            roughness: 0.5,
            metalness: 0.1,
            wireframe: false,
        }
    }
}

impl StandardMaterial {
    /// Creates a new standard material with base color.
    #[must_use]
    pub fn new(color: Color) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }

    /// Sets roughness.
    #[must_use]
    pub const fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness;
        self
    }

    /// Sets metalness.
    #[must_use]
    pub const fn with_metalness(mut self, metalness: f32) -> Self {
        self.metalness = metalness;
        self
    }

    /// Enables wireframe rendering.
    #[must_use]
    pub const fn wireframe(mut self) -> Self {
        self.wireframe = true;
        self
    }

    /// Uses filled solid rendering.
    #[must_use]
    pub const fn solid(mut self) -> Self {
        self.wireframe = false;
        self
    }
}
