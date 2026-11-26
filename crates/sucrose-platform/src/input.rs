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
    /// The wheel turned; `delta` is the raw amount (120 per notch, positive away from the user).
    Wheel {
        /// The raw wheel amount.
        delta: i32,
        /// Cursor x on the virtual screen.
        x: i32,
        /// Cursor y on the virtual screen.
        y: i32,
    },
    /// A key went down or up.
    Key {
        /// The Windows virtual-key code.
        vk: u16,
        /// The scan code.
        scan: u16,
        /// `true` for down, `false` for up.
        down: bool,
        /// Whether the key is an extended key (the `E0` prefix).
        extended: bool,
    },
    /// The one click a keyboard-only wallpaper needs before its first key, so the page has the keyboard focus. The
    /// renderer makes it itself; it never comes from the OS.
    FocusClick,
}

impl InputEvent {
    /// The cursor position this event carries, if it carries one.
    #[must_use]
    pub fn position(&self) -> Option<(i32, i32)> {
        match *self {
            Self::MouseMove { x, y } | Self::Button { x, y, .. } | Self::Wheel { x, y, .. } => Some((x, y)),
            Self::Key { .. } | Self::FocusClick => None,
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

/// The legacy mouse lParam: the y coordinate in the high word, the x coordinate in the low word (`(Y << 16) | X`).
/// Each coordinate is cut to 16 bits, as the message format has no room for more.
#[must_use]
pub fn mouse_lparam(x: i32, y: i32) -> isize {
    let packed = ((y as u32 & 0xFFFF) << 16) | (x as u32 & 0xFFFF);
    packed as usize as isize
}

/// The legacy message for a mouse event, as `(message, wParam, lParam)`, with the position made relative to
/// `origin` (the left and top of the wallpaper's rectangle on the virtual screen). `None` for an event that is not a
/// plain mouse message.
#[must_use]
pub fn mouse_message(event: &InputEvent, origin: (i32, i32)) -> Option<(u32, usize, isize)> {
    match *event {
        InputEvent::MouseMove { x, y } => Some((
            WM_MOUSEMOVE,
            MOVE_WPARAM,
            mouse_lparam(x.saturating_sub(origin.0), y.saturating_sub(origin.1)),
        )),
        InputEvent::Button { button, down, x, y } => {
            let (message, wparam) = match (button, down) {
                (MouseButton::Left, true) => (WM_LBUTTONDOWN, LEFT_WPARAM),
                (MouseButton::Left, false) => (WM_LBUTTONUP, LEFT_WPARAM),
                (MouseButton::Right, true) => (WM_RBUTTONDOWN, RIGHT_WPARAM),
                (MouseButton::Right, false) => (WM_RBUTTONUP, RIGHT_WPARAM),
                (MouseButton::Middle, true) => (WM_MBUTTONDOWN, MIDDLE_WPARAM),
                (MouseButton::Middle, false) => (WM_MBUTTONUP, MIDDLE_WPARAM),
            };
            Some((
                message,
                wparam,
                mouse_lparam(x.saturating_sub(origin.0), y.saturating_sub(origin.1)),
            ))
        }
        InputEvent::Wheel { .. } | InputEvent::Key { .. } | InputEvent::FocusClick => None,
    }
}

/// The legacy lParam of a key message: repeat count 1, the scan code in bits 16 to 23, the extended-key bit always set
/// (as the legacy app does) and, for a key up, the previous-state and transition bits (`3 << 30`). The context
/// (Alt) bit is never set: Alt combinations are not promised, as in the legacy app.
#[must_use]
pub fn key_lparam(scan: u16, down: bool) -> isize {
    let _ = (scan, down);
    unimplemented!("key_lparam")
}

/// Whether the result of `MapVirtualKey(vk, MAPVK_VK_TO_CHAR)` marks a dead key (its high bit).
#[must_use]
pub fn is_dead_key(mapped: u32) -> bool {
    let _ = mapped;
    unimplemented!("is_dead_key")
}

/// The key event of one raw keyboard record, or `None` for a dead key, which the legacy app skips. `flags` is
/// `RAWKEYBOARD.Flags` (bit 0 break, bit 1 `E0`) and `mapped_char` the `MapVirtualKey` result for `vk`.
#[must_use]
pub fn key_event(vk: u16, scan: u16, flags: u16, mapped_char: u32) -> Option<InputEvent> {
    let _ = (vk, scan, flags, mapped_char);
    unimplemented!("key_event")
}

/// The two messages of the legacy one-time click at (99999, 99999), far outside the page, that gives it the keyboard
/// focus: left button down, then up.
#[must_use]
pub fn focus_click_messages() -> [(u32, usize, isize); 2] {
    unimplemented!("focus_click_messages")
}

/// Whether the foreground window class is the desktop: `WorkerW`, `SHELLDLL_DefView` or `Progman` (`InputDesktop`).
#[must_use]
pub fn is_desktop_class(class_name: &str) -> bool {
    let _ = class_name;
    unimplemented!("is_desktop_class")
}

/// The devices raw input has to be registered for under `input_type`: `(mouse, keyboard)`. `Close` registers nothing.
#[must_use]
pub fn devices(input_type: InputType) -> (bool, bool) {
    match input_type {
        InputType::Close => (false, false),
        InputType::OnlyMouse => (true, false),
        InputType::OnlyKeyboard => (false, true),
        InputType::MouseKeyboard => (true, true),
    }
}

/// Whether the cursor at `(x, y)` is inside `rect` (left, top, right, bottom; the right and bottom edges are outside).
#[must_use]
pub fn cursor_inside(rect: [i32; 4], (x, y): (i32, i32)) -> bool {
    let [left, top, right, bottom] = rect;
    x >= left && x < right && y >= top && y < bottom
}

/// The raw mouse button flags (`RAWMOUSE.usButtonFlags`) the legacy app reacts to.
mod ri {
    pub const LEFT_DOWN: u16 = 0x0001;
    pub const LEFT_UP: u16 = 0x0002;
    pub const RIGHT_DOWN: u16 = 0x0004;
    pub const RIGHT_UP: u16 = 0x0008;
    pub const MIDDLE_DOWN: u16 = 0x0010;
    pub const MIDDLE_UP: u16 = 0x0020;
}

/// Turns the button flags of one raw mouse record into events at the cursor position `(x, y)`: a record without any
/// button flag is a movement (the legacy app sends a move only for those), each button flag is one press or release.
pub fn mouse_events(button_flags: u16, button_data: u16, (x, y): (i32, i32), mut emit: impl FnMut(InputEvent)) {
    let _ = button_data;
    let buttons = [
        (ri::LEFT_DOWN, MouseButton::Left, true),
        (ri::LEFT_UP, MouseButton::Left, false),
        (ri::RIGHT_DOWN, MouseButton::Right, true),
        (ri::RIGHT_UP, MouseButton::Right, false),
        (ri::MIDDLE_DOWN, MouseButton::Middle, true),
        (ri::MIDDLE_UP, MouseButton::Middle, false),
    ];
    if button_flags == 0 {
        emit(InputEvent::MouseMove { x, y });
        return;
    }
    for (flag, button, down) in buttons {
        if button_flags & flag != 0 {
            emit(InputEvent::Button { button, down, x, y });
        }
    }
}

/// Whether a renderer forwards `event`: `cursor_inside` is whether the cursor is inside this renderer's rectangle,
/// `desktop_foreground` whether the desktop is the foreground window.
#[must_use]
pub fn should_forward(filter: &InputFilter, event: &InputEvent, cursor_inside: bool, desktop_foreground: bool) -> bool {
    let _ = (filter, desktop_foreground);
    match event {
        InputEvent::MouseMove { .. } | InputEvent::Button { .. } | InputEvent::Wheel { .. } => cursor_inside,
        InputEvent::Key { .. } | InputEvent::FocusClick => unimplemented!("keys"),
    }
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
        let button = |button, down| {
            mouse_message(
                &InputEvent::Button {
                    button,
                    down,
                    x: 10,
                    y: 20,
                },
                (0, 0),
            )
        };
        let lparam = (20 << 16) | 10;

        assert_eq!(button(MouseButton::Left, true), Some((WM_LBUTTONDOWN, 0x1, lparam)));
        assert_eq!(button(MouseButton::Left, false), Some((WM_LBUTTONUP, 0x1, lparam)));
        assert_eq!(button(MouseButton::Right, true), Some((WM_RBUTTONDOWN, 0x2, lparam)));
        assert_eq!(button(MouseButton::Right, false), Some((WM_RBUTTONUP, 0x2, lparam)));
        assert_eq!(button(MouseButton::Middle, true), Some((WM_MBUTTONDOWN, 0x10, lparam)));
        assert_eq!(button(MouseButton::Middle, false), Some((WM_MBUTTONUP, 0x10, lparam)));
    }

    #[test]
    fn the_cursor_rule_takes_the_left_and_top_edges_in_and_the_right_and_bottom_edges_out() {
        let rect = [1920, 0, 3840, 1080];

        assert!(cursor_inside(rect, (1920, 0)));
        assert!(cursor_inside(rect, (3839, 1079)));
        assert!(!cursor_inside(rect, (1919, 500)));
        assert!(!cursor_inside(rect, (3840, 500)));
        assert!(!cursor_inside(rect, (2000, 1080)));
    }

    #[test]
    fn a_raw_record_without_button_flags_is_a_move_and_each_button_flag_is_one_event() {
        let collect = |flags| {
            let mut events = Vec::new();
            mouse_events(flags, 0, (7, 8), |event| events.push(event));
            events
        };

        assert_eq!(collect(0), [InputEvent::MouseMove { x: 7, y: 8 }]);
        assert_eq!(
            collect(0x0001),
            [InputEvent::Button {
                button: MouseButton::Left,
                down: true,
                x: 7,
                y: 8
            }]
        );
        assert_eq!(
            collect(0x0020),
            [InputEvent::Button {
                button: MouseButton::Middle,
                down: false,
                x: 7,
                y: 8
            }]
        );
        assert_eq!(collect(0x0004 | 0x0002).len(), 2);
    }

    const ALL_OFF: InputFilter = InputFilter {
        input_type: InputType::Close,
        desktop_only: false,
    };
    const MOUSE_EVENT: InputEvent = InputEvent::MouseMove { x: 1, y: 1 };
    const WHEEL_EVENT: InputEvent = InputEvent::Wheel { delta: 120, x: 1, y: 1 };
    const KEY_EVENT: InputEvent = InputEvent::Key { vk: 0x41, scan: 0x1E, down: true, extended: false };

    fn filter(input_type: InputType, desktop_only: bool) -> InputFilter {
        InputFilter { input_type, desktop_only }
    }

    #[test]
    fn closed_forwards_nothing_and_the_two_single_device_types_drop_the_other_device() {
        for event in [MOUSE_EVENT, WHEEL_EVENT, KEY_EVENT, InputEvent::FocusClick] {
            assert!(!should_forward(&ALL_OFF, &event, true, true), "{event:?}");
        }
        let only_mouse = filter(InputType::OnlyMouse, false);
        assert!(should_forward(&only_mouse, &MOUSE_EVENT, true, true));
        assert!(should_forward(&only_mouse, &WHEEL_EVENT, true, true));
        assert!(!should_forward(&only_mouse, &KEY_EVENT, true, true));
        let only_keyboard = filter(InputType::OnlyKeyboard, false);
        assert!(should_forward(&only_keyboard, &KEY_EVENT, true, true));
        assert!(!should_forward(&only_keyboard, &MOUSE_EVENT, true, true));
        assert!(!should_forward(&only_keyboard, &WHEEL_EVENT, true, true));
        for event in [MOUSE_EVENT, WHEEL_EVENT, KEY_EVENT] {
            assert!(should_forward(&MOUSE_KEYBOARD, &event, true, true), "{event:?}");
        }
    }

    #[test]
    fn keys_follow_the_cursor_rule_like_the_mouse() {
        assert!(!should_forward(&MOUSE_KEYBOARD, &KEY_EVENT, false, true));
        assert!(!should_forward(&MOUSE_KEYBOARD, &WHEEL_EVENT, false, true));
    }

    #[test]
    fn with_input_desktop_on_nothing_is_forwarded_unless_the_desktop_is_in_front() {
        let desktop_only = filter(InputType::MouseKeyboard, true);

        for event in [MOUSE_EVENT, WHEEL_EVENT, KEY_EVENT] {
            assert!(!should_forward(&desktop_only, &event, true, false), "{event:?}");
            assert!(should_forward(&desktop_only, &event, true, true), "{event:?}");
        }
    }

    #[test]
    fn the_key_lparam_is_the_legacy_value() {
        assert_eq!(key_lparam(0x1E, true), 0x011E_0001);
        assert_eq!(key_lparam(0x1E, false), (0x011E_0001_u32 | 0xC000_0000) as usize as isize);
        assert_eq!(key_lparam(0xFF, true) >> 16 & 0xFF, 0xFF, "the scan code is bits 16 to 23");
    }

    #[test]
    fn a_dead_key_is_not_forwarded_and_a_live_key_keeps_its_flags() {
        assert!(is_dead_key(0x8000_005E));
        assert!(!is_dead_key(0x41));
        assert_eq!(key_event(0xDE, 0x28, 0, 0x8000_0027), None);
        assert_eq!(
            key_event(0x41, 0x1E, 0, 0x41),
            Some(InputEvent::Key { vk: 0x41, scan: 0x1E, down: true, extended: false })
        );
        assert_eq!(
            key_event(0x25, 0x4B, 0x1 | 0x2, 0),
            Some(InputEvent::Key { vk: 0x25, scan: 0x4B, down: false, extended: true })
        );
    }

    #[test]
    fn the_focus_click_is_a_left_click_far_outside_the_page() {
        let [down, up] = focus_click_messages();

        assert_eq!(down, (WM_LBUTTONDOWN, 0x1, 0x869F_869F_u32 as usize as isize));
        assert_eq!(up, (WM_LBUTTONUP, 0x1, 0x869F_869F_u32 as usize as isize));
    }

    #[test]
    fn the_wheel_flag_gives_a_signed_delta_at_the_cursor() {
        let collect = |data| {
            let mut events = Vec::new();
            mouse_events(0x0400, data, (7, 8), |event| events.push(event));
            events
        };

        assert_eq!(collect(120), [InputEvent::Wheel { delta: 120, x: 7, y: 8 }]);
        assert_eq!(collect(0xFF88), [InputEvent::Wheel { delta: -120, x: 7, y: 8 }]);
    }

    #[test]
    fn the_desktop_is_progman_workerw_or_the_icons_window() {
        assert!(is_desktop_class("Progman"));
        assert!(is_desktop_class("WorkerW"));
        assert!(is_desktop_class("SHELLDLL_DefView"));
        assert!(!is_desktop_class("Chrome_WidgetWin_1"));
        assert!(!is_desktop_class(""));
    }

    #[test]
    fn a_key_event_never_shows_which_key_it_was_when_printed() {
        let text = format!("{KEY_EVENT:?}");

        assert!(!text.contains("65") && !text.contains("30") && !text.contains("0x"), "{text}");
        assert!(text.contains("Key"), "{text}");
    }

    #[test]
    fn raw_input_is_registered_only_for_the_devices_the_input_type_leaves_on() {
        assert_eq!(devices(InputType::Close), (false, false));
        assert_eq!(devices(InputType::OnlyMouse), (true, false));
        assert_eq!(devices(InputType::OnlyKeyboard), (false, true));
        assert_eq!(devices(InputType::MouseKeyboard), (true, true));
    }

    #[test]
    fn a_mouse_event_is_forwarded_only_while_the_cursor_is_inside_the_renderers_rectangle() {
        let moved = InputEvent::MouseMove { x: 5, y: 5 };

        assert!(should_forward(&MOUSE_KEYBOARD, &moved, true, true));
        assert!(!should_forward(&MOUSE_KEYBOARD, &moved, false, true));
    }
}
