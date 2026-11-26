//! Runs a downloaded installer through the shell (D-15). Sucrose never asks for elevation itself: ShellExecuteEx with the
//! default verb lets Windows read the installer's manifest and show its own UAC prompt where the installer needs one.

use std::{os::windows::ffi::OsStrExt, path::Path};

use windows::{
    Win32::{
        Foundation::{CloseHandle, WAIT_OBJECT_0},
        System::Threading::{GetExitCodeProcess, WaitForSingleObject},
        UI::{
            Shell::{SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW},
            WindowsAndMessaging::SW_SHOWNORMAL,
        },
    },
    core::PCWSTR,
};

use crate::PlatformError;

/// How long an installer may run before the wait gives up (the installer keeps running; Sucrose stops waiting).
const WAIT_LIMIT_MS: u32 = 30 * 60 * 1000;

/// NUL-terminated UTF-16 of `text`.
fn wide_os(text: &std::ffi::OsStr) -> Vec<u16> {
    text.encode_wide().chain(std::iter::once(0)).collect()
}

/// See [`crate::Platform::run_installer`].
pub(crate) fn run_installer(path: &Path, args: &str) -> Result<i32, PlatformError> {
    let file = wide_os(path.as_os_str());
    let parameters = wide_os(std::ffi::OsStr::new(args));
    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(parameters.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..SHELLEXECUTEINFOW::default()
    };
    // SAFETY: `info` is fully initialised and its two strings are NUL-terminated and outlive the call; with
    // SEE_MASK_NOCLOSEPROCESS a successful call leaves a process handle in `hProcess` that is closed below.
    unsafe { ShellExecuteExW(&raw mut info) }.map_err(|error| PlatformError::Os {
        context: "starting the installer",
        code: error.code().0,
    })?;
    let process = info.hProcess;
    if process.is_invalid() {
        return Err(PlatformError::Failed {
            context: "starting the installer",
            message: "the shell returned no process to wait for".to_owned(),
        });
    }
    // SAFETY: `process` is the valid handle ShellExecuteExW returned and is closed exactly once, after the wait and the
    // exit code are read.
    unsafe {
        let waited = WaitForSingleObject(process, WAIT_LIMIT_MS);
        let result = if waited == WAIT_OBJECT_0 {
            let mut code = 0_u32;
            GetExitCodeProcess(process, &raw mut code)
                .map(|()| code as i32)
                .map_err(|error| PlatformError::Os {
                    context: "reading the installer's exit code",
                    code: error.code().0,
                })
        } else {
            Err(PlatformError::Failed {
                context: "waiting for the installer",
                message: "the installer did not finish in time".to_owned(),
            })
        };
        let _ = CloseHandle(process);
        result
    }
}
