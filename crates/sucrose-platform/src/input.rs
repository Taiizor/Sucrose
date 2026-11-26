//! Input forwarding to a web wallpaper, as pure values: the events a renderer receives from raw input, the legacy
//! Win32 messages they become and the filters that decide whether one is forwarded at all (WAPI-07, engines.md section 5).
//!
//! Nothing here touches the OS, so it is built and tested on every platform. The Windows side (RawInput registration and
//! decoding) lives in `os_windows/raw_input.rs` and only produces [`InputEvent`]s.

use sucrose_core::settings::InputType;

/// `WM_MOUSEMOVE`.
pub const WM_MOUSEMOVE: u32 = 0x0200;
/// `WM_LBUTTONDOWN`.
pub const WM_LBUTTONDOWN: u32 = 0x0201;
/// `WM_LBUTTONUP`.
pub const WM_LBUTTONUP: u32 = 0x0202;
/// `WM_RBUTTONDOWN`.
pub const WM_RBUTTONDOWN: u32 = 0x0204;
/// `WM_RBUTTONUP`.
pub const WM_RBUTTONUP: u32 = 0x0205;
/// `WM_MBUTTONDOWN`.
pub const WM_MBUTTONDOWN: u32 = 0x0207;
/// `WM_MBUTTONUP`.
pub const WM_MBUTTONUP: u32 = 0x0208;

/// The wParam the legacy app sends with a mouse move.
pub const MOVE_WPARAM: usize = 0x20;
/// The wParam the legacy app sends with the left button messages.
pub const LEFT_WPARAM: usize = 0x1;
/// The wParam the legacy app sends with the right button messages.
pub const RIGHT_WPARAM: usize = 0x2;
/// The wParam the legacy app sends with the middle button messages.
pub const MIDDLE_WPARAM: usize = 0x10;

/// A mouse button the page can see.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// The left button.
    Left,
    /// The right button.
    Right,
    /// The middle button (the wheel's).
    Middle,
}

/// One input event from the OS. Coordinates are physical pixels on the virtual screen (the cursor position when the
/// event arrived), not relative to any wallpaper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    /// The cursor moved.
    MouseMove {
        /// Cursor x on the virtual screen.
        x: i32,
        /// Cursor y on the virtual screen.
        y: i32,
    },
    /// A button went down or up.
    Button {
        /// Which button.
        button: MouseButton,
        /// `true` for down, `false` for up.
        down: bool,
        /// Cursor x on the virtual screen.
        x: i32,
        /// Cursor y on the virtual screen.
        y: i32,
    },
}

impl InputEvent {
    /// The cursor position this event carries, if it carries one.
    #[must_use]
    pub fn position(&self) -> Option<(i32, i32)> {
        match *self {
            Self::MouseMove { x, y } | Self::Button { x, y, .. } => Some((x, y)),
        }
    }
}

/// What the user configured about input forwarding (`InputType`, `InputDesktop`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputFilter {
    /// Which devices are forwarded.
    pub input_type: InputType,
    /// Forward only while the desktop is the foreground window (`InputDesktop`).
    pub desktop_only: bool,
}

/// The legacy message for a mouse event, as `(message, wParam, lParam)`, with the position made relative to
/// `origin` (the left and top of the wallpaper's rectangle on the virtual screen). `None` for an event that is not a
/// plain mouse message.
#[must_use]
pub fn mouse_message(event: &InputEvent, origin: (i32, i32)) -> Option<(u32, usize, isize)> {
    let _ = (event, origin);
    unimplemented!("mouse_message")
}

/// Whether a renderer forwards `event`: `cursor_inside` is whether the cursor is inside this renderer's rectangle,
/// `desktop_foreground` whether the desktop is the foreground window.
#[must_use]
pub fn should_forward(filter: &InputFilter, event: &InputEvent, cursor_inside: bool, desktop_foreground: bool) -> bool {
    let _ = (filter, event, cursor_inside, desktop_foreground);
    unimplemented!("should_forward")
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOUSE_KEYBOARD: InputFilter = InputFilter {
        input_type: InputType::MouseKeyboard,
        desktop_only: false,
    };

    #[test]
    fn a_mouse_move_is_made_relative_to_the_monitor_and_carries_the_move_flag() {
        let message = mouse_message(&InputEvent::MouseMove { x: 2100, y: 300 }, (1920, 0));

        assert_eq!(message, Some((WM_MOUSEMOVE, 0x20, (300 << 16) | 180)));
    }

    #[test]
    fn the_buttons_map_to_the_legacy_messages_and_wparams() {
        let button = |button, down| mouse_message(&InputEvent::Button { button, down, x: 10, y: 20 }, (0, 0));
        let lparam = (20 << 16) | 10;

        assert_eq!(button(MouseButton::Left, true), Some((WM_LBUTTONDOWN, 0x1, lparam)));
        assert_eq!(button(MouseButton::Left, false), Some((WM_LBUTTONUP, 0x1, lparam)));
        assert_eq!(button(MouseButton::Right, true), Some((WM_RBUTTONDOWN, 0x2, lparam)));
        assert_eq!(button(MouseButton::Right, false), Some((WM_RBUTTONUP, 0x2, lparam)));
        assert_eq!(button(MouseButton::Middle, true), Some((WM_MBUTTONDOWN, 0x10, lparam)));
        assert_eq!(button(MouseButton::Middle, false), Some((WM_MBUTTONUP, 0x10, lparam)));
    }

    #[test]
    fn a_mouse_event_is_forwarded_only_while_the_cursor_is_inside_the_renderers_rectangle() {
        let moved = InputEvent::MouseMove { x: 5, y: 5 };

        assert!(should_forward(&MOUSE_KEYBOARD, &moved, true, true));
        assert!(!should_forward(&MOUSE_KEYBOARD, &moved, false, true));
    }
}
