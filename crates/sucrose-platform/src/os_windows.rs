//! Windows implementation: user identity, monitors, DPI awareness, the message loop and the desktop surface.

mod display;
mod event_loop;
mod job;
mod refresh;
mod surface;
mod tray;

use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, HLOCAL, LocalFree},
        Security::{Authorization::ConvertSidToStringSidW, GetTokenInformation, TOKEN_QUERY, TOKEN_USER, TokenUser},
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    },
    core::PWSTR,
};

use crate::{Capabilities, MonitorInfo, Platform, PlatformError};

pub(crate) use display::init_dpi_awareness;
pub use event_loop::{EventLoop, LoopProxy};
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
            ..Capabilities::default()
        }
    }

    fn user_key(&self) -> Result<String, PlatformError> {
        current_user_sid()
    }

    fn monitors(&self) -> Vec<MonitorInfo> {
        display::monitors()
    }
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
    fn the_user_key_is_an_account_sid() {
        let key = Current.user_key().expect("user key");

        assert!(key.starts_with("S-1-5-"), "{key}");
        assert!(key.chars().all(|c| c.is_ascii_digit() || c == 'S' || c == '-'), "{key}");
    }
}
