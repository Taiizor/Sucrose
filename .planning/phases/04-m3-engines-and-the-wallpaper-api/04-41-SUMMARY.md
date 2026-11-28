---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 41
subsystem: service
tags: [display-modes, per-display, monitors, placement, settings, d-042, desk-03]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-40: MonitorInfo::stable_id and the spike verdict (device path stable across replug: yes)"
provides:
  - "Engine.DisplayAssignments (DisplayAssignment { MonitorId, DeviceName, Theme }) with the lenient reader and the 64-entry cap"
  - "apps/sucrose/src/placement.rs: Target, Placement, plan_targets, derived_assignments, target_monitor, with_assignment, groups, themes, actions"
  - "Fleet: a job per theme, one renderer per assigned monitor, a master clock and a sound per theme group, Use acting on the target monitor"
  - "xtask synth-library --display-mode, --screen-device and --assign"
  - "D-042 and the Per display parity row"
affects: [04-42, 04-43, 04-46, 04-56, DESK-03]
actuals:
  tokens: 35000
  tasks: 3
  commits: 3
plan_head_before: c2c9ac2676e0d4293743a068bdce99c2e8d651b4
plan_head_after: 5452e0d6426c0ddbc221756225779e9bf180c4c9
tech-stack:
  added: []
  patterns: ["pure placement rules (plan_targets) feed the fleet; the fleet keeps jobs by theme folder and groups renderers by theme", "the legacy ScreenDevice meaning is derived once and stored as assignments (D-03)"]
key-files:
  created:
    - apps/sucrose/src/placement.rs
  modified:
    - crates/sucrose-config/src/model.rs
    - apps/sucrose/src/main.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/tests/common/mod.rs
    - xtask/src/synth_library.rs
    - docs/DECISIONS.md
    - docs/FEATURES.md
    - docs/ARCHITECTURE.md
key-decisions:
  - "D-042: Engine.DisplayAssignments remembered by the stable monitor id (device path, GDI fallback); no EDID second key because 04-40 said device path stable across replug: yes"
  - "Fleet.job became Fleet.jobs (a map by theme folder) with job() for the selected theme; Fleet.monitors now means the monitors that have a renderer"
  - "Renderers keep groups (slot to theme) and masters (theme to slot): election, clock relay and sound are per theme"
  - "A Use of a renderer that led its old theme demotes it with SetRole(Follower) and the election gives it the lead of the new theme only when that theme has none"
requirements-completed: []
coverage:
  - id: D1
    description: "Two monitors play two different themes from stored DisplayAssignments, one renderer each, a master clock and a sound per theme"
    requirement: DESK-03
    verification:
      - kind: unit
        ref: "apps/sucrose/src/placement.rs tests; service tests each_theme_has_its_own_master_and_a_clock_reaches_only_its_own_theme, a_monitor_without_an_assignment_loses_its_renderer_and_the_others_play_on"
        status: pass
      - kind: integration
        ref: "target/place-lib run: two 'wallpaper placed' lines (gif-anim on DISPLAY1, webp-anim on DISPLAY9), 'renderers started' count=2, two master clocks elected, two BuiltInGif starts"
        status: pass
    human_judgment: false
  - id: D2
    description: "Migrated Per display + ScreenDevice play only on that monitor and the service stores the derived entry"
    requirement: DESK-03
    verification:
      - kind: integration
        ref: "target/place-legacy run: one 'wallpaper placed' line on DISPLAY9, Sucrose.json gained exactly one DisplayAssignments entry for DISPLAY9 with the stable id"
        status: pass
    human_judgment: false
  - id: D3
    description: "Use in Per display acts on the target monitor only"
    requirement: DESK-03
    verification:
      - kind: unit
        ref: "service tests a_use_on_the_target_monitor_loads_only_its_renderer, a_restart_on_the_target_monitor_closes_only_its_renderer, the_renderer_that_led_a_theme_and_moves_to_another_gives_the_clock_up, decide_use per-target tests"
        status: pass
    human_judgment: true
    rationale: "handle_use itself was not driven on the desktop in this plan (no Portal in the run); the Portal side comes in 04-42 and 04-46 and the final desktop proof in 04-56"
duration: 25min
completed: 2026-11-28
status: complete
---

# Phase 4 Plan 41: Per display wallpapers per monitor Summary

**Per display now holds one wallpaper per monitor from Engine.DisplayAssignments (matched by the stable monitor id), with one renderer per assigned monitor, a master clock and a sound per theme, the legacy ScreenDevice meaning kept for migrated settings, and Use acting on the target monitor only.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-11-28T15:00:53Z
- **Completed:** 2026-11-28T15:26Z
- **Tasks:** 3
- **Files modified:** 9 (1 created)

## Accomplishments

- `Engine.DisplayAssignments` is read leniently (non-objects, missing MonitorId or Theme, themes that are not plain folder names and repeated ids are dropped with one warning, 64 entries at most; an absent key stays absent, an empty list stays empty) and written back in order (T-04-90, T-04-91).
- `placement.rs` holds the pure rules; the same list in another order and the same monitors in another enumeration order give the same targets (unit tests), and a monitor is matched by `stable_id`, not by its device name.
- The fleet keeps a job per theme and starts a renderer only for a target monitor. On the maintainer machine (DISPLAY1 + DISPLAY9) the service log shows two `wallpaper placed` lines with their own themes, `renderers started` count=2, two `master clock elected` lines (one per theme) and two built-in GIF renderer starts.
- A migrated Per display file with ScreenDevice DISPLAY9 and no DisplayAssignments plays only on DISPLAY9 and the service stores the single entry (stable id, device name, theme) once (D-03).
- Use in Per display replaces the target monitor's entry and loads, restarts or starts only that monitor's renderer; a renderer that changes theme joins the other theme's group and gives its old clock up (T-04-92).
- D-042, the FEATURES row and the ARCHITECTURE line are written.

## Task Commits

1. **Task 1: Two monitors play two themes from Engine.DisplayAssignments** - `bbc89443` (feat)
2. **Task 2: Migrated Per display keeps its meaning and Use acts on the target monitor** - `ab7b249c` (feat)
3. **Task 3: D-042, FEATURES row, architecture line** - `5452e0d6` (docs)

**Plan metadata:** the commit that adds this SUMMARY and the STATE/ROADMAP update (docs).

## Files Created/Modified

- `apps/sucrose/src/placement.rs` - Target, Placement, plan_targets, derived_assignments, target_monitor, with_assignment, groups, themes, actions (new)
- `crates/sucrose-config/src/model.rs` - DisplayAssignment, MAX_DISPLAY_ASSIGNMENTS, Engine.display_assignments, the reader and its tests
- `apps/sucrose/src/service.rs` - jobs by theme, placement, groups and per-group masters, per-target Use, the legacy derivation at start, the `wallpaper placed` log line
- `apps/sucrose/src/main.rs` - `mod placement`
- `apps/sucrose/tests/common/mod.rs` - the harness plays one theme on every monitor
- `xtask/src/synth_library.rs` - --display-mode, --screen-device, --assign
- `docs/DECISIONS.md`, `docs/FEATURES.md`, `docs/ARCHITECTURE.md`

## Decisions Made

D-042 (see docs/DECISIONS.md). Notable implementation choices: the group key is the theme's folder name; `Fleet::job()` is the selected theme's job and what the tray names; `Fleet::monitors` now holds the monitors that have a renderer; the migrated assignment is only derived when the selected theme passed the gate.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The desktop integration tests assumed Per display plays on every monitor**
- **Found during:** Task 1 (`cargo test -p sucrose`)
- **Issue:** four tests in `apps/sucrose/tests/portal.rs` wait for one renderer per connected monitor; with D-03 the default Per display plays on one monitor only.
- **Fix:** `TestService::start` now writes `Engine.DisplayScreenType = SameDuplicate` as a base setting (tests can override it), so those tests still exercise every monitor's renderer.
- **Files modified:** apps/sucrose/tests/common/mod.rs (not in the plan's file list)
- **Commit:** bbc89443

**2. [Rule 1 - Plan defect] The plan's Task 1 verify command expects BuiltInGif starts but never selects the built-in engine**
- **Found during:** Task 1 desktop run
- **Issue:** the default Gif engine is MpvPlayerLive, so the first run logged `engine="Mpv"` twice and the `BuiltInGif` count was 0.
- **Fix:** both desktop runs (place-lib and place-legacy) were generated with `--engine Gif=XavierLive`; with it the two renderers log `engine started engine="BuiltInGif"`.
- **Files modified:** none
- **Commit:** none

**3. [Rule 2 - Missing critical] Rebuild paths read the one selected job**
- **Found during:** Task 1
- **Issue:** `restart_for_engine`, `restart_wallpapers` and `resume_wallpaper` (library move) rebuilt only the selected theme's job; with a job per theme the other themes would keep old paths or engines.
- **Fix:** they now rebuild and restart every affected job; `other_jobs` builds the jobs of the assigned themes.
- **Files modified:** apps/sucrose/src/service.rs
- **Commit:** bbc89443

**4. [Rule 3 - Blocking] Existing fleet tests adapted to the job map**
- **Found during:** Task 1
- **Issue:** tests read `fleet.job` and `master_slot(current)` and used a theme folder of "" (no folder name, so no target).
- **Fix:** `fleet.job()`, `master_slot(group, current)`, a fixture theme folder "main", and `put_everywhere` before `load_source` or `restart_with` in the three tests that switch theme directly.
- **Files modified:** apps/sucrose/src/service.rs (tests)
- **Commit:** bbc89443

---

**Total deviations:** 4 auto-fixed (1 plan defect, 1 missing critical, 2 blocking). **Impact:** none on scope; the harness base setting is the only change outside the plan's file list.

## Known gaps (not stubs)

- A change of the assignments while the service runs (for example from the Display Preferences page) is not applied to renderers that already run: `apply_snapshot` places new and removed monitors but does not change the theme of a monitor that keeps its renderer. Use does it for the target; the Portal-driven edit comes with 04-42 and 04-46.
- Use in Per display was verified by unit tests of the fleet and of the decisions, not by a desktop Use through the Portal (this plan has no Portal in its runs).
- Span and Duplicate still put one theme on every monitor (04-43).
- TDD: the tests were written together with the code, no separate RED run was recorded.

## Issues Encountered

None beyond the deviations. No flaky test appeared; the first desktop test failures were real and were fixed (deviation 1). No process of this run is left (`tasklist.exe` shows no Rust Sucrose process).

## Line endings

Restored to CRLF before each commit: crates/sucrose-config/src/model.rs, apps/sucrose/src/service.rs, apps/sucrose/src/main.rs, docs/ARCHITECTURE.md and docs/FEATURES.md (the last two keep no final newline). Left as they are: docs/DECISIONS.md, xtask/src/synth_library.rs and apps/sucrose/tests/common/mod.rs (LF), apps/sucrose/src/placement.rs (new, LF).

## Threat Flags

None: the new settings key is validated at the boundary (T-04-90, T-04-91) and the clock relay is per theme (T-04-92, unit test).

## Next Phase Readiness

04-42 can build Display Preferences on `Engine.DisplayAssignments`, `placement::with_assignment` and `Fleet::retarget`; 04-43 builds Span and Duplicate on the placement modes. Reboot stability of the stable id is still unchecked (04-56).

## Self-Check: PASSED

All created and modified files exist, the three task commits are ancestors of HEAD, `cargo xtask ci` exits 0.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-28*
