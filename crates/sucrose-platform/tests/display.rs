//! Reads the real displays of this machine; the test is ignored so CI (which has no desktop) skips it.
#![cfg(windows)]
#![allow(clippy::print_stdout)]

use std::collections::HashSet;

#[test]
#[ignore = "reads the real displays of this machine; plan 40 runs it"]
fn monitors_have_stable_ids() {
    let monitors = sucrose_platform::current().monitors();
    assert!(!monitors.is_empty(), "the OS reported no monitor");
    for monitor in &monitors {
        println!(
            "monitor id={} stable_id={} edid={:?} friendly={:?} primary={}",
            monitor.id,
            monitor.stable_id(),
            monitor.edid,
            monitor.friendly,
            monitor.primary
        );
        assert!(!monitor.stable_id().is_empty(), "{} has an empty stable id", monitor.id);
    }
    let unique: HashSet<&str> = monitors.iter().map(|monitor| monitor.stable_id()).collect();
    assert_eq!(unique.len(), monitors.len(), "two monitors share a stable id");
}
