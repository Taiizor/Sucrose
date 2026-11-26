//! CEF engine for web wallpapers (D-013, D-32).
//!
//! The pinned CEF dist is linked through `CEF_PATH` and `libcef.dll` is delay-loaded by the executables that host
//! CEF: only renderers and CEF subprocesses ever load it, so the service and the mpv renderers stay without it.
//! Nothing in this crate calls a CEF function unless the process is a CEF subprocess (see [`run_if_subprocess`]).

// Engine/FFI crate: allowed to opt out of the workspace `unsafe_code` lint (D-011).
#![allow(unsafe_code)]

pub mod browser;
pub mod engine;
pub mod scheme;

use std::ffi::OsString;

/// Whether this build contains CEF: only Windows on x86_64 and aarch64 has a pinned dist (D-028), so the service
/// offers CEF on those targets only.
pub const BUILT: bool = cfg!(all(windows, any(target_arch = "x86_64", target_arch = "aarch64")));

/// The CEF version of the pinned dist (`xtask/engines.toml`), without the commit and Chromium suffix.
pub const CEF_VERSION: &str = "156.0.31";

/// The switch CEF puts on every command line it relaunches this executable with (`--type=renderer`, `gpu-process`, ...).
const SUBPROCESS_SWITCH: &str = "--type=";

/// Whether a command line is a CEF subprocess's: some argument starts with `--type=`. Pure and the same on every
/// target, so the Sucrose binary can ask it before it starts anything.
#[must_use]
pub fn is_cef_subprocess(args: &[OsString]) -> bool {
    args.iter()
        .any(|arg| arg.to_str().is_some_and(|text| text.starts_with(SUBPROCESS_SWITCH)))
}

/// Runs CEF's subprocess entry when this process was started by CEF and returns its exit code; `None` for every other
/// command line, in which case no CEF function was called and `libcef.dll` was not loaded. Call it first in `main`,
/// before the log, the single-instance check or the command-line parser exist (D-32).
#[must_use]
pub fn run_if_subprocess() -> Option<i32> {
    let args: Vec<OsString> = std::env::args_os().collect();
    if !is_cef_subprocess(&args) {
        return None;
    }
    subprocess::run()
}

#[cfg(all(windows, any(target_arch = "x86_64", target_arch = "aarch64")))]
mod subprocess {
    use cef::{args::Args, *};

    // The application object of a CEF subprocess: no customisation yet (the renderer-side script bridge comes with
    // the web wallpaper host).
    wrap_app! {
        struct SubprocessApp;

        impl App {}
    }

    pub(super) fn run() -> Option<i32> {
        // CEF requires the API version to be declared before any other call; this is also the first call that
        // loads libcef.dll, which is why it only ever runs in a CEF subprocess.
        let _ = cef::api_hash(cef::sys::CEF_API_VERSION_LAST, 0);
        let args = Args::new();
        let mut app = SubprocessApp::new();
        let code = cef::execute_process(Some(args.as_main_args()), Some(&mut app), std::ptr::null_mut());
        if code < 0 {
            // CEF says this is the browser process although the command line named a subprocess type: never fall
            // through to the Sucrose parser with such a line.
            tracing::error!(code, "CEF did not accept the subprocess command line");
            return Some(1);
        }
        Some(code)
    }
}

/// Targets without a CEF build (x86 Windows, other systems): there are no CEF subprocesses, so nothing is routed.
#[cfg(not(all(windows, any(target_arch = "x86_64", target_arch = "aarch64"))))]
mod subprocess {
    pub(super) fn run() -> Option<i32> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    #[test]
    fn a_type_switch_with_a_value_is_a_cef_subprocess() {
        assert!(is_cef_subprocess(&args(&["--type=gpu-process", "--lang=en-US"])));
        assert!(is_cef_subprocess(&args(&[
            "sucrose.exe",
            "--no-sandbox",
            "--type=renderer"
        ])));
        assert!(is_cef_subprocess(&args(&["--type=utility"])));
    }

    #[test]
    fn sucrose_command_lines_are_not_cef_subprocesses() {
        assert!(!is_cef_subprocess(&args(&[])));
        assert!(!is_cef_subprocess(&args(&["--render", "1"])));
        assert!(!is_cef_subprocess(&args(&["--no-portal"])));
        assert!(
            !is_cef_subprocess(&args(&["--type"])),
            "--type without = is not CEF's switch"
        );
        assert!(!is_cef_subprocess(&args(&[
            "--typed=1",
            "type=renderer",
            "-type=renderer"
        ])));
    }

    #[test]
    fn this_test_process_is_not_a_subprocess_and_loads_no_cef() {
        assert_eq!(run_if_subprocess(), None);
    }

    #[test]
    fn the_cef_version_is_the_pinned_one() {
        assert_eq!(CEF_VERSION, "156.0.31");
    }
}
