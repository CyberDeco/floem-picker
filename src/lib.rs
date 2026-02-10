//! # floem-picker
//!
//! A color picker widget for [Floem](https://github.com/lapce/floem).
//!
//! Provides an inline HSB color picker with 2D saturation/brightness area, hue
//! slider, optional alpha slider, numeric inputs, and hex editing.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use floem::prelude::*;
//! use floem_solid::{solid_picker, SolidColor};
//!
//! let color = RwSignal::new(SolidColor::from_hex("3B82F6").unwrap());
//! // Use `solid_picker(color)` in your Floem view tree.
//! ```

mod color;

#[cfg(feature = "alpha")]
mod alpha_slider;
mod brightness_slider;
#[cfg(feature = "alpha")]
mod checkerboard;
mod color_editor;
mod color_wheel;
mod constants;
#[cfg(all(feature = "eyedropper", target_os = "macos"))]
mod eyedropper;
mod inputs;
mod math;

pub use color::SolidColor;

use std::sync::Once;

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::text::FONT_SYSTEM;

static LOAD_LUCIDE_FONT: Once = Once::new();

/// Creates the top-level color picker view.
///
/// The picker reads from and writes to `color`. Any external changes to the
/// signal are reflected in the UI, and user edits update the signal.
pub fn solid_picker(color: RwSignal<SolidColor>) -> impl IntoView {
    LOAD_LUCIDE_FONT.call_once(|| {
        FONT_SYSTEM
            .lock()
            .db_mut()
            .load_font_data(lucide_icons::LUCIDE_FONT_BYTES.to_vec());
    });
    color_editor::color_editor(color)
}
