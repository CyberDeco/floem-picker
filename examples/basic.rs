//! Standalone demo: opens a window with the color picker.

use floem::prelude::*;
use floem::window::WindowConfig;
use floem_picker::{solid_picker, SolidColor};

fn main() {
    let color = RwSignal::new(SolidColor::from_hex("FFFFFF").unwrap());

    floem::Application::new()
        .window(
            move |_| {
                solid_picker(color).on_event_stop(floem::event::EventListener::WindowClosed, |_| {
                    floem::quit_app()
                })
            },
            Some(
                WindowConfig::default()
                    .size((232.0, 460.0))
                    .title("floem-picker"),
            ),
        )
        .run();
}
