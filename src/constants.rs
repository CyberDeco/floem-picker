//! Sizing, color, and styling constants for the picker.

/// 1D slider track height
pub const SLIDER_HEIGHT: f32 = 16.0;

/// Cursor circle radius on the 2D picker
pub const CURSOR_RADIUS: f64 = 8.0;

/// Thumb radius on 1D sliders
pub const THUMB_RADIUS: f64 = 7.0;

/// Border radius for slider tracks
pub const RADIUS: f32 = 4.0;

/// Gap between picker elements
pub const GAP: f32 = 8.0;

/// Padding around the whole picker
pub const PADDING: f32 = 8.0;

/// Input field width
pub const INPUT_WIDTH: f32 = 28.0;

/// Hex input field width
pub const HEX_INPUT_WIDTH: f32 = 64.0;

/// Input font size
pub const INPUT_FONT: f32 = 11.0;

/// Label font size
pub const LABEL_FONT: f32 = 10.0;

/// Checkerboard cell size (for alpha backgrounds)
#[cfg(feature = "alpha")]
pub const CHECKER_CELL: f64 = 5.0;
