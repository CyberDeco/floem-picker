//! Alpha slider with checkerboard background + transparent-to-opaque gradient.

use floem::kurbo::{Rect, Shape};
use floem::peniko::{Color, Gradient};

use floem::reactive::{create_effect, RwSignal, SignalGet, SignalUpdate};
use floem::views::Decorators;
use floem::{
    context::{ComputeLayoutCx, EventCx, PaintCx, UpdateCx},
    event::{Event, EventPropagation},
    View, ViewId,
};
use floem_renderer::Renderer;

use crate::checkerboard;
use crate::constants;

enum AlphaUpdate {
    Alpha(f64),
    BaseColor(f64, f64, f64),
}

pub struct AlphaSlider {
    id: ViewId,
    held: bool,
    alpha: f64,
    base_r: f64,
    base_g: f64,
    base_b: f64,
    size: floem::taffy::prelude::Size<f32>,
    on_change: Option<Box<dyn Fn(f64)>>,
}

/// Creates an alpha slider.
///
/// - `alpha_signal`: 0.0 (transparent) to 1.0 (opaque).
/// - `base_color_fn`: returns the current (r, g, b) in 0.0–1.0 for the gradient overlay.
pub fn alpha_slider(
    alpha_signal: RwSignal<f64>,
    base_color_fn: impl Fn() -> (f64, f64, f64) + 'static,
) -> AlphaSlider {
    let id = ViewId::new();

    create_effect(move |_| {
        let a = alpha_signal.get();
        id.update_state(AlphaUpdate::Alpha(a));
    });

    create_effect(move |_| {
        let (r, g, b) = base_color_fn();
        id.update_state(AlphaUpdate::BaseColor(r, g, b));
    });

    AlphaSlider {
        id,
        held: false,
        alpha: 1.0,
        base_r: 0.5,
        base_g: 0.5,
        base_b: 0.5,
        size: Default::default(),
        on_change: Some(Box::new(move |a| {
            alpha_signal.set(a);
        })),
    }
    .style(|s| {
        s.height(constants::SLIDER_HEIGHT)
            .border_radius(constants::THUMB_RADIUS as f32)
            .cursor(floem::style::CursorStyle::Pointer)
    })
}

impl AlphaSlider {
    fn update_from_pointer(&mut self, x: f64) {
        let w = self.size.width as f64;
        let r = constants::THUMB_RADIUS;
        let usable = w - 2.0 * r;
        if usable > 0.0 {
            // Left = opaque, right = transparent
            self.alpha = 1.0 - ((x - r) / usable).clamp(0.0, 1.0);
        }
    }
}

impl View for AlphaSlider {
    fn id(&self) -> ViewId {
        self.id
    }

    fn update(&mut self, _cx: &mut UpdateCx, state: Box<dyn std::any::Any>) {
        if let Ok(update) = state.downcast::<AlphaUpdate>() {
            match *update {
                AlphaUpdate::Alpha(a) => self.alpha = a,
                AlphaUpdate::BaseColor(r, g, b) => {
                    self.base_r = r;
                    self.base_g = g;
                    self.base_b = b;
                }
            }
            self.id.request_layout();
        }
    }

    fn event_before_children(
        &mut self,
        cx: &mut EventCx,
        event: &Event,
    ) -> EventPropagation {
        match event {
            Event::PointerDown(e) => {
                cx.update_active(self.id());
                self.held = true;
                self.update_from_pointer(e.pos.x);
                if let Some(cb) = &self.on_change {
                    cb(self.alpha);
                }
                self.id.request_layout();
                EventPropagation::Stop
            }
            Event::PointerMove(e) => {
                if self.held {
                    self.update_from_pointer(e.pos.x);
                    if let Some(cb) = &self.on_change {
                        cb(self.alpha);
                    }
                    self.id.request_layout();
                    EventPropagation::Stop
                } else {
                    EventPropagation::Continue
                }
            }
            Event::PointerUp(_) => {
                self.held = false;
                EventPropagation::Continue
            }
            Event::FocusLost => {
                self.held = false;
                EventPropagation::Continue
            }
            _ => EventPropagation::Continue,
        }
    }

    fn compute_layout(&mut self, _cx: &mut ComputeLayoutCx) -> Option<Rect> {
        let layout = self.id.get_layout().unwrap_or_default();
        self.size = layout.size;
        None
    }

    fn paint(&mut self, cx: &mut PaintCx) {
        let w = self.size.width as f64;
        let h = self.size.height as f64;
        let rect = Rect::new(0.0, 0.0, w, h);
        let rrect = rect.to_rounded_rect(constants::THUMB_RADIUS);

        // Checkerboard background
        cx.save();
        cx.clip(&rrect);
        checkerboard::paint_checkerboard(cx, rect);

        // Opaque (left) → transparent (right)
        let solid = Color::rgba(self.base_r, self.base_g, self.base_b, 1.0);
        let transparent = Color::rgba(self.base_r, self.base_g, self.base_b, 0.0);
        let gradient =
            Gradient::new_linear((0.0, h / 2.0), (w, h / 2.0)).with_stops([solid, transparent]);
        // Convert to BezPath so the vello renderer uses the general path
        // handler (its Rect fast-path only supports solid colors).
        let path = rect.to_path(0.1);
        cx.fill(&path, &gradient, 0.0);
        cx.restore();

        // Slider outline
        cx.stroke(
            &rrect,
            Color::rgba8(0, 0, 0, 40),
            &floem::kurbo::Stroke::new(1.0),
        );

        // Thumb (circular ring; left = 1.0, right = 0.0)
        let radius = constants::THUMB_RADIUS;
        let thumb_x = radius + (1.0 - self.alpha) * (w - 2.0 * radius);
        let thumb_cy = h / 2.0;
        let circle = floem::kurbo::Circle::new((thumb_x, thumb_cy), radius);
        cx.stroke(
            &circle,
            Color::rgba8(0, 0, 0, 80),
            &floem::kurbo::Stroke::new(1.0),
        );
        let inner = floem::kurbo::Circle::new((thumb_x, thumb_cy), radius - 1.5);
        cx.stroke(
            &inner,
            Color::WHITE,
            &floem::kurbo::Stroke::new(2.0),
        );
        let innermost = floem::kurbo::Circle::new((thumb_x, thumb_cy), radius - 3.0);
        cx.stroke(
            &innermost,
            Color::rgba8(0, 0, 0, 80),
            &floem::kurbo::Stroke::new(1.0),
        );
    }
}
