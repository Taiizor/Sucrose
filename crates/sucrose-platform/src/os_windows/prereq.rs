//! The Visual C++ Redistributable CEF needs, read from the registry the way the legacy app does (D-15, ENG-06):
//! `HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\{x86,x64,ARM64}` with the values `Installed` (a DWORD)
//! and `Version` (text such as `v14.44.35211.00`). Only read; nothing is written.

use windows::{
    Win32::{
        Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS},
        System::Registry::{
            HKEY, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE, KEY_WOW64_64KEY, REG_DWORD, REG_EXPAND_SZ, REG_SZ,
            REG_VALUE_TYPE, RegCloseKey, RegOpenKeyExW, RegQueryValueExW,
        },
    },
    core::PCWSTR,
};

use crate::{Arch, VcRedist};

/// The key of the runtime for `arch`, or `None` for a processor Visual C++ has no runtime key for.
fn runtime_key(arch: Arch) -> Option<&'static str> {
    match arch {
        Arch::X86 => Some(r"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x86"),
        Arch::X64 => Some(r"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64"),
        Arch::Arm64 => Some(r"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\ARM64"),
        Arch::Other => None,
    }
}

/// NUL-terminated UTF-16 for a Win32 string argument.
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Closes a registry key when dropped.
struct Key(HKEY);

impl Drop for Key {
    fn drop(&mut self) {
        // SAFETY: the handle came from a successful RegOpenKeyExW and is closed exactly once, here. A failed close
        // cannot be acted on.
        let _ = unsafe { RegCloseKey(self.0) };
    }
}

/// The type and bytes of the value `name` of `key`; `None` when it is missing or unreadable.
fn read_value(key: &Key, name: &str) -> Option<(REG_VALUE_TYPE, Vec<u8>)> {
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
    if status != ERROR_SUCCESS && status != ERROR_MORE_DATA {
        return None;
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
    if status != ERROR_SUCCESS {
        return None;
    }
    buffer.truncate(capacity as usize);
    Some((kind, buffer))
}

/// The number a DWORD value holds.
fn dword(bytes: &[u8]) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(..4)?.try_into().ok()?))
}

/// The text of a string value: UTF-16 up to the first NUL.
fn text(bytes: &[u8]) -> String {
    let (pairs, _) = bytes.as_chunks::<2>();
    let units: Vec<u16> = pairs.iter().map(|pair| u16::from_le_bytes(*pair)).collect();
    let end = units.iter().position(|unit| *unit == 0).unwrap_or(units.len());
    String::from_utf16_lossy(&units[..end])
}

/// Reads the Visual C++ runtime of `arch` through the 64-bit registry view, whatever the process is. A runtime that has
/// no key is reported as not installed; a processor without a runtime key has no answer.
pub(crate) fn vc_redist(arch: Arch) -> Option<VcRedist> {
    let subkey = wide(runtime_key(arch)?);
    let mut handle = HKEY::default();
    // SAFETY: `subkey` is NUL-terminated and outlives the call; `handle` is a valid out pointer.
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(subkey.as_ptr()),
            None,
            KEY_QUERY_VALUE | KEY_WOW64_64KEY,
            &raw mut handle,
        )
    };
    if status != ERROR_SUCCESS {
        return Some(VcRedist {
            installed: false,
            version: String::new(),
        });
    }
    let key = Key(handle);
    let installed = read_value(&key, "Installed")
        .filter(|(kind, _)| *kind == REG_DWORD)
        .and_then(|(_, bytes)| dword(&bytes))
        == Some(1);
    let version = read_value(&key, "Version")
        .filter(|(kind, _)| *kind == REG_SZ || *kind == REG_EXPAND_SZ)
        .map(|(_, bytes)| text(&bytes))
        .unwrap_or_default();
    Some(VcRedist { installed, version })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_processor_with_a_runtime_has_its_own_key() {
        assert!(runtime_key(Arch::X86).is_some_and(|key| key.ends_with(r"\x86")));
        assert!(runtime_key(Arch::X64).is_some_and(|key| key.ends_with(r"\x64")));
        assert!(runtime_key(Arch::Arm64).is_some_and(|key| key.ends_with(r"\ARM64")));
        assert_eq!(runtime_key(Arch::Other), None);
    }

    #[test]
    fn a_dword_and_a_version_text_are_read_from_their_bytes() {
        assert_eq!(dword(&1_u32.to_le_bytes()), Some(1));
        assert_eq!(dword(&[1, 0]), None);
        let bytes: Vec<u8> = "v14.44.35211.00\0junk"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        assert_eq!(text(&bytes), "v14.44.35211.00");
    }

    #[test]
    fn the_registry_answers_for_this_machines_architecture_without_failing() {
        let found = vc_redist(Arch::current());

        assert!(found.is_some());
        if let Some(found) = found.filter(|found| found.installed) {
            assert!(!found.version.is_empty());
        }
        assert_eq!(vc_redist(Arch::Other), None);
    }
}
