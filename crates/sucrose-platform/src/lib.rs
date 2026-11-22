//! The only crate for OS-specific code (D-002). Each OS implements [`Platform`]; anything an OS cannot do is
//! reported through [`Capabilities`] instead of being faked. This crate may opt out of the workspace `unsafe_code`
//! lint with `#![allow(unsafe_code)]` once it needs FFI (D-011).

// D-011: the Windows module calls Win32 APIs; each unsafe block carries a SAFETY comment.
#![allow(unsafe_code)]

#[cfg(target_os = "linux")]
mod os_linux;
#[cfg(target_os = "macos")]
mod os_macos;
#[cfg(target_os = "windows")]
mod os_windows;

#[cfg(target_os = "linux")]
use os_linux as imp;
#[cfg(target_os = "macos")]
use os_macos as imp;
#[cfg(target_os = "windows")]
use os_windows as imp;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
compile_error!("Sucrose supports Windows, macOS and Linux only");

/// What the current OS and session support. The UI hides or explains anything that is `false`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// A surface behind the desktop icons can host a wallpaper.
    pub desktop_wallpaper: bool,
    /// Another application's window can be embedded (Application wallpapers).
    pub embed_application: bool,
    /// Fullscreen applications can be detected.
    pub fullscreen_detection: bool,
}

/// Why a platform call failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PlatformError {
    /// This OS does not provide the feature (reported through capabilities where the UI needs to know).
    #[error("{0} is not supported on this OS")]
    Unsupported(&'static str),
    /// An OS call failed.
    #[error("{context} failed with OS error {code}")]
    Os {
        /// What was being done.
        context: &'static str,
        /// The OS error code (the HRESULT on Windows).
        code: i32,
    },
}

/// OS integration used by the service.
pub trait Platform: Send + Sync {
    /// Short OS name for logs, the same value as `std::env::consts::OS`.
    fn name(&self) -> &'static str;

    /// Features this implementation provides.
    fn capabilities(&self) -> Capabilities;

    /// A string that identifies the current user on this machine (the account SID on Windows), used to name the
    /// per-user service endpoint and to restrict it to that user.
    fn user_key(&self) -> Result<String, PlatformError> {
        Err(PlatformError::Unsupported("user identity"))
    }
}

/// The implementation for the OS this binary was built for.
pub fn current() -> Box<dyn Platform> {
    Box::new(imp::Current)
}

#[cfg(test)]
mod tests {
    #[test]
    fn current_matches_the_build_target() {
        assert_eq!(super::current().name(), std::env::consts::OS);
    }
}
