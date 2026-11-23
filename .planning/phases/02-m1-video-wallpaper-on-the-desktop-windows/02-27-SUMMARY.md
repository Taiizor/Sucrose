---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 27
subsystem: service
tags: [rust, tray-status, renderer-reports, slot-map, fullscreen-rule]
requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: handle_report with the WR-15/WR-18 drop rules, SlotMap retire/release, RuleClosed
provides:
  - handle_report drops a report whose slot no longer belongs to a connected monitor or whose renderer the fullscreen rule closed (WR-19)
  - FEATURES row 'Video' states exactly when a renderer's report is ignored (IN-25)
  - current doc comments of handle_report, Fleet::apply_snapshot and Fleet::apply (IN-28, service part)
affects: [02-28, 02-29, tray status, docs/FEATURES.md]
actuals:
  tokens: 2500
  tasks: 2
  commits: 2
plan_head_before: d3670ec4894a2322929d39caa715118c5e5eef7a
plan_head_after: 4019eb0ea3aec694b4e068974d0a8c2fbd5401c7
tech-stack:
  added: []
  patterns:
    - "Group the facts about a slot's holder into one SlotState argument instead of widening a function past clippy's 7-parameter limit"
key-files:
  created: []
  modified:
    - apps/sucrose/src/service.rs
    - docs/FEATURES.md
key-decisions:
  - "A report of an unassigned slot is dropped whether or not the renderer is still connected (retired monitor or rule-closed renderer), so neither the status nor the master clock changes"
  - "Fleet::slot_assigned uses SlotMap::monitor_of, not active_slots, to avoid building a Vec per report"
requirements-completed: [TRAY-01]
coverage:
  - id: D1
    description: "A late or live report of a renderer whose monitor was removed or whose renderer the fullscreen rule closed no longer reaches the tray status"
    requirement: "TRAY-01"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#a_late_failed_of_a_removed_monitor_is_dropped"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/service.rs#a_report_of_a_renderer_whose_slot_is_not_assigned_is_dropped_even_while_connected"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/service.rs#slot_assigned_follows_the_monitor_and_the_fullscreen_rule"
        status: pass
    human_judgment: false
  - id: D2
    description: "A real failure of a connected monitor's renderer still reaches the tooltip (WR-18 kept)"
    requirement: "TRAY-01"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#a_late_failed_of_an_assigned_slot_still_counts"
        status: pass
    human_judgment: false
  - id: D3
    description: "FEATURES row 'Video' and the service doc comments state the current behavior"
    verification: []
    human_judgment: true
    rationale: "Wording accuracy of documentation is judged by reading; greps only prove the text is present"
duration: 4min
completed: 2026-11-23
status: complete
---

# Phase 02 Plan 27: Report drop for removed or rule-closed monitors Summary

**handle_report now takes a SlotState (link pid, running processes, assigned flag) and drops any report whose slot lost its monitor or was closed by the fullscreen rule, so a late Failed no longer turns the tray tooltip to "no wallpaper" while the other monitors play (WR-19).**

## Performance

- **Duration:** about 4 min
- **Started:** 2026-11-23T18:47:50Z
- **Completed:** 2026-11-23T18:51:59Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- `SlotState<'a>` and `Fleet::slot_assigned` (true while `SlotMap::monitor_of(slot)` is some and the rule has not closed the slot); `handle_report` keeps 6 parameters, no clippy allow.
- The service loop builds the holder from the connection's pid, the supervisor's pids and `fleet.slot_assigned(slot)`; the debug line for a dropped report now logs `assigned`.
- Four new unit tests (removed monitor, unassigned-but-connected, assigned slot still counts, `slot_assigned` across rule close, forget, retire, release); the nine existing handle_report calls run with `assigned: true` through a `holder` test helper and keep their expectations.
- FEATURES row 'Video' lists the exact drop rules (IN-25); doc comments of `handle_report`, `Fleet::apply_snapshot` and `Fleet::apply` match the code (IN-28, service part).

## Task Commits

1. **Task 1: late Failed of a removed monitor's renderer no longer reaches the tray status (tracer)** - `4216f96e` (fix)
2. **Task 2: parity record and doc comments** - `4019eb0e` (docs)

The tracer gate: the exact-name run of the four new tests printed four `... ok` lines and the full `cargo test -p sucrose` (137 passed) and clippy were clean before Task 2 started.

TDD note: RED was the compile failure of the new tests against the old signature (`SlotState` and `slot_assigned` missing), recorded before the implementation; RED and GREEN went into one commit because the plan defines a single task commit.

## Files Created/Modified
- `apps/sucrose/src/service.rs` - SlotState, Fleet::slot_assigned, handle_report with the assigned check, the call site, tests, doc comments
- `docs/FEATURES.md` - row 'Video': when a renderer's report is ignored

## Decisions Made
None beyond the plan: the removed-monitor and rule-closed drop is one `!holder.assigned` term in the existing `replaced` condition.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking, tooling] Line endings kept through the edit tool and rustfmt**
- **Found during:** Task 1
- **Issue:** the REPL file reader returns LF text for the CRLF file, and the first `cargo fmt --all` after my write left service.rs with LF lines (checked with `grep -vc $'\r$'`, 3005).
- **Fix:** converted the file back to CRLF and re-ran `cargo fmt --all` and `cargo fmt --all -- --check` (stable, 0 non-CRLF lines) before every commit; docs/FEATURES.md written with CRLF (197 lines, 0 non-CRLF).
- **Files modified:** apps/sucrose/src/service.rs, docs/FEATURES.md
- **Committed in:** 4216f96e, 4019eb0e

**2. [Plan wording] `grep -c 'fn slot_assigned'` prints 2, not 1**
- **Issue:** the acceptance criterion expects 1, but the mandated test `fn slot_assigned_follows_the_monitor_and_the_fullscreen_rule` also contains the substring. The intent (exactly one `Fleet::slot_assigned` definition) holds: `grep -c 'fn slot_assigned(' ` prints 1.
- **Impact:** none.

---

**Total deviations:** 1 tooling fix, 1 criterion wording mismatch. **Impact on plan:** none on behavior or scope.

## Review Findings
- **Addressed:** WR-19, IN-25, IN-28 (service part; the install_location_watch comment is plan 02-28).
- **Resolved before this round (no task):** IN-24, IN-10.
- **Deferred (reasons from the plan's ledger):**
  - IN-19: order guard no longer limits a failing restore; robustness cost only, bounded by the shell's event rate, fix is in surface.rs outside this round.
  - IN-20: a Quit queued while the surface attaches is not seen until attach ends; the renderer still ends after the 2 s grace, fix is `wait_pumping` in event_loop.rs.
  - IN-21: a stuck renderer link is only logged; needs its own decision on the service-renderer contract (D-22 budget).
  - IN-22: a refused Hello after a shell restart costs a crash strike; robustness cost only.
  - IN-23: `stopped_reason` can name another monitor's failure; needs a per-slot failure memory that changes the global tooltip (D-13, D-22).
  - IN-26: tests duplicate the budget-spending loop; test quality only.
  - IN-27: an outside-window surface is warned twice and on every move; log volume only, fix is in render.rs.
  The review disposition file is not edited by hand; the next `/gsd-code-review` run records these.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Known Stubs
None.

## Threat Flags
None. T-02-54 and T-02-55 are mitigated: unassigned slots are dropped, assigned slots behave as before (tests above).

## Next Phase Readiness
Ready for 02-28. No sucrose.exe was started; G-02-7 and 02-UAT.md are untouched.

## Self-Check: PASSED
- apps/sucrose/src/service.rs and docs/FEATURES.md exist and changed; commits 4216f96e and 4019eb0e are ancestors of HEAD.
- `cargo xtask ci` exit 0; CRLF intact in both files.
