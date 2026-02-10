//! Brightness slider (0.0–1.0).
//!
//! Renders a horizontal gradient from the current color at full brightness
//! (left) to black (right) as a rasterized image, avoiding vger's broken
//! linear gradient coordinate handling.

use std::sync::Arc;

use floem::kurbo::Rect;
use floem::peniko::{self, Blob, Color};

use floem::reactive::{create_effect, RwSignal, SignalGet, SignalUpdate};
use floem::views::Decorators;
use floem::{
    context::{ComputeLayoutCx, EventCx, PaintCx, UpdateCx},
    event::{Event, EventPropagation},
    View, ViewId,
};
use floem_renderer::Renderer;

use crate::constants;
use crate::math;

/// Rasterize a horizontal gradient: `(r, g, b)` on the left → black on the right.
fn rasterize_brightness_gradient(width: u32, height: u32, r: f64, g: f64, b: f64) -> Vec<u8> {
    let mut buf = vec![0u8; (width * height * 4) as usize];
    for px in 0..width {
        let t = px as f64 / (width - 1).max(1) as f64; // 0 at left, 1 at right
        let cr = ((1.0 - t) * r * 255.0 + 0.5) as u8;
        let cg = ((1.0 - t) * g * 255.0 + 0.5) as u8;
        let cb = ((1.0 - t) * b * 255.0 + 0.5) as u8;
        for py in 0..height {
            let offset = ((py * width + px) * 4) as usize;
            buf[offset] = cr;
            buf[offset + 1] = cg;
            buf[offset + 2] = cb;
            buf[offset + 3] = 255;
        }
    }
    buf
}

enum BrightnessUpdate {
    Value(f64),
    BaseColor(f64, f64, f64),
}

pub struct BrightnessSlider {
    id: ViewId,
    held: bool,
    brightness: f64,
    base_r: f64,
    base_g: f64,
    base_b: f64,
    size: floem::taffy::prelude::Size<f32>,
    on_change: Option<Box<dyn Fn(f64)>>,
    /// Cached gradient image.
    grad_img: Option<peniko::Image>,
    grad_hash: Vec<u8>,
    cached_color: (u8, u8, u8),
    cached_dims: (u32, u32),
}

/// Creates a horizontal brightness slider.
///
/// - `hue`, `saturation`: read-only, used to compute the gradient's end color.
/// - `brightness`: 0.0 (black, left) to 1.0 (full color, right).
pub fn brightness_slider(
    hue: RwSignal<f64>,
    saturation: RwSignal<f64>,
    brightness: RwSignal<f64>,
) -> BrightnessSlider {
    let id = ViewId::new();

    create_effect(move |_| {
        let b = brightness.get();
        id.update_state(BrightnessUpdate::Value(b));
    });

    create_effect(move |_| {
        let h = hue.get();
        let s = saturation.get();
        let (r, g, b) = math::hsb_to_rgb(h, s, 1.0);
        id.update_state(BrightnessUpdate::BaseColor(r, g, b));
    });

    let (r, g, b) = math::hsb_to_rgb(
        hue.get_untracked(),
        saturation.get_untracked(),
        1.0,
    );

    BrightnessSlider {
        id,
        held: false,
        brightness: brightness.get_untracked(),
        base_r: r,
        base_g: g,
        base_b: b,
        size: Default::default(),
        on_change: Some(Box::new(move |val| {
            brightness.set(val);
        })),
        grad_img: None,
        grad_hash: Vec::new(),
        cached_color: (0, 0, 0),
        cached_dims: (0, 0),
    }
    .style(|s| {
        s.height(constants::SLIDER_HEIGHT)
            .border_radius(constants::THUMB_RADIUS as f32)
            .cursor(floem::style::CursorStyle::Pointer)
    })
}

impl BrightnessSlider {
    fn update_from_pointer(&mut self, x: f64) {
        let w = self.size.width as f64;
        let r = constants::THUMB_RADIUS;
        let usable = w - 2.0 * r;
        if usable > 0.0 {
            // Left = full brightness, right = black
            self.brightness = 1.0 - ((x - r) / usable).clamp(0.0, 1.0);
        }
    }

    fn ensure_gradient_image(&mut self, scale: f64) {
        let s = scale.max(1.0);
        let pw = (self.size.width as f64 * s).round() as u32;
        let ph = (self.size.height as f64 * s).round() as u32;
        if pw == 0 || ph == 0 {
            return;
        }

        let color_key = (
            (self.base_r * 255.0 + 0.5) as u8,
            (self.base_g * 255.0 + 0.5) as u8,
            (self.base_b * 255.0 + 0.5) as u8,
        );
        let dims = (pw, ph);
        if self.cached_dims == dims && self.cached_color == color_key {
            return;
        }

        let pixels = rasterize_brightness_gradient(pw, ph, self.base_r, self.base_g, self.base_b);
        let blob = Blob::new(Arc::new(pixels));
        let img = peniko::Image::new(blob.clone(), peniko::Format::Rgba8, pw, ph);

        let id = blob.id();
        self.grad_hash = id.to_le_bytes().to_vec();
        self.grad_img = Some(img);
        self.cached_color = color_key;
        self.cached_dims = dims;
    }
}

impl View for BrightnessSlider {
    fn id(&self) -> ViewId {
        self.id
    }

    fn update(&mut self, _cx: &mut UpdateCx, state: Box<dyn std::any::Any>) {
        if let Ok(update) = state.downcast::<BrightnessUpdate>() {
            match *update {
                BrightnessUpdate::Value(val) => self.brightness = val,
                BrightnessUpdate::BaseColor(r, g, b) => {
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
                    cb(self.brightness);
                }
                self.id.request_layout();
                EventPropagation::Stop
            }
            Event::PointerMove(e) => {
                if self.held {
                    self.update_from_pointer(e.pos.x);
                    if let Some(cb) = &self.on_change {
                        cb(self.brightness);
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
        let rect = Rect::new(0.0, 0.0, w, h);
        let rrect = rect.to_rounded_rect(constants::THUMB_RADIUS);

        // Clip to rounded rect for rounded ends
        cx.save();
        cx.clip(&rrect);

        // Full-brightness color (left) → black (right) as an image
        let scale = cx.scale();
        self.ensure_gradient_image(scale);
        if let Some(ref img) = self.grad_img {
            cx.draw_img(
                floem_renderer::Img {
                    img: img.clone(),
                    hash: &self.grad_hash,
                },
                rect,
            );
        }

        cx.restore();

        // Slider outline
        cx.stroke(
            &rrect,
            Color::rgba8(0, 0, 0, 40),
            &floem::kurbo::Stroke::new(1.0),
        );

        // Thumb (circular ring; left = 1.0, right = 0.0)
        let radius = constants::THUMB_RADIUS;
        let thumb_x = radius + (1.0 - self.brightness) * (w - 2.0 * radius);
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
