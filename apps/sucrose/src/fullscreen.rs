//! The legacy fullscreen rule (PERF-05): a foreground window that covers a whole monitor makes the wallpaper step
//! aside. Everything here is pure: the service reads the foreground window and the monitors, feeds them in and
//! performs what comes out, so the rule is tested without a desktop.

use sucrose_platform::{ForegroundWindow, MonitorInfo};

/// The loop timer that samples the foreground window. It only exists while the rule is on (D-008).
pub const FULLSCREEN_TIMER: u32 = 3;

/// How often the foreground window is sampled, as in the legacy Backgroundog.
pub const FULLSCREEN_POLL_MS: u32 = 1000;

/// Consecutive positive samples that switch the rule on.
pub const TRIGGER_SAMPLES: u8 = 5;

/// Consecutive negative samples that switch the rule off again.
pub const RECOVER_SAMPLES: u8 = 3;

/// Window classes that are the desktop itself or the taskbar: being "in front" and covering a monitor does not make
/// them a fullscreen application (D-04).
const SHELL_CLASSES: [&str; 5] = [
    "Progman",
    "WorkerW",
    "SHELLDLL_DefView",
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
];

/// The ids of the monitors whose whole rectangle the foreground window covers. Empty when there is no foreground
/// window, when it is not a real application window (minimized, invisible, cloaked, the desktop, the taskbar) or
/// when it belongs to Sucrose itself (`own_pids`: the service and its renderers).
pub fn covered_monitors(fg: Option<&ForegroundWindow>, monitors: &[MonitorInfo], own_pids: &[u32]) -> Vec<String> {
    let Some(fg) = fg else {
        return Vec::new();
    };
    if !fg.visible || fg.minimized || fg.cloaked || own_pids.contains(&fg.process_id) {
        return Vec::new();
    }
    if SHELL_CLASSES.contains(&fg.class_name.as_str()) {
        return Vec::new();
    }
    monitors
        .iter()
        .filter(|monitor| fg.rect.covers(&monitor.rect))
        .map(|monitor| monitor.id.clone())
        .collect()
}

/// Turns a stream of yes/no samples into state flips: `trigger` equal samples in a row switch it on, `recover` equal
/// samples in a row switch it off. A single odd sample inside a run starts the count over.
#[derive(Debug, Clone)]
pub struct Debounce {
    trigger: u8,
    recover: u8,
    on: bool,
    run: u8,
}

impl Debounce {
    /// A debounce that is off, needing `trigger` positive samples to turn on and `recover` negative ones to turn off.
    pub fn new(trigger: u8, recover: u8) -> Self {
        Self {
            trigger,
            recover,
            on: false,
            run: 0,
        }
    }

    /// Takes one sample; returns the new state only when it flipped.
    pub fn feed(&mut self, active: bool) -> Option<bool> {
        if active == self.on {
            self.run = 0;
            return None;
        }
        self.run = self.run.saturating_add(1);
        let needed = if self.on { self.recover } else { self.trigger };
        if self.run >= needed {
            self.on = active;
            self.run = 0;
            Some(self.on)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use sucrose_platform::RectPx;

    use super::*;

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RectPx {
        RectPx {
            left,
            top,
            right,
            bottom,
        }
    }

    fn monitor(id: &str, area: RectPx) -> MonitorInfo {
        MonitorInfo {
            id: id.to_owned(),
            rect: area,
            work: area,
            dpi: 96,
            primary: id == "D1",
        }
    }

    fn window(class: &str, area: RectPx) -> ForegroundWindow {
        ForegroundWindow {
            rect: area,
            class_name: class.to_owned(),
            process_id: 4242,
            visible: true,
            minimized: false,
            cloaked: false,
        }
    }

    fn two_monitors() -> Vec<MonitorInfo> {
        vec![
            monitor("D1", rect(0, 0, 1920, 1080)),
            monitor("D2", rect(1920, 0, 3840, 1080)),
        ]
    }

    #[test]
    fn a_window_covering_a_monitor_covers_that_monitor_only() {
        let fg = window("Game", rect(0, 0, 1920, 1080));

        assert_eq!(covered_monitors(Some(&fg), &two_monitors(), &[]), ["D1"]);
    }

    #[test]
    fn a_window_that_stops_short_of_the_taskbar_is_not_fullscreen() {
        // Maximized on a 1920x1080 monitor with the taskbar visible: the work area only.
        let fg = window("Chrome_WidgetWin_1", rect(-8, -8, 1928, 1032));

        assert!(covered_monitors(Some(&fg), &two_monitors(), &[]).is_empty());
    }

    #[test]
    fn a_borderless_window_larger_than_the_monitor_is_fullscreen() {
        let fg = window("Game", rect(-8, -8, 1928, 1088));

        assert_eq!(covered_monitors(Some(&fg), &two_monitors(), &[]), ["D1"]);
    }

    #[test]
    fn a_window_spanning_both_monitors_covers_both() {
        let fg = window("Game", rect(0, 0, 3840, 1080));

        assert_eq!(covered_monitors(Some(&fg), &two_monitors(), &[]), ["D1", "D2"]);
    }

    #[test]
    fn no_foreground_window_covers_nothing() {
        assert!(covered_monitors(None, &two_monitors(), &[]).is_empty());
    }

    #[test]
    fn the_desktop_and_the_taskbar_are_never_fullscreen_applications() {
        for class in SHELL_CLASSES {
            let fg = window(class, rect(0, 0, 1920, 1080));

            assert!(covered_monitors(Some(&fg), &two_monitors(), &[]).is_empty(), "{class}");
        }
    }

    #[test]
    fn minimized_invisible_and_cloaked_windows_are_ignored() {
        let area = rect(0, 0, 1920, 1080);
        for change in [
            |fg: &mut ForegroundWindow| fg.minimized = true,
            |fg: &mut ForegroundWindow| fg.visible = false,
            |fg: &mut ForegroundWindow| fg.cloaked = true,
        ] {
            let mut fg = window("Game", area);
            change(&mut fg);

            assert!(covered_monitors(Some(&fg), &two_monitors(), &[]).is_empty());
        }
    }

    #[test]
    fn sucroses_own_windows_are_ignored() {
        let fg = window("Game", rect(0, 0, 1920, 1080));

        assert!(covered_monitors(Some(&fg), &two_monitors(), &[1, 4242]).is_empty());
        assert_eq!(covered_monitors(Some(&fg), &two_monitors(), &[1, 2]), ["D1"]);
    }

    #[test]
    fn five_positive_samples_in_a_row_switch_the_rule_on() {
        let mut debounce = Debounce::new(TRIGGER_SAMPLES, RECOVER_SAMPLES);

        for _ in 0..4 {
            assert_eq!(debounce.feed(true), None);
        }
        assert_eq!(debounce.feed(true), Some(true));
        assert_eq!(debounce.feed(true), None);
    }

    #[test]
    fn a_negative_sample_restarts_the_count_to_trigger() {
        let mut debounce = Debounce::new(TRIGGER_SAMPLES, RECOVER_SAMPLES);

        for _ in 0..4 {
            debounce.feed(true);
        }
        assert_eq!(debounce.feed(false), None);
        for _ in 0..4 {
            assert_eq!(debounce.feed(true), None);
        }
        assert_eq!(debounce.feed(true), Some(true));
    }

    #[test]
    fn three_negative_samples_in_a_row_switch_the_rule_off() {
        let mut debounce = Debounce::new(TRIGGER_SAMPLES, RECOVER_SAMPLES);
        for _ in 0..TRIGGER_SAMPLES {
            debounce.feed(true);
        }

        assert_eq!(debounce.feed(false), None);
        assert_eq!(debounce.feed(false), None);
        assert_eq!(debounce.feed(false), Some(false));
        assert_eq!(debounce.feed(false), None);
    }

    #[test]
    fn a_positive_sample_while_recovering_keeps_the_rule_on() {
        let mut debounce = Debounce::new(TRIGGER_SAMPLES, RECOVER_SAMPLES);
        for _ in 0..TRIGGER_SAMPLES {
            debounce.feed(true);
        }

        debounce.feed(false);
        debounce.feed(false);
        assert_eq!(debounce.feed(true), None);
        assert_eq!(debounce.feed(false), None);
        assert_eq!(debounce.feed(false), None);
        assert_eq!(debounce.feed(false), Some(false));
    }
}
