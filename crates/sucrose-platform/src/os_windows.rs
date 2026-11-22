//! Windows implementation: user identity, monitors, DPI awareness, the message loop and the desktop surface.

mod display;
mod event_loop;
mod foreground;
mod gpu;
mod job;
mod refresh;
mod surface;
mod tray;

use std::{path::Path, sync::OnceLock};

use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, HLOCAL, LocalFree},
        Security::{Authorization::ConvertSidToStringSidW, GetTokenInformation, TOKEN_QUERY, TOKEN_USER, TokenUser},
        System::{
            Performance::{QueryPerformanceCounter, QueryPerformanceFrequency},
            Threading::{GetCurrentProcess, OpenProcessToken},
        },
    },
    core::PWSTR,
};

use crate::{Capabilities, ForegroundWindow, GpuPreference, GpuPreferenceChange, MonitorInfo, Platform, PlatformError};

pub(crate) use display::init_dpi_awareness;
pub use event_loop::{EventLoop, LoopProxy, LoopTimers};
pub use job::RendererJob;
pub(crate) use refresh::refresh_desktop;
pub use surface::DesktopSurface;
pub use tray::Tray;

pub(crate) struct Current;

impl Platform for Current {
    fn name(&self) -> &'static str {
        "windows"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            desktop_wallpaper: true,
            fullscreen_detection: true,
            gpu_preference: true,
            ..Capabilities::default()
        }
    }

    fn user_key(&self) -> Result<String, PlatformError> {
        current_user_sid()
    }

    fn monitors(&self) -> Vec<MonitorInfo> {
        display::monitors()
    }

    fn foreground_window(&self) -> Option<ForegroundWindow> {
        foreground::foreground_window()
    }

    fn set_gpu_preference(&self, exe: &Path, preference: GpuPreference) -> Result<GpuPreferenceChange, PlatformError> {
        gpu::set_gpu_preference(exe, preference)
    }
}

/// QueryPerformanceCounter scaled to nanoseconds. The counter is system-wide, so the service and its renderers read
/// comparable values (RESEARCH A4). 0 when the OS refuses, which a caller sees as "no clock".
pub(crate) fn monotonic_ns() -> u64 {
    static FREQUENCY: OnceLock<i64> = OnceLock::new();
    let frequency = *FREQUENCY.get_or_init(|| {
        let mut frequency = 0_i64;
        // SAFETY: `frequency` is a valid out pointer for the duration of the call.
        match unsafe { QueryPerformanceFrequency(&mut frequency) } {
            Ok(()) => frequency,
            Err(_) => 0,
        }
    });
    let mut counter = 0_i64;
    // SAFETY: `counter` is a valid out pointer for the duration of the call.
    if frequency <= 0 || unsafe { QueryPerformanceCounter(&mut counter) }.is_err() || counter < 0 {
        return 0;
    }
    let nanoseconds = i128::from(counter) * 1_000_000_000 / i128::from(frequency);
    u64::try_from(nanoseconds).unwrap_or(u64::MAX)
}

fn os_error(context: &'static str, error: &windows::core::Error) -> PlatformError {
    PlatformError::Os {
        context,
        code: error.code().0,
    }
}

/// Closes a process token handle when dropped.
struct Token(HANDLE);

impl Drop for Token {
    fn drop(&mut self) {
        // SAFETY: the handle came from a successful OpenProcessToken and is closed exactly once, here.
        // A failed close cannot be acted on.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

/// The account SID of the user this process runs as, in the `S-1-5-21-...` string form.
fn current_user_sid() -> Result<String, PlatformError> {
    let mut handle = HANDLE::default();
    // SAFETY: GetCurrentProcess returns a pseudo handle that needs no closing, and `handle` is a valid out pointer.
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle) }
        .map_err(|e| os_error("OpenProcessToken", &e))?;
    let token = Token(handle);

    // First call: ask how many bytes TOKEN_USER needs (it fails with ERROR_INSUFFICIENT_BUFFER by design).
    let mut needed = 0_u32;
    // SAFETY: a null buffer with length 0 is the documented way to query the required size; `needed` is valid.
    let _ = unsafe { GetTokenInformation(token.0, TokenUser, None, 0, &mut needed) };
    if needed == 0 {
        return Err(PlatformError::Os {
            context: "GetTokenInformation (size)",
            code: 0,
        });
    }

    // u64 elements keep the buffer 8-byte aligned, which TOKEN_USER requires.
    let mut buffer = vec![0_u64; (needed as usize).div_ceil(8)];
    // SAFETY: `buffer` is at least `needed` bytes, writable and aligned for TOKEN_USER; `needed` is a valid out pointer.
    unsafe {
        GetTokenInformation(
            token.0,
            TokenUser,
            Some(buffer.as_mut_ptr().cast()),
            needed,
            &mut needed,
        )
    }
    .map_err(|e| os_error("GetTokenInformation", &e))?;

    // SAFETY: the call above filled `buffer` with a TOKEN_USER whose SID points inside the same buffer, which
    // stays alive until the end of this function.
    let sid = unsafe { (*buffer.as_ptr().cast::<TOKEN_USER>()).User.Sid };
    let mut wide = PWSTR::null();
    // SAFETY: `sid` is a valid SID (see above) and `wide` is a valid out pointer.
    unsafe { ConvertSidToStringSidW(sid, &mut wide) }.map_err(|e| os_error("ConvertSidToStringSidW", &e))?;
    // SAFETY: on success `wide` points to a NUL-terminated UTF-16 string allocated with LocalAlloc.
    let text = unsafe { wide.to_string() };
    // SAFETY: the string was allocated by ConvertSidToStringSidW for us to free with LocalFree, once.
    unsafe { LocalFree(Some(HLOCAL(wide.0.cast()))) };
    text.map_err(|_| PlatformError::Os {
        context: "ConvertSidToStringSidW (text)",
        code: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_advances_and_is_in_nanoseconds() {
        let first = monotonic_ns();
        std::thread::sleep(std::time::Duration::from_millis(20));
        let second = monotonic_ns();

        assert!(first > 0);
        let elapsed = second - first;
        assert!((15_000_000..2_000_000_000).contains(&elapsed), "{elapsed}");
    }

    #[test]
    fn the_user_key_is_an_account_sid() {
        let key = Current.user_key().expect("user key");

        assert!(key.starts_with("S-1-5-"), "{key}");
        assert!(key.chars().all(|c| c.is_ascii_digit() || c == 'S' || c == '-'), "{key}");
    }
}
