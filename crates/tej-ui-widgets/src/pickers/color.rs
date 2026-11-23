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
        let c = self.v * self.s;
        let h_prime = self.h / 60.0;
        let x = c * (1.0 - (h_prime.rem_euclid(2.0) - 1.0).abs());
        let m = self.v - c;

        let (r1, g1, b1) = match h_prime as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        let r = ((r1 + m) * 255.0).round() as u8;
        let g = ((g1 + m) * 255.0).round() as u8;
        let b = ((b1 + m) * 255.0).round() as u8;
        let a = (self.a * 255.0).round() as u8;

        (r, g, b, a)
    }

    /// Construct from 8-bit RGBA values.
    #[must_use]
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        let rf = r as f32 / 255.0;
        let gf = g as f32 / 255.0;
        let bf = b as f32 / 255.0;
        let af = a as f32 / 255.0;

        let max = rf.max(gf).max(bf);
        let min = rf.min(gf).min(bf);
        let delta = max - min;

        let h = if delta <= 0.0 {
            0.0
        } else if (max - rf).abs() < f32::EPSILON {
            60.0 * (((gf - bf) / delta).rem_euclid(6.0))
        } else if (max - gf).abs() < f32::EPSILON {
            60.0 * (((bf - rf) / delta) + 2.0)
        } else {
            60.0 * (((rf - gf) / delta) + 4.0)
        };

        let s = if max <= 0.0 { 0.0 } else { delta / max };
        let v = max;

        Self::new(h, s, v, af)
    }

    /// Format as standard `#RRGGBB` or `#RRGGBBAA` hexadecimal color string.
    #[must_use]
    pub fn to_hex_string(self) -> String {
        let (r, g, b, a) = self.to_rgba8();
        if a == 255 {
            format!("#{r:02X}{g:02X}{b:02X}")
        } else {
            format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
        }
    }

    /// Parse hex color string `#RGB`, `#RRGGBB`, or `#RRGGBBAA`.
    pub fn from_hex_str(hex: &str) -> Option<Self> {
        let s = hex.strip_prefix('#').unwrap_or(hex);
        match s.len() {
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                Some(Self::from_rgba8(r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&s[0..2], 16).ok()?;
                let g = u8::from_str_radix(&s[2..4], 16).ok()?;
                let b = u8::from_str_radix(&s[4..6], 16).ok()?;
                let a = u8::from_str_radix(&s[6..8], 16).ok()?;
                Some(Self::from_rgba8(r, g, b, a))
            }
            3 => {
                let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
                Some(Self::from_rgba8(r, g, b, 255))
            }
            _ => None,
        }
    }
}
