//! The foreground window, read for the fullscreen rule and for where a keystroke goes. Only geometry, the window
//! class, the owning process, the keyboard focus and state flags are read: a window title can hold private text and is
//! never touched (T-02-28).

use std::mem::size_of;

use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute},
    UI::WindowsAndMessaging::{
        AllowSetForegroundWindow, GUITHREADINFO, GetClassNameW, GetForegroundWindow, GetGUIThreadInfo, GetWindowRect,
        GetWindowThreadProcessId, IsChild, IsIconic, IsWindowVisible,
    },
};

use crate::{ForegroundWindow, PlatformError, RectPx};

/// Lets process `pid` bring a window to the front once, on behalf of the caller, which must hold the foreground
/// right itself (the process the user just started does).
pub(crate) fn allow_set_foreground(pid: u32) -> Result<(), PlatformError> {
    // SAFETY: the call takes a process id by value and only changes the foreground permission of the window manager;
    // an id that names no process makes it fail, which is reported below.
    unsafe { AllowSetForegroundWindow(pid) }.map_err(|error| PlatformError::Os {
        context: "allowing a process to take the foreground",
        code: error.code().0,
    })
}

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

/// Whether the keyboard focus of the foreground window's thread is `ancestor` or one of its descendants, i.e. whether a
/// key the user types now is delivered by Windows to a window inside `ancestor`. A child window of another thread
/// shares that thread's input state (Windows attaches the queues), so a browser widget inside a desktop surface can
/// hold the focus of the shell's desktop thread while the desktop is in front. `false` when nothing can be read.
pub(crate) fn keyboard_focus_within(ancestor: HWND) -> bool {
    // SAFETY: the call takes no arguments and only reads the window manager's focus state.
    let foreground = unsafe { GetForegroundWindow() };
    if foreground.is_invalid() {
        return false;
    }
    // SAFETY: any handle value is accepted; a window that closed in the meantime gives thread id 0, handled below.
    let thread = unsafe { GetWindowThreadProcessId(foreground, None) };
    if thread == 0 {
        return false;
    }
    let mut info = GUITHREADINFO {
        cbSize: size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: `info` is a valid out pointer whose cbSize is set to its size, as the call requires; it only reads the
    // input state of the thread, which may have ended (the call then fails).
    if unsafe { GetGUIThreadInfo(thread, &mut info) }.is_err() {
        return false;
    }
    is_within(ancestor, info.hwndFocus)
}

/// Whether `window` is `ancestor` itself or a child window somewhere below it (across processes too).
fn is_within(ancestor: HWND, window: HWND) -> bool {
    if window.is_invalid() || ancestor.is_invalid() {
        return false;
    }
    // SAFETY: both are plain handle values; IsChild only walks the parent chain and is false for a window that is gone.
    window == ancestor || unsafe { IsChild(ancestor, window) }.as_bool()
}

#[cfg(test)]
mod tests {
    use windows::{
        Win32::UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow, WINDOW_EX_STYLE, WS_CHILD, WS_POPUP},
        core::w,
    };

    use super::*;

    /// A hidden window of the built-in STATIC class (no window procedure of ours), a child of `parent` when given.
    fn static_window(parent: Option<HWND>) -> HWND {
        let style = if parent.is_some() { WS_CHILD } else { WS_POPUP };
        // SAFETY: a system class and literal strings; no menu, instance or creation data; nothing is shown.
        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                None,
                style,
                0,
                0,
                10,
                10,
                parent,
                None,
                None,
                None,
            )
        }
        .expect("a STATIC window")
    }

    #[test]
    fn a_window_is_within_itself_and_its_ancestors_but_not_its_children_or_strangers() {
        let surface = static_window(None);
        let browser = static_window(Some(surface));
        let widget = static_window(Some(browser));
        let stranger = static_window(None);

        let seen = [
            is_within(surface, surface),
            is_within(surface, browser),
            is_within(surface, widget),
            is_within(browser, surface),
            is_within(surface, stranger),
            is_within(surface, HWND::default()),
            // None of these hidden windows of a test thread can hold the foreground window's keyboard focus.
            keyboard_focus_within(surface),
        ];
        for window in [widget, browser, surface, stranger] {
            // SAFETY: a window created above by this thread.
            let _ = unsafe { DestroyWindow(window) };
        }

        assert_eq!(seen, [true, true, true, false, false, false, false]);
    }

    #[test]
    fn granting_the_foreground_to_a_process_that_does_not_exist_fails_without_panicking() {
        // Neither a pid nobody has nor the lack of a foreground right of a test runner may panic; the result is an error.
        assert!(allow_set_foreground(u32::MAX - 1).is_err());
    }

    #[test]
    fn the_snapshot_never_panics_and_is_self_consistent() {
        // A test runner may have no foreground window at all (service session, locked screen).
        if let Some(window) = foreground_window() {
            assert!(!window.class_name.is_empty());
            assert!(window.rect.right >= window.rect.left || window.minimized);
        }
    }
}
