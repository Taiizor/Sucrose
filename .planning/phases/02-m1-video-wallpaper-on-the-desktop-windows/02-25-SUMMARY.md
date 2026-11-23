---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 25
subsystem: service
tags: [topology, display-change, set-rect, fleet, legacy-parity, g-02-7]
requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "Checked surface placement (02-23) and the location watch (02-24); Fleet::apply_snapshot, move_to and the SetRect message from earlier plans"
provides:
  - "topology::replace_all: the actions after a display change, a SetRect for every monitor that stays plus the closes and spawns of diff"
  - "Fleet::apply_snapshot re-places every surface after each applied display change, also when no monitor changed; the master election runs only when monitors changed"
  - "Fleet-level tests with recording renderer links that prove SetRect reaches the renderer of an unchanged monitor"
  - "docs/FEATURES.md row 'Monitor changes, shell restart recovery (CrashExplorer)' states the re-place-all rule"
affects: [02-26, G-02-7]
actuals:
  tokens: 4600
  tasks: 2
  commits: 2
plan_head_before: d17c41e647192549bf9db6fdd3c317808151d6e0
plan_head_after: b68cbd5bee76794f65c2ea5540ee1f7476aafb20
coverage:
  - id: D1
    description: "replace_all gives a SetRect for every surviving monitor (origin shift, primary switch, unchanged layout), closes first and spawns last, nothing for an empty snapshot"
    requirement: DESK-07
    verification:
      - kind: unit
        ref: "apps/sucrose/src/topology.rs#replace_all_* (7 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "apply_snapshot sends SetRect to the renderer of an unchanged monitor and of a moved one, updates both assignments, and re-places every renderer of an unchanged snapshot without any role message"
    requirement: DESK-07
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#replace_all_an_origin_shift_reaches_the_unchanged_monitors_renderer"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/service.rs#replace_all_an_unchanged_snapshot_re_places_every_renderer"
        status: pass
    human_judgment: false
  - id: D3
    description: "FEATURES row 'Monitor changes, shell restart recovery (CrashExplorer)' says every surface is re-placed after each applied display change"
    requirement: DESK-07
    verification:
      - kind: other
        ref: "grep -c 'every surface is re-placed' docs/FEATURES.md (1); grep -c 'per-monitor diffs' docs/FEATURES.md (0)"
        status: pass
    human_judgment: false
  - id: D4
    description: "On a real desktop, a primary switch and an origin-only change put every monitor's surface back on its monitor"
    requirement: DESK-07
    verification: []
    human_judgment: true
    rationale: "Where a surface shows depends on where explorer keeps its desktop window; only the maintainer's desktop exercises that (02-26)"
tech-stack:
  added: []
  patterns:
    - "diff and replace_all share one private walk (actions_between) so their order and id rules cannot drift"
    - "Fleet tests use recording Links (a writer thread pushing into a shared Vec) and a Close sentinel through the same queue to know every earlier message was written"
key-files:
  created: []
  modified:
    - apps/sucrose/src/topology.rs
    - apps/sucrose/src/service.rs
    - docs/FEATURES.md
key-decisions:
  - "replace_all is a separate function and diff stays as it is, because the initial start uses diff(&[], monitors) and must send spawns only"
  - "The master election runs only when diff reports a change: an unchanged snapshot cannot change the master, so role messages stay exactly as before"
requirements-completed: [DESK-07]
duration: 5 min
completed: 2026-11-23
status: complete
---

# Phase 02 Plan 25: Re-place every surface after a display change Summary

**After every applied display change the service sends SetRect with the current rectangle to every renderer that stays (legacy parity), through a pure topology::replace_all, so an origin-only change or a primary switch no longer leaves an unchanged monitor's surface dragged off its monitor.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-11-23T17:12:00Z
- **Completed:** 2026-11-23T17:17:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- `topology::replace_all(old, new)`: closes, then a SetRect for every monitor in both snapshots (changed or not), then spawns, each in id order; an empty `new` gives nothing. `diff` is unchanged apart from sharing the walk.
- `Fleet::apply_snapshot` carries out `replace_all` and no longer returns early for an unchanged snapshot; it logs "re-placing every surface" (changed, surfaces) and "applying a monitor change" only when monitors changed. `move_to` logs "re-placing its surface" with a `changed` field. `elect` runs only when monitors changed.
- 9 tests with `replace_all` in their names: 7 pure ones in topology.rs and 2 Fleet tests in service.rs over recording links (origin shift reaches the unchanged monitor and updates both assignments; an unchanged snapshot re-places both renderers, sends no SetRole and runs no election). The tracer test was seen failing (empty message list for the unchanged monitor) with the old `diff`-driven apply before it passed.
- FEATURES row updated. `cargo test -p sucrose` (133 passed), clippy and `cargo xtask ci` are green.

## Task Commits

1. **Task 1: An origin shift re-places the renderer of an unchanged monitor (tracer)** - `d19efd89` (feat)
2. **Task 2: Every applied display change re-places every surface; FEATURES row** - `b68cbd5b` (feat)

**Plan metadata:** the docs commit that carries this file.

## Files Created/Modified
- `apps/sucrose/src/topology.rs` - `replace_all`, shared `actions_between`, 7 tests
- `apps/sucrose/src/service.rs` - `apply_snapshot` and `move_to` changes, recording-fleet test helpers, 2 Fleet tests
- `docs/FEATURES.md` - the re-place-all sentence in the monitor-change row

## Decisions Made
- `diff` stays for the initial start; `replace_all` is used for display changes only.
- Role messages are unchanged: no election for an unchanged snapshot.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] clippy type_complexity on the test helper**
- **Found during:** Task 1
- **Issue:** the recording helper's return type tripped `clippy::type_complexity` under `-D warnings`
- **Fix:** added a `SentLogs` type alias in the tests module
- **Files modified:** apps/sucrose/src/service.rs
- **Committed in:** d19efd89 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** None on behavior. The tracer's tests were written together with the implementation; RED was confirmed afterwards by temporarily restoring the old `diff`-driven apply (the Fleet test failed), then reverting.

## Issues Encountered
None. Line endings of the three files were kept CRLF (an intermediate edit helper briefly wrote LF; restored before the first commit).

## Gap Status
This plan contributes to G-02-7 (service side) and does not close it: the plan lists no gap_ids by design. G-02-7 stays open in 02-UAT.md until the 02-26 decision is carried out.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
Ready for 02-26: the maintainer exercises a primary switch and an origin-only change on the desktop; the logs now carry "re-placing every surface" and the per-surface "re-placing its surface" lines (with `changed`) from the service, plus the Placement verdicts of 02-23.

## Known Stubs
None.

## Threat Flags
None. T-02-51 is mitigated as planned: one SetRect per connected renderer per applied change, bounded by the existing 2000 ms + 500 ms debounce.

## Self-Check: PASSED
- Files found: topology.rs, service.rs, FEATURES.md
- Commits found: d19efd89, b68cbd5b
- Acceptance greps: `pub fn replace_all` 1, `replace_all(` in service.rs >= 1, `fn replace_all_` in topology.rs 7, `re-placing every surface` 1, `every surface is re-placed` 1, `per-monitor diffs` 0

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-23*
