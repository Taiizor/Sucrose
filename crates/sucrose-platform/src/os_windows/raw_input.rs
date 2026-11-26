//! RawInput on the event loop's hidden window (WAPI-07): the registration with `RIDEV_INPUTSINK`, so the window also
//! gets input while another window is in front, and the decoding of `WM_INPUT` into [`InputEvent`]s.
//!
//! The cursor position comes from `GetCursorPos` (physical pixels, the renderer is Per-Monitor-V2 aware), as in the
//! legacy app: a raw mouse record only carries a relative movement. Nothing read here is logged: the events are
//! the user's keystrokes (T-04-45).

use std::mem::size_of;

use windows::Win32::{
    Foundation::{HWND, LPARAM, POINT},
    UI::{
        Input::{
            GetRawInputData, HRAWINPUT,
            KeyboardAndMouse::{MAPVK_VK_TO_CHAR, MapVirtualKeyW},
            RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER, RID_INPUT, RIDEV_INPUTSINK, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
            RegisterRawInputDevices,
        },
        WindowsAndMessaging::GetCursorPos,
    },
};

use super::os_error;
use crate::{
    PlatformError,
    input::{InputEvent, key_event, mouse_events},
};

/// HID usage page of generic desktop controls.
const USAGE_PAGE_GENERIC: u16 = 0x01;
/// HID usage of a mouse.
const USAGE_MOUSE: u16 = 0x02;
/// HID usage of a keyboard.
const USAGE_KEYBOARD: u16 = 0x06;

/// The cursor position in physical pixels on the virtual screen.
pub(crate) fn cursor_position() -> Option<(i32, i32)> {
    let mut point = POINT::default();
    // SAFETY: `point` is a valid out pointer for the duration of the call.
    unsafe { GetCursorPos(&mut point) }.ok()?;
    Some((point.x, point.y))
}

/// Registers the mouse and/or the keyboard for raw input on `hwnd`, in the background too (`RIDEV_INPUTSINK`).
pub(super) fn register(hwnd: HWND, mouse: bool, keyboard: bool) -> Result<(), PlatformError> {
    let device = |usage| RAWINPUTDEVICE {
        usUsagePage: USAGE_PAGE_GENERIC,
        usUsage: usage,
        dwFlags: RIDEV_INPUTSINK,
        hwndTarget: hwnd,
    };
    let mut devices = Vec::with_capacity(2);
    if mouse {
        devices.push(device(USAGE_MOUSE));
    }
    if keyboard {
        devices.push(device(USAGE_KEYBOARD));
    }
    if devices.is_empty() {
        return Ok(());
    }
    // SAFETY: `devices` is a live slice of fully initialized RAWINPUTDEVICE values, the size passed is that of one
    // element, and `hwnd` is the window of the calling thread's event loop; the call keeps no pointer.
    unsafe { RegisterRawInputDevices(&devices, size_of::<RAWINPUTDEVICE>() as u32) }
        .map_err(|error| os_error("RegisterRawInputDevices", &error))
}

/// Reads the raw input record `lparam` of a `WM_INPUT` message and hands the events it stands for to `emit`. A record
/// that cannot be read, or that is neither mouse nor keyboard, gives no event.
pub(super) fn decode(lparam: LPARAM, mut emit: impl FnMut(InputEvent)) {
    // RAWINPUT is 48 bytes for a mouse or keyboard record on 64-bit Windows; u64 elements keep the buffer aligned.
    let mut buffer = [0_u64; 16];
    let mut size = size_of_val(&buffer) as u32;
    let handle = HRAWINPUT(lparam.0 as *mut core::ffi::c_void);
    // SAFETY: `buffer` is writable for `size` bytes and aligned for RAWINPUT, `size` is a valid in/out pointer and the
    // header size is that of RAWINPUTHEADER, as the call requires; `handle` comes from the WM_INPUT message being handled
    // and the call only reads the record it names.
    let copied = unsafe {
        GetRawInputData(
            handle,
            RID_INPUT,
            Some(buffer.as_mut_ptr().cast()),
            &mut size,
            size_of::<RAWINPUTHEADER>() as u32,
        )
    };
    if copied == u32::MAX || (copied as usize) < size_of::<RAWINPUTHEADER>() {
        return;
    }
    // SAFETY: the call above filled `buffer` with a RAWINPUT record (header and data), and the buffer is aligned for it.
    let record = unsafe { &*buffer.as_ptr().cast::<RAWINPUT>() };
    if record.header.dwType == RIM_TYPEMOUSE.0 {
        // SAFETY: the header says this is a mouse record, so the `mouse` member of the union is the initialized one.
        let mouse = unsafe { record.data.mouse };
        // SAFETY: both members of this union overlay the same 32 bits, which are initialized.
        let (flags, data) = unsafe {
            let buttons = mouse.Anonymous.Anonymous;
            (buttons.usButtonFlags, buttons.usButtonData)
        };
        if let Some(position) = cursor_position() {
            mouse_events(flags, data, position, emit);
        }
    } else if record.header.dwType == RIM_TYPEKEYBOARD.0 {
        // SAFETY: the header says this is a keyboard record, so the `keyboard` member of the union is the initialized one.
        let key = unsafe { record.data.keyboard };
        // SAFETY: a plain lookup by value; it reads the keyboard layout and keeps no pointer.
        let mapped = unsafe { MapVirtualKeyW(u32::from(key.VKey), MAPVK_VK_TO_CHAR) };
        if let Some(event) = key_event(key.VKey, key.MakeCode, key.Flags, mapped) {
            emit(event);
        }
    }
}
