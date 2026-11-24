//! Sucrose's own start entry on Windows: a value of the per-user Run key (D-08, D-034), and Task Manager's on/off flag
//! for it under StartupApproved. The Run value of the C# app ("Sucrose") and its scheduled task are only looked for,
//! never written, changed or removed.

use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use windows::{
    Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS, WIN32_ERROR},
        System::Registry::{
            HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_EXPAND_SZ, REG_OPTION_NON_VOLATILE,
            REG_SAM_FLAGS, REG_SZ, REG_VALUE_TYPE, RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW,
            RegQueryValueExW, RegSetValueExW,
        },
    },
    core::{HRESULT, PCWSTR},
};

use super::spawn_detached;
use crate::{
    PlatformError,
    autostart::{AutostartState, RUN_VALUE_NAME, disabled_by_os, run_value_data},
};

/// The per-user Run key: what Windows starts at sign-in.
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// Task Manager's Startup apps keep their on/off flag here, one binary value per Run value.
const APPROVED_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
/// The Run value of the C# app. Only read.
const LEGACY_RUN_VALUE: &str = "Sucrose";
/// The scheduled task of the C# app (Scheduler startup). Only queried.
const LEGACY_TASK: &str = r"\Sucrose\Autorun for Sucrose";
/// How long the task query may take before it counts as "no task".
const QUERY_TIMEOUT: Duration = Duration::from_secs(5);

/// NUL-terminated UTF-16 for a Win32 string argument.
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn win32_error(context: &'static str, error: WIN32_ERROR) -> PlatformError {
    PlatformError::Os {
        context,
        code: HRESULT::from_win32(error.0).0,
    }
}

fn is_missing(status: WIN32_ERROR) -> bool {
    status == ERROR_FILE_NOT_FOUND || status == ERROR_PATH_NOT_FOUND
}

/// Closes a registry key when dropped.
struct Key(HKEY);

impl Drop for Key {
    fn drop(&mut self) {
        // SAFETY: the handle came from a successful RegOpenKeyExW or RegCreateKeyExW and is closed exactly once, here.
        // A failed close cannot be acted on.
        let _ = unsafe { RegCloseKey(self.0) };
    }
}

/// Opens the HKCU key `subkey`; `None` when it does not exist.
fn open(subkey: &str, access: REG_SAM_FLAGS) -> Result<Option<Key>, PlatformError> {
    let subkey = wide(subkey);
    let mut handle = HKEY::default();
    // SAFETY: `subkey` is NUL-terminated and outlives the call; `handle` is a valid out pointer.
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            None,
            access,
            &raw mut handle,
        )
    };
    if is_missing(status) {
        return Ok(None);
    }
    if status != ERROR_SUCCESS {
        return Err(win32_error("RegOpenKeyExW", status));
    }
    Ok(Some(Key(handle)))
}

/// Opens the HKCU key `subkey` for writing, creating it when it is missing.
fn open_or_create(subkey: &str) -> Result<Key, PlatformError> {
    let subkey = wide(subkey);
    let mut handle = HKEY::default();
    // SAFETY: `subkey` is NUL-terminated and outlives the call; no class, security attributes or disposition are
    // requested and `handle` is a valid out pointer.
    let status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            None,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_QUERY_VALUE | KEY_SET_VALUE,
            None,
            &raw mut handle,
            None,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(win32_error("RegCreateKeyExW", status));
    }
    Ok(Key(handle))
}

/// The type and bytes of the value `name` of `key`; `None` when it does not exist.
fn read_value(key: &Key, name: &str) -> Result<Option<(REG_VALUE_TYPE, Vec<u8>)>, PlatformError> {
    let name = wide(name);
    let mut kind = REG_VALUE_TYPE(0);
    let mut bytes = 0_u32;
    // SAFETY: `name` is NUL-terminated and outlives the call; with no data buffer the call only reports the type and
    // size through the valid out pointers.
    let status = unsafe {
        RegQueryValueExW(
            key.0,
            PCWSTR(name.as_ptr()),
            None,
            Some(&raw mut kind),
            None,
            Some(&raw mut bytes),
        )
    };
    if is_missing(status) {
        return Ok(None);
    }
    if status != ERROR_SUCCESS && status != ERROR_MORE_DATA {
        return Err(win32_error("RegQueryValueExW", status));
    }
    let mut buffer = vec![0_u8; bytes as usize];
    let mut capacity = bytes;
    // SAFETY: `buffer` holds `capacity` writable bytes, `name` is NUL-terminated and `kind`/`capacity` are valid out
    // pointers for the duration of the call.
    let status = unsafe {
        RegQueryValueExW(
            key.0,
            PCWSTR(name.as_ptr()),
            None,
            Some(&raw mut kind),
            Some(buffer.as_mut_ptr()),
            Some(&raw mut capacity),
        )
    };
    if is_missing(status) {
        return Ok(None);
    }
    if status != ERROR_SUCCESS {
        return Err(win32_error("RegQueryValueExW", status));
    }
    buffer.truncate(capacity as usize);
    Ok(Some((kind, buffer)))
}

/// The text of a string value, or `None` when it is missing or of another type.
fn read_text(key: &Key, name: &str) -> Result<Option<String>, PlatformError> {
    let Some((kind, bytes)) = read_value(key, name)? else {
        return Ok(None);
    };
    if kind != REG_SZ && kind != REG_EXPAND_SZ {
        return Ok(None);
    }
    let (pairs, _) = bytes.as_chunks::<2>();
    let units: Vec<u16> = pairs.iter().map(|pair| u16::from_le_bytes(*pair)).collect();
    let end = units.iter().position(|unit| *unit == 0).unwrap_or(units.len());
    Ok(Some(String::from_utf16_lossy(&units[..end])))
}

/// Writes the string value `name` of `key`.
fn write_text(key: &Key, name: &str, text: &str) -> Result<(), PlatformError> {
    let name = wide(name);
    let data: Vec<u8> = wide(text).iter().flat_map(|unit| unit.to_le_bytes()).collect();
    // SAFETY: `name` is NUL-terminated and `data` is the NUL-terminated UTF-16 text REG_SZ expects; both outlive the call.
    let status = unsafe { RegSetValueExW(key.0, PCWSTR(name.as_ptr()), None, REG_SZ, Some(&data)) };
    if status != ERROR_SUCCESS {
        return Err(win32_error("RegSetValueExW", status));
    }
    Ok(())
}

/// Deletes the value `name` of `key`; a missing value is success.
fn delete_value(key: &Key, name: &str) -> Result<(), PlatformError> {
    let name = wide(name);
    // SAFETY: `name` is NUL-terminated and outlives the call; `key` is an open key with KEY_SET_VALUE access.
    let status = unsafe { RegDeleteValueW(key.0, PCWSTR(name.as_ptr())) };
    if status != ERROR_SUCCESS && !is_missing(status) {
        return Err(win32_error("RegDeleteValueW", status));
    }
    Ok(())
}

/// Writes the Run value (the quoted `exe`, no arguments) and resets Task Manager's flag for it, so an entry that was
/// turned off in Task Manager starts again once the user chooses Normal (RESEARCH Pitfall 9).
pub(crate) fn enable_autostart(exe: &Path) -> Result<(), PlatformError> {
    if !exe.is_absolute() {
        return Err(PlatformError::Failed {
            context: "startup entry",
            message: "the executable path is not absolute".to_owned(),
        });
    }
    let data = run_value_data(exe)?;
    write_text(&open_or_create(RUN_KEY)?, RUN_VALUE_NAME, &data)?;
    // The entry is written; a flag that cannot be reset is shown to the user as "turned off by Task Manager".
    match open(APPROVED_KEY, KEY_SET_VALUE) {
        Ok(Some(approved)) => {
            if let Err(error) = delete_value(&approved, RUN_VALUE_NAME) {
                tracing::warn!(%error, "the Task Manager flag of the startup entry could not be reset");
            }
        }
        Ok(None) => {}
        Err(error) => tracing::warn!(%error, "the Task Manager flag of the startup entry could not be reset"),
    }
    Ok(())
}

/// Removes the entry in two steps: the Run value first, then Task Manager's flag.
fn remove_entry(
    run: impl FnOnce() -> Result<(), PlatformError>,
    flag: impl FnOnce() -> Result<(), PlatformError>,
) -> Result<(), PlatformError> {
    run()?;
    flag()?;
    Ok(())
}

/// Deletes Sucrose's own Run value and its Task Manager flag; nothing else.
pub(crate) fn disable_autostart() -> Result<(), PlatformError> {
    if let Some(run) = open(RUN_KEY, KEY_SET_VALUE)? {
        delete_value(&run, RUN_VALUE_NAME)?;
    }
    if let Some(approved) = open(APPROVED_KEY, KEY_SET_VALUE)? {
        delete_value(&approved, RUN_VALUE_NAME)?;
    }
    Ok(())
}

/// Whether Sucrose's own Run value starts `exe`, and whether Task Manager turned it off. An unreadable key counts as
/// absent.
pub(crate) fn autostart_state(exe: &Path) -> AutostartState {
    let found = match open(RUN_KEY, KEY_QUERY_VALUE).and_then(|run| match run {
        Some(run) => read_text(&run, RUN_VALUE_NAME),
        None => Ok(None),
    }) {
        Ok(found) => found,
        Err(error) => {
            tracing::debug!(%error, "the startup entry cannot be read");
            None
        }
    };
    let Some(found) = found else {
        return AutostartState::default();
    };
    let disabled = match open(APPROVED_KEY, KEY_QUERY_VALUE).and_then(|approved| match approved {
        Some(approved) => read_value(&approved, RUN_VALUE_NAME),
        None => Ok(None),
    }) {
        Ok(Some((_, bytes))) => disabled_by_os(&bytes),
        Ok(None) => false,
        Err(error) => {
            tracing::debug!(%error, "the Task Manager flag cannot be read");
            false
        }
    };
    AutostartState {
        entry_present: run_value_data(exe).is_ok_and(|expected| found.eq_ignore_ascii_case(&expected)),
        disabled_by_os: disabled,
        legacy_entry: false,
    }
}

/// Whether the C# app has a start entry: its Run value, or its scheduled task. Read-only (D-08).
pub(crate) fn legacy_entry() -> bool {
    legacy_run_value() || legacy_task()
}

fn legacy_run_value() -> bool {
    open(RUN_KEY, KEY_QUERY_VALUE)
        .and_then(|run| match run {
            Some(run) => read_value(&run, LEGACY_RUN_VALUE),
            None => Ok(None),
        })
        .is_ok_and(|value| value.is_some())
}

/// `schtasks.exe` from the system folder, never found through the search path.
fn schtasks_path() -> PathBuf {
    std::env::var_os("SystemRoot")
        .map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from)
        .join("System32")
        .join("schtasks.exe")
}

/// Asks the Task Scheduler whether the C# task exists: exit code 0 means it does. The task folder cannot be read by a
/// standard user, so the query is the way to ask (RESEARCH A10). Only a query is ever sent.
fn legacy_task() -> bool {
    let mut command = Command::new(schtasks_path());
    command
        .args(["/Query", "/TN", LEGACY_TASK])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let Ok(mut child) = spawn_detached(&mut command) else {
        return false;
    };
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) if started.elapsed() < QUERY_TIMEOUT => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                // A query that hangs counts as "no task"; the process is not left behind.
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    fn failure() -> PlatformError {
        PlatformError::Os {
            context: "test",
            code: 5,
        }
    }

    #[test]
    fn a_flag_that_cannot_be_removed_after_the_run_value_went_is_not_an_error() {
        let flag_called = Cell::new(false);

        let result = remove_entry(
            || Ok(()),
            || {
                flag_called.set(true);
                Err(failure())
            },
        );

        assert!(flag_called.get(), "the flag removal is still tried");
        assert!(result.is_ok(), "the Run value is gone, so the entry is off: {result:?}");
    }

    #[test]
    fn a_run_value_that_cannot_be_removed_is_an_error_and_the_flag_is_left() {
        let flag_called = Cell::new(false);

        let result = remove_entry(
            || Err(failure()),
            || {
                flag_called.set(true);
                Ok(())
            },
        );

        assert!(result.is_err(), "an entry that is still active is not a success");
        assert!(!flag_called.get(), "the flag stays while the Run value is there");
    }

    #[test]
    fn both_steps_succeeding_is_ok() {
        assert!(remove_entry(|| Ok(()), || Ok(())).is_ok());
    }
}
