//! The Memory group (stub for the failing-test commit).

use serde_json::{Value, json};

use crate::SysInfoError;

/// The Memory group of the legacy default: State false, empty names, zero figures.
#[must_use]
pub fn memory_unavailable() -> Value {
    json!({
        "Name": "",
        "State": false,
        "MemoryLoad": 0.0,
        "MemoryUsed": 0.0,
        "VirtualName": "",
        "MemoryAvailable": 0.0,
        "VirtualMemoryLoad": 0.0,
        "VirtualMemoryUsed": 0.0,
        "VirtualMemoryAvailable": 0.0
    })
}

/// The Memory group.
///
/// # Errors
/// Not implemented yet.
pub fn memory_snapshot() -> Result<Value, SysInfoError> {
    Err(SysInfoError::Unsupported("the memory status"))
}
