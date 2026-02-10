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

/// Checkerboard cell size (for alpha backgrounds)
#[cfg(feature = "alpha")]
pub(crate) const CHECKER_CELL: f64 = 5.0;
