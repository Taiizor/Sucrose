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

/// What to do to go from the monitors `old` to the monitors `new`: first the closes, then the rectangle changes,
/// then the spawns, each group ordered by monitor id. An empty `new` gives nothing: the OS reports no monitor for
/// a moment while a monitor is plugged in or out, which must not stop the wallpaper.
pub fn diff(old: &[MonitorInfo], new: &[MonitorInfo]) -> Vec<TopologyAction> {
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
            && (before.rect != now.rect || before.work != now.work || before.dpi != now.dpi)
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

    /// Whether `slot` belongs to a removed monitor whose renderer has not ended yet.
    pub fn is_draining(&self, slot: u8) -> bool {
        self.draining.contains(&slot)
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
        assert!(slots.is_draining(0));
        assert_eq!(slots.assign("C"), Some(2), "slot 0 still has a renderer");

        assert!(slots.release(0));
        assert!(!slots.release(0), "released once");
        assert_eq!(slots.assign("D"), Some(0));
        assert_eq!(slots.retire("unknown"), None);
    }
}
