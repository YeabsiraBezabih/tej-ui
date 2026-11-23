//! DOM event translation to unified cross-platform `UiEvent`s.

use tej_ui_hal::event::{MouseButton, UiEvent};
use tej_ui_layout::Point;

/// Translator converting DOM event data payloads into `UiEvent`s.
#[derive(Clone, Copy, Debug, Default)]
pub struct WebEventBridge;

impl WebEventBridge {
    /// Translates DOM `pointermove` or `mousemove` coordinates.
    #[must_use]
    pub const fn on_mouse_move(x: f32, y: f32) -> UiEvent {
        UiEvent::MouseMove {
            position: Point::new(x, y),
        }
    }

    /// Translates DOM `pointerdown` or `mousedown`.
    #[must_use]
    pub fn on_mouse_down(button: u16, x: f32, y: f32) -> UiEvent {
        let b = match button {
            0 => MouseButton::Left,
            2 => MouseButton::Right,
            _ => MouseButton::Middle,
        };
        UiEvent::MouseDown {
            button: b,
            position: Point::new(x, y),
        }
    }

    /// Translates DOM `pointerup` or `mouseup`.
    #[must_use]
    pub fn on_mouse_up(button: u16, x: f32, y: f32) -> UiEvent {
        let b = match button {
            0 => MouseButton::Left,
            2 => MouseButton::Right,
            _ => MouseButton::Middle,
        };
        UiEvent::MouseUp {
            button: b,
            position: Point::new(x, y),
        }
    }

    /// Translates DOM `wheel` event with delta normalization.
    #[must_use]
    pub const fn on_wheel(delta_x: f32, delta_y: f32, x: f32, y: f32) -> UiEvent {
        UiEvent::Scroll {
            delta_x,
            delta_y,
            position: Point::new(x, y),
        }
    }

    /// Translates DOM `keydown` event.
    #[must_use]
    pub const fn on_key_down(key_code: u32) -> UiEvent {
        UiEvent::KeyDown { key_code }
    }

    /// Translates DOM `keyup` event.
    #[must_use]
    pub const fn on_key_up(key_code: u32) -> UiEvent {
        UiEvent::KeyUp { key_code }
    }

    /// Translates DOM `input` or printable character commits.
    #[must_use]
    pub fn on_text_input(text: &str) -> UiEvent {
        UiEvent::TextInput {
            text: text.to_string(),
        }
    }

    /// Translates window resize or CSS container query changes.
    #[must_use]
    pub const fn on_resize(width: f32, height: f32) -> UiEvent {
        UiEvent::Resize { width, height }
    }

    /// Translates `window.devicePixelRatio` changes.
    #[must_use]
    pub const fn on_dpr_change(scale_factor: f32) -> UiEvent {
        UiEvent::ScaleFactorChanged { scale_factor }
    }
}
