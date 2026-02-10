//! Color math — direct conversions without external dependencies.
//! All functions use normalized f64 in 0.0–1.0 for internal use.

/// HSB/HSV → RGB. All values 0.0–1.0.
pub(crate) fn hsb_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    if s == 0.0 {
        return (v, v, v);
    }
    let h6 = (h * 6.0) % 6.0;
    let i = h6.floor() as u32;
    let f = h6 - h6.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

/// RGB → HSB/HSV. All values 0.0–1.0.
pub(crate) fn rgb_to_hsb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;
    let s = if max == 0.0 { 0.0 } else { delta / max };

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        ((g - b) / delta).rem_euclid(6.0) / 6.0
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };

    (h, s, v)
}

/// HSL → HSB. All values 0.0–1.0.
pub(crate) fn hsl_to_hsb(h: f64, s_hsl: f64, l: f64) -> (f64, f64, f64) {
    let v = l + s_hsl * l.min(1.0 - l);
    let s_hsb = if v == 0.0 { 0.0 } else { 2.0 * (1.0 - l / v) };
    (h, s_hsb, v)
}

/// HSB → HSL. All values 0.0–1.0.
pub(crate) fn hsb_to_hsl(h: f64, s_hsb: f64, v: f64) -> (f64, f64, f64) {
    let l = v * (1.0 - s_hsb / 2.0);
    let s_hsl = if l == 0.0 || l == 1.0 {
        0.0
    } else {
        (v - l) / l.min(1.0 - l)
    };
    (h, s_hsl, l)
}

/// Normalize a hex string: uppercase, expand shorthand, default to gray if invalid.
///
/// Always returns 8 chars (RRGGBBAA).
pub(crate) fn normalize_hex(hex: &str) -> String {
    let stripped = hex.trim_start_matches('#');
    if !stripped.chars().all(|c| c.is_ascii_hexdigit()) {
        return "808080FF".to_string();
    }
    match stripped.len() {
        3 => {
            let mut out = String::with_capacity(8);
            for c in stripped.chars() {
                out.push(c);
                out.push(c);
            }
            out.push_str("FF");
            out.to_uppercase()
        }
        6 => format!("{}FF", stripped.to_uppercase()),
        8 => stripped.to_uppercase(),
        _ => "808080FF".to_string(),
    }
}
