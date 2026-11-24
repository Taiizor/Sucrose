---
phase: 03-m2-service-ipc-and-portal-connected
plan: 13
subsystem: service
tags: [rust, modules, tray, memory, sysinfo, ipc, d-008, d-19]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: PortalRegistry and PortalEvent::Memory with its droppable writer, SettingChange::AppVisible, the settings host, publish_runtime (plans 06, 07, 09)
provides:
  - "start_module(name, enabled, create): the gate every optional service module goes through; the constructor is never called when off, and each module logs one line module started, module off or module failed with its name"
  - "Live tray toggle: tray_action(current, wanted) gives Create, Drop or Keep; Notification Area Icon in the Portal builds or removes the tray at once, one change at a time on the main thread so quick toggles end in the last value"
  - "MemorySampler and MemoryModule: while a Portal is registered a 1 s timer (MEMORY_TIMER = 4) samples the working set of the service, each renderer (in slot order) and the Portal, and sends PortalEvent::Memory; with no Portal there is no timer and no sysinfo System"
  - "The fullscreen rule starts through start_module (off with Resume or with no wallpaper)"
affects: [03-16, 03-18, 03-20]
actuals:
  tokens: 6400
  tasks: 2
  commits: 3
plan_head_before: febf3ed27fd5572d2d5a63adb5f071ad52402b3c
plan_head_after: f3c6835ca42efbb99388a794ae9227f73887aaff
tech-stack:
  added: ["sysinfo (already a workspace dependency, now also used by apps/sucrose; no new crate)"]
  patterns: ["a module is an Option that exists only while wanted; the pure decision (tray_action, MemoryModule::follow) is unit-tested and the main thread only applies it", "Wake handler re-derives module state from the registry at the end of each batch, so a Portal dropped by a failed send also stops the memory module"]
key-files:
  created:
    - apps/sucrose/src/modules.rs
  modified:
    - apps/sucrose/Cargo.toml
    - Cargo.lock
    - apps/sucrose/src/main.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/portal_link.rs
    - apps/sucrose/tests/portal.rs
key-decisions:
  - "MemoryModule remembers whether a Portal was connected when it last changed, so a sampler that fails to open is logged once per Portal instead of on every loop message"
  - "The memory module logs module off once at service start, so all three modules (tray, fullscreen, memory) log one line at start as the plan requires"
  - "A tray that cannot be created still logs the old hint that Sucrose can only be stopped from Task Manager, through create_tray, in addition to module failed"
requirements-completed: [ARCH-03]
coverage:
  - id: D1
    description: "An optional module that is off starts nothing: no tray with AppVisible false, no fullscreen timer with Resume, no memory timer or sampler without a Portal; each logs module off with its name"
    requirement: ARCH-03
    verification:
      - kind: unit
        ref: "apps/sucrose/src/modules.rs#with_app_visible_false_no_tray_is_built"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/modules.rs#the_memory_module_exists_only_while_a_portal_is_connected"
        status: pass
      - kind: integration
        ref: "service run on a cargo xtask synth-library folder: Service log has module off for tray, memory and fullscreen and no module started"
        status: pass
    human_judgment: false
  - id: D2
    description: "The tray follows Notification Area Icon live and quick toggles end in the last value"
    requirement: ARCH-03
    verification:
      - kind: unit
        ref: "apps/sucrose/src/modules.rs#the_tray_is_created_when_wanted_and_absent"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#the_tray_follows_the_notification_area_switch_live"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#quick_toggles_leave_the_state_of_the_last_value"
        status: pass
    human_judgment: false
  - id: D3
    description: "A connected Portal receives the memory of the service, its renderers and itself about once a second; only those pids are read"
    requirement: ARCH-03
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#memory_reports_arrive_while_connected"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/modules.rs#the_sampler_reads_this_process_and_reports_a_missing_one_as_zero"
        status: pass
    human_judgment: false
duration: 17min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 13: Service, IPC and Portal connected, modules Summary

**Optional service modules go through one gate (start_module), the tray appears and disappears live with the Portal's Notification Area Icon switch, and a 1 s memory report of the service, renderers and Portal runs only while a Portal is connected.**

## Performance

- **Duration:** 17 min
- **Started:** 2026-11-24T07:47:00Z
- **Completed:** 2026-11-24T08:04:00Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- `apps/sucrose/src/modules.rs` holds `start_module`, `module_off`, `tray_action`, `MemorySampler` and `MemoryModule`; `start_tray` and its tests moved there.
- Registration of a Portal arms `MEMORY_TIMER`; `Timer(MEMORY_TIMER)` samples the service pid, `supervisor.pids()` (slot order) and the registered Portal pid and sends `PortalEvent::Memory`, which the writer drops when the Portal is slow (D-19). The timer is killed and the sampler dropped when the Portal is gone or the service quits.
- `SettingChange::AppVisible` accepted by the settings host builds or drops the tray on the main thread; the Runtime event then carries `tray_visible`.
- Process-level check on a synthetic folder: the Service log has `module off` for tray, memory and fullscreen and no `module started` line before a Portal connects.

## Task Commits

1. **Task 1: memory of all Sucrose processes, only while connected (tracer)** - `a36bf971` (feat)
2. **Task 2: live tray toggle, fullscreen through the helper (TDD)** - RED `c72bb36d` (test), GREEN `f3c6835c` (feat)

## TDD Gate Compliance (Task 2)

- RED: `the_tray_is_created_when_wanted_and_absent` and `the_tray_is_dropped_when_present_and_no_longer_wanted` failed on their assertions against a stub that always returns Keep; `the_tray_follows_the_notification_area_switch_live` failed with "no Runtime with tray_visible true" because the service did not create the tray. `quick_toggles_leave_the_state_of_the_last_value` passed in RED (the tray never existed), so it guards the behavior rather than proving the change.
- GREEN: all pass; `cargo xtask ci` exit 0.

## Files Created/Modified

- `apps/sucrose/src/modules.rs` - module gate, tray decision, memory sampler and module
- `apps/sucrose/src/service.rs` - tray through start_module and create_tray, live toggle, memory timer wiring, start_fullscreen
- `apps/sucrose/src/portal_link.rs` - `PortalRegistry::registered_pid`
- `apps/sucrose/src/main.rs`, `apps/sucrose/Cargo.toml`, `Cargo.lock` - module declaration and the existing sysinfo dependency
- `apps/sucrose/tests/portal.rs` - memory, live tray and quick toggle integration tests

## Decisions Made

See key-decisions above; the plan was otherwise followed as specified.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Cargo.lock updated**
- **Found during:** Task 1
- **Issue:** Adding `sysinfo` to `apps/sucrose/Cargo.toml` changes the lock file, which the plan's file list does not name.
- **Fix:** Committed the one-line `Cargo.lock` change with Task 1.
- **Commit:** a36bf971

**2. [Rule 1 - Bug] Test helper loop could never time out**
- **Found during:** Task 2 RED run
- **Issue:** a first draft of `wait_for_tray` looped on `next_event`, whose 5 s timeout restarts with every Memory event (one per second), so a failing test hung.
- **Fix:** an overall 5 s deadline in `wait_for_tray` and in the quick toggle loop.
- **Commit:** c72bb36d

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug). **Impact:** none on scope.

## Issues Encountered

- `the_tray_follows_the_notification_area_switch_live` creates a real tray icon for a moment on the desktop it runs on and removes it again before the test ends, so no ghost icon is left. It needs an interactive Windows session (it is skipped where `service_runs` is false).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The Portal status bar (plan 18) can show `PortalEvent::Memory`; the FEATURES.md notes for D-19 and the tray module are written by plans 18 and 16.
- The pre-existing dirty working-tree files (line-ending artifacts in crates/ and some apps/sucrose sources) were left alone.

## Self-Check: PASSED

- Files exist: apps/sucrose/src/modules.rs; commits a36bf971, c72bb36d, f3c6835c are ancestors of HEAD.
- `grep -c 'pub fn start_module'` = 1, `fn start_tray` in service.rs = 0, `MEMORY_TIMER` in service.rs = 4, `fn tray_action` = 1, `cargo test -p sucrose modules` 9 passed.
- `cargo xtask ci` exit 0; clippy -D warnings clean.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
