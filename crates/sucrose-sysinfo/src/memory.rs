//! The Memory group: the physical memory and the page file as the legacy `GetMemoryInfo` writes them, from
//! `GlobalMemoryStatusEx`. The figures are single-precision numbers like the C# `float` ones, memory in GiB and the
//! loads in percent.

use serde_json::{Value, json};

use crate::{SysInfoError, float};

const GIB: f32 = (1024u32 * 1024 * 1024) as f32;

/// What `GlobalMemoryStatusEx` reports, in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) struct MemoryStatus {
    pub total_physical: u64,
    pub available_physical: u64,
    pub total_page_file: u64,
    pub available_page_file: u64,
}

/// The percentage of `total` in use, as the legacy `100.0f - (100.0f * available / total)`; 0 when the OS reports no total
/// (the C# formula gives NaN there, which is not a JSON number).
fn load(total: u64, available: u64) -> f32 {
    if total == 0 {
        return 0.0;
    }
    100.0 - (100.0 * available as f32 / total as f32)
}

fn gib(bytes: u64) -> f32 {
    bytes as f32 / GIB
}

/// The legacy object of `status`: the keys in the order of `Data.cs`.
pub(crate) fn snapshot(status: &MemoryStatus) -> Value {
    json!({
        "Name": "Total Memory",
        "State": true,
        "MemoryLoad": float(load(status.total_physical, status.available_physical)),
        "MemoryUsed": float(gib(status.total_physical.saturating_sub(status.available_physical))),
        "VirtualName": "Virtual Memory",
        "MemoryAvailable": float(gib(status.available_physical)),
        "VirtualMemoryLoad": float(load(status.total_page_file, status.available_page_file)),
        "VirtualMemoryUsed": float(gib(status.total_page_file.saturating_sub(status.available_page_file))),
        "VirtualMemoryAvailable": float(gib(status.available_page_file))
    })
}

/// The Memory group of the legacy default, which a page sees while the memory cannot be read: `State` false, empty names,
/// zero figures.
#[must_use]
pub fn memory_unavailable() -> Value {
    json!({
        "Name": "",
        "State": false,
        "MemoryLoad": float(0.0),
        "MemoryUsed": float(0.0),
        "VirtualName": "",
        "MemoryAvailable": float(0.0),
        "VirtualMemoryLoad": float(0.0),
        "VirtualMemoryUsed": float(0.0),
        "VirtualMemoryAvailable": float(0.0)
    })
}

#[cfg(windows)]
fn read() -> Result<MemoryStatus, SysInfoError> {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    let mut status = MEMORYSTATUSEX {
        dwLength: size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    // SAFETY: `status` is a valid, exclusively borrowed MEMORYSTATUSEX whose `dwLength` is set, which is all the call
    // requires.
    unsafe { GlobalMemoryStatusEx(&mut status) }.map_err(|error| SysInfoError::Os {
        call: "GlobalMemoryStatusEx",
        message: error.to_string(),
    })?;
    Ok(MemoryStatus {
        total_physical: status.ullTotalPhys,
        available_physical: status.ullAvailPhys,
        total_page_file: status.ullTotalPageFile,
        available_page_file: status.ullAvailPageFile,
    })
}

#[cfg(not(windows))]
fn read() -> Result<MemoryStatus, SysInfoError> {
    Err(SysInfoError::Unsupported("the memory status"))
}

/// The Memory group for the memory as it is now.
///
/// # Errors
/// [`SysInfoError::Os`] when the OS refuses the call, [`SysInfoError::Unsupported`] on an OS without a provider.
pub fn memory_snapshot() -> Result<Value, SysInfoError> {
    Ok(snapshot(&read()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATUS: MemoryStatus = MemoryStatus {
        total_physical: 16 * 1024 * 1024 * 1024,
        available_physical: 4 * 1024 * 1024 * 1024,
        total_page_file: 20 * 1024 * 1024 * 1024,
        available_page_file: 15 * 1024 * 1024 * 1024,
    };

    #[test]
    fn the_figures_follow_the_legacy_formulas() {
        let value = snapshot(&STATUS);

        assert_eq!(value["MemoryLoad"].as_f64(), Some(75.0));
        assert_eq!(value["MemoryUsed"].as_f64(), Some(12.0));
        assert_eq!(value["MemoryAvailable"].as_f64(), Some(4.0));
        assert_eq!(value["VirtualMemoryLoad"].as_f64(), Some(25.0));
        assert_eq!(value["VirtualMemoryUsed"].as_f64(), Some(5.0));
        assert_eq!(value["VirtualMemoryAvailable"].as_f64(), Some(15.0));
        assert_eq!(value["State"], true);
    }

    #[test]
    fn a_missing_total_gives_a_zero_load_not_a_non_number() {
        let status = MemoryStatus {
            total_page_file: 0,
            available_page_file: 0,
            ..STATUS
        };

        assert_eq!(snapshot(&status)["VirtualMemoryLoad"].as_f64(), Some(0.0));
    }
}
