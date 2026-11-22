---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 10
subsystem: service-supervision
tags: [win32, job-object, supervisor, restart-budget, d-22, crash-recovery]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "plan 09 service channel, LoopProxy::wake, Status/tooltip and the ordered Quit; plan 07 renderer exit codes 2 and 4"
provides:
  - "sucrose_platform::RendererJob::{new, spawn, terminate_all}: kill-on-close job object, renderers start with CREATE_NO_WINDOW (portable fallback that only spawns)"
  - "apps/sucrose supervisor: Supervisor (waiter thread per renderer, Exited events, restart_after), ExitClass, classify_exit, RestartBudget, decide, Exhausted and the exit code constants CLEAN, ENGINE_MISSING, SURFACE_LOST, START_FAILED"
  - "Log lines: renderer spawned pid=.. slot=.., renderer restarted slot=..; service started gains pid as its first field"
affects: [02-12, 02-14]

actuals:
  tokens: 7800
  tasks: 2
  commits: 3

plan_head_before: 3f49a03fe93052f34377ceabfd36fad0dfe0bf8b
plan_head_after: 7eca68e29d266b28e9f3276b238d438c9a5c124e

tech-stack:
  added: []
  patterns:
    - "Blocking waiter thread per child reports the exit over the service channel and wakes the loop; no polling"
    - "Restart decisions are a pure function over an injected clock (decide, RestartBudget::allow(now))"
    - "A renderer is only ever ended through the job object or its own Close, never by pid"

key-files:
  created:
    - crates/sucrose-platform/src/os_windows/job.rs
    - apps/sucrose/src/supervisor.rs
  modified:
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - crates/sucrose-platform/src/os_linux.rs
    - crates/sucrose-platform/src/os_macos.rs
    - crates/sucrose-platform/src/unsupported.rs
    - apps/sucrose/src/main.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - docs/FEATURES.md

key-decisions:
  - "Shutdown kills survivors with RendererJob::terminate_all (TerminateJobObject) instead of per-child kill: the waiter thread owns each Child, and only renderers that ignored Close are left at that point"
  - "A renderer that cannot be started again is fed back as a START_FAILED exit, so it counts against the same crash budget"
  - "A spent budget shows the new Status::RendererStopped (\"Sucrose - no wallpaper on <monitor>: ...\") and leaves the monitor empty until Quit; a restarting Playing status falls back to Starting until the new renderer plays"
  - "Restart budgets live on the service main thread (HashMap per slot), not inside Supervisor"

patterns-established:
  - "Exit codes are defined once in supervisor.rs and re-used by render.rs"

requirements-completed: [DESK-07]

duration: 14 min
completed: 2026-11-22
status: complete
---

# Phase 2 Plan 10: Renderer job object and supervisor Summary

**Kill-on-close job object plus a supervisor that restarts a crashed renderer with 0.5/1/2 s backoff, at most 3 crashes per 60 s per monitor, and reports the reason when it gives up (D-22).**

## Performance

- **Duration:** 14 min
- **Started:** 2026-11-22T15:22:00Z
- **Completed:** 2026-11-22T15:36:00Z
- **Tasks:** 2 (3 code commits: tracer, TDD red, TDD green)
- **Files modified:** 12

## Accomplishments
- Every renderer starts in a `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` job without a console; killing the service with `taskkill /F` left no sucrose.exe behind after 3 s (A6 confirmed).
- Killing the renderer with `taskkill /PID` made the service start a new one after 500 ms and the wallpaper came back ("playback started" twice in the renderer log).
- Four kills in a row on the real machine: restarts after 500 ms, 1 s, 2 s, then `GiveUp(Crashes)` with the tray text "no wallpaper on \\.\DISPLAY1: the renderer stopped after 3 crashes in 60 s".
- 8 supervisor unit tests (classification table, budget, sliding window, surface-lost cap, never-restart classes, give-up reasons) plus 2 job tests and 2 Status tests; `cargo xtask ci` is green.

## Task Commits

1. **Task 1: job object + supervisor (tracer)** - `7e53aced` (feat)
2. **Task 2 RED: failing tests** - `c0aabb71` (test)
3. **Task 2 GREEN: classification and budget, wiring, FEATURES.md** - `7eca68e2` (feat)

**Tracer gate:** the tracer's `<verify>` was run end to end (renderer kill, restart, service kill, no orphan) and passed before Task 2.

## Files Created/Modified
- `crates/sucrose-platform/src/os_windows/job.rs` - RendererJob (create, spawn with CREATE_NO_WINDOW and assign, terminate_all, Drop) with SAFETY comments and two tests
- `apps/sucrose/src/supervisor.rs` - Supervisor, exit classification, RestartBudget, decide
- `apps/sucrose/src/service.rs` - uses the supervisor, handles Exited events, Status::RendererStopped, pid in the "service started" line
- `apps/sucrose/src/render.rs` - exit codes re-use the shared constants
- `docs/FEATURES.md` - Reliability bullet records the bounded restarts

## Decisions Made
See key-decisions above. The RED commit used stubs that return wrong values so every test failed on its assertion (8 of 8), not on a panic.

## Deviations from Plan

**1. [Rule 3 - Blocking] Child ownership changed the shutdown wiring**
- **Found during:** Task 1
- **Issue:** a waiter thread that blocks on `child.wait()` owns the `Child`, so `try_wait`/`kill` through the old `(slot, Child)` list is impossible; pids and the "running" state had to move out of `Renderers`.
- **Fix:** the supervisor tracks pids and running slots behind a mutex and condvar; `wait_exit` waits on the condvar, survivors are ended through the job object; `read_connection` asks the supervisor for the pid set.
- **Files modified:** apps/sucrose/src/service.rs, apps/sucrose/src/supervisor.rs
- **Committed in:** 7e53aced

**2. [Rule 2 - Missing critical] Spawn failures on restart count against the budget**
- **Found during:** Task 1
- **Issue:** a restart that cannot even start would otherwise be dropped silently (or retried forever).
- **Fix:** it is reported as a START_FAILED exit, which the D-22 budget bounds.
- **Committed in:** 7e53aced, 7eca68e2

**Total deviations:** 2 auto-fixed (1 blocking, 1 missing critical). **Impact:** none on scope.

## Issues Encountered
None. Two pre-existing working-tree modifications (player.rs, display.rs) were left untouched and uncommitted.

## Known Stubs
None. `RendererJob` on macOS and Linux only spawns (no renderer runs there until their ports), documented in the doc comment.

## Threat Flags
None beyond the plan's register (T-02-22 and T-02-23 are mitigated and verified live).

## Next Phase Readiness
Plan 12 can exit the renderer with `SURFACE_LOST` (3) and the service restarts it under its own cap of 5 per 60 s.

## Self-Check: PASSED
- job.rs and supervisor.rs exist; commits 7e53aced, c0aabb71, 7eca68e2 are on HEAD; `cargo xtask ci` exits 0; acceptance greps: KILL_ON_JOB_CLOSE 2, SAFETY 7, FEATURES.md line 1.
