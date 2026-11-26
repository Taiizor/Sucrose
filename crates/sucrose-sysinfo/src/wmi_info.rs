//! The groups the legacy app reads once through WMI: Bios (`Win32_BIOS`), Motherboard (`Win32_BaseBoard`) and the static
//! fields of the Processor group (`Win32_Processor`). Each query asks for the first instance only, as the legacy
//! `foreach … break` does, and a property WMI does not give is an empty text or a zero (`SSSHM.Check` with a default).

use serde_json::{Value, json};

use crate::SysInfoError;

/// The fields of the first `Win32_Processor` that the Processor group carries.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) struct ProcessorInfo {
    pub name: String,
    pub cores: u64,
    pub threads: u64,
}

/// The fields of the first `Win32_BIOS`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) struct BiosInfo {
    pub name: String,
    pub caption: String,
    pub version: String,
    pub description: String,
    pub release_date: String,
    pub manufacturer: String,
    pub serial_number: String,
    pub current_language: String,
}

/// The fields of the first `Win32_BaseBoard`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) struct MotherboardInfo {
    pub product: String,
    pub version: String,
    pub manufacturer: String,
}

/// The legacy Bios object of `info`: the keys in the order of `Data.cs`.
pub(crate) fn bios_payload(info: &BiosInfo, state: bool) -> Value {
    json!({
        "Name": info.name,
        "State": state,
        "Caption": info.caption,
        "Version": info.version,
        "Description": info.description,
        "ReleaseDate": info.release_date,
        "Manufacturer": info.manufacturer,
        "SerialNumber": info.serial_number,
        "CurrentLanguage": info.current_language
    })
}

/// The legacy Motherboard object of `info`; `Name` is the manufacturer and the product joined by a space, as the legacy
/// code builds it.
pub(crate) fn motherboard_payload(info: &MotherboardInfo, state: bool) -> Value {
    let name = if state {
        format!("{} {}", info.manufacturer, info.product)
    } else {
        String::new()
    };
    json!({
        "Name": name,
        "State": state,
        "Product": info.product,
        "Version": info.version,
        "Manufacturer": info.manufacturer
    })
}

/// The Bios group of the legacy default: `State` false and empty texts.
#[must_use]
pub fn bios_unavailable() -> Value {
    bios_payload(&BiosInfo::default(), false)
}

/// The Motherboard group of the legacy default: `State` false and empty texts.
#[must_use]
pub fn motherboard_unavailable() -> Value {
    motherboard_payload(&MotherboardInfo::default(), false)
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

#[cfg(windows)]
fn read_bios() -> Result<BiosInfo, SysInfoError> {
    let row = query::first_instance(
        "Win32_BIOS",
        "Name, Caption, Version, Description, ReleaseDate, Manufacturer, SerialNumber, CurrentLanguage",
    )?;
    Ok(BiosInfo {
        name: query::text(&row, "Name"),
        caption: query::text(&row, "Caption"),
        version: query::text(&row, "Version"),
        description: query::text(&row, "Description"),
        release_date: query::text(&row, "ReleaseDate"),
        manufacturer: query::text(&row, "Manufacturer"),
        serial_number: query::text(&row, "SerialNumber"),
        current_language: query::text(&row, "CurrentLanguage"),
    })
}

#[cfg(not(windows))]
fn read_bios() -> Result<BiosInfo, SysInfoError> {
    Err(SysInfoError::Unsupported("the BIOS data"))
}

#[cfg(windows)]
fn read_motherboard() -> Result<MotherboardInfo, SysInfoError> {
    let row = query::first_instance("Win32_BaseBoard", "Product, Version, Manufacturer")?;
    Ok(MotherboardInfo {
        product: query::text(&row, "Product"),
        version: query::text(&row, "Version"),
        manufacturer: query::text(&row, "Manufacturer"),
    })
}

#[cfg(not(windows))]
fn read_motherboard() -> Result<MotherboardInfo, SysInfoError> {
    Err(SysInfoError::Unsupported("the motherboard data"))
}

/// The Bios group, read from WMI now. The caller keeps the result: the data does not change while the service runs.
///
/// # Errors
/// [`SysInfoError::Os`] when WMI is unavailable or has no BIOS instance, [`SysInfoError::Unsupported`] on an OS
/// without a provider.
pub fn bios_snapshot() -> Result<Value, SysInfoError> {
    Ok(bios_payload(&read_bios()?, true))
}

/// The Motherboard group, read from WMI now. The caller keeps the result.
///
/// # Errors
/// [`SysInfoError::Os`] when WMI is unavailable or has no base board instance, [`SysInfoError::Unsupported`] on an OS
/// without a provider.
pub fn motherboard_snapshot() -> Result<Value, SysInfoError> {
    Ok(motherboard_payload(&read_motherboard()?, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_motherboard_name_joins_the_manufacturer_and_the_product() {
        let info = MotherboardInfo {
            product: "B550M".to_owned(),
            version: "Rev 1.0".to_owned(),
            manufacturer: "ASUSTeK".to_owned(),
        };

        let value = motherboard_payload(&info, true);

        assert_eq!(value["Name"], "ASUSTeK B550M");
        assert_eq!(value["State"], true);
    }

    #[test]
    fn the_unavailable_groups_have_state_false_and_empty_texts() {
        let bios = bios_unavailable();
        let board = motherboard_unavailable();

        assert_eq!(bios["State"], false);
        assert_eq!(bios["SerialNumber"], "");
        assert_eq!(board["State"], false);
        assert_eq!(board["Name"], "", "no manufacturer or product to join");
    }
}
