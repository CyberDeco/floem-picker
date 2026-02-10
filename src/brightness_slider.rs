//! Brightness slider (0.0–1.0).
//!
//! Horizontal gradient from current color at full brightness
//! (left) to black (right) as a rasterized image.

use std::sync::Arc;

use floem::kurbo::Rect;
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

/// Rasterize horizontal gradient: `(r, g, b)` on the left -> black on the right.
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

pub(crate) struct BrightnessSlider {
    id: ViewId,
    held: bool,
    brightness: f64,
    base_r: f64,
    base_g: f64,
    base_b: f64,
    size: floem::taffy::prelude::Size<f32>,
    on_change: Option<Box<dyn Fn(f64)>>,
    /// Cached gradient image, rasterized at a fixed resolution.
    grad_img: Option<peniko::Image>,
    grad_hash: Vec<u8>,
    cached_color: (u8, u8, u8),
}

/// Creates a horizontal brightness slider.
///
/// - `hue`, `saturation`: read-only, used to compute the gradient's end color.
/// - `brightness`: 0.0 (black, left) to 1.0 (full color, right).
pub(crate) fn brightness_slider(
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

    let (r, g, b) = math::hsb_to_rgb(hue.get_untracked(), saturation.get_untracked(), 1.0);

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

    /// Rasterize at a fixed resolution, only when the base color changes.
    /// The renderer scales the image to the actual widget size.
    fn ensure_gradient_image(&mut self) {
        let color_key = (
            (self.base_r * 255.0 + 0.5) as u8,
            (self.base_g * 255.0 + 0.5) as u8,
            (self.base_b * 255.0 + 0.5) as u8,
        );
        if self.grad_img.is_some() && self.cached_color == color_key {
            return;
        }

        let pw = constants::SLIDER_RASTER_WIDTH;
        let ph = constants::SLIDER_RASTER_HEIGHT;
        let pixels = rasterize_brightness_gradient(pw, ph, self.base_r, self.base_g, self.base_b);
        let blob = Blob::new(Arc::new(pixels));
        let img = peniko::Image::new(blob, peniko::Format::Rgba8, pw, ph);

        self.grad_hash = [
            b"bri" as &[u8],
            &color_key.0.to_le_bytes(),
            &color_key.1.to_le_bytes(),
            &color_key.2.to_le_bytes(),
        ]
        .concat();
        self.grad_img = Some(img);
        self.cached_color = color_key;
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

    fn event_before_children(&mut self, cx: &mut EventCx, event: &Event) -> EventPropagation {
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

        // Rounded ends on sliders
        cx.save();
        cx.clip(&rrect);

        // Full-brightness color (left) -> black (right) as raster
        self.ensure_gradient_image();
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

        // Filled thumbs (same pattern as color wheel cursor)
        let radius = constants::THUMB_RADIUS;
        let thumb_x = radius + (1.0 - self.brightness) * (w - 2.0 * radius);
        let thumb_cy = h / 2.0;
        cx.fill(
            &floem::kurbo::Circle::new((thumb_x, thumb_cy), radius + 1.5),
            Color::rgba8(0, 0, 0, 80),
            0.0,
        );
        cx.fill(
            &floem::kurbo::Circle::new((thumb_x, thumb_cy), radius),
            Color::WHITE,
            0.0,
        );
        cx.fill(
            &floem::kurbo::Circle::new((thumb_x, thumb_cy), radius - 2.0),
            Color::rgba8(0, 0, 0, 150),
            0.0,
        );
        cx.fill(
            &floem::kurbo::Circle::new((thumb_x, thumb_cy), radius - 3.0),
            Color::rgb(
                self.base_r * self.brightness,
                self.base_g * self.brightness,
                self.base_b * self.brightness,
            ),
            0.0,
        );
    }
}
