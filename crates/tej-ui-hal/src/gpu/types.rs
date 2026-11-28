//! Core GPU types, errors, uniform structures, and 128-byte instanced primitives.

use bytemuck::{Pod, Zeroable};
use std::fmt;

/// Errors arising during GPU initialization, surface management, or frame presentation.
#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    #[error("No compatible graphics adapter found")]
    AdapterNotFound,

    #[error("Failed to request logical GPU device: {0}")]
    DeviceRequestFailed(String),

    #[error("Surface creation failed: {0}")]
    SurfaceCreationFailed(String),

    #[error("Surface acquisition failed: {0}")]
    SurfaceAcquireFailed(String),

    #[error("Invalid dimensions for surface or texture: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },

    #[error("Invalid pixel buffer size: expected {expected} bytes, got {actual} bytes")]
    InvalidBufferSize { expected: usize, actual: usize },

    #[error("Atlas is full and could not allocate {width}x{height} glyph")]
    AtlasFull { width: u32, height: u32 },

    #[error("GPU readback failed: {0}")]
    ReadbackFailed(String),

    #[error("Clip stack underflow (attempted to pop the root viewport clip)")]
    ClipStackUnderflow,
}

/// GPU backend runtime configuration options.
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Preferred GPU power profile.
    pub power_preference: wgpu::PowerPreference,
    /// Whether to permit software-based fallback adapters (e.g. LLVMpipe, WARP).
    pub allow_software_fallback: bool,
    /// Presentation mode for windowed surfaces.
    pub present_mode: wgpu::PresentMode,
    /// Initial capacity (in quad instances) for the storage buffer.
    pub initial_instance_capacity: usize,
    /// Preferred texture format for targets.
    pub preferred_format: wgpu::TextureFormat,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::HighPerformance,
            allow_software_fallback: true,
            present_mode: wgpu::PresentMode::Fifo,
            initial_instance_capacity: 4096,
            preferred_format: wgpu::TextureFormat::Rgba8Unorm,
        }
    }
}

/// Converts a single normalized sRGB channel `[0.0, 1.0]` to linear light using
/// the IEC 61966-2-1 sRGB transfer function.
#[inline]
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Premultiplied linear RGBA color vector `[r, g, b, a]`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct PremulColor {
    pub channels: [f32; 4],
}

impl PremulColor {
    pub const TRANSPARENT: Self = Self {
        channels: [0.0, 0.0, 0.0, 0.0],
    };
    pub const BLACK: Self = Self {
        channels: [0.0, 0.0, 0.0, 1.0],
    };
    pub const WHITE: Self = Self {
        channels: [1.0, 1.0, 1.0, 1.0],
    };

    /// Create from straight 8-bit sRGB color channels, decoding RGB into linear space
    /// and premultiplying by linear alpha.
    #[must_use]
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        let alpha = f32::from(a) / 255.0;
        let r_f = srgb_to_linear(f32::from(r) / 255.0) * alpha;
        let g_f = srgb_to_linear(f32::from(g) / 255.0) * alpha;
        let b_f = srgb_to_linear(f32::from(b) / 255.0) * alpha;
        Self {
            channels: [r_f, g_f, b_f, alpha],
        }
    }

    /// Create from straight normalized f32 linear color channels.
    #[must_use]
    pub fn from_rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        let alpha = a.clamp(0.0, 1.0);
        Self {
            channels: [
                r.clamp(0.0, 1.0) * alpha,
                g.clamp(0.0, 1.0) * alpha,
                b.clamp(0.0, 1.0) * alpha,
                alpha,
            ],
        }
    }
}

/// Hardware primitive type recognized by the WGSL fragment shader.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveKind {
    /// Solid or rounded rectangle with anti-aliasing.
    Rect = 0,
    /// Rounded rectangle with independent borders.
    BorderedRect = 1,
    /// Analytic soft Gaussian drop shadow.
    DropShadow = 2,
}

/// Physical pixel scissor bounding box for hardware rasterizer clipping.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct ScissorRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ScissorRect {
    #[must_use]
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Computes intersection between two physical scissor boxes.
    #[must_use]
    pub fn intersect(&self, other: Self) -> Self {
        let x0 = self.x.max(other.x);
        let y0 = self.y.max(other.y);
        let x1 = (self.x.saturating_add(self.width)).min(other.x.saturating_add(other.width));
        let y1 = (self.y.saturating_add(self.height)).min(other.y.saturating_add(other.height));

        if x1 > x0 && y1 > y0 {
            Self {
                x: x0,
                y: y0,
                width: x1 - x0,
                height: y1 - y0,
            }
        } else {
            Self {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            }
        }
    }

    /// Clamps scissor rect to viewport dimensions.
    #[must_use]
    pub fn clamp_to_viewport(&self, vp_width: u32, vp_height: u32) -> Self {
        self.intersect(Self::new(0, 0, vp_width, vp_height))
    }
}

/// Viewport configuration passed as a 16-byte uniform buffer to shaders.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct ViewportUniform {
    /// Viewport size in physical pixels `[width, height]`.
    pub viewport_size: [f32; 2],
    /// Display scale factor (HiDPI DPI ratio).
    pub scale_factor: f32,
    /// Padding for 16-byte uniform alignment.
    pub pad: f32,
}

const _: () = assert!(core::mem::size_of::<ViewportUniform>() == 16);

/// Exactly 128-byte cache-line friendly instance primitive passed via GPU storage buffer.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct QuadInstance {
    /// Logical bounds `[x, y, width, height]`.
    pub bounds: [f32; 4],
    /// Logical secondary clip bounds `[x, y, width, height]`.
    pub clip_bounds: [f32; 4],
    /// Premultiplied fill color `[r, g, b, a]`.
    pub fill_color: [f32; 4],
    /// Premultiplied border color `[r, g, b, a]`.
    pub border_color: [f32; 4],
    /// Corner radii `[top_left, top_right, bottom_right, bottom_left]`.
    pub corner_radii: [f32; 4],
    /// Border widths `[top, right, bottom, left]` or `[blur_radius, spread, 0.0, 0.0]` for shadows.
    pub border_widths: [f32; 4],
    /// Clip rect corner radii `[top_left, top_right, bottom_right, bottom_left]`.
    pub clip_radii: [f32; 4],
    /// Metadata `[primitive_kind, flags, reserved0, reserved1]`.
    pub meta: [u32; 4],
}

const _: () = assert!(core::mem::size_of::<QuadInstance>() == 128);

impl QuadInstance {
    /// Construct a solid rectangle instance.
    #[must_use]
    pub fn solid_rect(bounds: [f32; 4], color: PremulColor) -> Self {
        Self {
            bounds,
            clip_bounds: [-1e6, -1e6, 2e6, 2e6],
            fill_color: color.channels,
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_widths: [0.0; 4],
            clip_radii: [0.0; 4],
            meta: [PrimitiveKind::Rect as u32, 0, 0, 0],
        }
    }

    /// Construct a rounded rectangle instance.
    #[must_use]
    pub fn rounded_rect(bounds: [f32; 4], radii: [f32; 4], color: PremulColor) -> Self {
        Self {
            bounds,
            clip_bounds: [-1e6, -1e6, 2e6, 2e6],
            fill_color: color.channels,
            border_color: [0.0; 4],
            corner_radii: radii,
            border_widths: [0.0; 4],
            clip_radii: [0.0; 4],
            meta: [PrimitiveKind::Rect as u32, 0, 0, 0],
        }
    }

    /// Construct a bordered rounded rectangle instance.
    #[must_use]
    pub fn bordered_rect(
        bounds: [f32; 4],
        radii: [f32; 4],
        fill: PremulColor,
        border: PremulColor,
        widths: [f32; 4],
    ) -> Self {
        Self {
            bounds,
            clip_bounds: [-1e6, -1e6, 2e6, 2e6],
            fill_color: fill.channels,
            border_color: border.channels,
            corner_radii: radii,
            border_widths: widths,
            clip_radii: [0.0; 4],
            meta: [PrimitiveKind::BorderedRect as u32, 0, 0, 0],
        }
    }

    /// Construct a soft drop shadow instance.
    #[must_use]
    pub fn drop_shadow(
        bounds: [f32; 4],
        radii: [f32; 4],
        color: PremulColor,
        blur_radius: f32,
        spread: f32,
    ) -> Self {
        Self {
            bounds,
            clip_bounds: [-1e6, -1e6, 2e6, 2e6],
            fill_color: color.channels,
            border_color: [0.0; 4],
            corner_radii: radii,
            border_widths: [blur_radius, spread, 0.0, 0.0],
            clip_radii: [0.0; 4],
            meta: [PrimitiveKind::DropShadow as u32, 0, 0, 0],
        }
    }
}

impl fmt::Display for PremulColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PremulColor({:.2}, {:.2}, {:.2}, {:.2})",
            self.channels[0], self.channels[1], self.channels[2], self.channels[3]
        )
    }
}
