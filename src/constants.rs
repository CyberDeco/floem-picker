//! Sizing, color, and styling constants for the picker.

/// Brightness and alpha sliders track height
pub(crate) const SLIDER_HEIGHT: f32 = 16.0;

/// Cursor circle radius for color wheel gradient picker
pub(crate) const CURSOR_RADIUS: f64 = 8.0;

/// Thumb radius on 1D sliders
pub(crate) const THUMB_RADIUS: f64 = 7.0;

/// Border radius for slider tracks
pub(crate) const RADIUS: f32 = 4.0;

/// Gap between picker elements
pub(crate) const GAP: f32 = 8.0;

/// Padding around the whole picker
pub(crate) const PADDING: f32 = 8.0;

/// Input field width
pub(crate) const INPUT_WIDTH: f32 = 28.0;

/// Hex input field width
pub(crate) const HEX_INPUT_WIDTH: f32 = 64.0;

/// Input font size
pub(crate) const INPUT_FONT: f32 = 11.0;

/// Label font size
pub(crate) const LABEL_FONT: f32 = 10.0;

/// Fixed raster size (in pixels) for the color wheel and slider gradients.
/// Rasterized once and scaled by the renderer, avoiding new texture-atlas
/// entries on every resize (which exhausts vger's fixed-size atlas).
pub(crate) const WHEEL_RASTER_SIZE: u32 = 1024;

/// Fixed raster width for slider gradients.
pub(crate) const SLIDER_RASTER_WIDTH: u32 = 256;

/// Fixed raster height for slider gradients.
pub(crate) const SLIDER_RASTER_HEIGHT: u32 = 32;

/// Checkerboard cell size (for alpha backgrounds)
#[cfg(feature = "alpha")]
pub(crate) const CHECKER_CELL: f64 = 5.0;
