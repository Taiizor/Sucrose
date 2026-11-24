---
phase: 03-m2-service-ipc-and-portal-connected
plan: 31
subsystem: service
tags: [rust, service, library-move, gap-closure, wr-01, wr-03, wr-07]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: library move (03-22), Fleet, Status and the topology debounce (plans 03-01..03-26)
provides:
  - "maintenance_decision: a MaintainLibrary during a library move is answered with NOTHING_MAINTAINED at once and starts no library-ops run"
  - "Fleet.pending_snapshot, apply_or_defer and finish_move: a monitor change that settles during a move is kept and applied after the job names the new library"
  - "Fleet::respawn_closed starts nothing while a move runs"
  - "Status::text passes every free-text field through printable"
affects: [03-verification, lib-09]
actuals:
  tokens: 3900
  tasks: 3
  commits: 6
plan_head_before: e7584894330eb51f4515a3e099401c1a9c309cd8
plan_head_after: b0835af0a571f93bd8f6c2f149c985f0cea4f848
tech-stack:
  added: []
  patterns:
    - "A library move quiesces the fleet: maintenance, respawn and topology wait for its end, which catches up in one order (adopt the job, apply the pending snapshot, respawn the closed slots)"
    - "Sanitizing happens in the one function that builds the sentence, not in each constructor"
key-files:
  created: []
  modified:
    - apps/sucrose/src/service.rs
key-decisions:
  - "A snapshot that settles during a move is applied by Fleet::finish_move only when the fleet has a job; with no job the topology is ignored, as it is outside a move"
  - "resume_wallpaper returns Starting also when the pending snapshot started a renderer although the move closed none"
requirements-completed: [LIB-09]
coverage:
  - id: D1
    description: "Maintenance asked for during a library move starts no run and is answered with nothing; a move during maintenance stays refused Busy"
    requirement: LIB-09
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#maintenance_is_answered_with_nothing_during_a_library_move"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/service.rs#maintenance_waits_for_the_run_that_is_going_during_no_library_move"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/service.rs#maintenance_starts_when_nothing_else_touches_the_library_move_free"
        status: pass
    human_judgment: false
  - id: D2
    description: "The fleet starts nothing on the old library during a move and a monitor change that settled meanwhile is applied when the move ends"
    requirement: LIB-09
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#respawn_closed_waits_while_the_library_move_runs"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/service.rs#a_monitor_change_during_a_library_move_is_applied_when_the_move_ends"
        status: pass
      - kind: integration
        ref: "cargo test -p sucrose --test portal library"
        status: pass
    human_judgment: false
  - id: D3
    description: "No text from a theme or the settings reaches the log or the tray tooltip with control characters"
    requirement: LIB-09
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#a_no_wallpaper_reason_with_control_characters_is_printable"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/service.rs#every_free_text_field_of_a_status_is_printable"
        status: pass
    human_judgment: false
duration: 7min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 31: Library move without interference Summary

**A library move now quiesces the service: maintenance is answered with nothing, renderers and monitor changes wait for its end and catch up in order, and every Status sentence is printable.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-11-24T15:05:38Z
- **Completed:** 2026-11-24T15:12:19Z
- **Tasks:** 3
- **Files modified:** 1

## Accomplishments

- WR-01: `maintenance_decision(running, moving)` gates the `MaintainLibrary` arm. During a move the Portal gets `LibraryMaintained(NOTHING_MAINTAINED)` at once and a debug line says why; `MoveEnv::start` already refuses a move during maintenance, so the two are exclusive both ways.
- WR-03: `Fleet::apply_or_defer` keeps a settled snapshot in `pending_snapshot` during a move (latest wins) and drops it while quitting. `Fleet::respawn_closed` returns at once during a move. `resume_wallpaper` now adopts the job, then `Fleet::finish_move` applies the pending snapshot and respawns the closed slots whose monitor is still there.
- WR-07: `Status::text()` passes every free-text field through `printable`, so the log line and the tooltip (built from `text()`) cannot carry a forged line or an escape sequence, also after a move.

## Task Commits

1. **Task 1: maintenance during a move (tracer)** - `cac2a094` (test, RED), `e10cc6e4` (fix, GREEN)
2. **Task 2: pending snapshot and guarded respawn** - `44fdcc17` (test, RED), `b6fa8dc0` (fix, GREEN)
3. **Task 3: printable status text** - `9f094034` (test, RED), `b0835af0` (fix, GREEN)

Tracer verified end-to-end (`cargo test -p sucrose --bin sucrose library_move`) before the expansion tasks. The RED commits fail on the planned assertions (stubs for Tasks 1 and 2), not on compile errors.

**Plan metadata:** the docs commit that adds this summary.

## Files Created/Modified

- `apps/sucrose/src/service.rs` - MaintenanceDecision and maintenance_decision, Fleet.pending_snapshot, apply_or_defer, finish_move, the respawn_closed guard, the reworked resume_wallpaper, Status::text sanitizing, and eight new tests

## Decisions Made

- The pending snapshot is applied only when the fleet has a job after the move. With no job (the selected theme is gone) the topology is ignored, exactly as the timer arm already does outside a move.
- `resume_wallpaper` reports `Starting` when the pending snapshot started a renderer even though the move closed none (for example the service waited for a monitor), so the tray does not keep the no-monitor text.

## Deviations from Plan

None - plan executed exactly as written. Two small additions inside the plan's scope: a third test (`a_monitor_change_that_settles_while_quitting_is_dropped`) for the quitting rule the truth names, and `the_monitor_wait_is_still_recognized_after_printable_text` for the NO_MONITOR behavior line.

**Total deviations:** 0
**Impact on plan:** none.

## Issues Encountered

- A test helper detail: `sent_to` leaves its own sentinel Close in the log, so an earlier call changes what a later one returns. The "nothing was told during the move" check reads the log directly instead.

## Verification

- `cargo test -p sucrose --bin sucrose library_move`, `cargo test -p sucrose --test portal library`, `cargo test -p sucrose` (256 + 26 passed), `cargo clippy -p sucrose --all-targets -- -D warnings` and `cargo xtask ci` all pass; the existing move tests still pass.
- Acceptance greps: `maintenance_decision` defined once, `pending_snapshot` appears 6 times, the WR-07 test name appears once.
- Not run: a live move with a real renderer fleet; the behavior is covered at unit level only.

## Known Stubs

None.

## Threat Flags

None - no new surface; T-03-70, T-03-71 and T-03-72 are mitigated as planned.

## Next Phase Readiness

Ready for 03-32 (WR-02, requests while quitting).

## Self-Check: PASSED

- FOUND: apps/sucrose/src/service.rs
- FOUND commits: cac2a094, e10cc6e4, 44fdcc17, b6fa8dc0, 9f094034, b0835af0
