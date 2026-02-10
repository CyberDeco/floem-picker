//! SolidColor type — the public color representation for floem-picker.
//!
//! Stores RGBA as f64 values in 0.0–1.0 range. Uses direct math for color
//! space conversions and hex parsing/formatting.

use crate::math;

/// RGBA color with components in the 0.0–1.0 range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidColor {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

impl SolidColor {
    /// Red component (0.0–1.0).
    pub fn r(&self) -> f64 {
        self.r
    }
    /// Green component (0.0–1.0).
    pub fn g(&self) -> f64 {
        self.g
    }
    /// Blue component (0.0–1.0).
    pub fn b(&self) -> f64 {
        self.b
    }
    /// Alpha component (0.0–1.0).
    pub fn a(&self) -> f64 {
        self.a
    }
}

impl Default for SolidColor {
    fn default() -> Self {
        Self {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        }
    }
}

impl SolidColor {
    /// Create from 0–255 RGB values with full opacity.
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: 1.0,
        }
    }

    /// Convert to 0–255 RGB tuple.
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        (
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8,
        )
    }

    /// Parse a hex string (with or without `#`, 3, 6, or 8 chars).
    ///
    /// 8-char hex is interpreted as RRGGBBAA. 3 and 6-char hex default to full opacity.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let stripped = hex.trim_start_matches('#');
        if !stripped.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        match stripped.len() {
            3 => {
                let r = u8::from_str_radix(&stripped[0..1], 16).ok()?;
                let g = u8::from_str_radix(&stripped[1..2], 16).ok()?;
                let b = u8::from_str_radix(&stripped[2..3], 16).ok()?;
                Some(Self {
                    r: (r * 17) as f64 / 255.0,
                    g: (g * 17) as f64 / 255.0,
                    b: (b * 17) as f64 / 255.0,
                    a: 1.0,
                })
            }
            6 => {
                let r = u8::from_str_radix(&stripped[0..2], 16).ok()?;
                let g = u8::from_str_radix(&stripped[2..4], 16).ok()?;
                let b = u8::from_str_radix(&stripped[4..6], 16).ok()?;
                Some(Self {
                    r: r as f64 / 255.0,
                    g: g as f64 / 255.0,
                    b: b as f64 / 255.0,
                    a: 1.0,
                })
            }
            8 => {
                let r = u8::from_str_radix(&stripped[0..2], 16).ok()?;
                let g = u8::from_str_radix(&stripped[2..4], 16).ok()?;
                let b = u8::from_str_radix(&stripped[4..6], 16).ok()?;
                let a = u8::from_str_radix(&stripped[6..8], 16).ok()?;
                Some(Self {
                    r: r as f64 / 255.0,
                    g: g as f64 / 255.0,
                    b: b as f64 / 255.0,
                    a: a as f64 / 255.0,
                })
            }
            _ => None,
        }
    }

    /// Format as uppercase hex (no `#` prefix).
    ///
    /// Returns 6 chars (RRGGBB) when alpha is 1.0 or the color is black.
    /// Returns 8 chars (RRGGBBAA) otherwise.
    pub fn to_hex(&self) -> String {
        let (r, g, b) = self.to_rgb();
        let is_black = r == 0 && g == 0 && b == 0;
        if (self.a - 1.0).abs() < 0.001 || is_black {
            format!("{:02X}{:02X}{:02X}", r, g, b)
        } else {
            let a = (self.a * 255.0).round() as u8;
            format!("{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
        }
    }

    /// Create from HSB/HSV values (all 0.0–1.0).
    pub fn from_hsb(h: f64, s: f64, b: f64, a: f64) -> Self {
        let (r, g, bl) = math::hsb_to_rgb(h, s, b);
        Self { r, g, b: bl, a }
    }

    /// Convert to HSB (all 0.0–1.0). Returns (h, s, b).
    pub fn to_hsb(&self) -> (f64, f64, f64) {
        math::rgb_to_hsb(self.r, self.g, self.b)
    }

    /// Create from HSL values (all 0.0–1.0).
    pub fn from_hsl(h: f64, s: f64, l: f64, a: f64) -> Self {
        let (hb, sb, vb) = math::hsl_to_hsb(h, s, l);
        let (r, g, bl) = math::hsb_to_rgb(hb, sb, vb);
        Self { r, g, b: bl, a }
    }

    /// Convert to HSL (all 0.0–1.0). Returns (h, s, l).
    pub fn to_hsl(&self) -> (f64, f64, f64) {
        let (h, s, v) = math::rgb_to_hsb(self.r, self.g, self.b);
        math::hsb_to_hsl(h, s, v)
    }

    /// Create from f64 RGBA (all 0.0–1.0).
    pub fn from_rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }
}
