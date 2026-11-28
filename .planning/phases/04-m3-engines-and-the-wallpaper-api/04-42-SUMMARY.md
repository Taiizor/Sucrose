---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 42
subsystem: service
tags: [portal-protocol, display-modes, per-display, monitors, remove-wallpaper, desk-03]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-41: Engine.DisplayAssignments, placement.rs, one job per theme, Use acting on the target monitor"
provides:
  - "Portal protocol 9: RuntimeState.displays (DisplayView), SettingChange::ScreenDevice with PortalSettings.screen_device, PortalRequest::RemoveWallpaper"
  - "placement::{display_views, display_number, remove_entry, id_matches}"
  - "Service: displays and the target-based selected theme in every runtime state, ScreenDevice that only moves the next Use, Remove Wallpaper for one monitor or the single wallpaper"
affects: [04-43, 04-46, 04-56, DESK-03]
actuals:
  tokens: 12000
  tasks: 2
  commits: 3
plan_head_before: f184f827ad68a3352da24660f0b5da9a6a96115b
plan_head_after: 677431b7558f4067a730cdabd1b223c912c2699d
tech-stack:
  added: []
  patterns: ["the Portal is told the placement as a pure view (display_views) built from the connected snapshot, the planned targets and the stored entries", "Remove Wallpaper edits the assignments and closes the monitor's renderer through the monitor-gone path"]
key-files:
  created: []
  modified:
    - crates/sucrose-ipc/src/portal.rs
    - apps/sucrose/src/placement.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/settings_host.rs
    - apps/sucrose/src/portal_link.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose-portal/src/library/card.rs
    - apps/sucrose-portal/src/settings/other.rs
    - apps/sucrose-portal/src/settings/wallpaper.rs
    - docs/DECISIONS.md
key-decisions:
  - "RuntimeState.selected is the theme on the target monitor in Per display (none when that monitor has no wallpaper), the Library.Selected theme in Span and Duplicate"
  - "Remove Wallpaper(None) in Span and Duplicate clears Library.Selected and closes every renderer; in Per display it is ignored (one monitor at a time)"
  - "Texts in a DisplayView are cut at 128 bytes (MAX_DISPLAY_TEXT); RemoveWallpaper matches the full or the cut id"
  - "The last removed entry leaves an empty list in the file, so nothing plays on restart"
requirements-completed: []
coverage:
  - id: D1
    description: "RuntimeState.displays lists every connected monitor left to right with theme, running flag and engine, then the stored entries of monitors that are not connected"
    requirement: DESK-03
    verification:
      - kind: unit
        ref: "apps/sucrose/src/placement.rs tests (the_displays_list_the_monitors_left_to_right_whatever_the_enumeration_order, an_assignment_of_a_monitor_that_is_not_connected_follows_the_connected_ones, with_no_monitor_connected_..., with_no_entry_...)"
        status: pass
      - kind: integration
        ref: "cargo test -p sucrose --test portal -- --ignored the_target_monitor_gets_the_use_and_remove_frees_it (DISPLAY1 + DISPLAY9)"
        status: pass
    human_judgment: false
  - id: D2
    description: "ScreenDevice moves the target of the next Use without changing a renderer; Use then restarts only that monitor's renderer"
    requirement: DESK-03
    verification:
      - kind: integration
        ref: "the_target_monitor_gets_the_use_and_remove_frees_it: ScreenDevice(DISPLAY9), selected follows, no restart; Use gif-three restarts exactly one renderer"
        status: pass
      - kind: unit
        ref: "settings_host a_screen_device_change_is_stored_shown_and_bounded; service the_selected_theme_is_the_one_on_the_target_monitor_..."
        status: pass
    human_judgment: false
  - id: D3
    description: "Remove Wallpaper frees one monitor (entry gone, renderer ended, others play on) and stops the single wallpaper of Span and Duplicate"
    requirement: DESK-03
    verification:
      - kind: integration
        ref: "the_target_monitor_gets_the_use_and_remove_frees_it: displays show no theme on DISPLAY9, the file keeps only DISPLAY1's entry, DISPLAY1 plays on, the theme folder is untouched"
        status: pass
      - kind: unit
        ref: "service removing_one_monitors_wallpaper_..., removing_the_entry_of_a_monitor_that_is_not_connected_restarts_nothing, remove_all_stops_the_single_wallpaper_..."
        status: pass
    human_judgment: false
  - id: D4
    description: "A monitor that returns gets its wallpaper back (D-08)"
    requirement: DESK-03
    verification:
      - kind: unit
        ref: "service a_monitor_that_goes_keeps_its_entry_and_one_that_returns_gets_its_theme_back (placement of the returning snapshot gives Spawn with its theme and its job exists); the spawn itself is the 04-41 path"
        status: pass
    human_judgment: true
    rationale: "The unit test proves the placement and the Spawn action; a real unplug and replug of DISPLAY9 was not driven in this plan (04-56 records the desktop proof)"
duration: 17min
completed: 2026-11-28
status: complete
---

# Phase 4 Plan 42: Portal-facing per-monitor placement Summary

**The Portal now hears what every monitor shows (RuntimeState.displays, protocol 9), can move the target monitor with Engine.ScreenDevice, and can remove one monitor's wallpaper (or the single wallpaper of Span and Duplicate) with RemoveWallpaper; a monitor that comes back is placed with its stored theme by the 04-41 snapshot path.**

## Performance

- **Duration:** 17 min
- **Started:** 2026-11-28T15:32:00Z
- **Completed:** 2026-11-28T15:49:41Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- `DisplayView` and `RuntimeState.displays` (at most 64, texts cut at 128 bytes): connected monitors left to right (device number from the name, else the position), each with its planned theme, running flag and engine, then the stored entries of monitors that are not connected; published again whenever it changes (it is part of the runtime state the service already diffs).
- `SettingChange::ScreenDevice` is validated (at most MAX_PATH_TEXT bytes, no NUL), stored as `Engine.ScreenDevice`, shown in the settings view and moves only the target of the next Use; the fleet's placement follows at once.
- `PortalRequest::RemoveWallpaper { monitor_id }`: Some(id) drops that entry of `Engine.DisplayAssignments` and closes that monitor's renderer (Per display); an entry of a monitor that is not connected is removed and nothing restarts; None in Span and Duplicate clears `Library.Selected` and closes every renderer. Nothing is deleted from the library.
- `RuntimeState.selected` is the theme on the target monitor in Per display, so the card's Selected pill follows the target (a card test pins it).
- Two-monitor desktop proof on DISPLAY1 + DISPLAY9 passed twice: both renderers placed from the file, ScreenDevice moved the target with no restart, a Use restarted exactly one renderer, Remove Wallpaper ended it and left one entry in the file.

## Task Commits

1. **Task 1: protocol 9, placement views, service handling, integration test** - `71fc651a` (feat)
2. **Task 2: the Portal builds with protocol 9, card rule on the target** - `1bacfe07` (feat)
3. **Amendment to D-042** - `677431b7` (docs)

**Plan metadata:** the commit that adds this SUMMARY and the STATE/ROADMAP update (docs).

## Files Created/Modified

See the front matter; `crates/sucrose-ipc/src/portal.rs` (protocol), `apps/sucrose/src/placement.rs` (pure views), `apps/sucrose/src/service.rs` (Fleet.connected, displays, shown_theme, drop_monitor, handle_remove, tests), `apps/sucrose/tests/portal.rs` (the two-monitor test).

## Decisions Made

See key-decisions; recorded as an amendment of D-042 in docs/DECISIONS.md.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The integration test had to restart the service after writing its themes**
- **Found during:** Task 1 desktop run
- **Issue:** `TestService::start` creates the library, the themes can only be written after it, and the service reads its assignments at start (no renderer started, "the folder of the selected theme does not exist").
- **Fix:** the test writes the three GIF themes, then `service.restart()`.
- **Files modified:** apps/sucrose/tests/portal.rs
- **Commit:** 71fc651a

**2. [Rule 1 - Plan defect] The restart wait raced the Runtime event**
- **Found during:** Task 1 desktop run
- **Issue:** the runtime state with the new theme arrives before the target's renderer ends and starts again, so reading the start counts right then saw one start.
- **Fix:** the test waits for a second engine start (30 s deadline) before checking that exactly one renderer restarted.
- **Commit:** 71fc651a

**3. [Rule 3 - Blocking] The test uses GIF themes written by `write_gif_theme`, not synth-library fixtures**
- **Found during:** Task 1
- **Issue:** the plan names gif-anim, webp-anim and apng-anim from synth-library; the test runs from its own scratch folder and needs only three distinct themes on the built-in GIF renderer, so it writes three GIF themes (gif-anim, gif-two, gif-three) and sets `Engine.Gif = XavierLive` (the default Gif engine is mpv).
- **Files modified:** apps/sucrose/tests/portal.rs

**4. [Rule 2 - Missing critical] Display text bound**
- **Found during:** Task 1 (T-04-94)
- **Issue:** stored monitor ids are not bounded, so 64 views could pass the frame limit.
- **Fix:** `MAX_DISPLAY_TEXT` (128 bytes) cuts the view texts; `id_matches` lets RemoveWallpaper name a monitor by the cut id.
- **Commit:** 71fc651a

---

**Total deviations:** 4 auto-fixed (1 plan defect, 1 missing critical, 2 blocking). **Impact:** none on scope.

## Issues Encountered

None beyond the deviations. No flaky test appeared; `cargo xtask ci` passed (exit 0).

## Known gaps (not stubs)

- A live edit of `Engine.DisplayAssignments` that is not a Use or a Remove (Display Preferences, 04-46) still does not change the theme of a monitor that keeps its renderer (04-41 gap, unchanged).
- The return of a monitor (D-08) is proven at the placement level with a unit test, not by unplugging DISPLAY9.
- `Fleet.connected` holds the rectangles of the last applied snapshot, which are the area the wallpaper covers (work area or display bounds), not always the raw monitor rectangle.
- Removing in Span or Duplicate with a monitor id only removes the stored entry; no renderer changes there.
- TDD: tests were written together with the code, no separate RED run was recorded.

## Line endings

Restored to CRLF before the commit: apps/sucrose/src/service.rs, apps/sucrose/src/settings_host.rs. Left as they are (LF): crates/sucrose-ipc/src/portal.rs, apps/sucrose/src/placement.rs, apps/sucrose/src/portal_link.rs, apps/sucrose/tests/portal.rs, the three Portal files, docs/DECISIONS.md.

## Threat Flags

None: ScreenDevice is bounded (T-04-93) and the displays list is capped and cut (T-04-94).

## Next Phase Readiness

04-43 (Span and Duplicate) can build on `Fleet.placement.mode`; 04-46 (Display Preferences) has everything it needs from the service: `RuntimeState.displays`, `ScreenDevice`, `RemoveWallpaper` and `Use` on the target.

## Self-Check: PASSED

Files exist, the three commits are ancestors of HEAD, `cargo xtask ci` exits 0, the two-monitor test passes and no Rust Sucrose process is left.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-28*
