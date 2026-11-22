//! Proves the DPI awareness a fresh process ends up with. Integration tests are separate binaries, so this test owns its
//! process and no window or other test has set the awareness first.
#![cfg(windows)]

use sucrose_platform::{DpiAwareness, init_dpi_awareness};

#[test]
fn init_dpi_awareness_reports_per_monitor_v2() {
    assert_eq!(init_dpi_awareness(), DpiAwareness::PerMonitorV2);
}
