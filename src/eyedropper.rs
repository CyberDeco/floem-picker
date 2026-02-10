//! macOS native eyedropper (screen color sampler) integration.
//!
//! Uses `NSColorSampler` via Objective-C FFI to invoke the system-wide
//! screen color picker. The sampler runs asynchronously — macOS shows a
//! magnifying-glass overlay, the user clicks a pixel, and the callback
//! fires with the sampled color.

use std::cell::Cell;

use block2::RcBlock;
use objc2::rc::{Allocated, Id};
use objc2::runtime::{AnyClass, AnyObject};
use objc2::{msg_send, msg_send_id};

use floem::prelude::*;
use floem::reactive::{RwSignal, SignalUpdate};

use crate::color::SolidColor;

/// Invokes the macOS native screen color sampler.
///
/// When the user picks a pixel, `on_pick` is called with the sampled color
/// (converted to sRGB). If the user cancels (Esc), nothing happens.
///
/// Must be called from the main thread (Floem event handlers satisfy this).
pub(crate) fn sample_color(on_pick: impl FnOnce(SolidColor) + 'static) {
    let cls = match AnyClass::get("NSColorSampler") {
        Some(c) => c,
        None => return,
    };

    let sampler: Allocated<AnyObject> = unsafe { msg_send_id![cls, alloc] };
    let sampler: Id<AnyObject> = unsafe { msg_send_id![sampler, init] };

    type Callback = Cell<Option<Box<dyn FnOnce(SolidColor)>>>;
    let callback: Callback = Cell::new(Some(Box::new(on_pick)));

    let block = RcBlock::new(move |color_ptr: *mut AnyObject| {
        if color_ptr.is_null() {
            return;
        }
        unsafe {
            let ns_cs_cls = match AnyClass::get("NSColorSpace") {
                Some(c) => c,
                None => return,
            };
            let srgb: *const AnyObject = msg_send![ns_cs_cls, sRGBColorSpace];
            if srgb.is_null() {
                return;
            }
            let srgb_color: *const AnyObject =
                msg_send![&*color_ptr, colorUsingColorSpace: &*srgb];
            if srgb_color.is_null() {
                return;
            }
            let mut r: f64 = 0.0;
            let mut g: f64 = 0.0;
            let mut b: f64 = 0.0;
            let mut a: f64 = 0.0;
            let _: () = msg_send![
                &*srgb_color,
                getRed: &mut r,
                green: &mut g,
                blue: &mut b,
                alpha: &mut a
            ];
            if let Some(cb) = callback.take() {
                cb(SolidColor::from_rgba(r, g, b, a));
            }
        }
    });

    unsafe {
        let _: () = msg_send![&*sampler, showSamplerWithSelectionHandler: &*block];
    }
}

/// A small button that invokes the macOS screen color sampler.
///
/// On click, opens the system eyedropper; the picked color is written
/// to `color`. Styled to match `copy_button`.
pub(crate) fn eyedropper_button(color: RwSignal<SolidColor>) -> impl IntoView {
    let pressed = RwSignal::new(false);
    label(|| lucide_icons::Icon::Pipette.unicode().to_string())
        .style(move |s| {
            let c = if pressed.get() {
                Color::rgb8(80, 80, 80)
            } else {
                Color::rgb8(120, 120, 120)
            };
            s.font_size(18.0)
                .font_family("lucide".to_string())
                .cursor(floem::style::CursorStyle::Pointer)
                .border_radius(3.0)
                .padding(2.0)
                .color(c)
                .hover(|s| s.background(Color::rgb8(230, 230, 230)))
        })
        .on_event_stop(floem::event::EventListener::PointerDown, move |_| {
            pressed.set(true);
        })
        .on_event_stop(floem::event::EventListener::PointerUp, move |_| {
            pressed.set(false);
            sample_color(move |picked| {
                color.set(picked);
            });
        })
}
