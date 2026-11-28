//! Color picker state machine and RGB/HSV/HEX conversion mathematics.

use std::fmt::Debug;

/// HSV (Hue, Saturation, Value, Alpha) color model representation.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ColorHsv {
    /// Hue angle in degrees `[0.0, 360.0)`.
    pub h: f32,
    /// Saturation percentage `[0.0, 1.0]`.
    pub s: f32,
    /// Value / Brightness percentage `[0.0, 1.0]`.
    pub v: f32,
    /// Alpha transparency `[0.0, 1.0]`.
    pub a: f32,
}

impl Default for ColorHsv {
    fn default() -> Self {
        Self {
            h: 0.0,
            s: 1.0,
            v: 1.0,
            a: 1.0,
        }
    }
}

impl ColorHsv {
    #[must_use]
    pub fn new(h: f32, s: f32, v: f32, a: f32) -> Self {
        Self {
            h: h.rem_euclid(360.0),
            s: s.clamp(0.0, 1.0),
            v: v.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    /// Convert HSV to 8-bit RGBA channels `(r, g, b, a)`.
    #[must_use]
    pub fn to_rgba8(self) -> (u8, u8, u8, u8) {
        let chroma = self.v * self.s;
        let h_prime = self.h / 60.0;
        let sec = chroma * (1.0 - (h_prime.rem_euclid(2.0) - 1.0).abs());
        let match_v = self.v - chroma;

        let (r1, g1, b1) = match h_prime as u32 {
            0 => (chroma, sec, 0.0),
            1 => (sec, chroma, 0.0),
            2 => (0.0, chroma, sec),
            3 => (0.0, sec, chroma),
            4 => (sec, 0.0, chroma),
            _ => (chroma, 0.0, sec),
        };

        let red = ((r1 + match_v) * 255.0).round() as u8;
        let green = ((g1 + match_v) * 255.0).round() as u8;
        let blue = ((b1 + match_v) * 255.0).round() as u8;
        let alpha = (self.a * 255.0).round() as u8;

        (red, green, blue, alpha)
    }

    /// Construct from 8-bit RGBA values.
    #[must_use]
    pub fn from_rgba8(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        let rf = f32::from(red) / 255.0;
        let gf = f32::from(green) / 255.0;
        let bf = f32::from(blue) / 255.0;
        let af = f32::from(alpha) / 255.0;

        let max = rf.max(gf).max(bf);
        let min = rf.min(gf).min(bf);
        let delta = max - min;

        let hue = if delta <= 0.0 {
            0.0
        } else if (max - rf).abs() < f32::EPSILON {
            60.0 * (((gf - bf) / delta).rem_euclid(6.0))
        } else if (max - gf).abs() < f32::EPSILON {
            60.0 * (((bf - rf) / delta) + 2.0)
        } else {
            60.0 * (((rf - gf) / delta) + 4.0)
        };

        let sat = if max <= 0.0 { 0.0 } else { delta / max };
        let val = max;

        Self::new(hue, sat, val, af)
    }

    /// Format as standard `#RRGGBB` or `#RRGGBBAA` hexadecimal color string.
    #[must_use]
    pub fn to_hex_string(self) -> String {
        let (red, green, blue, alpha) = self.to_rgba8();
        if alpha == 255 {
            format!("#{red:02X}{green:02X}{blue:02X}")
        } else {
            format!("#{red:02X}{green:02X}{blue:02X}{alpha:02X}")
        }
    }

    /// Parse hex color string `#RGB`, `#RRGGBB`, or `#RRGGBBAA`.
    pub fn from_hex_str(hex: &str) -> Option<Self> {
        let digits = hex.strip_prefix('#').unwrap_or(hex);
        match digits.len() {
            6 => {
                let red = u8::from_str_radix(&digits[0..2], 16).ok()?;
                let green = u8::from_str_radix(&digits[2..4], 16).ok()?;
                let blue = u8::from_str_radix(&digits[4..6], 16).ok()?;
                Some(Self::from_rgba8(red, green, blue, 255))
            }
            8 => {
                let red = u8::from_str_radix(&digits[0..2], 16).ok()?;
                let green = u8::from_str_radix(&digits[2..4], 16).ok()?;
                let blue = u8::from_str_radix(&digits[4..6], 16).ok()?;
                let alpha = u8::from_str_radix(&digits[6..8], 16).ok()?;
                Some(Self::from_rgba8(red, green, blue, alpha))
            }
            3 => {
                let red = u8::from_str_radix(&digits[0..1], 16).ok()? * 17;
                let green = u8::from_str_radix(&digits[1..2], 16).ok()? * 17;
                let blue = u8::from_str_radix(&digits[2..3], 16).ok()? * 17;
                Some(Self::from_rgba8(red, green, blue, 255))
            }
            _ => None,
        }
    }
}
