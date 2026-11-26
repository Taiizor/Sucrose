//! What the legacy app reads once through WMI: the static fields of the Processor group (`Win32_Processor`); Bios and
//! Motherboard follow in the same module. Each query asks for the first instance only, as the legacy
//! `foreach … break` does, and a property WMI does not give is an empty text or a zero (`SSSHM.Check` with a default).

use serde_json::{Value, json};

use crate::SysInfoError;

/// The Bios group of the legacy default: `State` false and empty texts.
#[must_use]
pub fn bios_unavailable() -> Value {
    json!({
        "Name": "", "State": false, "Caption": "", "Version": "", "Description": "", "ReleaseDate": "",
        "Manufacturer": "", "SerialNumber": "", "CurrentLanguage": ""
    })
}

/// The Motherboard group of the legacy default: `State` false and empty texts.
#[must_use]
pub fn motherboard_unavailable() -> Value {
    json!({ "Name": "", "State": false, "Product": "", "Version": "", "Manufacturer": "" })
}

/// The Bios group.
///
/// # Errors
/// Not implemented yet.
pub fn bios_snapshot() -> Result<Value, SysInfoError> {
    Err(SysInfoError::Unsupported("the BIOS data"))
}

/// The Motherboard group.
///
/// # Errors
/// Not implemented yet.
pub fn motherboard_snapshot() -> Result<Value, SysInfoError> {
    Err(SysInfoError::Unsupported("the motherboard data"))
}

/// The fields of the first `Win32_Processor` that the Processor group carries.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) struct ProcessorInfo {
    pub name: String,
    pub cores: u64,
    pub threads: u64,
}

#[cfg(windows)]
mod query {
    use std::collections::HashMap;

    use wmi::{Variant, WMIConnection};

    use crate::SysInfoError;

    /// One instance of a WMI class: property name to value.
    pub(super) type Instance = HashMap<String, Variant>;

    fn failed(call: &'static str, error: &wmi::WMIError) -> SysInfoError {
        SysInfoError::Os {
            call,
            message: error.to_string(),
        }
    }

    /// The first instance of `class` with the listed properties. The connection lives only for this call: WMI
    /// connections belong to the thread that made them, and the data is read once, so nothing is kept open.
    pub(super) fn first_instance(class: &'static str, properties: &str) -> Result<Instance, SysInfoError> {
        let connection = WMIConnection::new().map_err(|error| failed("WMI connect", &error))?;
        let mut rows: Vec<Instance> = connection
            .raw_query(format!("SELECT {properties} FROM {class}"))
            .map_err(|error| failed("WMI query", &error))?;
        if rows.is_empty() {
            return Err(SysInfoError::Os {
                call: "WMI query",
                message: format!("{class} has no instance"),
            });
        }
        Ok(rows.swap_remove(0))
    }

    /// A text property, empty when WMI gives none.
    pub(super) fn text(instance: &Instance, key: &str) -> String {
        match instance.get(key) {
            Some(Variant::String(text)) => text.clone(),
            _ => String::new(),
        }
    }

    /// A whole-number property, 0 when WMI gives none.
    pub(super) fn number(instance: &Instance, key: &str) -> u64 {
        match instance.get(key) {
            Some(Variant::UI1(value)) => u64::from(*value),
            Some(Variant::UI2(value)) => u64::from(*value),
            Some(Variant::UI4(value)) => u64::from(*value),
            Some(Variant::UI8(value)) => *value,
            Some(Variant::I1(value)) => u64::try_from(*value).unwrap_or(0),
            Some(Variant::I2(value)) => u64::try_from(*value).unwrap_or(0),
            Some(Variant::I4(value)) => u64::try_from(*value).unwrap_or(0),
            Some(Variant::I8(value)) => u64::try_from(*value).unwrap_or(0),
            _ => 0,
        }
    }
}

/// The name and the core and thread counts of the first processor.
#[cfg(windows)]
pub(crate) fn processor_info() -> Result<ProcessorInfo, SysInfoError> {
    let row = query::first_instance("Win32_Processor", "Name, NumberOfCores, NumberOfLogicalProcessors")?;
    Ok(ProcessorInfo {
        name: query::text(&row, "Name"),
        cores: query::number(&row, "NumberOfCores"),
        threads: query::number(&row, "NumberOfLogicalProcessors"),
    })
}

/// The name and the core and thread counts of the first processor.
#[cfg(not(windows))]
pub(crate) fn processor_info() -> Result<ProcessorInfo, SysInfoError> {
    Err(SysInfoError::Unsupported("the processor information"))
}
