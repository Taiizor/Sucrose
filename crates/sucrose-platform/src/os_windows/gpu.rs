//! The GPU preference of Windows for an executable (Settings > Display > Graphics), written the way the C# app does
//! for its engine executables (DESK-12), plus the DXGI adapter list and the adapter a default Direct3D 11 device gets.

use std::path::Path;

use windows::{
    Win32::{
        Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS, WIN32_ERROR},
        System::Registry::{
            HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, REG_VALUE_TYPE,
            RegCloseKey, RegCreateKeyExW, RegQueryValueExW, RegSetValueExW,
        },
    },
    core::{HRESULT, PCWSTR},
};

use crate::{GpuPreferenceChange, PlatformError};
use sucrose_core::settings::GpuPreference;

/// The per-user key that holds one value per executable, named by the executable's full path.
const PREFERENCES_SUBKEY: &str = r"Software\Microsoft\DirectX\UserGpuPreferences";

/// The pair inside the value that carries the preference.
const PREFERENCE_KEY: &str = "GpuPreference";

/// Sets the GPU preference of `exe` with the legacy rules (see [`merged_value`]).
pub(crate) fn set_gpu_preference(exe: &Path, preference: GpuPreference) -> Result<GpuPreferenceChange, PlatformError> {
    let name = value_name(exe)?;
    write_preference(PREFERENCES_SUBKEY, &name, preference.legacy_ordinal())
}

/// The registry value name for an executable: its UTF-8 path without a verbatim `\\?\` prefix.
fn value_name(exe: &Path) -> Result<String, PlatformError> {
    let text = exe.to_str().ok_or_else(|| PlatformError::Failed {
        context: "GPU preference",
        message: "the executable path is not valid UTF-8".to_owned(),
    })?;
    let stripped = text.strip_prefix(r"\\?\").filter(|rest| {
        let mut chars = rest.chars();
        matches!((chars.next(), chars.next()), (Some(letter), Some(':')) if letter.is_ascii_alphabetic())
    });
    Ok(stripped.unwrap_or(text).to_owned())
}

/// The `Key=Value` pairs of a value, parsed like the C# app: parts split on ';', trimmed, split at the first '=' that
/// has text on both sides, keys compared case-insensitively (a later duplicate overwrites the value and keeps the
/// position and spelling of the first).
fn parse_pairs(value: &str) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    for part in value.split(';') {
        let part = part.trim();
        let Some(index) = part.find('=') else { continue };
        if index == 0 || index == part.len() - 1 {
            continue;
        }
        let (key, rest) = part.split_at(index);
        let key = key.trim();
        let rest = rest[1..].trim();
        if key.is_empty() {
            continue;
        }
        match pairs.iter_mut().find(|(known, _)| known.eq_ignore_ascii_case(key)) {
            Some(pair) => pair.1 = rest.to_owned(),
            None => pairs.push((key.to_owned(), rest.to_owned())),
        }
    }
    pairs
}

/// The preference a value carries: the `GpuPreference` pair when it is the number 0, 1 or 2, else Default (0).
fn current_ordinal(pairs: &[(String, String)]) -> u32 {
    pairs
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(PREFERENCE_KEY))
        .and_then(|(_, value)| value.parse::<i64>().ok())
        .and_then(|number| u32::try_from(number).ok())
        .filter(|number| *number <= 2)
        .unwrap_or(0)
}

/// The value to store so that `existing` carries preference `ordinal`, or `None` when nothing has to be written (the value
/// already carries it, a missing value counting as Default). Other pairs are kept in place.
fn merged_value(existing: Option<&str>, ordinal: u32) -> Option<String> {
    let mut pairs = parse_pairs(existing.unwrap_or_default());
    if current_ordinal(&pairs) == ordinal {
        return None;
    }
    let number = ordinal.to_string();
    match pairs
        .iter_mut()
        .find(|(key, _)| key.eq_ignore_ascii_case(PREFERENCE_KEY))
    {
        Some(pair) => pair.1 = number,
        None => pairs.push((PREFERENCE_KEY.to_owned(), number)),
    }
    let mut value = String::new();
    for (key, rest) in &pairs {
        value.push_str(key);
        value.push('=');
        value.push_str(rest);
        value.push(';');
    }
    Some(value)
}

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

/// Closes a registry key when dropped.
struct Key(HKEY);

impl Drop for Key {
    fn drop(&mut self) {
        // SAFETY: the handle came from a successful RegCreateKeyExW and is closed exactly once, here. A failed close
        // cannot be acted on.
        let _ = unsafe { RegCloseKey(self.0) };
    }
}

/// Reads a REG_SZ value; a missing value or one of another type is `None`.
fn read_string(key: &Key, name: &[u16]) -> Result<Option<String>, PlatformError> {
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
    if status != ERROR_SUCCESS && status != ERROR_MORE_DATA {
        // Not found (or unreadable): the value counts as missing, like the legacy GetValue.
        return Ok(None);
    }
    if kind != REG_SZ {
        return Ok(None);
    }
    // One extra element keeps the text terminated even when the stored data is not.
    let mut buffer = vec![0_u16; (bytes as usize).div_ceil(2) + 1];
    let mut capacity = u32::try_from((buffer.len() - 1) * 2).unwrap_or(u32::MAX);
    // SAFETY: `buffer` holds at least `capacity` writable bytes, `name` is NUL-terminated and `kind`/`capacity` are valid
    // out pointers for the duration of the call.
    let status = unsafe {
        RegQueryValueExW(
            key.0,
            PCWSTR(name.as_ptr()),
            None,
            Some(&raw mut kind),
            Some(buffer.as_mut_ptr().cast()),
            Some(&raw mut capacity),
        )
    };
    if status != ERROR_SUCCESS {
        return Err(win32_error("RegQueryValueExW", status));
    }
    let length = buffer.iter().position(|unit| *unit == 0).unwrap_or(buffer.len());
    Ok(Some(String::from_utf16_lossy(&buffer[..length])))
}

/// Applies `ordinal` to the value `value_name` of the HKCU key `subkey` (created when missing). Only that one value is read or
/// written.
fn write_preference(subkey: &str, value_name: &str, ordinal: u32) -> Result<GpuPreferenceChange, PlatformError> {
    let subkey = wide(subkey);
    let name = wide(value_name);
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
    let key = Key(handle);

    let existing = read_string(&key, &name)?;
    let Some(value) = merged_value(existing.as_deref(), ordinal) else {
        return Ok(GpuPreferenceChange::Unchanged);
    };
    let data: Vec<u8> = wide(&value).iter().flat_map(|unit| unit.to_le_bytes()).collect();
    // SAFETY: `name` is NUL-terminated and `data` is the NUL-terminated UTF-16 text REG_SZ expects; both outlive the call.
    let status = unsafe { RegSetValueExW(key.0, PCWSTR(name.as_ptr()), None, REG_SZ, Some(&data)) };
    if status != ERROR_SUCCESS {
        return Err(win32_error("RegSetValueExW", status));
    }
    Ok(GpuPreferenceChange::Written)
}

#[cfg(test)]
mod tests {
    use windows::Win32::System::Registry::RegDeleteTreeW;

    use super::*;

    #[test]
    fn gpu_value_is_created_for_a_missing_one() {
        assert_eq!(merged_value(None, 2), Some("GpuPreference=2;".to_owned()));
    }

    #[test]
    fn gpu_value_is_left_alone_when_it_already_carries_the_preference() {
        assert_eq!(merged_value(Some("GpuPreference=2;"), 2), None);
    }

    #[test]
    fn gpu_default_without_a_value_writes_nothing() {
        assert_eq!(merged_value(None, 0), None);
    }

    #[test]
    fn gpu_other_pairs_are_kept() {
        assert_eq!(
            merged_value(Some("SwapEffectUpgradeEnable=1;"), 2),
            Some("SwapEffectUpgradeEnable=1;GpuPreference=2;".to_owned())
        );
        assert_eq!(
            merged_value(Some("GpuPreference=1;AutoHDREnable=1;"), 2),
            Some("GpuPreference=2;AutoHDREnable=1;".to_owned())
        );
    }

    #[test]
    fn gpu_keys_match_case_insensitively_and_keep_their_spelling() {
        assert_eq!(
            merged_value(Some("gpupreference=2;"), 0),
            Some("gpupreference=0;".to_owned())
        );
    }

    #[test]
    fn gpu_malformed_pairs_are_dropped_like_the_legacy_parser() {
        assert_eq!(
            merged_value(Some("GpuPreference=abc; =x;y=;"), 2),
            Some("GpuPreference=2;".to_owned())
        );
    }

    #[test]
    fn gpu_an_out_of_range_preference_counts_as_default() {
        assert_eq!(merged_value(Some("GpuPreference=7;"), 0), None);
        assert_eq!(merged_value(Some("GpuPreference=-1;"), 0), None);
        assert_eq!(
            merged_value(Some("GpuPreference=7;"), 1),
            Some("GpuPreference=1;".to_owned())
        );
    }

    #[test]
    fn gpu_value_name_drops_the_verbatim_prefix_before_a_drive_letter() {
        assert_eq!(
            value_name(Path::new(r"\\?\C:\Sucrose\sucrose.exe")).as_deref(),
            Ok(r"C:\Sucrose\sucrose.exe")
        );
        assert_eq!(
            value_name(Path::new(r"C:\Sucrose\sucrose.exe")).as_deref(),
            Ok(r"C:\Sucrose\sucrose.exe")
        );
        assert_eq!(
            value_name(Path::new(r"\\?\UNC\host\share\a.exe")).as_deref(),
            Ok(r"\\?\UNC\host\share\a.exe")
        );
    }

    /// Deletes the scratch key tree even when an assertion fails.
    struct ScratchKey(String);

    impl Drop for ScratchKey {
        fn drop(&mut self) {
            let subkey = wide(&self.0);
            // SAFETY: `subkey` is NUL-terminated and outlives the call; it names the scratch tree of this test under
            // HKCU, never the real DirectX key.
            let _ = unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, PCWSTR(subkey.as_ptr())) };
        }
    }

    fn read_back(subkey: &str, name: &str) -> Option<String> {
        let subkey = wide(subkey);
        let name = wide(name);
        let mut handle = HKEY::default();
        // SAFETY: `subkey` is NUL-terminated and outlives the call; `handle` is a valid out pointer.
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
        assert_eq!(status, ERROR_SUCCESS);
        read_string(&Key(handle), &name).expect("read")
    }

    #[test]
    fn gpu_registry_round_trip_on_a_scratch_key() {
        let subkey = format!(r"Software\SucroseTest\UserGpuPreferences-{}", std::process::id());
        let _guard = ScratchKey(subkey.clone());
        let name = r"C:\Scratch\sucrose.exe";
        let other = r"C:\Scratch\other.exe";

        assert_eq!(write_preference(&subkey, other, 1), Ok(GpuPreferenceChange::Written));
        assert_eq!(write_preference(&subkey, name, 0), Ok(GpuPreferenceChange::Unchanged));
        assert_eq!(read_back(&subkey, name), None);

        assert_eq!(write_preference(&subkey, name, 2), Ok(GpuPreferenceChange::Written));
        assert_eq!(read_back(&subkey, name).as_deref(), Some("GpuPreference=2;"));
        assert_eq!(write_preference(&subkey, name, 2), Ok(GpuPreferenceChange::Unchanged));

        // Another value of the key is never touched.
        assert_eq!(read_back(&subkey, other).as_deref(), Some("GpuPreference=1;"));
    }

    #[test]
    fn gpu_registry_write_keeps_the_pairs_it_did_not_own() {
        let subkey = format!(r"Software\SucroseTest\UserGpuPreferences-keep-{}", std::process::id());
        let _guard = ScratchKey(subkey.clone());
        let name = r"C:\Scratch\sucrose.exe";
        seed(&subkey, name, "SwapEffectUpgradeEnable=1;");

        assert_eq!(write_preference(&subkey, name, 2), Ok(GpuPreferenceChange::Written));
        assert_eq!(
            read_back(&subkey, name).as_deref(),
            Some("SwapEffectUpgradeEnable=1;GpuPreference=2;")
        );
    }

    fn seed(subkey: &str, name: &str, text: &str) {
        let subkey = wide(subkey);
        let name = wide(name);
        let mut handle = HKEY::default();
        // SAFETY: `subkey` is NUL-terminated and outlives the call; `handle` is a valid out pointer.
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
        assert_eq!(status, ERROR_SUCCESS);
        let key = Key(handle);
        let data: Vec<u8> = wide(text).iter().flat_map(|unit| unit.to_le_bytes()).collect();
        // SAFETY: `name` is NUL-terminated and `data` is NUL-terminated UTF-16 text; both outlive the call.
        let status = unsafe { RegSetValueExW(key.0, PCWSTR(name.as_ptr()), None, REG_SZ, Some(&data)) };
        assert_eq!(status, ERROR_SUCCESS);
    }
}
