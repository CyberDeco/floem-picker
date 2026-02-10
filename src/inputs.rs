//! Numeric input components for color channel editing.

use floem::event::EventPropagation;
use floem::prelude::*;
use floem::reactive::{RwSignal, SignalGet, SignalUpdate, create_effect};

use crate::constants;

/// A numeric input that maps a normalized 0.0–1.0 signal to a display range.
///
/// For example, hue maps 0.0–1.0 → 0–360, saturation maps 0.0–1.0 → 0–100.
pub(crate) fn number_input(
    lbl: &'static str,
    signal: RwSignal<f64>,
    max_display: f64,
) -> impl IntoView {
    let text = RwSignal::new(format_value(signal.get_untracked(), max_display));

    // Signal → text (external updates)
    create_effect(move |_| {
        let val = signal.get();
        let display = (val * max_display).round();
        let current = text.get_untracked();
        let expected = format!("{}", display as i64);
        if current != expected {
            text.set(expected);
        }
    });

    let on_commit = move || {
        let raw = text.get_untracked();
        if let Ok(num) = raw.parse::<f64>() {
            let clamped = num.clamp(0.0, max_display);
            let new_display = clamped.round() as i64;
            let old_display = (signal.get_untracked() * max_display).round() as i64;
            if new_display != old_display {
                signal.set(clamped / max_display);
            }
            let formatted = format!("{}", new_display);
            if raw != formatted {
                text.set(formatted);
            }
        } else {
            // Reset to current signal value
            let formatted = format!("{}", (signal.get_untracked() * max_display).round() as i64);
            if raw != formatted {
                text.set(formatted);
            }
        }
    };

    let on_commit_clone = on_commit;

    v_stack((
        text_input(text)
            .style(|s| {
                s.width(constants::INPUT_WIDTH)
                    .padding(2.0)
                    .font_size(constants::INPUT_FONT)
                    .font_family("monospace".to_string())
                    .background(Color::WHITE)
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(3.0)
            })
            .on_event_stop(floem::event::EventListener::FocusLost, move |_| {
                on_commit();
            })
            .on_event(floem::event::EventListener::KeyDown, move |e| {
                if let floem::event::Event::KeyDown(ke) = e
                    && ke.key.logical_key
                        == floem::keyboard::Key::Named(floem::keyboard::NamedKey::Enter)
                {
                    on_commit_clone();
                    return EventPropagation::Stop;
                }
                EventPropagation::Continue
            }),
        label(move || lbl).style(|s| {
            s.font_size(constants::LABEL_FONT)
                .color(Color::rgb8(120, 120, 120))
                .justify_content(Some(floem::taffy::AlignContent::Center))
        }),
    ))
    .style(|s| s.items_center().gap(1.0))
}

fn format_value(normalized: f64, max: f64) -> String {
    let display = (normalized * max).round() as i64;
    format!("{}", display)
}

/// A hex input field that syncs bidirectionally with an RwSignal<String>.
///
/// Updates the color dynamically as the user types valid hex values.
pub(crate) fn hex_input(hex_signal: RwSignal<String>) -> impl IntoView {
    let text = RwSignal::new(hex_signal.get_untracked());

    // External hex_signal → text (only update if not equivalent)
    create_effect(move |_| {
        let val = hex_signal.get();
        let current = text.get_untracked();
        let current_normalized = current.trim_start_matches('#').to_uppercase();
        if current_normalized != val {
            text.set(val);
        }
    });

    // Dynamic: text → hex_signal on every valid keystroke
    create_effect(move |_| {
        let raw = text.get();
        let trimmed = raw.trim_start_matches('#');
        if (trimmed.len() == 6 || trimmed.len() == 8)
            && trimmed.chars().all(|c| c.is_ascii_hexdigit())
        {
            let upper = trimmed.to_uppercase();
            if hex_signal.get_untracked() != upper {
                hex_signal.set(upper);
            }
        }
    });

    let on_commit = move || {
        let raw = text.get_untracked();
        let normalized = crate::math::normalize_hex(&raw);
        if raw != normalized {
            text.set(normalized.clone());
        }
        if hex_signal.get_untracked() != normalized {
            hex_signal.set(normalized);
        }
    };
    let on_commit_clone = on_commit;

    h_stack((
        label(|| "#").style(|s| {
            s.font_size(constants::INPUT_FONT)
                .font_family("monospace".to_string())
                .color(Color::rgb8(120, 120, 120))
        }),
        text_input(text)
            .style(|s| {
                s.width(constants::HEX_INPUT_WIDTH)
                    .padding(2.0)
                    .font_size(constants::INPUT_FONT)
                    .font_family("monospace".to_string())
                    .background(Color::WHITE)
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(3.0)
            })
            .on_event_stop(floem::event::EventListener::FocusLost, move |_| {
                on_commit();
            })
            .on_event_stop(floem::event::EventListener::KeyDown, move |e| {
                if let floem::event::Event::KeyDown(ke) = e
                    && ke.key.logical_key
                        == floem::keyboard::Key::Named(floem::keyboard::NamedKey::Enter)
                {
                    on_commit_clone();
                }
            }),
    ))
    .style(|s| s.items_center().gap(1.0))
}

/// An editable percentage input for alpha (0–100%).
///
/// Shows a numeric text field with a `%` label to its right. The user types
/// a plain number; it is committed on Enter or focus-lost and clamped to 0–100.
#[cfg(feature = "alpha")]
pub(crate) fn alpha_input(signal: RwSignal<f64>) -> impl IntoView {
    let text = RwSignal::new(format!(
        "{}",
        (signal.get_untracked() * 100.0).round() as i64
    ));

    // Signal → text
    create_effect(move |_| {
        let val = signal.get();
        let display = format!("{}", (val * 100.0).round() as i64);
        if text.get_untracked() != display {
            text.set(display);
        }
    });

    let on_commit = move || {
        let raw = text.get_untracked();
        if let Ok(num) = raw.trim().parse::<f64>() {
            let clamped = num.clamp(0.0, 100.0);
            let new_display = clamped.round() as i64;
            let old_display = (signal.get_untracked() * 100.0).round() as i64;
            if new_display != old_display {
                signal.set(clamped / 100.0);
            }
            let formatted = format!("{}", new_display);
            if raw.trim() != formatted {
                text.set(formatted);
            }
        } else {
            let formatted = format!("{}", (signal.get_untracked() * 100.0).round() as i64);
            if raw != formatted {
                text.set(formatted);
            }
        }
    };
    let on_commit_clone = on_commit;

    h_stack((
        text_input(text)
            .style(|s| {
                s.width(28.0)
                    .padding(2.0)
                    .font_size(constants::INPUT_FONT)
                    .font_family("monospace".to_string())
                    .background(Color::WHITE)
                    .border(1.0)
                    .border_color(Color::rgb8(200, 200, 200))
                    .border_radius(3.0)
            })
            .on_event_stop(floem::event::EventListener::FocusLost, move |_| {
                on_commit();
            })
            .on_event(floem::event::EventListener::KeyDown, move |e| {
                if let floem::event::Event::KeyDown(ke) = e
                    && ke.key.logical_key
                        == floem::keyboard::Key::Named(floem::keyboard::NamedKey::Enter)
                {
                    on_commit_clone();
                    return EventPropagation::Stop;
                }
                EventPropagation::Continue
            }),
        label(|| "%").style(|s| {
            s.font_size(constants::LABEL_FONT)
                .color(Color::rgb8(120, 120, 120))
        }),
    ))
    .style(|s| s.items_center().gap(2.0))
}

/// A small copy button that copies the result of `get_text` to the clipboard.
pub(crate) fn copy_button(get_text: impl Fn() -> String + 'static) -> impl IntoView {
    let pressed = RwSignal::new(false);
    container(
        label(|| lucide_icons::Icon::Copy.unicode().to_string()).style(move |s| {
            let c = if pressed.get() {
                Color::rgb8(80, 80, 80)
            } else {
                Color::rgb8(120, 120, 120)
            };
            s.font_size(14.0).font_family("lucide".to_string()).color(c)
        }),
    )
    .style(|s| {
        s.size(20.0, 20.0)
            .items_center()
            .justify_center()
            .border_radius(3.0)
            .cursor(floem::style::CursorStyle::Pointer)
            .align_self(Some(floem::taffy::AlignItems::Start))
            .hover(|s| s.background(Color::rgb8(230, 230, 230)))
    })
    .on_event_stop(floem::event::EventListener::PointerDown, move |_| {
        pressed.set(true);
    })
    .on_event_stop(floem::event::EventListener::PointerUp, move |_| {
        pressed.set(false);
        copy_to_clipboard(&get_text());
    })
}

fn copy_to_clipboard(text: &str) {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(text);
    }
}
