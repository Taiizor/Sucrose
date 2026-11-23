---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 23
subsystem: platform
tags: [win32, progman, desktop-surface, placement, geometry, tracing, windows-crate]
requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "DesktopSurface, set_rect and the 02-16 order guard in surface.rs"
provides:
  - "RectPx::intersection, Placement, PlacementVerdict and the pure placement_verdict (geometry.rs)"
  - "DesktopSurface::set_rect returning the Placement it reads back (Windows and the unsupported stub)"
  - "Renderer logs: 'surface moved' with target, local rectangle, parent and verdict; warnings for a surface outside the desktop window or off its target"
  - "'surface attached' with parent and verdict, and the same outside-the-window warning at attach"
affects: [02-24, 02-25, 02-26, G-02-7]
actuals:
  tokens: 5300
  tasks: 2
  commits: 2
plan_head_before: dd51e9c28f5f5659d2e2045af7b1a7f811f0b7e5
plan_head_after: 750ae7ff4d8147b80d5cb02e410c5a1bb3c7319b
tech-stack:
  added: []
  patterns:
    - "Read back after every OS move and judge it with a pure function that is unit-tested on every OS"
    - "A non-Shown verdict is data for the caller to log, never an error"
key-files:
  created: []
  modified:
    - crates/sucrose-platform/src/geometry.rs
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/unsupported.rs
    - crates/sucrose-platform/src/os_windows/surface.rs
    - apps/sucrose/src/render.rs
key-decisions:
  - "A PartlyHidden or Hidden verdict never fails an attach or a move: the renderer keeps playing and logs, so a stale explorer does not burn the restart budget"
  - "Misplaced (actual rectangle differs from target) wins over the coverage check"
requirements-completed: [DESK-07, DESK-01]
duration: 5 min
completed: 2026-11-23
status: complete
---

# Phase 02 Plan 23: Checked surface placement Summary

**Every desktop-surface move is read back and judged (Shown, Misplaced, PartlyHidden, Hidden) against its target and the parent's client area; the UAT G-02-7 layout C replay is Hidden in a pure test and with real hidden windows, and the renderer logs parent, local rectangle and verdict at attach and at every move.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-11-23T16:55:00Z
- **Completed:** 2026-11-23T17:00:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Pure `placement_verdict`, `RectPx::intersection`, `Placement` and `PlacementVerdict` in geometry.rs with 8 unit tests (UAT layout C replay parent [-1920,0,1920,1080], target [1920,0,3840,1080] is Hidden; touching edges do not overlap).
- `set_rect` now returns the `Placement` (target, local, parent client area, actual, virtual screen, verdict) via `place_checked` / `read_placement`; the move itself (same origin read, same SetWindowPos flags) is unchanged.
- The renderer logs "surface moved" with target, local, parent and verdict, and warns "the surface lies outside the desktop window" (target, parent, virtual screen) or "the surface did not land on its target"; the attach line carries parent and verdict and warns the same way, never failing the attach.
- `placement_check_against_a_real_parent_window` proves Hidden, Shown and PartlyHidden with real hidden windows (parent [-200,0,200,100]).
- The 02-16 order guard is untouched; macOS and Linux keep compiling through the stub signature.

## Task Commits

1. **Task 1: A surface move is checked against its target and its parent, end to end into the renderer log** - `9fe20ab8` (feat)
2. **Task 2: The attach reports its verdict, and real hidden windows prove the check** - `750ae7ff` (feat)

**Plan metadata:** recorded in the docs(02-23) commit that adds this file.

_TDD: Task 1's RED was observed on assertions (stubbed `placement_verdict` and `intersection`: 6 of 8 tests failed on their assertions, the 2 trivially-true ones passed) and then the implementation made all 8 pass; both ran in one commit per the plan's per-task commit rule. Task 2's real-window test passed on first run because the implementation it exercises (`place_checked`) came from Task 1._

## Files Created/Modified
- `crates/sucrose-platform/src/geometry.rs` - intersection, Placement, PlacementVerdict, placement_verdict, tests
- `crates/sucrose-platform/src/lib.rs` - re-exports Placement and PlacementVerdict
- `crates/sucrose-platform/src/unsupported.rs` - set_rect with the Placement signature
- `crates/sucrose-platform/src/os_windows/surface.rs` - client_rect_on_screen, virtual_screen, read_placement, place_checked, checked set_rect, attach verdict, real-window test
- `apps/sucrose/src/render.rs` - Wake handler logs the Placement

## Decisions Made
- A non-Shown verdict is returned, not an error, so the renderer attaches and keeps playing (as in the plan's design notes).
- `Placement.local` is derived from the parent's client origin read back after the move, which is the same origin `place` used.

## Deviations from Plan

None - plan executed exactly as written.

(Environment note, not a deviation: `cargo fmt` rewrote geometry.rs and surface.rs with LF line endings; CRLF was restored before each commit so the diffs stay minimal and `cargo fmt --check` still passes.)

## Issues Encountered
None

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Gap status

Contributes to G-02-7 (renderer side, check) and does not close it: the plan lists no gap_ids on purpose. The behavior on the real desktop (attach verdict, log lines during a primary switch and an origin-only change) is exercised by the maintainer in 02-26.

## Threat Flags

None - no new endpoint, auth path or file access; the new log fields are rectangles and a verdict (T-02-50 accepted), no crate or windows feature was added (T-02-SC).

## Next Phase Readiness
- 02-24 can follow Progman's location with the `Placement` type and `read_placement` helper in place.
- Verification run: `cargo test -p sucrose-platform` (48 lib tests, 9 placement), `cargo build -p sucrose`, `cargo clippy ... -D warnings` and `cargo xtask ci` all green.

## Self-Check: PASSED

- Files modified exist on disk; commits `9fe20ab8` and `750ae7ff` are ancestors of HEAD.
- Acceptance criteria: placement_verdict 1, intersection 1, place_checked 1, Result<Placement, PlatformError> 1 (surface.rs) and 1 (unsupported.rs), "verdict" in render.rs 2, "outside the desktop window" in render.rs 1 and in surface.rs 1, placement_check_against_a_real_parent_window 1; 9 tests containing "placement" pass (needed 7).

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-23*
