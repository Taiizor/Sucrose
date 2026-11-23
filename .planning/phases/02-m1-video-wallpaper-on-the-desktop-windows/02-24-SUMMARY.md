---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 24
subsystem: platform
tags: [win32, winevent, progman, desktop-surface, location-watch, tracing, windows-crate]
requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "Placement, place_checked and read_placement (02-23), the 02-16 order guard"
provides:
  - "Location watch: a WinEvent hook on EVENT_OBJECT_LOCATIONCHANGE, limited to the thread that owns the surface's parent, re-places the surface at its stored target when explorer moves or resizes the parent"
  - "Pure location_reaction, outside_episode and record_placement decision functions"
  - "One warning when a surface starts lying outside its desktop window and one info line when it is inside again, from the attach, set_rect and the watch"
  - "docs/FEATURES.md row 'Monitor changes, shell restart recovery (CrashExplorer)' describes the placement check and the watch"
affects: [02-25, 02-26, G-02-7]
actuals:
  tokens: 6800
  tasks: 2
  commits: 2
plan_head_before: 1160f5d569d282484f05e146e43375967d7b0bf3
plan_head_after: 96e7cec51df7c742115898933f210c0a0353d2ed
tech-stack:
  added: []
  patterns:
    - "A second thread-local registry (WATCHED) found by hook handle, short try_borrow, never held while windows move (same shape as GUARDS)"
    - "Decision functions are pure and take lazy inputs (location_reaction asks for the parent's rectangle only after the cheap comparisons agree)"
key-files:
  created: []
  modified:
    - crates/sucrose-platform/src/os_windows/surface.rs
    - docs/FEATURES.md
key-decisions:
  - "The watch is a second hook with its own registry; the 02-16 order guard is unchanged"
  - "The hook is filtered on the process and thread of the parent's root (explorer's desktop thread), so taskbar, File Explorer and cursor location events never wake the renderer"
  - "A Misplaced verdict changes nothing in an outside episode; PartlyHidden and Hidden count as outside"
requirements-completed: [DESK-07, DESK-01]
duration: 7 min
completed: 2026-11-23
status: complete
---

# Phase 02 Plan 24: Location watch Summary

**An event-driven WinEvent location watch re-places a desktop surface at its stored target whenever explorer moves or resizes its parent window, with one warning when an outside episode starts and one line when it ends.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-11-23T17:03:00Z
- **Completed:** 2026-11-23T17:10:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- `install_location_watch` installs SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE) after every successful attach (all three strategies), limited to the thread and process of the parent's root; `Drop` removes it before the window is destroyed.
- `location_watch_proc` finds its entry by hook handle, asks the pure `location_reaction` (OBJID_WINDOW only, window is the parent or its root, parent client rectangle differs from the recorded one) and re-places through `place_checked`, logging "the desktop window moved, surface re-placed" with the previous and new parent rectangle, target, local rectangle and verdict.
- `outside_episode` and `record_placement` keep one outside-episode record per surface; the attach, `set_rect` and the watch all feed it, so a surface outside its desktop window warns once and logs "the surface is inside the desktop window again" once. `set_rect` stores the newest target, so a later parent move re-places there.
- 12 tests with "watch" in their names (6 reaction cases, the real-window `the_watch_re_places_after_its_parent_moved` that drives `location_watch_proc` with hidden windows, 5 episode and record cases). `cargo xtask ci` is green; sucrose-platform has 60 lib tests.
- docs/FEATURES.md row 'Monitor changes, shell restart recovery (CrashExplorer)' now says how a surface is checked and re-placed.

## Task Commits

1. **Task 1: A parent move re-places the surface, end to end from the WinEvent hook to the log** - `f77d9103` (feat, tracer)
2. **Task 2: Outside episodes, set_rect keeps the watch target current, FEATURES row** - `96e7cec5` (feat)

**Plan metadata:** recorded in the docs(02-24) commit that adds this file.

_TDD: the tests were written together with the implementation and committed with it, one commit per task as in 02-23; no separate RED commit. Tracer gate: interactive end-of-phase mode, automated-only verify, re-run green (cargo test watch, clippy, cargo build -p sucrose), so expansion continued._

## Files Created/Modified
- `crates/sucrose-platform/src/os_windows/surface.rs` - Watched registry, install_location_watch, location_reaction, location_watch_proc, follow_parent, remove_watch, Episode, outside_episode, record_placement, note_placement, log_episode, set_rect and attach wiring, 12 tests
- `docs/FEATURES.md` - note in the shell restart recovery row

## Decisions Made
- The attach's own "outside the desktop window" warning moved into the episode record, so the warning is emitted once per episode wherever the placement came from; when the watch could not be installed the attach still warns directly.
- A busy or missing watch entry never fails a move: it is logged at debug and the entry stays as it was.

## Deviations from Plan

None - plan executed exactly as written.

(Environment note, not a deviation: `cargo fmt` rewrites files with LF; CRLF was restored before each commit.)

One observable overlap: the renderer's per-move warn from 02-23 (render.rs, "surface moved" with a PartlyHidden or Hidden verdict) stays as it is, so a move that starts an outside episode produces that line and the episode's one warning; the episode warning is what appears once.

## Issues Encountered
None

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Gap status

Contributes to G-02-7 (renderer side, follow) and does not close it: the plan lists no gap_ids on purpose. Hook delivery cannot be tested in-process (WINEVENT_SKIPOWNPROCESS); the maintainer run in 02-26 checks on the desktop that a parent move produces the "desktop window moved" line.

## Threat Flags

None - no new endpoint, auth path or file access. T-02-48 mitigated (hook limited to one thread and one event, O(1) comparisons, re-place only when the rectangle differs, own surface only); T-02-SC: no crate or windows feature added.

## Next Phase Readiness
- 02-25 can re-place every surface after every display change; the watch and set_rect already keep the stored target current.
- 02-26 maintainer run: look for "the desktop window moved, surface re-placed" during a primary switch and an origin-only change.

## Self-Check: PASSED

- surface.rs and docs/FEATURES.md exist; commits f77d9103 and 96e7cec5 are ancestors of HEAD.
- Acceptance criteria: EVENT_OBJECT_LOCATIONCHANGE in surface.rs 6 (needed >=1), "the desktop window moved" >=1, fn location_reaction 1, fn the_watch_re_places_after_its_parent_moved 1, fn outside_episode 1, fn record_placement 1, "inside the desktop window again" >=1, "explorer moves or resizes" in FEATURES.md 1; 12 watch tests pass (needed 11); cargo xtask ci exit 0.

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-23*
