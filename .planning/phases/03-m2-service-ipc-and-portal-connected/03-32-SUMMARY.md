---
phase: 03-m2-service-ipc-and-portal-connected
plan: 32
subsystem: service
tags: [rust, service, shutdown, library-move, gap-closure, wr-02]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: library move, Closing lifecycle, Portal requests (plans 03-16, 03-22, 03-31)
provides:
  - "acts_while_quitting and quitting_answer: once the service closes, only RefreshStartup is acted on; MoveLibrary is answered Refused(Busy), the other state-changing requests are dropped at debug level"
  - "may_exit and settle_exit: the Quit path and Close Applications wait for a running library move before Flow::Exit"
  - "resume_after_move: a move that ends while quitting answers the Portal but starts no renderer"
affects: [03-verification, tray-05, arch-05]
actuals:
  tokens: 2200
  tasks: 2
  commits: 2
plan_head_before: a5d9a38b08e994e9267dc44ba53dde494685334a
plan_head_after: 6422970856568e5de71c95ae1a531b2056b18273
tech-stack:
  added: []
  patterns:
    - "Small pure decision functions (acts_while_quitting, may_exit, resume_after_move) tested in isolation, called from the main loop"
key-files:
  created: []
  modified:
    - apps/sucrose/src/service.rs
key-decisions:
  - "Quit path no longer returns Flow::Exit from the ShutdownDone event; it records quit_done and the exit check at the end of the loop pass decides, so a move that ended in the same pass lets the service exit at once"
  - "A move that ends while quitting also drops fleet.pending_snapshot, so no monitor change is kept for a restart that never comes"
requirements-completed: [TRAY-05, ARCH-05]
duration: 4 min
completed: 2026-11-24
status: complete
coverage:
  - id: D1
    description: "A Portal request that changes state is not acted on while the service is quitting (WR-02): MoveLibrary refused Busy, Change/Use/SetStartup/MaintainLibrary dropped, RefreshStartup still answered"
    requirement: "ARCH-05"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#only_refresh_startup_acts_while_quitting"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/service.rs#a_move_asked_for_while_quitting_is_refused_busy"
        status: pass
    human_judgment: false
  - id: D2
    description: "The service never exits while a library move runs, and the end of that move starts no renderer while quitting"
    requirement: "TRAY-05"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#quitting_waits_for_the_library_move_before_exit"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/service.rs#a_move_that_ends_while_quitting_starts_no_renderer"
        status: pass
    human_judgment: false
  - id: D3
    description: "SC3: Close Applications closes every Sucrose process end to end (TRAY-05)"
    requirement: "TRAY-05"
    verification: []
    human_judgment: true
    rationale: "The tray click and the real process teardown are the 03-VERIFICATION behavior_unverified_items run by /gsd-verify-work; the loop decisions are unit-tested here but the end-to-end run needs a human"
---

# Phase 3 Plan 32: Quitting is final for Portal requests and never cuts a library move Summary

**Once the service closes, a Portal can no longer start a move, select a wallpaper or change a setting, and both exit paths (Quit, Close Applications) wait for a running library move instead of ending the process in the middle of it (WR-02).**

## Performance

- **Duration:** 4 min
- **Started:** 2026-11-24T15:43:32Z
- **Completed:** 2026-11-24T15:47:00Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- `Event::Portal` checks `quitting` right after the registration check: `acts_while_quitting` lets only `RefreshStartup` through; a `MoveLibrary` gets `MoveResult(Refused(Busy))`, `Change`, `Use`, `SetStartup` and `MaintainLibrary` are logged at debug and dropped (no request content logged). `Library.Selected` is therefore never written and no renderer is spawned after Close Applications or Quit (ARCH-05).
- `may_exit(done, moving)` guards every `Flow::Exit` for quitting: the Quit path (`ShutdownDone` with no `closing`, now `quit_done`), the `closing.done()` check of the loop pass and the CLOSE_TIMER arm. The check runs after `settle_move`, so the service exits in the same pass the move ends. The hold is logged once at info.
- `MoveEnv` carries `quitting`; `MoveEnv::end` still writes the result and answers the Portal but, via `resume_after_move`, skips `resume_wallpaper` and drops any pending monitor snapshot while quitting.
- Close Applications with no Portal and no move exits as before; the 03-16 lifecycle tests pass unchanged.

## Task Commits

1. **Task 1: Portal requests are not acted on once the service closes** - `2af1b190` (feat; tests written first and seen to fail on the planned assertions with stub functions, then implemented)
2. **Task 2: The service waits for a running library move before it exits** - `64229708` (feat; same flow)

**Plan metadata:** committed with this SUMMARY (docs: complete plan)

## Files Created/Modified

- `apps/sucrose/src/service.rs` - `acts_while_quitting`, `quitting_answer`, `may_exit`, `settle_exit`, `resume_after_move`, the gate in `Event::Portal`, `quit_done`/`exit_held` in the loop, `MoveEnv.quitting`, four unit tests

## Decisions Made

- The Quit path exits through the common end-of-pass check instead of returning from the `ShutdownDone` arm, so one rule (`may_exit`) governs every quitting exit.
- A move ending while quitting discards `pending_snapshot` (nothing restarts, so keeping it would only be dead state).

## Deviations from Plan

None - plan executed exactly as written. (The plan's `resume_after_move(quitting) -> bool` example name was used as given; `settle_exit` is a small extra helper that only adds the once-only hold log.)

## Issues Encountered

None. Line numbers in the plan had shifted after 03-31; the code was located by name.

## Verification

- `cargo test -p sucrose --bin sucrose quitting`: 5 passed (the four new tests and the existing monitor-change one).
- `cargo test -p sucrose`: 263 + 26 passed; `cargo clippy -p sucrose --all-targets -- -D warnings` clean; `cargo xtask ci` (fmt, clippy, tests) completed without error.
- `grep -c 'fn may_exit'` = 1, `grep -c 'fn acts_while_quitting'` = 1.
- 03-VERIFICATION.md still holds the SC3 item once (`grep -c` = 1); this plan did not touch it.

**SC3 stays human-pending and is not claimed here.** The end-to-end Close Applications run (03-VERIFICATION behavior_unverified_items "SC3: Exit / Close Applications closes every Sucrose process (TRAY-05)", the 03-16 tray check) is run by /gsd-verify-work after this round; it exercises the quitting gate and the exit hold changed here.

## Known Stubs

None.

## Threat Flags

None - the change only narrows what the service acts on while closing (T-03-73, T-03-74 mitigated); no new surface.

## Next Phase Readiness

WR-02 is fixed. Remaining Phase 3 work continues with the other gap-closure plans; SC3 end-to-end verification is pending for /gsd-verify-work.

## Self-Check: PASSED

- apps/sucrose/src/service.rs: FOUND
- commits 2af1b190, 64229708: FOUND in HEAD ancestry
