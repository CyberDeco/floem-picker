//! Color editor: consolidated panel showing HSB, HSL, and RGB input rows
//! alongside the color wheel, brightness slider, alpha slider, hex input,
//! and color swatch.

use std::cell::Cell;
use std::rc::Rc;

use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, create_effect};

use crate::brightness_slider::brightness_slider;
use crate::color::SolidColor;
use crate::color_wheel::color_wheel;
use crate::constants;
#[cfg(all(feature = "eyedropper", target_os = "macos"))]
use crate::eyedropper::eyedropper_button;
#[cfg(feature = "alpha")]
use crate::inputs::alpha_input;
use crate::inputs::{copy_button, hex_input, number_input};
use crate::math;

#[cfg(feature = "alpha")]
use crate::alpha_slider::alpha_slider;

/// Creates a consolidated color editor with HSB, HSL, and RGB input rows.
pub(crate) fn color_editor(color: RwSignal<SolidColor>) -> impl IntoView {
    // HSB signals (ground-truth)
    let h = RwSignal::new(0.0_f64);
    let s = RwSignal::new(0.0_f64);
    let b = RwSignal::new(1.0_f64);
    let a = RwSignal::new(1.0_f64);
    let hex = RwSignal::new("808080FF".to_string());

    // HSL derived signals
    let s_hsl = RwSignal::new(0.0_f64);
    let l = RwSignal::new(0.5_f64);

    // RGB derived signals
    let r = RwSignal::new(0.5_f64);
    let g = RwSignal::new(0.5_f64);
    let bl = RwSignal::new(0.5_f64);

    // Non-reactive guards to break forward→back-sync cycles between color signals.
    let hsl_from_hsb = Rc::new(Cell::new(false));
    let rgb_from_hsb = Rc::new(Cell::new(false));

    // Initialize from current color
    {
        let c = color.get_untracked();
        let (ch, cs, cb) = c.to_hsb();
        h.set(ch);
        s.set(cs);
        b.set(cb);
        a.set(c.a());
        hex.set(c.to_hex());
        let (_, sh, lv) = math::hsb_to_hsl(ch, cs, cb);
        s_hsl.set(sh);
        l.set(lv);
        r.set(c.r());
        g.set(c.g());
        bl.set(c.b());
    }

    // ── HSB → color (when any HSB component changes) ───────────────────
    create_effect(move |_| {
        let hv = h.get();
        let sv = s.get();
        let bv = b.get();
        let av = a.get();
        let new_color = SolidColor::from_hsb(hv, sv, bv, av);
        let current = color.get_untracked();
        if (new_color.r() - current.r()).abs() > 0.001
            || (new_color.g() - current.g()).abs() > 0.001
            || (new_color.b() - current.b()).abs() > 0.001
            || (new_color.a() - current.a()).abs() > 0.001
        {
            color.set(new_color);
            let new_hex = new_color.to_hex();
            if hex.get_untracked() != new_hex {
                hex.set(new_hex);
            }
        }
    });

    // External color -> HSB
    create_effect(move |prev: Option<SolidColor>| {
        let c = color.get();
        if let Some(prev) = prev
            && (c.r() - prev.r()).abs() < 0.001
            && (c.g() - prev.g()).abs() < 0.001
            && (c.b() - prev.b()).abs() < 0.001
            && (c.a() - prev.a()).abs() < 0.001
        {
            return c;
        }
        let (er, eg, eb) =
            math::hsb_to_rgb(h.get_untracked(), s.get_untracked(), b.get_untracked());
        if (er - c.r()).abs() < 0.005
            && (eg - c.g()).abs() < 0.005
            && (eb - c.b()).abs() < 0.005
            && (a.get_untracked() - c.a()).abs() < 0.005
        {
            let new_hex = c.to_hex();
            if hex.get_untracked() != new_hex {
                hex.set(new_hex);
            }
            return c;
        }
        let (ch, cs, cb) = c.to_hsb();
        if cs > 0.001 && cb > 0.001 {
            h.set(ch);
        }
        s.set(cs);
        b.set(cb);
        a.set(c.a());
        let new_hex = c.to_hex();
        if hex.get_untracked() != new_hex {
            hex.set(new_hex);
        }
        c
    });

    // Hex -> color
    create_effect(move |_| {
        let hx = hex.get();
        if let Some(c) = SolidColor::from_hex(&hx) {
            let current = color.get_untracked();
            let rgb_changed = (c.r() - current.r()).abs() > 0.003
                || (c.g() - current.g()).abs() > 0.003
                || (c.b() - current.b()).abs() > 0.003;
            let alpha_changed = (c.a() - a.get_untracked()).abs() > 0.004;
            if rgb_changed || alpha_changed {
                let new_a = if alpha_changed {
                    c.a()
                } else {
                    a.get_untracked()
                };
                let new_color = SolidColor::from_rgba(c.r(), c.g(), c.b(), new_a);
                color.set(new_color);
                let (ch, cs, cb) = new_color.to_hsb();
                if cs > 0.001 && cb > 0.001 {
                    h.set(ch);
                }
                s.set(cs);
                b.set(cb);
                if alpha_changed {
                    a.set(new_a);
                }
            }
        }
    });

    // HSB -> HSL display sync
    let hsl_guard_fwd = hsl_from_hsb.clone();
    create_effect(move |_| {
        let hv = h.get();
        let sv = s.get();
        let bv = b.get();
        let (_, new_s_hsl, new_l) = math::hsb_to_hsl(hv, sv, bv);
        if (s_hsl.get_untracked() - new_s_hsl).abs() > 0.001
            || (l.get_untracked() - new_l).abs() > 0.001
        {
            hsl_guard_fwd.set(true);
            s_hsl.set(new_s_hsl);
            l.set(new_l);
            hsl_guard_fwd.set(false);
        }
    });

    // ── HSL → HSB back-sync (when HSL inputs change) ───────────────────
    let hsl_guard_back = hsl_from_hsb;
    create_effect(move |_| {
        let sh = s_hsl.get();
        let lv = l.get();
        if hsl_guard_back.get() {
            return;
        }
        let hv = h.get_untracked();
        let (_, new_s_hsb, new_b) = math::hsl_to_hsb(hv, sh, lv);
        if (s.get_untracked() - new_s_hsb).abs() > 0.001 {
            s.set(new_s_hsb);
        }
        if (b.get_untracked() - new_b).abs() > 0.001 {
            b.set(new_b);
        }
    });

    // HSB -> RGB display sync
    let rgb_guard_fwd = rgb_from_hsb.clone();
    create_effect(move |_| {
        let hv = h.get();
        let sv = s.get();
        let bv = b.get();
        let (nr, ng, nb) = math::hsb_to_rgb(hv, sv, bv);
        if (r.get_untracked() - nr).abs() > 0.002
            || (g.get_untracked() - ng).abs() > 0.002
            || (bl.get_untracked() - nb).abs() > 0.002
        {
            rgb_guard_fwd.set(true);
            r.set(nr);
            g.set(ng);
            bl.set(nb);
            rgb_guard_fwd.set(false);
        }
    });

    // RGB -> HSB back-sync (when RGB inputs change)
    let rgb_guard_back = rgb_from_hsb;
    create_effect(move |_| {
        let rv = r.get();
        let gv = g.get();
        let bv = bl.get();
        if rgb_guard_back.get() {
            return;
        }
        let (new_h, new_s, new_b) = math::rgb_to_hsb(rv, gv, bv);
        if new_s > 0.001 && new_b > 0.001 && (h.get_untracked() - new_h).abs() > 0.002 {
            h.set(new_h);
        }
        if (s.get_untracked() - new_s).abs() > 0.002 {
            s.set(new_s);
        }
        if (b.get_untracked() - new_b).abs() > 0.002 {
            b.set(new_b);
        }
    });

    // Build layout
    v_stack((
        // Color wheel (hue + saturation)
        color_wheel(h, s, b).style(|s| s.margin_top(12.0)),
        // Eyedropper + color swatch row
        h_stack((
            #[cfg(all(feature = "eyedropper", target_os = "macos"))]
            eyedropper_button(color),
            // Spacer pushes swatch to the right
            empty().style(|s| s.flex_grow(1.0)),
            {
                let color_copy = color;
                empty().style(move |st| {
                    let c = color_copy.get();
                    st.width(32.0)
                        .height(32.0)
                        .border_radius(constants::RADIUS)
                        .border(1.0)
                        .border_color(Color::rgb8(180, 180, 180))
                        .background(Color::rgba(c.r(), c.g(), c.b(), c.a()))
                })
            },
        ))
        .style(|st| st.items_center().margin_horiz(8.0)),
        // Brightness slider
        brightness_slider(h, s, b).style(|s| s.margin_horiz(8.0)),
        // Alpha slider + percentage (feature-gated)
        #[cfg(feature = "alpha")]
        h_stack((
            alpha_slider(a, move || {
                let (r, g, bl) = math::hsb_to_rgb(h.get(), s.get(), b.get());
                (r, g, bl)
            })
            .style(|s| s.flex_grow(1.0)),
            alpha_input(a),
        ))
        .style(|s| s.margin_horiz(8.0).gap(4.0)),
        // Hex + copy row
        h_stack((hex_input(hex), copy_button(move || hex.get().to_string())))
            .style(|st| st.gap(constants::GAP).items_center().justify_center()),
        // HSB inputs row
        h_stack((
            number_input("H", h, 360.0),
            number_input("S", s, 100.0),
            number_input("B", b, 100.0),
            copy_button(move || {
                format!(
                    "{}, {}, {}",
                    (h.get() * 360.0).round() as i64,
                    (s.get() * 100.0).round() as i64,
                    (b.get() * 100.0).round() as i64,
                )
            }),
        ))
        .style(|st| st.gap(constants::GAP / 2.0).items_center().justify_center()),
        // HSL inputs row
        h_stack((
            number_input("H", h, 360.0),
            number_input("S", s_hsl, 100.0),
            number_input("L", l, 100.0),
            copy_button(move || {
                format!(
                    "{}, {}, {}",
                    (h.get() * 360.0).round() as i64,
                    (s_hsl.get() * 100.0).round() as i64,
                    (l.get() * 100.0).round() as i64,
                )
            }),
        ))
        .style(|st| st.gap(constants::GAP / 2.0).items_center().justify_center()),
        // RGB inputs row
        h_stack((
            number_input("sR", r, 255.0),
            number_input("G", g, 255.0),
            number_input("B", bl, 255.0),
            copy_button(move || {
                format!(
                    "{}, {}, {}",
                    (r.get() * 255.0).round() as i64,
                    (g.get() * 255.0).round() as i64,
                    (bl.get() * 255.0).round() as i64,
                )
            }),
        ))
        .style(|st| st.gap(constants::GAP / 2.0).items_center().justify_center()),
    ))
    .style(|st| {
        st.gap(constants::GAP)
            .padding_horiz(constants::PADDING)
            .padding_bottom(constants::PADDING)
            .padding_top(2.0)
            .size_full()
            .justify_center()
            .background(Color::rgb8(242, 242, 242))
    })
}
