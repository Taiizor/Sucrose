//! The Processor group (stub for the failing-test commit).

use serde_json::{Value, json};

use crate::SysInfoError;

/// The Processor group of the legacy default: State false, empty text, the minimum at 100.
#[must_use]
pub fn processor_unavailable() -> Value {
    json!({
        "Max": 0.0,
        "Min": 100.0,
        "Now": 0.0,
        "Core": 0,
        "Name": "",
        "Type": "",
        "Cores": [],
        "State": false,
        "Thread": 0,
        "CoresMax": 0.0,
        "CoresMin": 100.0,
        "CoresNow": 0.0,
        "ProcessorCount": 0
    })
}

/// Reads the processor load.
pub struct ProcessorSampler;

impl ProcessorSampler {
    /// Opens the counters.
    ///
    /// # Errors
    /// Not implemented yet.
    pub fn new() -> Result<Self, SysInfoError> {
        Err(SysInfoError::Unsupported("the processor counters"))
    }

    /// The group with the latest figures.
    pub fn sample(&mut self) -> Value {
        processor_unavailable()
    }
}
