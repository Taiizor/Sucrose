//! The legacy fullscreen rule (PERF-05): a foreground window that covers a whole monitor makes the wallpaper step
//! aside. Everything here is pure: the service reads the foreground window and the monitors, feeds them in and
//! performs what comes out, so the rule is tested without a desktop.

use sucrose_core::settings::FullScreenAction;
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

/// The surfaces a fullscreen window covers, given the monitors it covers (`covered`) and the surfaces that have a
/// renderer (`surfaces`). Per display and Duplicate have one surface per monitor, so the monitors are the surfaces. In Span
/// one surface (`span`) lies over every monitor: a fullscreen window on any monitor covers it, and the rule acts on that
/// one renderer.
pub fn covered_surfaces(covered: Vec<String>, surfaces: &[String], span: &str) -> Vec<String> {
    if !surfaces.iter().any(|surface| surface == span) {
        return covered;
    }
    if covered.is_empty() {
        covered
    } else {
        vec![span.to_owned()]
    }
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

    /// Whether the rule is currently on.
    pub fn is_on(&self) -> bool {
        self.on
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

/// Which renderers the rule acts on; `Backgroundog.FullScreenScope` chooses (D-05), all monitors by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionScope {
    /// Every monitor's renderer, whichever monitor the fullscreen window is on (the legacy behavior).
    AllMonitors,
    /// Only the renderers of the monitors the fullscreen window covers.
    AffectedMonitor,
}

/// What the service does to the renderer of one monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleAction {
    /// Pause playback.
    Pause,
    /// Play again after a pause.
    Resume,
    /// Stop the renderer.
    Close,
    /// Start the renderer again after a Close.
    Respawn,
}

/// What to do for `action` when the rule flips: `became_active` on the way in, otherwise on the way out. `affected`
/// are the monitors the fullscreen window covers (on the way out: the ones the rule acted on), `all` every monitor
/// that has a renderer. Resume is the disabled rule and plans nothing.
pub fn plan_action(
    action: FullScreenAction,
    scope: ActionScope,
    became_active: bool,
    affected: &[String],
    all: &[String],
) -> Vec<(String, RuleAction)> {
    let step = match (action, became_active) {
        (FullScreenAction::Resume, _) => return Vec::new(),
        (FullScreenAction::Pause, true) => RuleAction::Pause,
        (FullScreenAction::Pause, false) => RuleAction::Resume,
        (FullScreenAction::Close, true) => RuleAction::Close,
        (FullScreenAction::Close, false) => RuleAction::Respawn,
    };
    let targets = match scope {
        ActionScope::AllMonitors => all,
        ActionScope::AffectedMonitor => affected,
    };
    targets.iter().map(|id| (id.clone(), step)).collect()
}

/// The fullscreen rule while it is on: the debounce of the samples and the monitors it acted on.
#[derive(Debug, Clone)]
pub struct Rule {
    action: FullScreenAction,
    scope: ActionScope,
    debounce: Debounce,
    acted: Vec<String>,
}

impl Rule {
    /// A rule that is off until five samples in a row see a fullscreen window.
    pub fn new(action: FullScreenAction, scope: ActionScope) -> Self {
        Self {
            action,
            scope,
            debounce: Debounce::new(TRIGGER_SAMPLES, RECOVER_SAMPLES),
            acted: Vec::new(),
        }
    }

    /// Takes one sample: `covered` are the monitors a fullscreen window covers now, `all` the monitors that have a
    /// renderer. Returns what to do, which is empty except on a flip.
    pub fn tick(&mut self, covered: &[String], all: &[String]) -> Vec<(String, RuleAction)> {
        let Some(active) = self.debounce.feed(!covered.is_empty()) else {
            return Vec::new();
        };
        if active {
            let plan = plan_action(self.action, self.scope, true, covered, all);
            self.acted = plan.iter().map(|(id, _)| id.clone()).collect();
            plan
        } else {
            // Undo what was done, on the monitors it was done to, even if the window has moved since.
            let acted = std::mem::take(&mut self.acted);
            plan_action(self.action, self.scope, false, &acted, all)
        }
    }

    /// Whether the renderer of `monitor` has to stay closed, for a monitor that appears while the rule is on.
    pub fn closes(&self, monitor: &str) -> bool {
        self.action == FullScreenAction::Close && self.acts_on(monitor)
    }

    fn acts_on(&self, monitor: &str) -> bool {
        self.debounce.is_on() && (self.scope == ActionScope::AllMonitors || self.acted.iter().any(|id| id == monitor))
    }

    /// Whether the renderer of `monitor` has to be paused, for a renderer that connects while the rule is on.
    pub fn pauses(&self, monitor: &str) -> bool {
        self.action == FullScreenAction::Pause && self.acts_on(monitor)
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
            device_path: None,
            edid: None,
            friendly: None,
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
    fn in_span_a_fullscreen_window_on_any_monitor_covers_the_one_surface() {
        let span = vec!["span:all".to_owned()];
        let monitors = vec!["D1".to_owned(), "D9".to_owned()];
        assert_eq!(covered_surfaces(vec!["D9".to_owned()], &span, "span:all"), span);
        assert_eq!(
            covered_surfaces(monitors.clone(), &span, "span:all"),
            span,
            "both is still one surface"
        );
        assert!(covered_surfaces(Vec::new(), &span, "span:all").is_empty());
        assert_eq!(
            covered_surfaces(vec!["D9".to_owned()], &monitors, "span:all"),
            ["D9"],
            "one surface per monitor leaves the monitors as they are"
        );
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

    fn ids(list: &[&str]) -> Vec<String> {
        list.iter().map(|id| (*id).to_owned()).collect()
    }

    fn acts(list: &[(&str, RuleAction)]) -> Vec<(String, RuleAction)> {
        list.iter().map(|(id, action)| ((*id).to_owned(), *action)).collect()
    }

    #[test]
    fn pause_on_all_monitors_pauses_every_renderer_and_resumes_them() {
        let (affected, all) = (ids(&["A"]), ids(&["A", "B"]));

        assert_eq!(
            plan_action(FullScreenAction::Pause, ActionScope::AllMonitors, true, &affected, &all),
            acts(&[("A", RuleAction::Pause), ("B", RuleAction::Pause)])
        );
        assert_eq!(
            plan_action(
                FullScreenAction::Pause,
                ActionScope::AllMonitors,
                false,
                &affected,
                &all
            ),
            acts(&[("A", RuleAction::Resume), ("B", RuleAction::Resume)])
        );
    }

    #[test]
    fn pause_on_the_affected_monitor_touches_only_that_monitor() {
        let (affected, all) = (ids(&["A"]), ids(&["A", "B"]));

        assert_eq!(
            plan_action(
                FullScreenAction::Pause,
                ActionScope::AffectedMonitor,
                true,
                &affected,
                &all
            ),
            acts(&[("A", RuleAction::Pause)])
        );
        assert_eq!(
            plan_action(
                FullScreenAction::Pause,
                ActionScope::AffectedMonitor,
                false,
                &affected,
                &all
            ),
            acts(&[("A", RuleAction::Resume)])
        );
    }

    #[test]
    fn close_stops_the_renderers_and_recovery_starts_them_again() {
        let (affected, all) = (ids(&["A"]), ids(&["A", "B"]));

        assert_eq!(
            plan_action(FullScreenAction::Close, ActionScope::AllMonitors, true, &affected, &all),
            acts(&[("A", RuleAction::Close), ("B", RuleAction::Close)])
        );
        assert_eq!(
            plan_action(
                FullScreenAction::Close,
                ActionScope::AllMonitors,
                false,
                &affected,
                &all
            ),
            acts(&[("A", RuleAction::Respawn), ("B", RuleAction::Respawn)])
        );
        assert_eq!(
            plan_action(
                FullScreenAction::Close,
                ActionScope::AffectedMonitor,
                true,
                &affected,
                &all
            ),
            acts(&[("A", RuleAction::Close)])
        );
    }

    #[test]
    fn resume_is_the_disabled_rule_and_plans_nothing() {
        let (affected, all) = (ids(&["A"]), ids(&["A", "B"]));

        for scope in [ActionScope::AllMonitors, ActionScope::AffectedMonitor] {
            for active in [true, false] {
                assert!(plan_action(FullScreenAction::Resume, scope, active, &affected, &all).is_empty());
            }
        }
    }

    #[test]
    fn a_rule_acts_on_the_fifth_positive_sample_and_undoes_it_on_the_third_negative_one() {
        let mut rule = Rule::new(FullScreenAction::Pause, ActionScope::AllMonitors);
        let (covered, all) = (ids(&["A"]), ids(&["A", "B"]));

        for _ in 0..4 {
            assert!(rule.tick(&covered, &all).is_empty());
        }
        assert_eq!(
            rule.tick(&covered, &all),
            acts(&[("A", RuleAction::Pause), ("B", RuleAction::Pause)])
        );
        assert!(rule.pauses("A") && rule.pauses("B"));
        assert!(rule.tick(&covered, &all).is_empty());
        assert!(rule.tick(&[], &all).is_empty());
        assert!(rule.tick(&[], &all).is_empty());
        assert_eq!(
            rule.tick(&[], &all),
            acts(&[("A", RuleAction::Resume), ("B", RuleAction::Resume)])
        );
        assert!(!rule.pauses("A"));
    }

    #[test]
    fn a_rule_with_the_affected_scope_resumes_the_monitor_it_paused_even_if_the_window_moved() {
        let mut rule = Rule::new(FullScreenAction::Pause, ActionScope::AffectedMonitor);
        let all = ids(&["A", "B"]);
        for _ in 0..TRIGGER_SAMPLES {
            rule.tick(&ids(&["A"]), &all);
        }

        assert!(rule.pauses("A"));
        assert!(!rule.pauses("B"));
        // The window moved to B, then went away: the recovery still undoes what was done, on A.
        rule.tick(&ids(&["B"]), &all);
        rule.tick(&[], &all);
        rule.tick(&[], &all);
        assert_eq!(rule.tick(&[], &all), acts(&[("A", RuleAction::Resume)]));
    }

    #[test]
    fn a_resume_rule_never_acts() {
        let mut rule = Rule::new(FullScreenAction::Resume, ActionScope::AllMonitors);
        let all = ids(&["A"]);

        for _ in 0..10 {
            assert!(rule.tick(&ids(&["A"]), &all).is_empty());
        }
        assert!(!rule.pauses("A"));
    }

    #[test]
    fn a_close_rule_does_not_ask_for_a_pause() {
        let mut rule = Rule::new(FullScreenAction::Close, ActionScope::AllMonitors);
        let all = ids(&["A"]);
        for _ in 0..TRIGGER_SAMPLES {
            rule.tick(&ids(&["A"]), &all);
        }

        assert!(!rule.pauses("A"));
    }

    #[test]
    fn a_close_rule_keeps_a_monitor_that_appears_meanwhile_closed() {
        let mut rule = Rule::new(FullScreenAction::Close, ActionScope::AllMonitors);
        let all = ids(&["A"]);
        assert!(!rule.closes("NEW"));
        for _ in 0..TRIGGER_SAMPLES {
            rule.tick(&ids(&["A"]), &all);
        }

        assert!(rule.closes("A") && rule.closes("NEW"));
    }
}
