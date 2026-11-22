//! The GPU preference of Windows for an executable (Settings > Display > Graphics), written the way the C# app does
//! for its engine executables (DESK-12), plus the DXGI adapter list and the adapter a default Direct3D 11 device gets.

use std::path::Path;

use windows::{
    Win32::{
        Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS, HMODULE, LUID, WIN32_ERROR},
        Graphics::{
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::{D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION, D3D11CreateDevice, ID3D11Device},
            Dxgi::{CreateDXGIFactory1, DXGI_ERROR_NOT_FOUND, IDXGIAdapter, IDXGIDevice, IDXGIFactory1},
        },
        System::Registry::{
            HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ, REG_VALUE_TYPE,
            RegCloseKey, RegCreateKeyExW, RegQueryValueExW, RegSetValueExW,
        },
    },
    core::{HRESULT, Interface, PCWSTR},
};

use crate::{GpuAdapter, GpuPreferenceChange, PlatformError};
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

/// The OS enumeration order of the DXGI adapters is **not** evidence of which adapter a Direct3D device of this
/// process gets: the per-app GPU preference may change what a default device picks without reordering this list. Use
/// [`default_gpu_adapter`] for that.
///
/// Every adapter, software ones included, with its LUID in the form typeperf prints in GPU instance names. Empty when
/// DXGI cannot be used.
pub(crate) fn gpu_adapters() -> Vec<GpuAdapter> {
    // SAFETY: CreateDXGIFactory1 has no preconditions; the result is checked.
    let factory = match unsafe { CreateDXGIFactory1::<IDXGIFactory1>() } {
        Ok(factory) => factory,
        Err(error) => {
            tracing::debug!(error = %error, "CreateDXGIFactory1 failed, no adapter list");
            return Vec::new();
        }
    };
    let mut adapters = Vec::new();
    for index in 0.. {
        // SAFETY: `factory` is a live COM interface; running past the last adapter is reported as an error.
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(error) if error.code() == DXGI_ERROR_NOT_FOUND => break,
            Err(error) => {
                tracing::debug!(error = %error, index, "EnumAdapters1 failed, adapter list ends here");
                break;
            }
        };
        // SAFETY: `adapter` is a live COM interface returned by EnumAdapters1.
        match unsafe { adapter.GetDesc1() } {
            Ok(desc) => adapters.push(adapter_info(
                &desc.Description,
                desc.AdapterLuid,
                desc.DedicatedVideoMemory,
            )),
            Err(error) => tracing::debug!(error = %error, index, "GetDesc1 failed, adapter skipped"),
        }
    }
    adapters
}

/// The adapter a Direct3D 11 device gets when it names no adapter, read back from such a device
/// (`IDXGIDevice::GetAdapter`), so it reflects the GPU preference the OS applied to this process. libmpv makes the same
/// call while no `d3d11-adapter` is set (see `embedding_options` in sucrose-engine-mpv), so this is the adapter its video
/// device uses. The cost is one short-lived device on the adapter libmpv is about to use anyway. `None` where no
/// hardware device can be created.
pub(crate) fn default_gpu_adapter() -> Option<GpuAdapter> {
    let mut device: Option<ID3D11Device> = None;
    // SAFETY: no adapter, no software module, no flags and no feature levels are valid arguments; `device` is a valid
    // out pointer and neither the feature level nor an immediate context is requested.
    let created = unsafe {
        D3D11CreateDevice(
            None::<&IDXGIAdapter>,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_FLAG(0),
            None,
            D3D11_SDK_VERSION,
            Some(&raw mut device),
            None,
            None,
        )
    };
    if let Err(error) = created {
        tracing::debug!(error = %error, "D3D11CreateDevice failed, no default adapter");
        return None;
    }
    let dxgi: IDXGIDevice = match device?.cast() {
        Ok(dxgi) => dxgi,
        Err(error) => {
            tracing::debug!(error = %error, "the Direct3D device has no IDXGIDevice");
            return None;
        }
    };
    // SAFETY: `dxgi` is a live COM interface.
    let adapter = match unsafe { dxgi.GetAdapter() } {
        Ok(adapter) => adapter,
        Err(error) => {
            tracing::debug!(error = %error, "IDXGIDevice::GetAdapter failed");
            return None;
        }
    };
    // SAFETY: `adapter` is a live COM interface returned by GetAdapter.
    match unsafe { adapter.GetDesc() } {
        Ok(desc) => Some(adapter_info(
            &desc.Description,
            desc.AdapterLuid,
            desc.DedicatedVideoMemory,
        )),
        Err(error) => {
            tracing::debug!(error = %error, "IDXGIAdapter::GetDesc failed");
            None
        }
    }
}

/// The adapter fields Sucrose reports, from a DXGI description.
fn adapter_info(description: &[u16], luid: LUID, dedicated_video_memory: usize) -> GpuAdapter {
    let length = description
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(description.len());
    GpuAdapter {
        name: String::from_utf16_lossy(&description[..length]),
        luid: luid_text(luid.HighPart, luid.LowPart),
        dedicated_video_memory_mb: u64::try_from(dedicated_video_memory).unwrap_or(u64::MAX) / (1024 * 1024),
    }
}

/// A LUID as typeperf prints it in GPU instance names: `0x{HighPart:08X}_0x{LowPart:08X}`.
fn luid_text(high: i32, low: u32) -> String {
    format!("0x{high:08X}_0x{low:08X}")
}

#[cfg(test)]
mod tests {
    use windows::Win32::System::Registry::RegDeleteTreeW;

    use super::*;

    #[test]
    fn gpu_luid_text_is_high_part_then_low_part_in_the_typeperf_form() {
        assert_eq!(luid_text(0, 0xA1B2), "0x00000000_0x0000A1B2");
        assert_eq!(luid_text(1, 0xDEAD_BEEF), "0x00000001_0xDEADBEEF");
    }

    fn is_luid_text(text: &str) -> bool {
        let hex = |part: &str| part.len() == 8 && part.chars().all(|c| c.is_ascii_digit() || ('A'..='F').contains(&c));
        text.split_once('_')
            .and_then(|(high, low)| Some((high.strip_prefix("0x")?, low.strip_prefix("0x")?)))
            .is_some_and(|(high, low)| hex(high) && hex(low))
    }

    #[test]
    fn gpu_adapter_list_names_at_least_one_adapter() {
        let adapters = gpu_adapters();

        assert!(!adapters.is_empty());
        for adapter in &adapters {
            assert!(!adapter.name.is_empty());
            assert!(is_luid_text(&adapter.luid), "{}", adapter.luid);
        }
    }

    #[test]
    fn gpu_default_adapter_is_one_of_the_listed_adapters_when_a_device_exists() {
        // None is fine where no hardware Direct3D 11 device can be created (a runner without a GPU).
        let Some(adapter) = default_gpu_adapter() else { return };

        assert!(!adapter.name.is_empty());
        assert!(is_luid_text(&adapter.luid), "{}", adapter.luid);
        assert!(gpu_adapters().iter().any(|listed| listed.luid == adapter.luid));
    }

    #[test]
    fn gpu_adapter_info_stops_the_name_at_the_first_nul_and_reports_mebibytes() {
        let mut description = [0_u16; 128];
        for (slot, unit) in description.iter_mut().zip("Test GPU".encode_utf16()) {
            *slot = unit;
        }
        description[20] = u16::from(b'x');
        let luid = LUID {
            LowPart: 0xA1B2,
            HighPart: 0,
        };

        let adapter = adapter_info(&description, luid, 4 * 1024 * 1024 * 1024);

        assert_eq!(adapter.name, "Test GPU");
        assert_eq!(adapter.luid, "0x00000000_0x0000A1B2");
        assert_eq!(adapter.dedicated_video_memory_mb, 4096);
    }

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
