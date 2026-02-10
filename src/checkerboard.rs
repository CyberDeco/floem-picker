//! Checkerboard background renderer for alpha slider.

use floem::context::PaintCx;
use floem::kurbo::Rect;
use floem::peniko::Color;
use floem_renderer::Renderer;

use crate::constants;

const LIGHT: Color = Color::rgb8(255, 255, 255);
const DARK: Color = Color::rgb8(204, 204, 204);

/// Paint a checkerboard pattern into `rect`.
pub(crate) fn paint_checkerboard(cx: &mut PaintCx, rect: Rect) {
    let cell = constants::CHECKER_CELL;
    // Fill with light first
    cx.fill(&rect, LIGHT, 0.0);
    // Then draw dark cells
    let cols = (rect.width() / cell).ceil() as usize;
    let rows = (rect.height() / cell).ceil() as usize;
    for row in 0..rows {
        for col in 0..cols {
            if (row + col) % 2 == 1 {
                let x = rect.x0 + col as f64 * cell;
                let y = rect.y0 + row as f64 * cell;
                let cell_rect = Rect::new(x, y, (x + cell).min(rect.x1), (y + cell).min(rect.y1));
                cx.fill(&cell_rect, DARK, 0.0);
            }
        }
    }
}
