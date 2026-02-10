//! Circular HSB color wheel.
//!
//! Renders a color wheel where angle maps to hue and radius maps to
//! saturation, with a brightness overlay. Uses three layers:
//! 1. Sweep gradient — full hue spectrum around the circle
//! 2. Radial gradient — white at center fading to transparent at edge
//! 3. Black overlay — darkens based on current brightness

use std::f64::consts::TAU;

use floem::kurbo::{Circle, Point, Rect, Shape};
use floem::peniko::{Color, Gradient};

use floem::reactive::{create_effect, RwSignal, SignalGet, SignalUpdate};
use floem::views::Decorators;
use floem::{
    context::{ComputeLayoutCx, EventCx, PaintCx, UpdateCx},
    event::{Event, EventPropagation},
    View, ViewId,
};
use floem_renderer::Renderer;

use bigcolor::BigColor;

use crate::constants;

enum WheelUpdate {
    HueSat(f64, f64),
    Brightness(f64),
}

pub struct ColorWheel {
    id: ViewId,
    held: bool,
    hue: f64,
    saturation: f64,
    brightness: f64,
    size: floem::taffy::prelude::Size<f32>,
    on_change: Option<Box<dyn Fn(f64, f64)>>,
}

/// Creates a circular color wheel.
///
/// - `hue`: 0.0–1.0 (angle around the wheel)
/// - `saturation`: 0.0 (center) to 1.0 (edge)
/// - `brightness`: read-only, used for the darkening overlay
pub fn color_wheel(
    hue: RwSignal<f64>,
    saturation: RwSignal<f64>,
    brightness: RwSignal<f64>,
) -> ColorWheel {
    let id = ViewId::new();

    create_effect(move |_| {
        let h = hue.get();
        let s = saturation.get();
        id.update_state(WheelUpdate::HueSat(h, s));
    });

    create_effect(move |_| {
        let b = brightness.get();
        id.update_state(WheelUpdate::Brightness(b));
    });

    ColorWheel {
        id,
        held: false,
        hue: hue.get_untracked(),
        saturation: saturation.get_untracked(),
        brightness: brightness.get_untracked(),
        size: Default::default(),
        on_change: Some(Box::new(move |h, s| {
            hue.set(h);
            saturation.set(s);
        })),
    }
    .style(|s| {
        s.flex_grow(1.0)
            .min_height(100.0)
            .cursor(floem::style::CursorStyle::Default)
    })
}

impl ColorWheel {
    fn radius(&self) -> f64 {
        let w = self.size.width as f64;
        let h = self.size.height as f64;
        w.min(h) / 2.0
    }

    fn center(&self) -> (f64, f64) {
        let w = self.size.width as f64;
        let h = self.size.height as f64;
        (w / 2.0, h / 2.0)
    }

    fn update_from_pointer(&mut self, pos: Point) {
        let (cx, cy) = self.center();
        let max_r = self.radius();
        if max_r <= 0.0 {
            return;
        }

        let dx = pos.x - cx;
        let dy = pos.y - cy;
        let angle = dy.atan2(dx); // -PI to PI
        let dist = (dx * dx + dy * dy).sqrt();
        let sat = (dist / max_r).clamp(0.0, 1.0);

        // Map angle to hue: 0 at the right (3 o'clock), going clockwise.
        // atan2 gives -PI..PI, we map to 0..1
        let mut h = angle / TAU; // -0.5..0.5
        if h < 0.0 {
            h += 1.0;
        }

        self.hue = h;
        self.saturation = sat;
    }

    fn cursor_position(&self) -> (f64, f64) {
        let (cx, cy) = self.center();
        let max_r = self.radius();
        let angle = self.hue * TAU;
        let r = self.saturation * max_r;
        (cx + angle.cos() * r, cy + angle.sin() * r)
    }
}

impl View for ColorWheel {
    fn id(&self) -> ViewId {
        self.id
    }

    fn update(&mut self, _cx: &mut UpdateCx, state: Box<dyn std::any::Any>) {
        if let Ok(update) = state.downcast::<WheelUpdate>() {
            match *update {
                WheelUpdate::HueSat(h, s) => {
                    self.hue = h;
                    self.saturation = s;
                }
                WheelUpdate::Brightness(b) => {
                    self.brightness = b;
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
                self.update_from_pointer(e.pos);
                if let Some(cb) = &self.on_change {
                    cb(self.hue, self.saturation);
                }
                self.id.request_layout();
                EventPropagation::Stop
            }
            Event::PointerMove(e) => {
                if self.held {
                    self.update_from_pointer(e.pos);
                    if let Some(cb) = &self.on_change {
                        cb(self.hue, self.saturation);
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
        if w == 0.0 || h == 0.0 {
            return;
        }

        let (center_x, center_y) = self.center();
        let radius = self.radius();
        let center_pt = Point::new(center_x, center_y);
        let circle = Circle::new(center_pt, radius);
        let path = circle.to_path(0.1);

        // Layer 1: Sweep gradient — hue around the circle
        // 361 stops at every 1° for pixel-perfect hue accuracy
        let stops: [Color; 361] = std::array::from_fn(|i| {
            let bc = BigColor::from_hsv(i as f32, 1.0, 1.0, 1.0);
            let rgb = bc.to_rgb();
            Color::rgb(rgb.r as f64 / 255.0, rgb.g as f64 / 255.0, rgb.b as f64 / 255.0)
        });
        let sweep = Gradient::new_sweep(center_pt, 0.0, TAU as f32).with_stops(stops);
        cx.fill(&path, &sweep, 0.0);

        // Layer 2: Radial gradient — white at center, transparent at edge
        let radial = Gradient::new_radial(center_pt, radius as f32).with_stops([
            Color::WHITE,
            Color::rgba(1.0, 1.0, 1.0, 0.0),
        ]);
        cx.fill(&path, &radial, 0.0);

        // Layer 3: Black overlay — darkens based on brightness
        let overlay_alpha = 1.0 - self.brightness;
        if overlay_alpha > 0.001 {
            cx.fill(&path, Color::rgba(0.0, 0.0, 0.0, overlay_alpha), 0.0);
        }

        // Draw cursor
        let (cur_x, cur_y) = self.cursor_position();
        let cur_pt = Point::new(cur_x, cur_y);
        let outer = Circle::new(cur_pt, constants::CURSOR_RADIUS + 1.0);
        cx.stroke(
            &outer,
            Color::rgba8(0, 0, 0, 80),
            &floem::kurbo::Stroke::new(1.0),
        );
        let cursor = Circle::new(cur_pt, constants::CURSOR_RADIUS);
        cx.stroke(
            &cursor,
            Color::WHITE,
            &floem::kurbo::Stroke::new(2.0),
        );
        let inner = Circle::new(cur_pt, constants::CURSOR_RADIUS - 1.5);
        cx.stroke(
            &inner,
            Color::rgba8(0, 0, 0, 80),
            &floem::kurbo::Stroke::new(1.0),
        );
    }
}
