//! Color math — conversions powered by `BigColor`.
//! All functions use normalized f64 in 0.0–1.0 for internal use.

use bigcolor::BigColor;

/// HSB/HSV → RGB. All values 0.0–1.0.
pub fn hsb_to_rgb(h: f64, s: f64, b: f64) -> (f64, f64, f64) {
    let bc = BigColor::from_hsv((h * 360.0) as f32, s as f32, b as f32, 1.0);
    let rgb = bc.to_rgb();
    (
        rgb.r as f64 / 255.0,
        rgb.g as f64 / 255.0,
        rgb.b as f64 / 255.0,
    )
}

/// RGB → HSB/HSV. All values 0.0–1.0.
pub fn rgb_to_hsb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let bc = BigColor::from_rgb(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        1.0,
    );
    let hsv = bc.to_hsv();
    (hsv.h as f64 / 360.0, hsv.s as f64, hsv.v as f64)
}

/// HSL → HSB. All values 0.0–1.0.
pub fn hsl_to_hsb(h: f64, s_hsl: f64, l: f64) -> (f64, f64, f64) {
    let bc = BigColor::from_hsl((h * 360.0) as f32, s_hsl as f32, l as f32, 1.0);
    let hsv = bc.to_hsv();
    (hsv.h as f64 / 360.0, hsv.s as f64, hsv.v as f64)
}

/// HSB → HSL. All values 0.0–1.0.
pub fn hsb_to_hsl(h: f64, s_hsb: f64, b: f64) -> (f64, f64, f64) {
    let bc = BigColor::from_hsv((h * 360.0) as f32, s_hsb as f32, b as f32, 1.0);
    let hsl = bc.to_hsl();
    (hsl.h as f64 / 360.0, hsl.s as f64, hsl.l as f64)
}

/// Normalize a hex string: uppercase, expand shorthand, default to gray if invalid.
///
/// Always returns 8 chars (RRGGBBAA).
pub(crate) fn normalize_hex(hex: &str) -> String {
    let stripped = hex.trim_start_matches('#');
    let bc = BigColor::new(format!("#{stripped}"));
    if bc.is_valid() {
        bc.to_hex8(false).to_uppercase()
    } else {
        "808080FF".to_string()
    }
}