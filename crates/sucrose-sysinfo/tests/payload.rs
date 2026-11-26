//! The payload of each system data group against the legacy shape (`Data.cs`, docs/legacy/services.md section 1.2): the
//! same keys in the same order with the same kinds of value, so a page written against the C# payload reads the same
//! names. The tests that read the machine run on Windows; other systems report the providers as unsupported.

use std::{thread, time::Duration};

use serde_json::Value;
use sucrose_sysinfo::{
    ProcessorSampler, bios_snapshot, bios_unavailable, memory_snapshot, memory_unavailable, motherboard_snapshot,
    motherboard_unavailable, processor_unavailable,
};

/// The keys of the legacy `Data.cs` objects, in the order it writes them.
const MEMORY_KEYS: [&str; 9] = [
    "Name",
    "State",
    "MemoryLoad",
    "MemoryUsed",
    "VirtualName",
    "MemoryAvailable",
    "VirtualMemoryLoad",
    "VirtualMemoryUsed",
    "VirtualMemoryAvailable",
];
const PROCESSOR_KEYS: [&str; 13] = [
    "Max",
    "Min",
    "Now",
    "Core",
    "Name",
    "Type",
    "Cores",
    "State",
    "Thread",
    "CoresMax",
    "CoresMin",
    "CoresNow",
    "ProcessorCount",
];
const BIOS_KEYS: [&str; 9] = [
    "Name",
    "State",
    "Caption",
    "Version",
    "Description",
    "ReleaseDate",
    "Manufacturer",
    "SerialNumber",
    "CurrentLanguage",
];
const MOTHERBOARD_KEYS: [&str; 5] = ["Name", "State", "Product", "Version", "Manufacturer"];
/// The fields of the legacy `CoreSensor` struct, in declaration order (Newtonsoft writes fields that way).
const CORE_KEYS: [&str; 7] = ["Index", "IsMax", "IsMin", "Now", "Name", "TypeText", "Type"];

fn keys(value: &Value) -> Vec<&str> {
    value
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect()
}

fn load(value: &Value, key: &str) -> f64 {
    let number = value[key]
        .as_f64()
        .unwrap_or_else(|| panic!("{key} is a number: {}", value[key]));
    assert!((0.0..=100.0).contains(&number), "{key} is a percentage: {number}");
    number
}

#[test]
fn the_unavailable_memory_group_is_the_legacy_default() {
    let value = memory_unavailable();

    assert_eq!(keys(&value), MEMORY_KEYS);
    assert_eq!(value["State"], false);
    assert_eq!(value["Name"], "");
    assert_eq!(value["VirtualName"], "");
    assert_eq!(value["MemoryLoad"].as_f64(), Some(0.0));
}

#[test]
fn the_unavailable_processor_group_is_the_legacy_default() {
    let value = processor_unavailable();

    assert_eq!(keys(&value), PROCESSOR_KEYS);
    assert_eq!(value["State"], false);
    assert_eq!(value["Name"], "");
    assert_eq!(value["Type"], "");
    assert_eq!(value["Cores"], Value::Array(Vec::new()));
    assert_eq!(value["Max"].as_f64(), Some(0.0));
    assert_eq!(value["Min"].as_f64(), Some(100.0), "the legacy minimum starts at 100");
    assert_eq!(value["CoresMin"].as_f64(), Some(100.0));
}

#[test]
fn the_unavailable_bios_and_motherboard_groups_are_the_legacy_default() {
    let bios = bios_unavailable();
    let board = motherboard_unavailable();

    assert_eq!(keys(&bios), BIOS_KEYS);
    assert_eq!(bios["State"], false);
    assert!(
        BIOS_KEYS
            .iter()
            .filter(|key| **key != "State")
            .all(|key| bios[*key] == "")
    );
    assert_eq!(keys(&board), MOTHERBOARD_KEYS);
    assert_eq!(board["State"], false);
    assert!(
        MOTHERBOARD_KEYS
            .iter()
            .filter(|key| **key != "State")
            .all(|key| board[*key] == "")
    );
}

#[cfg(windows)]
#[test]
fn the_bios_group_has_the_legacy_keys_and_texts() {
    let value = bios_snapshot().expect("WMI gives the BIOS");

    assert_eq!(keys(&value), BIOS_KEYS);
    assert_eq!(value["State"], true);
    for key in BIOS_KEYS.iter().filter(|key| **key != "State") {
        assert!(value[*key].is_string(), "{key} is a text: {}", value[*key]);
    }
    assert!(
        value["Manufacturer"].as_str().is_some_and(|text| !text.is_empty()),
        "WMI knows the BIOS manufacturer"
    );
}

#[cfg(windows)]
#[test]
fn the_motherboard_group_has_the_legacy_keys_and_the_joined_name() {
    let value = motherboard_snapshot().expect("WMI gives the base board");

    assert_eq!(keys(&value), MOTHERBOARD_KEYS);
    assert_eq!(value["State"], true);
    for key in MOTHERBOARD_KEYS.iter().filter(|key| **key != "State") {
        assert!(value[*key].is_string(), "{key} is a text: {}", value[*key]);
    }
    let name = format!(
        "{} {}",
        value["Manufacturer"].as_str().expect("a text"),
        value["Product"].as_str().expect("a text")
    );
    assert_eq!(
        value["Name"], name,
        "Name is the manufacturer and the product, as the legacy code builds it"
    );
}

#[cfg(windows)]
#[test]
fn the_memory_group_has_the_legacy_keys_and_gib_values() {
    let value = memory_snapshot().expect("the memory status reads");

    assert_eq!(keys(&value), MEMORY_KEYS);
    assert_eq!(value["State"], true);
    assert_eq!(value["Name"], "Total Memory");
    assert_eq!(value["VirtualName"], "Virtual Memory");
    load(&value, "MemoryLoad");
    load(&value, "VirtualMemoryLoad");
    let used = value["MemoryUsed"].as_f64().expect("MemoryUsed is a number");
    let available = value["MemoryAvailable"].as_f64().expect("MemoryAvailable is a number");
    assert!(
        used > 0.0 && available > 0.0,
        "GiB values: {used} used, {available} available"
    );
    assert!(
        (used + available) < 16384.0,
        "the sum is the installed memory in GiB, not bytes: {}",
        used + available
    );
    assert!(value["VirtualMemoryUsed"].as_f64().is_some_and(|gib| gib > 0.0));
    assert!(value["VirtualMemoryAvailable"].as_f64().is_some());
}

#[cfg(windows)]
#[test]
fn the_second_processor_sample_has_the_legacy_keys_and_consistent_figures() {
    let mut sampler = ProcessorSampler::new().expect("the processor counters open");
    thread::sleep(Duration::from_millis(1200));

    let value = sampler.sample();

    assert_eq!(keys(&value), PROCESSOR_KEYS);
    let now = load(&value, "Now");
    let max = load(&value, "Max");
    let min = load(&value, "Min");
    assert!(min <= now && now <= max, "Min {min} <= Now {now} <= Max {max}");
    let logical = std::thread::available_parallelism().expect("a processor count").get();
    assert_eq!(value["ProcessorCount"].as_u64(), Some(logical as u64));
    let cores = value["Cores"].as_array().expect("Cores is an array");
    assert_eq!(cores.len(), logical, "one entry per logical processor");
    let mut highest: f64 = 0.0;
    for (index, core) in cores.iter().enumerate() {
        assert_eq!(keys(core), CORE_KEYS);
        assert_eq!(core["Index"].as_u64(), Some(index as u64));
        assert_eq!(core["Name"], format!("Core #{index}"));
        assert_eq!(core["TypeText"], "Timer100NsInverse");
        assert_eq!(core["Type"].as_i64(), Some(557_909_248));
        assert!(core["IsMax"].is_boolean() && core["IsMin"].is_boolean());
        highest = highest.max(load(core, "Now"));
    }
    let cores_now = load(&value, "CoresNow");
    assert!(
        (cores_now - highest).abs() < 1e-3,
        "CoresNow {cores_now} is the highest core {highest}"
    );
    assert_eq!(value["Type"], "Timer100NsInverse");
    assert_eq!(value["State"], true);
    assert!(
        value["Name"].as_str().is_some_and(|name| !name.is_empty()),
        "the WMI name"
    );
    assert!(value["Core"].as_u64().is_some_and(|cores| cores >= 1));
    assert!(value["Thread"].as_u64().is_some_and(|threads| threads >= 1));
}

#[cfg(windows)]
#[test]
fn a_processor_sampler_can_be_created_twice_in_one_process() {
    for _ in 0..2 {
        let mut sampler = ProcessorSampler::new().expect("the processor counters open");
        thread::sleep(Duration::from_millis(300));
        let value = sampler.sample();
        assert_eq!(keys(&value), PROCESSOR_KEYS);
        // The sampler is dropped here, which must release its PDH query.
    }
}

#[cfg(not(windows))]
#[test]
fn other_systems_report_bios_and_motherboard_as_unsupported() {
    assert!(matches!(
        bios_snapshot(),
        Err(sucrose_sysinfo::SysInfoError::Unsupported(_))
    ));
    assert!(matches!(
        motherboard_snapshot(),
        Err(sucrose_sysinfo::SysInfoError::Unsupported(_))
    ));
}

#[cfg(not(windows))]
#[test]
fn other_systems_report_memory_and_processor_as_unsupported() {
    assert!(matches!(
        memory_snapshot(),
        Err(sucrose_sysinfo::SysInfoError::Unsupported(_))
    ));
    assert!(matches!(
        ProcessorSampler::new(),
        Err(sucrose_sysinfo::SysInfoError::Unsupported(_))
    ));
}
