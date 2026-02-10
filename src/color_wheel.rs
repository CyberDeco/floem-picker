//! Circular HSB color wheel.
//!
//! Renders a color wheel where angle maps to hue and radius maps to
//! saturation. The wheel is rasterized to an RGBA8 pixel buffer and
//! drawn as a single image — works on both vello and vger.

use std::f64::consts::TAU;
use std::sync::Arc;

use floem::kurbo::{BezPath, Circle, Point, Rect};
use floem::peniko::{self, Blob, Color};

use floem::reactive::{RwSignal, SignalGet, SignalUpdate, create_effect};
use floem::views::Decorators;
use floem::{
    View, ViewId,
    context::{ComputeLayoutCx, EventCx, PaintCx, UpdateCx},
    event::{Event, EventPropagation},
};
use floem_renderer::Renderer;

use crate::constants;
use crate::math;

/// Build a closed `BezPath` circle from line segments (no cubic curves).
fn circle_path(center: Point, radius: f64) -> BezPath {
    let mut path = BezPath::new();
    for i in 0..64 {
        let angle = TAU * i as f64 / 64.0;
        let pt = Point::new(
            center.x + angle.cos() * radius,
            center.y + angle.sin() * radius,
        );
        if i == 0 {
            path.move_to(pt);
        } else {
            path.line_to(pt);
        }
    }
    path.close_path();
    path
}

/// Feather width in pixels for anti-aliasing the circle edge.
const FEATHER: f64 = 1.5;

/// Rasterize the color wheel at full brightness (V=1.0) to an RGBA8 buffer.
///
/// `width`/`height` are in physical pixels. The circle is inset by
/// [`FEATHER`] so the full anti-alias gradient fits inside the buffer.
fn rasterize_wheel_base(width: u32, height: u32) -> Vec<u8> {
    let cx = width as f64 / 2.0;
    let cy = height as f64 / 2.0;
    let radius = cx.min(cy) - FEATHER;

    let mut buf = vec![0u8; (width * height * 4) as usize];

    for py in 0..height {
        let dy = py as f64 + 0.5 - cy;
        let row_offset = (py * width * 4) as usize;

        for px in 0..width {
            let dx = px as f64 + 0.5 - cx;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > radius + FEATHER {
                continue; // fully outside
            }

            // Anti-alias: smooth fade over FEATHER pixels at the edge
            let alpha = ((radius + FEATHER - dist) / FEATHER).clamp(0.0, 1.0);

            let sat = (dist / radius).min(1.0);
            let angle = dy.atan2(dx);
            let mut hue = angle / TAU;
            if hue < 0.0 {
                hue += 1.0;
            }

            let (r, g, b) = math::hsb_to_rgb(hue, sat, 1.0);
            let offset = row_offset + (px * 4) as usize;
            buf[offset] = (r * 255.0 + 0.5) as u8;
            buf[offset + 1] = (g * 255.0 + 0.5) as u8;
            buf[offset + 2] = (b * 255.0 + 0.5) as u8;
            buf[offset + 3] = (alpha * 255.0 + 0.5) as u8;
        }
    }

    buf
}

enum WheelUpdate {
    HueSat(f64, f64),
    Brightness(f64),
}

pub(crate) struct ColorWheel {
    id: ViewId,
    held: bool,
    hue: f64,
    saturation: f64,
    brightness: f64,
    size: floem::taffy::prelude::Size<f32>,
    on_change: Option<Box<dyn Fn(f64, f64)>>,
    /// Cached full-brightness wheel image, regenerated only on resize.
    wheel_img: Option<peniko::Image>,
    wheel_hash: Vec<u8>,
    /// The physical pixel dimensions of the cached image.
    cached_dims: (u32, u32),
}

/// Creates a circular color wheel.
///
/// - `hue`: 0.0–1.0 (angle around the wheel)
/// - `saturation`: 0.0 (center) to 1.0 (edge)
/// - `brightness`: read-only, used for the darkening overlay
pub(crate) fn color_wheel(
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
        wheel_img: None,
        wheel_hash: Vec::new(),
        cached_dims: (0, 0),
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

    fn ensure_wheel_image(&mut self, scale: f64) {
        let s = scale.max(1.0);
        let pw = (self.size.width as f64 * s).round() as u32;
        let ph = (self.size.height as f64 * s).round() as u32;
        if pw == 0 || ph == 0 {
            return;
        }

        let dims = (pw, ph);
        if self.cached_dims == dims {
            return; // cache is valid — only resize triggers re-rasterize
        }

        let pixels = rasterize_wheel_base(pw, ph);
        let blob = Blob::new(Arc::new(pixels));
        let img = peniko::Image::new(blob.clone(), peniko::Format::Rgba8, pw, ph);

        let id = blob.id();
        self.wheel_hash = id.to_le_bytes().to_vec();
        self.wheel_img = Some(img);
        self.cached_dims = dims;
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

    fn event_before_children(&mut self, cx: &mut EventCx, event: &Event) -> EventPropagation {
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

        // Draw the full-brightness wheel image (stable cache, only changes on resize)
        let scale = cx.scale();
        self.ensure_wheel_image(scale);
        if let Some(ref img) = self.wheel_img {
            let rect = Rect::new(0.0, 0.0, w, h);
            cx.draw_img(
                floem_renderer::Img {
                    img: img.clone(),
                    hash: &self.wheel_hash,
                },
                rect,
            );
        }

        // Brightness overlay: darken the wheel with semi-transparent black
        let overlay_alpha = 1.0 - self.brightness;
        if overlay_alpha > 0.001 {
            let overlay = circle_path(center_pt, radius - FEATHER / scale.max(1.0));
            cx.fill(&overlay, Color::rgba(0.0, 0.0, 0.0, overlay_alpha), 0.0);
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
        cx.stroke(&cursor, Color::WHITE, &floem::kurbo::Stroke::new(2.0));
        let inner = Circle::new(cur_pt, constants::CURSOR_RADIUS - 1.5);
        cx.stroke(
            &inner,
            Color::rgba8(0, 0, 0, 80),
            &floem::kurbo::Stroke::new(1.0),
        );
    }
}
