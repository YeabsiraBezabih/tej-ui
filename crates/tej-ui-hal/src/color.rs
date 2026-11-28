//! High-fidelity color representations, wide gamut color spaces (Display P3, sRGB),
//! and physically correct linear-space alpha blending.

/// Color space specification for rendering pipelines and ICC profile conversions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorSpace {
    /// Standard Red-Green-Blue (IEC 61966-2-1:1999) with gamma ~2.2.
    #[default]
    Srgb,
    /// Apple Wide Color Gamut Display P3 (DCI-P3 primaries with D65 white point).
    DisplayP3,
    /// Radiometrically linear RGB color space for accurate optical blending without dark fringes.
    LinearRgb,
}

/// Floating-point linear-space color with normalized [0.0, 1.0] channels.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct ColorLinear {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorLinear {
    /// Creates a new linear-space color.
    #[inline]
    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Converts linear RGB to standard 8-bit non-linear sRGB.
    #[must_use]
    pub fn to_srgb_color(&self) -> Color {
        let r = (linear_to_srgb_channel(self.r) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        let g = (linear_to_srgb_channel(self.g) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        let b = (linear_to_srgb_channel(self.b) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        let a = (self.a * 255.0).round().clamp(0.0, 255.0) as u8;
        Color::rgba(r, g, b, a)
    }

    /// Converts linear sRGB to Display P3 wide gamut using standard color transformation matrix.
    #[must_use]
    pub fn to_display_p3(&self) -> Self {
        // D65 sRGB to Display P3 matrix transformation
        let r_p3 = 0.8225 * self.r + 0.1775 * self.g;
        let g_p3 = 0.0332 * self.r + 0.9668 * self.g;
        let b_p3 = 0.0171 * self.r + 0.0724 * self.g + 0.9105 * self.b;
        Self {
            r: r_p3.clamp(0.0, 1.0),
            g: g_p3.clamp(0.0, 1.0),
            b: b_p3.clamp(0.0, 1.0),
            a: self.a,
        }
    }
}

/// Color representation with 8-bit RGBA channels.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    #[inline]
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[inline]
    #[must_use]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Construct a copy of this color with replaced alpha channel.
    #[inline]
    #[must_use]
    pub const fn with_alpha(self, a: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    /// Construct solid color from a 24-bit RGB integer (e.g. `0x1E293B`).
    #[inline]
    #[must_use]
    pub const fn from_rgb_u32(val: u32) -> Self {
        Self {
            r: ((val >> 16) & 0xFF) as u8,
            g: ((val >> 8) & 0xFF) as u8,
            b: (val & 0xFF) as u8,
            a: 255,
        }
    }

    /// Construct color from hexadecimal string (e.g. `"#1E293B"`, `"#3B82F6"`).
    #[must_use]
    pub fn from_hex(hex: &str) -> Option<Self> {
        let digits = hex.strip_prefix('#').unwrap_or(hex);
        match digits.len() {
            6 => {
                let red = u8::from_str_radix(&digits[0..2], 16).ok()?;
                let green = u8::from_str_radix(&digits[2..4], 16).ok()?;
                let blue = u8::from_str_radix(&digits[4..6], 16).ok()?;
                Some(Self::rgb(red, green, blue))
            }
            8 => {
                let red = u8::from_str_radix(&digits[0..2], 16).ok()?;
                let green = u8::from_str_radix(&digits[2..4], 16).ok()?;
                let blue = u8::from_str_radix(&digits[4..6], 16).ok()?;
                let alpha = u8::from_str_radix(&digits[6..8], 16).ok()?;
                Some(Self::rgba(red, green, blue, alpha))
            }
            _ => None,
        }
    }

    /// Converts non-linear sRGB color into linear floating-point space.
    #[must_use]
    pub fn to_linear(self) -> ColorLinear {
        ColorLinear {
            r: srgb_to_linear_channel(f32::from(self.r) / 255.0),
            g: srgb_to_linear_channel(f32::from(self.g) / 255.0),
            b: srgb_to_linear_channel(f32::from(self.b) / 255.0),
            a: f32::from(self.a) / 255.0,
        }
    }

    /// Performs physically correct linear-space alpha blending of `src` over `dst`.
    ///
    /// Eliminates dark edge artifacts and saturation loss common in naïve gamma-space blending.
    #[must_use]
    pub fn blend_linear(src: Self, dst: Self) -> Self {
        if src.a == 255 {
            return src;
        }
        if src.a == 0 {
            return dst;
        }

        let l_src = src.to_linear();
        let l_dst = dst.to_linear();

        let src_a = l_src.a;
        let dst_a = l_dst.a;
        let out_a = src_a + dst_a * (1.0 - src_a);

        if out_a <= 0.0001 {
            return Self::TRANSPARENT;
        }

        let out_r = (l_src.r * src_a + l_dst.r * dst_a * (1.0 - src_a)) / out_a;
        let out_g = (l_src.g * src_a + l_dst.g * dst_a * (1.0 - src_a)) / out_a;
        let out_b = (l_src.b * src_a + l_dst.b * dst_a * (1.0 - src_a)) / out_a;

        ColorLinear::new(out_r, out_g, out_b, out_a).to_srgb_color()
    }
}

/// Converts a normalized non-linear sRGB channel to linear space.
#[inline]
fn srgb_to_linear_channel(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Converts a normalized linear RGB channel to non-linear sRGB.
#[inline]
fn linear_to_srgb_channel(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}
