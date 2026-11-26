//! WebView2 engine for web wallpapers (D-013, D-14).
//!
//! WebView2 is the user-installed Microsoft Edge WebView2 Runtime: Sucrose links only the static loader
//! (`webview2-com-sys`) and never ships or downloads the runtime. Only a renderer creates a WebView2 environment; the
//! service asks [`runtime_version`], which reads the installed runtime's version without creating a WebView (D-008).

// Engine/FFI crate: allowed to opt out of the workspace `unsafe_code` lint (D-011).
#![allow(unsafe_code)]
#![warn(clippy::undocumented_unsafe_blocks)]

pub mod engine;
pub mod host;

/// Whether this build contains WebView2: only Windows has the runtime and its loader (D-002).
pub const BUILT: bool = cfg!(windows);

/// The version of the installed WebView2 Runtime, for example `158.0.4370.33`; `None` when no runtime is installed or
/// on a system without WebView2. It asks the loader for the version and creates no WebView and no browser process.
#[must_use]
pub fn runtime_version() -> Option<String> {
    imp::runtime_version()
}

#[cfg(windows)]
mod imp {
    use webview2_com::{Microsoft::Web::WebView2::Win32::GetAvailableCoreWebView2BrowserVersionString, take_pwstr};
    use windows::core::{PCWSTR, PWSTR};

    pub(super) fn runtime_version() -> Option<String> {
        let mut version = PWSTR::null();
        // SAFETY: no browser folder is given, so the loader looks at the installed runtime; the out pointer is valid
        // for the call, and the string it fills in (CoTaskMemAlloc) is freed by `take_pwstr`, which also reads a
        // null result as empty.
        unsafe { GetAvailableCoreWebView2BrowserVersionString(PCWSTR::null(), &mut version) }.ok()?;
        let version = take_pwstr(version);
        let version = version.trim();
        (!version.is_empty()).then(|| version.to_owned())
    }
}

#[cfg(not(windows))]
mod imp {
    pub(super) fn runtime_version() -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_runtime_version_is_none_or_a_dotted_number() {
        if let Some(version) = runtime_version() {
            assert!(
                version
                    .split('.')
                    .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())),
                "{version}"
            );
        }
    }

    #[test]
    fn only_windows_contains_webview2() {
        assert_eq!(BUILT, cfg!(windows));
    }
}
