//! Which monitor shows which theme (D-01, D-03, D-08).
//!
//! Pure rules, tested without a display: the service asks [`plan_targets`] for the monitors that get a renderer and the
//! theme each one plays. In Per display the stored `Engine.DisplayAssignments` decide, matched to the connected
//! monitors by their stable id; a migrated legacy file has no such list and the legacy `ScreenDevice` and
//! `Library.Selected` give it once ([`derived_assignments`]). Span and Duplicate keep one theme on every
//! monitor until their own plan builds them.

use std::collections::{BTreeMap, BTreeSet};

use sucrose_config::model::{DisplayAssignment, MAX_DISPLAY_ASSIGNMENTS, Settings};
use sucrose_core::settings::{DisplayScreenType, EngineType};
use sucrose_ipc::portal::{DisplayView, MAX_DISPLAY_TEXT, MAX_DISPLAYS};
use sucrose_platform::MonitorInfo;

use crate::topology::{TopologyAction, diff, replace_all};

/// One monitor that gets a renderer, and the theme (the folder name in the library) it plays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The monitor, its rectangle already the area to cover (`topology::with_area`).
    pub monitor: MonitorInfo,
    /// The folder name of the theme.
    pub theme: String,
}

/// What decides the targets: the display mode and the settings it reads, copied out of the settings file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placement {
    /// `Engine.DisplayScreenType`.
    pub mode: DisplayScreenType,
    /// `Engine.DisplayAssignments`; `None` when the file has no such key.
    pub assignments: Option<Vec<DisplayAssignment>>,
    /// `Library.Selected`: the theme of Span and Duplicate, and of the legacy single wallpaper.
    pub selected: String,
    /// `Engine.ScreenDevice`: where the next Use goes in Per display.
    pub screen_device: String,
}

impl Placement {
    /// The placement the settings describe now.
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            mode: settings.engine.display_screen_type,
            assignments: settings.engine.display_assignments.clone(),
            selected: settings.library.selected.clone(),
            screen_device: settings.engine.screen_device.clone(),
        }
    }

    /// Every monitor plays `selected`: what a fleet that was given one job and no settings does.
    pub fn everywhere(selected: String) -> Self {
        Self {
            mode: DisplayScreenType::SameDuplicate,
            assignments: None,
            selected,
            screen_device: String::new(),
        }
    }

    /// The targets for `monitors`.
    pub fn targets(&self, monitors: &[MonitorInfo]) -> Vec<Target> {
        plan_targets(
            self.mode,
            monitors,
            self.assignments.as_deref(),
            &self.selected,
            &self.screen_device,
        )
    }

    /// The assignments to store once for a legacy file (D-03), if any.
    pub fn derived(&self, monitors: &[MonitorInfo]) -> Option<Vec<DisplayAssignment>> {
        derived_assignments(
            self.mode,
            monitors,
            self.assignments.as_deref(),
            &self.selected,
            &self.screen_device,
        )
    }
}

/// The monitor `ScreenDevice` names, else the first one (the primary, else the lowest device name): the legacy
/// auto-choose. `None` for no monitor.
pub fn target_monitor<'a>(monitors: &'a [MonitorInfo], screen_device: &str) -> Option<&'a MonitorInfo> {
    let named = (!screen_device.is_empty())
        .then(|| {
            monitors
                .iter()
                .filter(|monitor| monitor.id.eq_ignore_ascii_case(screen_device))
                .min_by(|a, b| a.id.cmp(&b.id))
        })
        .flatten();
    named.or_else(|| {
        monitors
            .iter()
            .filter(|monitor| monitor.primary)
            .min_by(|a, b| a.id.cmp(&b.id))
            .or_else(|| monitors.iter().min_by(|a, b| a.id.cmp(&b.id)))
    })
}

/// The monitors that get a renderer and what each plays, in device-name order whatever the order of `monitors`
/// and `assignments`.
///
/// Per display with a stored list: a connected monitor with an entry (matched by stable id, the first entry of an id
/// wins) plays that theme and every other monitor gets nothing. Per display without a list (a legacy file): the target
/// monitor plays `selected` and nothing plays while `selected` is empty. Span and Duplicate: every monitor plays
/// `selected`.
pub fn plan_targets(
    mode: DisplayScreenType,
    monitors: &[MonitorInfo],
    assignments: Option<&[DisplayAssignment]>,
    selected: &str,
    screen_device: &str,
) -> Vec<Target> {
    let by_id = sorted(monitors);
    match (mode, assignments) {
        (DisplayScreenType::PerDisplay, Some(list)) => by_id
            .into_iter()
            .filter_map(|monitor| {
                let entry = list.iter().find(|entry| entry.monitor_id == monitor.stable_id())?;
                Some(Target {
                    monitor: monitor.clone(),
                    theme: entry.theme.clone(),
                })
            })
            .collect(),
        (DisplayScreenType::PerDisplay, None) => {
            if selected.is_empty() {
                return Vec::new();
            }
            target_monitor(monitors, screen_device)
                .map(|monitor| Target {
                    monitor: monitor.clone(),
                    theme: selected.to_owned(),
                })
                .into_iter()
                .collect()
        }
        (DisplayScreenType::SpanAcross | DisplayScreenType::SameDuplicate, _) => {
            if selected.is_empty() {
                return Vec::new();
            }
            by_id
                .into_iter()
                .map(|monitor| Target {
                    monitor: monitor.clone(),
                    theme: selected.to_owned(),
                })
                .collect()
        }
    }
}

/// The assignments a legacy file means (D-03): Per display with no stored list plays `selected` on the target monitor
/// only, and that is what is stored once. `None` when a list is stored already, in another mode, with no theme or
/// with no monitor.
pub fn derived_assignments(
    mode: DisplayScreenType,
    monitors: &[MonitorInfo],
    assignments: Option<&[DisplayAssignment]>,
    selected: &str,
    screen_device: &str,
) -> Option<Vec<DisplayAssignment>> {
    if mode != DisplayScreenType::PerDisplay || assignments.is_some() || selected.is_empty() {
        return None;
    }
    let monitor = target_monitor(monitors, screen_device)?;
    Some(vec![assignment_for(monitor, selected)])
}

/// The assignment that puts `theme` on `monitor`.
pub fn assignment_for(monitor: &MonitorInfo, theme: &str) -> DisplayAssignment {
    DisplayAssignment {
        monitor_id: monitor.stable_id().to_owned(),
        device_name: monitor.id.clone(),
        theme: theme.to_owned(),
    }
}

/// `assignments` with the entry of `entry.monitor_id` replaced, or added at the end; the other entries stay as they
/// were. `None` when the list is full ([`MAX_DISPLAY_ASSIGNMENTS`]) and the monitor has no entry.
pub fn with_assignment(
    assignments: Option<&[DisplayAssignment]>,
    entry: DisplayAssignment,
) -> Option<Vec<DisplayAssignment>> {
    let mut list = assignments.map(<[DisplayAssignment]>::to_vec).unwrap_or_default();
    if let Some(slot) = list.iter_mut().find(|known| known.monitor_id == entry.monitor_id) {
        *slot = entry;
    } else if list.len() < MAX_DISPLAY_ASSIGNMENTS {
        list.push(entry);
    } else {
        return None;
    }
    Some(list)
}

/// `assignments` without the entry of `monitor_id` (Remove Wallpaper, D-07); the other entries stay as they were. `None`
/// when no entry has that id: nothing changes then.
pub fn remove_entry(assignments: Option<&[DisplayAssignment]>, monitor_id: &str) -> Option<Vec<DisplayAssignment>> {
    let list = assignments?;
    let position = list
        .iter()
        .position(|entry| id_matches(&entry.monitor_id, monitor_id))?;
    let mut rest = list.to_vec();
    rest.remove(position);
    Some(rest)
}

/// Whether `requested` names the monitor stored under `full`: the id as it is, or as the Portal got it, cut to
/// [`MAX_DISPLAY_TEXT`] (see [`display_views`]).
pub fn id_matches(full: &str, requested: &str) -> bool {
    full == requested || cut(full) == requested
}

/// The number a monitor shows: the trailing number of its device name (`\\.\DISPLAY9` is 9), else its 1-based
/// `position` from the left. At most 255.
pub fn display_number(device_name: &str, position: usize) -> u8 {
    let digits = device_name.len() - device_name.trim_end_matches(|c: char| c.is_ascii_digit()).len();
    let number = device_name
        .get(device_name.len() - digits..)
        .and_then(|text| text.parse::<usize>().ok())
        .filter(|number| *number > 0)
        .unwrap_or(position);
    u8::try_from(number).unwrap_or(u8::MAX)
}

/// What every monitor shows, for the Portal (D-02, D-07, D-08): the connected `monitors` from left to right (then
/// top to bottom, then by device name, so the enumeration order of the OS changes nothing), each with the theme
/// `targets` place on it and whether a renderer plays there (`playing` maps a device name to its engine); then the
/// stored `assignments` whose monitor is not connected, in their stored order. At most [`MAX_DISPLAYS`].
pub fn display_views(
    monitors: &[MonitorInfo],
    targets: &[Target],
    assignments: &[DisplayAssignment],
    playing: &BTreeMap<String, EngineType>,
) -> Vec<DisplayView> {
    let mut ordered: Vec<&MonitorInfo> = monitors.iter().collect();
    ordered.sort_by(|a, b| (a.rect.left, a.rect.top, &a.id).cmp(&(b.rect.left, b.rect.top, &b.id)));
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    ordered.retain(|monitor| seen.insert(monitor.stable_id()));
    let mut views: Vec<DisplayView> = ordered
        .iter()
        .enumerate()
        .map(|(index, monitor)| {
            let engine = playing.get(&monitor.id).copied();
            DisplayView {
                monitor_id: cut(monitor.stable_id()),
                device_name: cut(&monitor.id),
                number: display_number(&monitor.id, index + 1),
                rect: [
                    monitor.rect.left,
                    monitor.rect.top,
                    monitor.rect.right,
                    monitor.rect.bottom,
                ],
                primary: monitor.primary,
                connected: true,
                theme: targets
                    .iter()
                    .find(|target| target.monitor.id == monitor.id)
                    .map(|target| cut(&target.theme)),
                running: engine.is_some(),
                engine,
            }
        })
        .collect();
    for entry in assignments {
        if !seen.insert(entry.monitor_id.as_str()) {
            continue;
        }
        let position = views.len() + 1;
        views.push(DisplayView {
            monitor_id: cut(&entry.monitor_id),
            device_name: cut(&entry.device_name),
            number: display_number(&entry.device_name, position),
            rect: [0; 4],
            primary: false,
            connected: false,
            theme: Some(cut(&entry.theme)),
            running: false,
            engine: None,
        });
    }
    views.truncate(MAX_DISPLAYS);
    views
}

/// `text` cut to [`MAX_DISPLAY_TEXT`] bytes at a character boundary.
fn cut(text: &str) -> String {
    let mut end = text.len().min(MAX_DISPLAY_TEXT);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.get(..end).unwrap_or_default().to_owned()
}

/// The monitor ids each theme plays on, for the groups that share a clock and a sound (D-04, D-05).
pub fn groups(targets: &[Target]) -> BTreeMap<String, Vec<String>> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for target in targets {
        groups
            .entry(target.theme.clone())
            .or_default()
            .push(target.monitor.id.clone());
    }
    for ids in groups.values_mut() {
        ids.sort();
    }
    groups
}

/// The themes the targets and the stored list name, selected first: every one of them needs its job.
pub fn themes(placement: &Placement) -> BTreeSet<String> {
    let mut themes = BTreeSet::new();
    if !placement.selected.is_empty() {
        themes.insert(placement.selected.clone());
    }
    for entry in placement.assignments.iter().flatten() {
        themes.insert(entry.theme.clone());
    }
    themes
}

/// What to do to go from the renderers on `old` to the ones on `new`: like [`replace_all`] (`every_survivor`) or
/// [`diff`], except that an empty `new` closes every renderer: here "no monitor" is a placement that shows nothing,
/// not the OS in the middle of a hot-plug (that case never reaches this function).
pub fn actions(old: &[MonitorInfo], new: &[MonitorInfo], every_survivor: bool) -> Vec<TopologyAction> {
    if new.is_empty() {
        return sorted(old)
            .into_iter()
            .map(|monitor| TopologyAction::Close(monitor.id.clone()))
            .collect();
    }
    if every_survivor {
        replace_all(old, new)
    } else {
        diff(old, new)
    }
}

fn sorted(monitors: &[MonitorInfo]) -> Vec<&MonitorInfo> {
    let mut sorted: Vec<&MonitorInfo> = monitors.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    sorted.dedup_by(|a, b| a.id == b.id);
    sorted
}

#[cfg(test)]
mod tests {
    use sucrose_platform::RectPx;

    use super::*;

    const D1: &str = "\\\\.\\DISPLAY1";
    const D2: &str = "\\\\.\\DISPLAY2";
    const D9: &str = "\\\\.\\DISPLAY9";

    fn monitor(id: &str, primary: bool) -> MonitorInfo {
        MonitorInfo {
            id: id.to_owned(),
            device_path: Some(format!("PATH#{id}")),
            edid: None,
            friendly: None,
            rect: RectPx {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            },
            work: RectPx {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1040,
            },
            dpi: 96,
            primary,
        }
    }

    fn entry(id: &str, theme: &str) -> DisplayAssignment {
        DisplayAssignment {
            monitor_id: format!("PATH#{id}"),
            device_name: id.to_owned(),
            theme: theme.to_owned(),
        }
    }

    fn pairs(targets: &[Target]) -> Vec<(&str, &str)> {
        targets
            .iter()
            .map(|target| (target.monitor.id.as_str(), target.theme.as_str()))
            .collect()
    }

    fn per_display(
        monitors: &[MonitorInfo],
        list: Option<&[DisplayAssignment]>,
        selected: &str,
        device: &str,
    ) -> Vec<Target> {
        plan_targets(DisplayScreenType::PerDisplay, monitors, list, selected, device)
    }

    #[test]
    fn per_display_gives_each_assigned_monitor_its_own_theme_in_monitor_order() {
        let monitors = [monitor(D1, true), monitor(D9, false)];
        let list = [entry(D9, "b"), entry(D1, "a")];
        let targets = per_display(&monitors, Some(&list), "a", "");
        assert_eq!(pairs(&targets), [(D1, "a"), (D9, "b")]);
    }

    #[test]
    fn a_legacy_per_display_plays_the_selected_theme_on_the_screen_device_only() {
        let monitors = [monitor(D1, true), monitor(D9, false)];
        assert_eq!(pairs(&per_display(&monitors, None, "a", D9)), [(D9, "a")]);
        assert_eq!(
            pairs(&per_display(&monitors, None, "a", "")),
            [(D1, "a")],
            "empty picks the primary"
        );
        assert_eq!(
            pairs(&per_display(&monitors, None, "a", "\\\\.\\DISPLAY7")),
            [(D1, "a")],
            "an unknown device picks the first monitor"
        );
        assert!(
            per_display(&monitors, None, "", D9).is_empty(),
            "no theme, nothing plays"
        );
    }

    #[test]
    fn the_first_monitor_is_the_primary_one_else_the_lowest_device_name() {
        let monitors = [monitor(D9, false), monitor(D2, true), monitor(D1, false)];
        assert_eq!(target_monitor(&monitors, "").map(|m| m.id.as_str()), Some(D2));
        let none_primary = [monitor(D9, false), monitor(D2, false)];
        assert_eq!(target_monitor(&none_primary, "").map(|m| m.id.as_str()), Some(D2));
        assert!(target_monitor(&[], D1).is_none());
    }

    #[test]
    fn nothing_connected_or_nothing_matching_gives_no_target_and_loses_no_assignment() {
        let list = [entry(D9, "b")];
        assert!(per_display(&[], Some(&list), "a", "").is_empty());
        let monitors = [monitor(D1, true)];
        assert!(per_display(&monitors, Some(&list), "a", "").is_empty());
        assert_eq!(list.len(), 1, "the stored entry is not consumed");
        assert!(
            per_display(&monitors, Some(&[]), "a", "").is_empty(),
            "an empty list plays nothing"
        );
        assert!(per_display(&monitors, None, "", "").is_empty());
    }

    #[test]
    fn the_order_of_the_list_and_of_the_monitors_changes_nothing() {
        let list = [entry(D1, "a"), entry(D9, "b"), entry(D2, "c")];
        let reversed = [entry(D2, "c"), entry(D9, "b"), entry(D1, "a")];
        let monitors = [monitor(D1, true), monitor(D2, false), monitor(D9, false)];
        let shuffled = [monitor(D9, false), monitor(D1, true), monitor(D2, false)];
        let expected = per_display(&monitors, Some(&list), "", "");
        assert_eq!(pairs(&expected), [(D1, "a"), (D2, "c"), (D9, "b")]);
        assert_eq!(per_display(&shuffled, Some(&reversed), "", ""), expected);
    }

    #[test]
    fn the_first_entry_of_a_repeated_monitor_id_wins() {
        let list = [entry(D1, "first"), entry(D1, "second")];
        let monitors = [monitor(D1, true)];
        assert_eq!(pairs(&per_display(&monitors, Some(&list), "", "")), [(D1, "first")]);
    }

    #[test]
    fn a_monitor_is_matched_by_its_stable_id_not_its_device_name() {
        let mut moved = monitor(D9, false);
        moved.device_path = Some(format!("PATH#{D1}"));
        let mut other = monitor(D1, true);
        other.device_path = Some("PATH#somewhere-else".to_owned());
        let list = [entry(D1, "a")];
        let targets = per_display(&[moved, other], Some(&list), "", "");
        assert_eq!(pairs(&targets), [(D9, "a")], "the entry follows the device path");
        let mut no_path = monitor(D2, false);
        no_path.device_path = None;
        let by_name = [DisplayAssignment {
            monitor_id: D2.to_owned(),
            device_name: String::new(),
            theme: "n".to_owned(),
        }];
        assert_eq!(pairs(&per_display(&[no_path], Some(&by_name), "", "")), [(D2, "n")]);
    }

    #[test]
    fn span_and_duplicate_put_the_selected_theme_on_every_monitor() {
        let monitors = [monitor(D9, false), monitor(D1, true)];
        for mode in [DisplayScreenType::SpanAcross, DisplayScreenType::SameDuplicate] {
            let targets = plan_targets(mode, &monitors, None, "a", D9);
            assert_eq!(pairs(&targets), [(D1, "a"), (D9, "a")]);
            assert!(plan_targets(mode, &monitors, None, "", D9).is_empty());
        }
    }

    #[test]
    fn duplicate_gives_each_monitor_its_own_renderer_with_the_selected_theme_and_one_monitor_gives_one() {
        let monitors = [monitor(D9, false), monitor(D1, true)];
        let targets = plan_targets(DisplayScreenType::SameDuplicate, &monitors, None, "a", "");
        assert_eq!(
            pairs(&targets),
            [(D1, "a"), (D9, "a")],
            "one target per monitor, in device-name order"
        );
        assert_eq!(
            groups(&targets)["a"],
            [D1, D9],
            "they form one group for the clock and the sound"
        );

        let alone = plan_targets(DisplayScreenType::SameDuplicate, &monitors[1..], None, "a", "");
        assert_eq!(pairs(&alone), [(D1, "a")], "with one monitor Duplicate is Per display");
        assert_eq!(
            pairs(&per_display(&monitors[1..], None, "a", "")),
            pairs(&alone),
            "the same as Per display with the legacy single wallpaper"
        );
        assert!(
            plan_targets(DisplayScreenType::SameDuplicate, &[], None, "a", "").is_empty(),
            "no monitor, nothing to place"
        );
        assert!(
            plan_targets(DisplayScreenType::SameDuplicate, &monitors, None, "", "").is_empty(),
            "an empty selection plays nothing"
        );
    }

    #[test]
    fn a_legacy_per_display_derives_one_assignment_for_the_screen_device() {
        let monitors = [monitor(D1, true), monitor(D9, false)];
        let derived = derived_assignments(DisplayScreenType::PerDisplay, &monitors, None, "a", D9).unwrap();
        assert_eq!(derived, [entry(D9, "a")]);
        assert_eq!(derived[0].monitor_id, "PATH#\\\\.\\DISPLAY9");
        assert_eq!(derived[0].device_name, D9);
        let auto = derived_assignments(DisplayScreenType::PerDisplay, &monitors, None, "a", "").unwrap();
        assert_eq!(auto, [entry(D1, "a")]);
        let stored = [entry(D1, "x")];
        assert_eq!(
            derived_assignments(DisplayScreenType::PerDisplay, &monitors, Some(&stored), "a", D9),
            None
        );
        assert_eq!(
            derived_assignments(DisplayScreenType::PerDisplay, &monitors, None, "", D9),
            None
        );
        assert_eq!(
            derived_assignments(DisplayScreenType::PerDisplay, &[], None, "a", D9),
            None
        );
        assert_eq!(
            derived_assignments(DisplayScreenType::SameDuplicate, &monitors, None, "a", D9),
            None
        );
    }

    #[test]
    fn what_a_legacy_file_derives_plays_the_same_targets_as_the_legacy_settings() {
        let monitors = [monitor(D1, true), monitor(D9, false)];
        let derived = derived_assignments(DisplayScreenType::PerDisplay, &monitors, None, "a", D9).unwrap();
        assert_eq!(
            per_display(&monitors, Some(&derived), "a", D9),
            per_display(&monitors, None, "a", D9)
        );
    }

    #[test]
    fn an_assignment_replaces_the_entry_of_its_monitor_and_keeps_the_others() {
        let list = [entry(D1, "a"), entry(D9, "b")];
        let next = with_assignment(Some(&list), entry(D9, "c")).unwrap();
        assert_eq!(next, [entry(D1, "a"), entry(D9, "c")]);
        let added = with_assignment(Some(&[entry(D1, "a")]), entry(D9, "c")).unwrap();
        assert_eq!(added, [entry(D1, "a"), entry(D9, "c")]);
        assert_eq!(with_assignment(None, entry(D1, "a")).unwrap(), [entry(D1, "a")]);
        let full: Vec<DisplayAssignment> = (0..MAX_DISPLAY_ASSIGNMENTS)
            .map(|n| entry(&format!("M{n}"), "t"))
            .collect();
        assert_eq!(
            with_assignment(Some(&full), entry(D1, "a")),
            None,
            "a full list takes no new monitor"
        );
        assert!(with_assignment(Some(&full), entry("M3", "other")).is_some());
    }

    #[test]
    fn themes_group_their_monitors_and_two_monitors_with_one_theme_form_one_group() {
        let monitors = [monitor(D1, true), monitor(D2, false), monitor(D9, false)];
        let list = [entry(D1, "a"), entry(D9, "a"), entry(D2, "b")];
        let targets = per_display(&monitors, Some(&list), "", "");
        let groups = groups(&targets);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups["a"], [D1, D9]);
        assert_eq!(groups["b"], [D2]);
    }

    #[test]
    fn the_themes_of_a_placement_are_the_selected_one_and_the_assigned_ones() {
        let placement = Placement {
            mode: DisplayScreenType::PerDisplay,
            assignments: Some(vec![entry(D1, "a"), entry(D9, "b")]),
            selected: "c".to_owned(),
            screen_device: String::new(),
        };
        assert_eq!(themes(&placement).into_iter().collect::<Vec<_>>(), ["a", "b", "c"]);
        assert!(themes(&Placement::everywhere(String::new())).is_empty());
    }

    #[test]
    fn an_empty_set_of_targets_closes_every_renderer_and_a_snapshot_never_does() {
        let old = [monitor(D9, false), monitor(D1, true)];
        assert_eq!(
            actions(&old, &[], true),
            [
                TopologyAction::Close(D1.to_owned()),
                TopologyAction::Close(D9.to_owned())
            ]
        );
        assert!(actions(&[], &[], true).is_empty());
        let new = [monitor(D1, true)];
        assert_eq!(actions(&old, &new, false), diff(&old, &new));
        assert_eq!(actions(&old, &new, true), replace_all(&old, &new));
    }

    fn at(id: &str, left: i32, primary: bool) -> MonitorInfo {
        let mut monitor = monitor(id, primary);
        monitor.rect.left = left;
        monitor.rect.right = left + 1920;
        monitor
    }

    fn views(monitors: &[MonitorInfo], list: &[DisplayAssignment], playing: &[(&str, EngineType)]) -> Vec<DisplayView> {
        let targets = per_display(monitors, Some(list), "", "");
        let playing: BTreeMap<String, EngineType> =
            playing.iter().map(|(id, engine)| ((*id).to_owned(), *engine)).collect();
        display_views(monitors, &targets, list, &playing)
    }

    #[test]
    fn a_display_number_is_the_trailing_number_of_the_device_name_else_the_position() {
        assert_eq!(display_number(D9, 1), 9);
        assert_eq!(display_number(D1, 2), 1);
        assert_eq!(display_number("\\\\.\\DISPLAY12", 1), 12);
        assert_eq!(display_number("Monitor", 3), 3, "no number in the name");
        assert_eq!(display_number("", 2), 2);
        assert_eq!(display_number("\\\\.\\DISPLAY0", 4), 4, "zero is no monitor number");
        assert_eq!(
            display_number("DISPLAY99999", 1),
            u8::MAX,
            "too large for a number badge"
        );
    }

    #[test]
    fn the_displays_list_the_monitors_left_to_right_whatever_the_enumeration_order() {
        let list = [entry(D1, "a"), entry(D9, "b")];
        let ordered = [at(D1, 0, true), at(D9, 1920, false)];
        let shuffled = [at(D9, 1920, false), at(D1, 0, true)];
        let views_a = views(&ordered, &list, &[(D1, EngineType::XavierLive)]);
        assert_eq!(views_a, views(&shuffled, &list, &[(D1, EngineType::XavierLive)]));
        assert_eq!(views_a.len(), 2);
        assert_eq!((views_a[0].device_name.as_str(), views_a[0].number), (D1, 1));
        assert_eq!((views_a[1].device_name.as_str(), views_a[1].number), (D9, 9));
        assert_eq!(views_a[0].theme.as_deref(), Some("a"));
        assert_eq!(views_a[1].theme.as_deref(), Some("b"));
        assert!(views_a[0].primary && views_a[0].connected && views_a[0].running);
        assert_eq!(views_a[0].engine, Some(EngineType::XavierLive));
        assert!(
            !views_a[1].running && views_a[1].engine.is_none(),
            "no renderer there yet"
        );
        assert_eq!(views_a[1].rect, [1920, 0, 3840, 1080]);
        assert_eq!(views_a[1].monitor_id, format!("PATH#{D9}"));
    }

    #[test]
    fn an_assignment_of_a_monitor_that_is_not_connected_follows_the_connected_ones() {
        let list = [entry(D2, "gone"), entry(D1, "a"), entry(D2, "again")];
        let monitors = [at(D1, 0, true)];
        let listed = views(&monitors, &list, &[]);
        assert_eq!(listed.len(), 2, "the repeated id is listed once");
        assert!(listed[0].connected);
        let away = &listed[1];
        assert!(!away.connected && !away.running && !away.primary);
        assert_eq!((away.device_name.as_str(), away.theme.as_deref()), (D2, Some("gone")));
        assert_eq!(away.monitor_id, format!("PATH#{D2}"));
        assert_eq!(away.rect, [0; 4]);
        assert_eq!(away.number, 2);
    }

    #[test]
    fn with_no_monitor_connected_every_stored_entry_is_listed_as_not_connected() {
        let list = [entry(D1, "a"), entry(D9, "b")];
        let listed = views(&[], &list, &[]);
        assert_eq!(listed.len(), 2);
        assert!(
            listed
                .iter()
                .all(|view| !view.connected && !view.running && view.theme.is_some())
        );
        assert_eq!(listed[0].device_name, D1);
        assert_eq!(listed[1].device_name, D9);
    }

    #[test]
    fn with_no_entry_every_connected_monitor_is_listed_with_no_theme() {
        let monitors = [at(D1, 0, true), at(D9, 1920, false)];
        let targets = per_display(&monitors, Some(&[]), "", "");
        let listed = display_views(&monitors, &targets, &[], &BTreeMap::new());
        assert_eq!(listed.len(), 2);
        assert!(
            listed
                .iter()
                .all(|view| view.connected && view.theme.is_none() && !view.running)
        );
        assert!(display_views(&[], &[], &[], &BTreeMap::new()).is_empty());
    }

    #[test]
    fn a_name_without_a_number_takes_its_position_from_the_left() {
        let monitors = [at("Right", 1920, false), at("Left", 0, true)];
        let listed = display_views(&monitors, &[], &[], &BTreeMap::new());
        assert_eq!(
            listed
                .iter()
                .map(|view| (view.device_name.as_str(), view.number))
                .collect::<Vec<_>>(),
            [("Left", 1), ("Right", 2)]
        );
    }

    #[test]
    fn the_displays_never_list_more_than_the_largest_runtime_allows() {
        let list: Vec<DisplayAssignment> = (0..MAX_DISPLAY_ASSIGNMENTS)
            .map(|n| entry(&format!("M{n}"), "t"))
            .collect();
        let monitors: Vec<MonitorInfo> = (0..8).map(|n| at(&format!("C{n}"), n * 1920, n == 0)).collect();
        let listed = views(&monitors, &list, &[]);
        assert_eq!(listed.len(), MAX_DISPLAYS);
        let long = "x".repeat(MAX_DISPLAY_TEXT * 2);
        let listed = display_views(&[], &[], &[entry(&long, &long)], &BTreeMap::new());
        assert_eq!(listed[0].monitor_id.len(), MAX_DISPLAY_TEXT, "long text is cut");
        assert!(id_matches(&format!("PATH#{long}"), &listed[0].monitor_id));
    }

    #[test]
    fn removing_an_entry_drops_exactly_that_monitor_and_an_unknown_id_changes_nothing() {
        let list = [entry(D1, "a"), entry(D9, "b")];
        let rest = remove_entry(Some(&list), &format!("PATH#{D9}")).unwrap();
        assert_eq!(rest, [entry(D1, "a")]);
        assert_eq!(remove_entry(Some(&list), "PATH#nowhere"), None);
        assert_eq!(remove_entry(Some(&[]), "x"), None);
        assert_eq!(remove_entry(None, "x"), None);
        let last = remove_entry(Some(&[entry(D1, "a")]), &format!("PATH#{D1}")).unwrap();
        assert!(
            last.is_empty(),
            "the list stays, empty: nothing plays, and the file keeps its meaning"
        );
    }

    #[test]
    fn an_id_cut_for_the_portal_still_names_its_monitor() {
        let long = "p".repeat(MAX_DISPLAY_TEXT + 40);
        assert!(id_matches(&long, &long));
        assert!(id_matches(&long, &long[..MAX_DISPLAY_TEXT]));
        assert!(!id_matches(&long, "other"));
        assert!(!id_matches("short", "shor"));
    }
}
