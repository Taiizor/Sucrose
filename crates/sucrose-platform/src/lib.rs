//! The only crate for OS-specific code (D-002). Each OS implements [`Platform`]; anything an OS cannot do is
//! reported through [`Capabilities`] instead of being faked. This crate may opt out of the workspace `unsafe_code`
//! lint with `#![allow(unsafe_code)]` once it needs FFI (D-011).

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

/// OS integration used by the service.
pub trait Platform: Send + Sync {
    /// Short OS name for logs, the same value as `std::env::consts::OS`.
    fn name(&self) -> &'static str;

    /// Features this implementation provides.
    fn capabilities(&self) -> Capabilities;
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
