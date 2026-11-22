//! The foreground window, read for the fullscreen rule. Only geometry, the window class, the owning process and
//! state flags are read: a window title can hold private text and is never touched (T-02-28).

use std::mem::size_of;

use windows::Win32::{
    Foundation::RECT,
    Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute},
    UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    },
};

use crate::{ForegroundWindow, RectPx};

/// The window the user is working in right now, or `None` when there is none (the secure desktop, a window that
/// closed between the calls) or it cannot be measured.
pub(crate) fn foreground_window() -> Option<ForegroundWindow> {
    // SAFETY: the call takes no arguments and only reads the window manager's focus state.
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return None;
    }

    let mut rect = RECT::default();
    // SAFETY: `hwnd` was just returned by the OS and `rect` is a valid out pointer; a window that has closed in the
    // meantime only makes the call fail.
    unsafe { GetWindowRect(hwnd, &mut rect) }.ok()?;

    let mut class = [0_u16; 256];
    // SAFETY: the slice is writable for its whole length, which is what the call is told; it returns the number of
    // UTF-16 units copied, without the terminating NUL, or 0 on failure.
    let copied = unsafe { GetClassNameW(hwnd, &mut class) };
    let copied = usize::try_from(copied)
        .ok()
        .filter(|len| *len > 0 && *len <= class.len())?;
    let class_name = String::from_utf16_lossy(&class[..copied]);

    let mut process_id = 0_u32;
    // SAFETY: `process_id` is a valid out pointer; the returned thread id is not needed.
    let _ = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut process_id)) };

    // SAFETY: both calls only read window state and accept any handle value.
    let (visible, minimized) = unsafe { (IsWindowVisible(hwnd).as_bool(), IsIconic(hwnd).as_bool()) };

    let mut cloaked = 0_u32;
    // SAFETY: `cloaked` is a writable u32 and the size passed is exactly its size, as DWMWA_CLOAKED requires.
    let cloaked_read =
        unsafe { DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, (&raw mut cloaked).cast(), size_of::<u32>() as u32) };

    Some(ForegroundWindow {
        rect: RectPx {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        },
        class_name,
        process_id,
        visible,
        minimized,
        // A failed read means the attribute does not apply to this window: it is not cloaked.
        cloaked: cloaked_read.is_ok() && cloaked != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_snapshot_never_panics_and_is_self_consistent() {
        // A test runner may have no foreground window at all (service session, locked screen).
        if let Some(window) = foreground_window() {
            assert!(!window.class_name.is_empty());
            assert!(window.rect.right >= window.rect.left || window.minimized);
        }
    }
}
