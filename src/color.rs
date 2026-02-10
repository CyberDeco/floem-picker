//! SolidColor type — the public color representation for floem-solid.
//!
//! Stores RGBA as f64 values in 0.0–1.0 range. Uses `BigColor` for color space
//! conversions and hex parsing/formatting. Independent of Floem types so consumers
//! can construct and inspect colors without importing the framework.

use bigcolor::BigColor;

/// An RGBA color with components in the 0.0–1.0 range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
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
        if !matches!(stripped.len(), 3 | 6 | 8)
            || !stripped.chars().all(|c| c.is_ascii_hexdigit())
        {
            return None;
        }
        let bc = BigColor::new(format!("#{stripped}"));
        if !bc.is_valid() {
            return None;
        }
        let rgb = bc.to_rgb();
        Some(Self {
            r: rgb.r as f64 / 255.0,
            g: rgb.g as f64 / 255.0,
            b: rgb.b as f64 / 255.0,
            a: bc.get_alpha() as f64,
        })
    }

    /// Format as uppercase hex (no `#` prefix).
    ///
    /// Returns 6 chars (RRGGBB) when alpha is 1.0 or the color is black.
    /// Returns 8 chars (RRGGBBAA) otherwise.
    pub fn to_hex(&self) -> String {
        let (r, g, b) = self.to_rgb();
        let bc = BigColor::from_rgb(r, g, b, self.a as f32);
        let is_black = r == 0 && g == 0 && b == 0;
        if (self.a - 1.0).abs() < 0.001 || is_black {
            bc.to_hex(false).to_uppercase()
        } else {
            bc.to_hex8(false).to_uppercase()
        }
    }

    /// Create from HSB/HSV values (all 0.0–1.0).
    pub fn from_hsb(h: f64, s: f64, b: f64, a: f64) -> Self {
        let bc = BigColor::from_hsv((h * 360.0) as f32, s as f32, b as f32, a as f32);
        let rgb = bc.to_rgb();
        Self {
            r: rgb.r as f64 / 255.0,
            g: rgb.g as f64 / 255.0,
            b: rgb.b as f64 / 255.0,
            a,
        }
    }

    /// Convert to HSB (all 0.0–1.0). Returns (h, s, b).
    pub fn to_hsb(&self) -> (f64, f64, f64) {
        let (r, g, b) = self.to_rgb();
        let bc = BigColor::from_rgb(r, g, b, 1.0);
        let hsv = bc.to_hsv();
        (hsv.h as f64 / 360.0, hsv.s as f64, hsv.v as f64)
    }

    /// Create from HSL values (all 0.0–1.0).
    pub fn from_hsl(h: f64, s: f64, l: f64, a: f64) -> Self {
        let bc = BigColor::from_hsl((h * 360.0) as f32, s as f32, l as f32, a as f32);
        let rgb = bc.to_rgb();
        Self {
            r: rgb.r as f64 / 255.0,
            g: rgb.g as f64 / 255.0,
            b: rgb.b as f64 / 255.0,
            a,
        }
    }

    /// Convert to HSL (all 0.0–1.0). Returns (h, s, l).
    pub fn to_hsl(&self) -> (f64, f64, f64) {
        let (r, g, b) = self.to_rgb();
        let bc = BigColor::from_rgb(r, g, b, 1.0);
        let hsl = bc.to_hsl();
        (hsl.h as f64 / 360.0, hsl.s as f64, hsl.l as f64)
    }

    /// Create from f64 RGBA (all 0.0–1.0).
    pub fn from_rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }
}
