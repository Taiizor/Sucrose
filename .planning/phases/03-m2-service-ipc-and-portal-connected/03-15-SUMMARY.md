---
phase: 03-m2-service-ipc-and-portal-connected
plan: 15
subsystem: service
tags: [rust, ipc, launcher, single-instance, lifecycle, foreground, portal]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: Launch and LaunchReply types, the registered Portal, spawn_detached, the tray Option and tray_action (plans 06, 08, 13)
provides:
  - "A second sucrose asks the running service over the endpoint to open or raise the Portal and exits 0 (Launch::OpenPortal, LaunchReply read with a 5 s limit)"
  - "service open_portal(page): starts the fixed sibling sucrose-portal with an argument vector chosen from the page enum; one spawn at a time (10 s window)"
  - "lifecycle.rs: pure start_actions (D-12) and should_exit (D-13), wired into the service start, PortalGone, renderer exits and AppVisible changes"
  - "sucrose_platform::allow_set_foreground(pid) (AllowSetForegroundWindow on Windows, Unsupported elsewhere)"
  - "Foreground handoff for a second sucrose and a second Portal: grant, then Launch::ForegroundGranted, then the service sends Focus to the open Portal"
affects: [03-16, 03-20, 03-24]
actuals:
  tokens: 8100
  tasks: 3
  commits: 4
plan_head_before: c3b92487fcd84f7bfa00e7c2cd7821270ab7f61c
plan_head_after: d7fbf2e1de9eb9a0439e01de3ef67c0d13e8771a
tech-stack:
  added: []
  patterns:
    - "a launcher or a turned-away Portal passes the foreground right on before the service focuses the open Portal"
    - "the connection reader thread asks the main thread for a verdict through a one-shot channel (Launch reply, turned_away) so the main thread stays the only decider"
    - "the exit rule is evaluated once per batch of events, only when a Portal left, a renderer ended or AppVisible changed"
key-files:
  created:
    - apps/sucrose/src/lifecycle.rs
  modified:
    - apps/sucrose/src/main.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/portal_link.rs
    - apps/sucrose/tests/common/mod.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose-portal/src/ipc.rs
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - crates/sucrose-platform/src/os_windows/foreground.rs
    - crates/sucrose-platform/src/unsupported.rs
    - crates/sucrose-platform/src/os_linux.rs
    - crates/sucrose-platform/src/os_macos.rs
key-decisions:
  - "A theme folder that exists counts as selected for the start rule even when the gate rejects it, like the legacy Launcher"
  - "A Portal that was just spawned and has not registered counts as connected for the exit rule for 10 s, so the service does not exit under it"
  - "The registered Portal is focused only after the newcomer passed on the foreground right (PortalRegistry::admit no longer sends Focus)"
requirements-completed: [ARCH-02]
duration: 13 min
completed: 2026-11-24
status: complete
coverage:
  - id: D1
    description: "A second sucrose reaches the running service through the endpoint, the service starts the sibling Portal, and the launcher exits 0"
    requirement: ARCH-02
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_launch_with_a_portal_registered_gets_portal_running"
        status: pass
      - kind: e2e
        ref: "Task 1 automated verify: second sucrose.exe exit 0, 'Sucrose is already running reply=Opened', Portal log 'connected to the service'"
        status: pass
    human_judgment: false
  - id: D2
    description: "The start rule (D-12) and the exit rule (D-13) as pure functions wired into the service"
    requirement: ARCH-02
    verification:
      - kind: unit
        ref: "apps/sucrose/src/lifecycle.rs tests (6)"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_service_with_the_tray_hidden_and_nothing_to_play_exits_when_its_only_portal_leaves"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_service_with_the_tray_visible_stays_when_its_only_portal_leaves"
        status: pass
    human_judgment: false
  - id: D3
    description: "The existing Portal window comes to the front (not only a flashing button) when Sucrose or the Portal is started again"
    requirement: ARCH-02
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_launcher_that_passes_on_the_foreground_gets_the_open_portal_focused"
        status: pass
    human_judgment: true
    rationale: "Windows foreground rules depend on real user input; the end-of-phase check opens the release Portal behind another window and starts Sucrose and the Portal again"
---

# Phase 3 Plan 15: Service, IPC and Portal connected, launcher and lifecycle Summary

**A second sucrose or Portal reuses the running instance through the endpoint and hands it the foreground, and the service starts by the legacy Launcher rule and exits by itself when the tray is hidden and nothing needs it**

## Performance

- **Duration:** 13 min
- **Started:** 2026-11-24T08:33:00Z
- **Completed:** 2026-11-24T08:46:00Z
- **Tasks:** 3
- **Files modified:** 13

## Accomplishments

- The launcher path: AlreadyRunning at bind connects, sends `Launch::OpenPortal`, reads one `LaunchReply` on a helper thread (5 s limit), logs "Sucrose is already running" with the reply and exits 0. `--no-portal` never asks for a Portal.
- The service answers on its main thread: no Portal registered starts the sibling `sucrose-portal` (`Opened`), a registered one gives `PortalRunning { pid }`, a stopping service `Refused`. A spawn in progress is not repeated for 10 s (T-03-35).
- `lifecycle::start_actions` and `should_exit` are pure and tested; the service applies the start rule once at start and evaluates the exit rule after the Portal left, a renderer ended or AppVisible changed, quitting through the normal shutdown path.
- `allow_set_foreground` in `sucrose-platform`. A second sucrose grants the foreground to the open Portal's pid and sends `ForegroundGranted`; a second Portal does the same on AlreadyOpen before it exits; only then does the service send Focus to the open Portal.
- End-to-end on a synthetic library: a second sucrose.exe exited 0 and the Portal it caused logged "connected to the service".

## Task Commits

1. **Task 1: a second sucrose opens the Portal through the running service** - `f1c86384` (feat)
2. **Task 2: legacy Launcher start rule and service exit rule** - `4251a10c` (test, RED), `f12539ff` (feat, GREEN)
3. **Task 3: foreground handoff for a second sucrose and a second Portal** - `d7fbf2e1` (feat)

**Plan metadata:** recorded in the docs(03-15) commit that follows.

## TDD Gate Compliance

Task 2 (tdd="true"): RED `4251a10c` failed on the intended assertions (5 of 6 unit tests on stub return values, the hidden-tray integration test on "it exits"); GREEN `f12539ff` passes all of them. No refactor commit.

## Files Created/Modified

- `apps/sucrose/src/lifecycle.rs` - start_actions, StartActions, should_exit with six tests
- `apps/sucrose/src/service.rs` - Event::Launch and Event::PortalFocus, serve_launcher, read_foreground_grant, open_portal, the start rule, the exit rule, Fleet::wallpaper_wanted
- `apps/sucrose/src/portal_link.rs` - ask_service_to_open_portal (with the grant), spawn_pending, portal_args; admit no longer focuses at once
- `apps/sucrose/src/main.rs` - passes the flag into service::run
- `apps/sucrose-portal/src/ipc.rs` - pass_on_foreground on AlreadyOpen
- `crates/sucrose-platform/src/*` - allow_set_foreground (Windows implementation, Unsupported stubs)
- `apps/sucrose/tests/*` - TestService starts with --no-portal and can wait for the process to exit; new launcher, grant and exit tests

## Decisions Made

- A theme folder that exists counts as selected for D-12 even when the gate rejects it (legacy behavior, the tooltip says why nothing plays).
- A just-spawned Portal counts as connected for the exit rule during its 10 s window.
- Focus of the open Portal moved behind the grant, as the plan asked; the grant is sent even when AllowSetForegroundWindow failed, so the worst case is the old flashing button.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The `portal_wanted` parameter of service::run arrived in Task 1**
- **Found during:** Task 1
- **Issue:** The launcher must not run for a `--no-portal` service (D-12 prohibition), so Task 1 already needed the flag the plan wired in Task 2.
- **Fix:** `service::run(portal_wanted)` added in Task 1.
- **Files modified:** apps/sucrose/src/service.rs, apps/sucrose/src/main.rs
- **Committed in:** f1c86384

**2. [Rule 1 - Bug] Existing portal integration tests broke under D-13**
- **Found during:** Task 2
- **Issue:** Tests that drop a Portal and connect another ended their service, since a hidden-tray service with nothing to play now exits.
- **Fix:** `a_portal_that_left_makes_room_for_the_next` and `a_failed_write_is_rejected...` start with AppVisible true; `quick_toggles_leave_the_state_of_the_last_value` now asserts the service exits after the Portal leaves (only true when the last toggle hid the tray).
- **Files modified:** apps/sucrose/tests/portal.rs
- **Committed in:** f12539ff

**3. [Rule 3 - Blocking] `PortalRegistry::admit` and the Portal-connected event lost their `page` field**
- **Found during:** Task 3
- **Issue:** Focus moved behind the foreground grant, so the hello page had no remaining use; the grant carries the page instead.
- **Fix:** removed the parameter and the event field; tests updated.
- **Files modified:** apps/sucrose/src/portal_link.rs, apps/sucrose/src/service.rs
- **Committed in:** d7fbf2e1

---

**Total deviations:** 3 auto-fixed (1 bug, 2 blocking). **Impact:** none on scope; all follow from the plan's own rules.

## Issues Encountered

- `cargo fmt` rewrites CRLF working copies as LF for some files; line endings were restored after each format run so only real changes show in the commits.
- The Task 1 end-to-end check starts the real Portal without `--no-focus` (the service spawns it with a fixed argument vector, deliberately), so a Portal window opened briefly on the desktop and was closed with taskkill.

## Known Gaps

- The exit rule runs on events only. A Portal the service spawned that never connects (crash at start) leaves a hidden-tray service running until the next event.
- The foreground handoff is verified only up to the messages; whether the window really comes to the front is the end-of-phase human check recorded in the plan (assumption A4).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `open_portal(page)` is ready for the tray's "Open Sucrose" item (plan 16); `should_exit` already follows AppVisible changes.
- The Portal-side Focus already restores and activates the window; plan 20 adds the settings pages that the grant's `page` selects.

## Threat Flags

None beyond the plan's threat model (T-03-34 and T-03-35 are applied: fixed sibling executable, enum-chosen arguments, one spawn at a time).

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*

## Self-Check: PASSED

- Created files found: apps/sucrose/src/lifecycle.rs
- Commits found: f1c86384, 4251a10c, f12539ff, d7fbf2e1
- cargo xtask ci exit 0
