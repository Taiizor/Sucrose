//! Which renderer serves which monitor, and what changes when the monitors change.
//!
//! Everything here is pure data in and out, so the multi-monitor and hot-plug rules are tested without a display:
//! the service takes monitor snapshots from the platform, turns the difference between two of them into actions
//! and applies those. Monitors are identified by their device name, never by their position in the list, because
//! the OS may enumerate the same monitors in another order.

use std::collections::{BTreeMap, BTreeSet};

use sucrose_core::settings::ScreenArea;
use sucrose_platform::{MonitorInfo, RectPx};

/// What has to happen to the renderers for the monitors to match a new snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopologyAction {
    /// A monitor appeared: start a renderer for it.
    Spawn(MonitorInfo),
    /// The monitor with this id is gone: close its renderer.
    Close(String),
    /// The monitor with this id moved, was resized or changed its DPI: move the renderer's surface, keep playing.
    SetRect(String, RectPx),
}

/// The monitor list with each monitor's rectangle replaced by the area a wallpaper covers (Engine.json ScreenType):
/// the whole monitor or its work area. Afterwards `rect` is the surface rectangle and `work` equals it, so a change
/// of the taskbar alone does not move a wallpaper that covers the whole monitor.
pub fn with_area(monitors: Vec<MonitorInfo>, area: ScreenArea) -> Vec<MonitorInfo> {
    monitors
        .into_iter()
        .map(|monitor| {
            let chosen = match area {
                ScreenArea::DisplayBound => monitor.rect,
                ScreenArea::WorkingArea => monitor.work,
            };
            MonitorInfo {
                rect: chosen,
                work: chosen,
                ..monitor
            }
        })
        .collect()
}

/// The primary monitor, found by its flag and not by its position in the list.
pub fn primary(monitors: &[MonitorInfo]) -> Option<&MonitorInfo> {
    monitors.iter().find(|monitor| monitor.primary)
}

/// The monitor whose renderer owns the persistent web profile (D-040): the one flagged primary, otherwise the one with
/// the lowest id (the fallback of the master election), `None` for an empty list.
pub fn web_profile_owner(monitors: &[MonitorInfo]) -> Option<&MonitorInfo> {
    primary(monitors).or_else(|| monitors.iter().min_by(|a, b| a.id.cmp(&b.id)))
}

/// What to do to go from the monitors `old` to the monitors `new`: first the closes, then the rectangle changes,
/// then the spawns, each group ordered by monitor id. An empty `new` gives nothing: the OS reports no monitor for
/// a moment while a monitor is plugged in or out, which must not stop the wallpaper.
pub fn diff(old: &[MonitorInfo], new: &[MonitorInfo]) -> Vec<TopologyAction> {
    actions_between(old, new, false)
}

/// What to do after a display change: the closes and spawns of [`diff`], and a SetRect with its current rectangle for
/// every monitor that stays, changed or not. A surface's position also depends on its parent window, which explorer
/// may move while the monitor's own rectangle stays the same (an origin-only change, a primary switch), so the
/// surfaces of unchanged monitors have to be put back too; the legacy app re-places every engine window after each
/// display change. Same order and id rules as [`diff`]: closes, SetRects, spawns, each in id order; a repeated id
/// counts once; an empty `new` gives nothing.
pub fn replace_all(old: &[MonitorInfo], new: &[MonitorInfo]) -> Vec<TopologyAction> {
    actions_between(old, new, true)
}

/// The shared walk of [`diff`] and [`replace_all`]; `every_survivor` also moves the monitors that did not change.
fn actions_between(old: &[MonitorInfo], new: &[MonitorInfo], every_survivor: bool) -> Vec<TopologyAction> {
    if new.is_empty() {
        return Vec::new();
    }
    let old = by_id(old);
    let new = by_id(new);
    let mut actions = Vec::new();
    for id in old.keys().filter(|id| !new.contains_key(*id)) {
        actions.push(TopologyAction::Close((*id).to_owned()));
    }
    for (id, now) in &new {
        if let Some(before) = old.get(id)
            && (every_survivor || before.rect != now.rect || before.work != now.work || before.dpi != now.dpi)
        {
            actions.push(TopologyAction::SetRect((*id).to_owned(), now.rect));
        }
    }
    for (id, now) in &new {
        if !old.contains_key(id) {
            actions.push(TopologyAction::Spawn((*now).clone()));
        }
    }
    actions
}

/// The monitors by id, in id order. A repeated id keeps its first entry.
fn by_id(monitors: &[MonitorInfo]) -> BTreeMap<&str, &MonitorInfo> {
    let mut map = BTreeMap::new();
    for monitor in monitors {
        map.entry(monitor.id.as_str()).or_insert(monitor);
    }
    map
}

/// Which renderer slot belongs to which monitor. A slot is a small number that stays with its monitor for as long as
/// the monitor is connected, and is only given to another monitor after the renderer that used it has ended.
#[derive(Debug, Default)]
pub struct SlotMap {
    monitors: BTreeMap<String, u8>,
    /// Slots of monitors that were removed, whose renderer has not ended yet.
    draining: BTreeSet<u8>,
}

impl SlotMap {
    /// The slot of monitor `id`: the one it has, or the lowest free one. `None` when all 256 are taken.
    pub fn assign(&mut self, id: &str) -> Option<u8> {
        if let Some(slot) = self.monitors.get(id) {
            return Some(*slot);
        }
        let slot = (0..=u8::MAX).find(|slot| !self.is_taken(*slot))?;
        self.monitors.insert(id.to_owned(), slot);
        Some(slot)
    }

    fn is_taken(&self, slot: u8) -> bool {
        self.draining.contains(&slot) || self.monitors.values().any(|taken| *taken == slot)
    }

    /// The slot of monitor `id`, if it has one.
    pub fn slot_of(&self, id: &str) -> Option<u8> {
        self.monitors.get(id).copied()
    }

    /// The slots of the monitors that are connected now, ascending.
    pub fn active_slots(&self) -> Vec<u8> {
        let mut slots: Vec<u8> = self.monitors.values().copied().collect();
        slots.sort_unstable();
        slots
    }

    /// The monitor that uses `slot`, if any.
    pub fn monitor_of(&self, slot: u8) -> Option<&str> {
        self.monitors
            .iter()
            .find_map(|(id, taken)| (*taken == slot).then_some(id.as_str()))
    }

    /// The monitor `id` is gone: its slot stays reserved until [`SlotMap::release`]. Returns the slot.
    pub fn retire(&mut self, id: &str) -> Option<u8> {
        let slot = self.monitors.remove(id)?;
        self.draining.insert(slot);
        Some(slot)
    }

    /// The renderer of a retired slot ended: the slot can be used again. `false` when `slot` was not retired.
    pub fn release(&mut self, slot: u8) -> bool {
        self.draining.remove(&slot)
    }
}

/// The timer that waits for a burst of display events to end.
pub const TOPOLOGY_DEBOUNCE: u32 = 1;
/// The timer that waits before the settle check.
pub const TOPOLOGY_SETTLE: u32 = 2;
/// How long the monitors have to stay quiet before they are looked at (the legacy app's debounce).
pub const DEBOUNCE_MS: u32 = 2000;
/// How long after the first look the second one is taken (the legacy app's settle check).
pub const SETTLE_MS: u32 = 500;

/// What the service has to do with one of its loop timers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerCommand {
    /// Start the timer (or restart it when it already runs).
    Arm {
        /// The timer's id.
        id: u32,
        /// Milliseconds until it fires.
        ms: u32,
    },
    /// Stop the timer. Window timers repeat, so each one is stopped as soon as it has done its job.
    Kill(u32),
}

/// What a step of the [`Debounce`] asks for.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct DebounceStep {
    /// Timers to start or stop, in order.
    pub timers: Vec<TimerCommand>,
    /// The monitors to apply (a stable snapshot), once display events have stopped and two looks agree.
    pub apply: Option<Vec<MonitorInfo>>,
}

#[derive(Debug, Default)]
enum Phase {
    /// No display event is pending and no timer runs.
    #[default]
    Idle,
    /// Display events came in; waiting for [`DEBOUNCE_MS`] without another one.
    Waiting,
    /// First look taken, waiting [`SETTLE_MS`] for the second.
    Settling(Vec<MonitorInfo>),
}

/// Turns bursts of display events into at most one application of the monitors: [`DEBOUNCE_MS`] after the last
/// event the monitors are looked at, [`SETTLE_MS`] later they are looked at again, and only two equal looks are
/// applied. It owns no timer: it says which timers to start and stop, so a fake sequence of events tests it.
#[derive(Debug, Default)]
pub struct Debounce {
    phase: Phase,
}

impl Debounce {
    /// A display event arrived: the debounce starts over.
    pub fn display_changed(&mut self) -> DebounceStep {
        let mut timers = Vec::new();
        if matches!(self.phase, Phase::Settling(_)) {
            timers.push(TimerCommand::Kill(TOPOLOGY_SETTLE));
        }
        self.phase = Phase::Waiting;
        timers.push(TimerCommand::Arm {
            id: TOPOLOGY_DEBOUNCE,
            ms: DEBOUNCE_MS,
        });
        DebounceStep { timers, apply: None }
    }

    /// The debounce timer fired and `snapshot` is the monitors now.
    pub fn debounce_fired(&mut self, snapshot: Vec<MonitorInfo>) -> DebounceStep {
        let mut timers = vec![TimerCommand::Kill(TOPOLOGY_DEBOUNCE)];
        // A stale firing (the timer was restarted after its message was queued) changes nothing.
        if matches!(self.phase, Phase::Waiting) {
            self.phase = Phase::Settling(snapshot);
            timers.push(TimerCommand::Arm {
                id: TOPOLOGY_SETTLE,
                ms: SETTLE_MS,
            });
        }
        DebounceStep { timers, apply: None }
    }

    /// The settle timer fired and `snapshot` is the monitors now: equal to the first look, it is applied; otherwise
    /// the monitors are still changing and the debounce starts over.
    pub fn settle_fired(&mut self, snapshot: Vec<MonitorInfo>) -> DebounceStep {
        let mut timers = vec![TimerCommand::Kill(TOPOLOGY_SETTLE)];
        match std::mem::take(&mut self.phase) {
            Phase::Settling(first) if first == snapshot => {
                return DebounceStep {
                    timers,
                    apply: Some(snapshot),
                };
            }
            Phase::Settling(_) => {
                self.phase = Phase::Waiting;
                timers.push(TimerCommand::Arm {
                    id: TOPOLOGY_DEBOUNCE,
                    ms: DEBOUNCE_MS,
                });
            }
            other => self.phase = other,
        }
        DebounceStep { timers, apply: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RectPx {
        RectPx {
            left,
            top,
            right,
            bottom,
        }
    }

    fn monitor(id: &str, area: RectPx, dpi: u32, primary: bool) -> MonitorInfo {
        MonitorInfo {
            id: id.to_owned(),
            rect: area,
            work: area,
            dpi,
            primary,
        }
    }

    fn left() -> MonitorInfo {
        monitor("\\\\.\\DISPLAY1", rect(0, 0, 1920, 1080), 96, true)
    }

    fn right() -> MonitorInfo {
        monitor("\\\\.\\DISPLAY2", rect(1920, 0, 3840, 1080), 96, false)
    }

    #[test]
    fn replace_all_an_origin_shift_moves_the_unchanged_monitor_too() {
        let old = [left(), monitor("\\\\.\\DISPLAY9", rect(1920, 0, 3840, 1080), 96, false)];
        let moved = monitor("\\\\.\\DISPLAY9", rect(-1920, 0, 0, 1080), 96, false);
        let new = [left(), moved.clone()];

        assert_eq!(
            replace_all(&old, &new),
            [
                TopologyAction::SetRect(left().id, left().rect),
                TopologyAction::SetRect(moved.id.clone(), moved.rect),
            ]
        );
        assert_eq!(diff(&old, &new), [TopologyAction::SetRect(moved.id, moved.rect)]);
    }

    #[test]
    fn replace_all_a_primary_switch_moves_both_monitors() {
        let old = [
            monitor("\\\\.\\DISPLAY1", rect(-1920, 0, 0, 1080), 96, false),
            monitor("\\\\.\\DISPLAY9", rect(0, 0, 1920, 1080), 96, true),
        ];
        let new = [
            monitor("\\\\.\\DISPLAY1", rect(1920, 0, 3840, 1080), 96, false),
            monitor("\\\\.\\DISPLAY9", rect(0, 0, 1920, 1080), 96, true),
        ];

        assert_eq!(
            replace_all(&old, &new),
            [
                TopologyAction::SetRect("\\\\.\\DISPLAY1".to_owned(), rect(1920, 0, 3840, 1080)),
                TopologyAction::SetRect("\\\\.\\DISPLAY9".to_owned(), rect(0, 0, 1920, 1080)),
            ]
        );
    }

    #[test]
    fn replace_all_the_same_monitors_give_one_set_rect_each() {
        let actions = replace_all(&[right(), left()], &[left(), right()]);

        assert_eq!(
            actions,
            [
                TopologyAction::SetRect(left().id, left().rect),
                TopologyAction::SetRect(right().id, right().rect),
            ]
        );
    }

    #[test]
    fn replace_all_orders_closes_then_set_rects_then_spawns() {
        let gone = monitor("\\\\.\\DISPLAY0", rect(-1920, 0, 0, 1080), 96, false);
        let added = monitor("\\\\.\\DISPLAY3", rect(3840, 0, 5760, 1080), 96, false);

        let actions = replace_all(&[right(), gone.clone(), left()], &[added.clone(), right(), left()]);

        assert_eq!(
            actions,
            [
                TopologyAction::Close(gone.id),
                TopologyAction::SetRect(left().id, left().rect),
                TopologyAction::SetRect(right().id, right().rect),
                TopologyAction::Spawn(added),
            ]
        );
    }

    #[test]
    fn replace_all_an_empty_new_snapshot_gives_nothing() {
        assert!(replace_all(&[left(), right()], &[]).is_empty());
    }

    #[test]
    fn replace_all_from_nothing_only_spawns() {
        assert_eq!(
            replace_all(&[], &[right(), left()]),
            [TopologyAction::Spawn(left()), TopologyAction::Spawn(right())]
        );
    }

    #[test]
    fn replace_all_a_repeated_id_gives_one_action() {
        let twin = monitor(&left().id, rect(5, 5, 10, 10), 96, false);

        assert_eq!(
            replace_all(&[left(), twin.clone()], &[twin, left()]),
            [TopologyAction::SetRect(left().id, rect(5, 5, 10, 10))]
        );
    }

    #[test]
    fn from_nothing_to_two_monitors_spawns_both_sorted_by_id() {
        let actions = diff(&[], &[right(), left()]);

        assert_eq!(actions, [TopologyAction::Spawn(left()), TopologyAction::Spawn(right())]);
    }

    #[test]
    fn the_same_monitors_in_another_order_give_no_action() {
        assert!(diff(&[left(), right()], &[right(), left()]).is_empty());
    }

    #[test]
    fn the_primary_monitor_is_found_by_its_flag_in_any_order() {
        assert_eq!(primary(&[left(), right()]), Some(&left()));
        assert_eq!(primary(&[right(), left()]), Some(&left()));
        assert_eq!(primary(&[right()]), None);
    }

    #[test]
    fn the_web_profile_owner_is_the_primary_in_any_order_else_the_lowest_id() {
        assert_eq!(web_profile_owner(&[left(), right()]), Some(&left()));
        assert_eq!(web_profile_owner(&[right(), left()]), Some(&left()));
        let mut a = monitor("\\\\.\\DISPLAY2", rect(0, 0, 10, 10), 96, false);
        let b = monitor("\\\\.\\DISPLAY1", rect(10, 0, 20, 10), 96, false);
        assert_eq!(web_profile_owner(&[a.clone(), b.clone()]), Some(&b));
        a.primary = true;
        assert_eq!(web_profile_owner(&[b, a.clone()]), Some(&a));
        assert_eq!(web_profile_owner(&[]), None);
    }

    #[test]
    fn a_changed_rect_gives_one_set_rect_and_nothing_else() {
        let moved = monitor(&left().id, rect(0, 0, 2560, 1440), 96, true);

        let actions = diff(&[left(), right()], &[moved.clone(), right()]);

        assert_eq!(actions, [TopologyAction::SetRect(left().id, moved.rect)]);
    }

    #[test]
    fn a_changed_dpi_alone_gives_a_set_rect_with_the_unchanged_rect() {
        let scaled = monitor(&left().id, left().rect, 144, true);

        let actions = diff(&[left()], &[scaled]);

        assert_eq!(actions, [TopologyAction::SetRect(left().id, left().rect)]);
    }

    #[test]
    fn a_removed_monitor_gives_one_close_for_that_monitor_only() {
        let actions = diff(&[left(), right()], &[left()]);

        assert_eq!(actions, [TopologyAction::Close(right().id)]);
    }

    #[test]
    fn an_empty_new_snapshot_gives_no_action() {
        assert!(diff(&[left(), right()], &[]).is_empty());
        assert!(diff(&[], &[]).is_empty());
    }

    #[test]
    fn closes_come_before_rect_changes_and_spawns() {
        let third = monitor("\\\\.\\DISPLAY3", rect(3840, 0, 5760, 1080), 96, false);
        let moved = monitor(&left().id, rect(0, 0, 2560, 1440), 96, true);

        let actions = diff(&[left(), right()], &[moved.clone(), third.clone()]);

        assert_eq!(
            actions,
            [
                TopologyAction::Close(right().id),
                TopologyAction::SetRect(left().id, moved.rect),
                TopologyAction::Spawn(third),
            ]
        );
    }

    #[test]
    fn adjacent_monitors_keep_their_exact_rects_without_overlap_or_gap() {
        let actions = diff(&[], &[left(), right()]);

        let rects: Vec<RectPx> = actions
            .iter()
            .filter_map(|action| match action {
                TopologyAction::Spawn(monitor) => Some(monitor.rect),
                _ => None,
            })
            .collect();
        assert_eq!(rects, [rect(0, 0, 1920, 1080), rect(1920, 0, 3840, 1080)]);
        assert_eq!(rects[0].right, rects[1].left, "the surfaces share an edge");
    }

    #[test]
    fn a_repeated_id_spawns_once() {
        let actions = diff(&[], &[left(), left()]);

        assert_eq!(actions, [TopologyAction::Spawn(left())]);
    }

    #[test]
    fn with_area_picks_the_whole_monitor_or_its_work_area() {
        let mut with_taskbar = left();
        with_taskbar.work = rect(0, 0, 1920, 1040);

        let bound = with_area(vec![with_taskbar.clone()], ScreenArea::DisplayBound);
        let working = with_area(vec![with_taskbar], ScreenArea::WorkingArea);

        assert_eq!(bound[0].rect, rect(0, 0, 1920, 1080));
        assert_eq!(working[0].rect, rect(0, 0, 1920, 1040));
        assert_eq!(working[0].work, working[0].rect);
    }

    fn arm(id: u32, ms: u32) -> TimerCommand {
        TimerCommand::Arm { id, ms }
    }

    #[test]
    fn debounce_a_burst_of_events_leads_to_exactly_one_application() {
        let mut debounce = Debounce::default();

        for _ in 0..5 {
            let step = debounce.display_changed();
            assert_eq!(step.timers, [arm(TOPOLOGY_DEBOUNCE, 2000)]);
            assert_eq!(step.apply, None);
        }
        let first = debounce.debounce_fired(vec![left()]);
        let second = debounce.settle_fired(vec![left()]);

        assert_eq!(
            first.timers,
            [TimerCommand::Kill(TOPOLOGY_DEBOUNCE), arm(TOPOLOGY_SETTLE, 500)]
        );
        assert_eq!(second.apply, Some(vec![left()]));
        assert_eq!(second.timers, [TimerCommand::Kill(TOPOLOGY_SETTLE)]);
        // Nothing is left over: a late settle message does not apply again.
        assert_eq!(debounce.settle_fired(vec![left()]).apply, None);
    }

    #[test]
    fn debounce_two_looks_that_differ_restart_the_wait_without_applying() {
        let mut debounce = Debounce::default();
        debounce.display_changed();
        debounce.debounce_fired(vec![left()]);

        let step = debounce.settle_fired(vec![left(), right()]);

        assert_eq!(step.apply, None);
        assert_eq!(
            step.timers,
            [TimerCommand::Kill(TOPOLOGY_SETTLE), arm(TOPOLOGY_DEBOUNCE, 2000)]
        );
        // The wait ran again and the monitors stayed the same this time.
        debounce.debounce_fired(vec![left(), right()]);
        assert_eq!(
            debounce.settle_fired(vec![left(), right()]).apply,
            Some(vec![left(), right()])
        );
    }

    #[test]
    fn debounce_an_event_during_the_settle_wait_starts_over() {
        let mut debounce = Debounce::default();
        debounce.display_changed();
        debounce.debounce_fired(vec![left()]);

        let step = debounce.display_changed();

        assert_eq!(
            step.timers,
            [TimerCommand::Kill(TOPOLOGY_SETTLE), arm(TOPOLOGY_DEBOUNCE, 2000)]
        );
        assert_eq!(
            debounce.settle_fired(vec![left()]).apply,
            None,
            "a stale settle message does nothing"
        );
    }

    #[test]
    fn debounce_timers_that_fire_while_idle_only_stop_themselves() {
        let mut debounce = Debounce::default();

        let debounce_step = debounce.debounce_fired(vec![left()]);
        let settle_step = debounce.settle_fired(vec![left()]);

        assert_eq!(debounce_step.timers, [TimerCommand::Kill(TOPOLOGY_DEBOUNCE)]);
        assert_eq!(settle_step.timers, [TimerCommand::Kill(TOPOLOGY_SETTLE)]);
        assert_eq!(debounce_step.apply, None);
        assert_eq!(settle_step.apply, None);
    }

    #[test]
    fn debounce_never_leaves_a_timer_running_after_it_applied() {
        let mut debounce = Debounce::default();
        let mut running: BTreeSet<u32> = BTreeSet::new();
        let mut run = |step: DebounceStep| {
            for command in step.timers {
                match command {
                    TimerCommand::Arm { id, .. } => running.insert(id),
                    TimerCommand::Kill(id) => running.remove(&id),
                };
            }
            step.apply
        };

        run(debounce.display_changed());
        run(debounce.debounce_fired(vec![left()]));
        let applied = run(debounce.settle_fired(vec![left()]));

        assert!(applied.is_some());
        assert!(running.is_empty(), "still running: {running:?}");
    }

    #[test]
    fn slots_follow_the_monitor_not_the_order() {
        let mut slots = SlotMap::default();

        assert_eq!(slots.assign("A"), Some(0));
        assert_eq!(slots.assign("B"), Some(1));
        assert_eq!(slots.assign("A"), Some(0), "a monitor keeps its slot");
        assert_eq!(slots.slot_of("B"), Some(1));
        assert_eq!(slots.monitor_of(1), Some("B"));
    }

    #[test]
    fn a_retired_slot_is_reserved_until_its_renderer_ended() {
        let mut slots = SlotMap::default();
        slots.assign("A");
        slots.assign("B");

        assert_eq!(slots.retire("A"), Some(0));
        assert_eq!(slots.assign("C"), Some(2), "slot 0 still has a renderer");

        assert!(slots.release(0));
        assert!(!slots.release(0), "released once");
        assert_eq!(slots.assign("D"), Some(0));
        assert_eq!(slots.retire("unknown"), None);
    }
}
