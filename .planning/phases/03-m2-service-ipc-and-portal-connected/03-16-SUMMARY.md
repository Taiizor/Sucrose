---
phase: 03-m2-service-ipc-and-portal-connected
plan: 16
subsystem: service
tags: [rust, tray, lifecycle, close-applications, tray-icon, sysinfo]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: the registered Portal, open_portal, should_exit, PortalEvent::Close, tray_action and start_module (plans 08, 13, 15)
provides:
  - "sucrose_platform::Tray::new(ico, tooltip, labels, on_action) with Open and exit items, a left double-click mapped to Open, set_open_enabled and set_exit_label; TrayAction and TrayLabels"
  - "service Event::Tray: Open starts the Portal (E14 tooltip on failure) or focuses the open one; the Open item is greyed out while a Portal is registered"
  - "Close Applications (AppExit on): renderers stopped, Portal told to close, Portal ended after 2 s only when the OS reports the sibling sucrose-portal executable, then the service exits"
  - "Exit (AppExit off): only the tray goes; a later launch shows it again while AppVisible is on; the exit label follows AppExit live"
  - "lifecycle::{Closing, is_sibling_portal, exit_label}, modules::end_portal, CLOSE_TIMER"
affects: [03-17, 03-20, 03-24]
actuals:
  tokens: 10300
  tasks: 2
  commits: 3
plan_head_before: dfb15c1a85aa5b5a2a117df69d12466ea2181a21
plan_head_after: ec7efbb3f2005cb1ca566781a2774810a4c8692b
tech-stack:
  added: []
  patterns:
    - "process-wide tray-icon and muda handlers installed once and reading the current wiring from a static, so a tray can be dropped and built again"
    - "Close Applications is a pure Closing state (renderers_done, portal_done) filled by events and a bounded timer; the main loop keeps pumping"
    - "a process is ended only after sysinfo confirms its executable path equals the sibling path"
key-files:
  created: []
  modified:
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows/tray.rs
    - crates/sucrose-platform/src/unsupported.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/modules.rs
    - apps/sucrose/src/lifecycle.rs
    - docs/FEATURES.md
key-decisions:
  - "The tray-icon and muda event handlers can be set only once per process (OnceCell/OnceLock), so they are installed once and route through a static wiring; Drop clears the wiring only when it is still its own"
  - "After the 2 s grace the Portal side of Closing counts as done whatever the outcome (ended, already gone, not the Portal executable, kill refused): a process that is not ours is never waited for"
  - "A left click on the icon does nothing and a double-click opens; the menu is on the right button"
requirements-completed: [TRAY-05]
coverage:
  - id: D1
    description: "The tray menu has Open Sucrose (also left double-click) and an exit item labelled by AppExit; the tray logs both labels"
    requirement: TRAY-05
    verification:
      - kind: integration
        ref: "cargo xtask synth-library + sucrose.exe run: log line 'tray icon ready open=Open Sucrose exit=Close Applications'"
        status: pass
    human_judgment: true
    rationale: "The menu, the double-click, the greyed-out Open item and the live relabel need a real tray click, which cannot be automated (no synthesized input)"
  - id: D2
    description: "Close Applications ends the renderers, the Portal and the service in order; Exit removes only the tray"
    requirement: TRAY-05
    verification:
      - kind: unit
        ref: "apps/sucrose/src/lifecycle.rs#closing_* and only_the_sibling_portal_executable_may_be_ended"
        status: pass
    human_judgment: true
    rationale: "The end-to-end close with a real Portal and wallpaper is the plan's human-check at the end of the phase"
duration: 15min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 16: Tray opens Sucrose and Exit closes everything Summary

**D-14 tray menu (Open Sucrose, Close Applications or Exit by AppExit) with a pure Closing state, a sibling-executable check before the Portal is ended, and a rebuild-safe tray event wiring**

## Performance

- **Duration:** 15 min
- **Started:** 2026-11-24T08:48:48Z
- **Completed:** 2026-11-24T09:03:30Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- The Phase 2 Quit-only tray is now the D-14 menu: Open Sucrose (disabled while the Portal is open, also left double-click) and an exit item that reads "Close Applications" with AppExit on and "Exit" with it off, relabelled live when AppExit changes.
- Close Applications stops the renderers (existing 2 s shutdown worker), tells the registered Portal to close, and after about 2 s ends a Portal still there, but only after sysinfo reports its executable as the sibling `sucrose-portal`; the service then exits. With nothing to wait for it exits at once.
- Exit with AppExit off drops only the tray and lets the D-13 rule decide; a later `sucrose` launch recreates the tray while AppVisible is on.
- A failed Open logs the error and sets the tooltip to "Sucrose couldn't open its window" (E14).
- FEATURES.md tray and close-all rows updated.

## Task Commits

1. **Task 1: tray shows Open Sucrose and the AppExit-labelled exit item and drives the service (tracer)** - `7e4fdcdc` (feat)
2. **Task 2: Close Applications and Exit, live label, tray back after a launch**
   - RED: `5f36ad3e` (test) - 7 tests failed on their assertions (stubs returned false)
   - GREEN: `ec7efbb3` (feat)

**Plan metadata:** committed with this summary (docs).

## Files Created/Modified
- `crates/sucrose-platform/src/os_windows/tray.rs` - two-item menu, double-click, set-once handlers over a static wiring, set_open_enabled, set_exit_label
- `crates/sucrose-platform/src/lib.rs` - TrayAction and TrayLabels (shared by the Windows tray and the stub)
- `crates/sucrose-platform/src/unsupported.rs` - stub with the same signatures (checked to compile on Linux in the rust Docker image)
- `apps/sucrose/src/service.rs` - Event::Tray, build_tray, Close Applications flow, CLOSE_TIMER, Exit, relaunch, relabel
- `apps/sucrose/src/modules.rs` - end_portal (sysinfo executable check, then kill)
- `apps/sucrose/src/lifecycle.rs` - Closing, is_sibling_portal, exit_label and the label constants
- `docs/FEATURES.md` - tray and "Close all Sucrose apps on exit" rows

## Decisions Made
See key-decisions above. Labels stay English constants until UI-06.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Tray event handlers could only be set once per process**
- **Found during:** Task 1 (reading tray-icon 0.26.1 and muda 0.21.2)
- **Issue:** both `set_event_handler` functions store into a set-once cell, so the old Drop that "cleared the handler" did nothing and a tray built again (AppVisible switch, relaunch after Exit) would keep the first tray's menu ids and callback.
- **Fix:** handlers are installed once and read a static `Wiring` (menu ids, icon id, callback); `Tray::new` replaces it and Drop clears it only when it is still its own.
- **Files modified:** crates/sucrose-platform/src/os_windows/tray.rs
- **Committed in:** 7e4fdcdc

**2. [Rule 3 - Structure] TrayAction and TrayLabels live in lib.rs**
- The plan listed them as exports of tray.rs; one shared definition in lib.rs serves the Windows tray and the stub without duplication (the acceptance check greps lib.rs for TrayAction). `exit_label` and its test were added with Task 1 (the tray needs the label at creation), so its test was not part of the Task 2 RED.

---

**Total deviations:** 1 auto-fixed bug, 1 structural note
**Impact on plan:** none on behavior.

## Issues Encountered
- A Portal spawned but not yet registered when Close Applications is chosen is not waited for: it connects, is told Close by the admission rule and the service has probably exited; it then sees the closed connection. Narrow window (the 10 s spawn window after Open), not covered by the plan's edges.
- After the 2 s grace the Portal counts as gone right after the kill call (TerminateProcess), without waiting for its connection to close.

## Known Stubs
None.

## Threat Flags
None beyond the plan's threat model (T-03-37 and T-03-38 are mitigated as written: job-object renderer end, executable-checked Portal end, bounded 2 s wait while the loop keeps pumping).

## Human check (deferred to the end of the phase)
With the release build playing a wallpaper and the Portal open (AppExit on): right-click the tray icon, check Open Sucrose is greyed out, choose Close Applications, then list processes with tasklist.exe filtered for sucrose. Expected: wallpaper stops, Portal closes, tray icon gone within about 3 s, no sucrose.exe or sucrose-portal.exe listed. Not run here: tray clicks cannot be automated.

## Next Phase Readiness
The tray menu and Close Applications are in place for the remaining Phase 3 plans. `cargo xtask ci` passes.

## Self-Check: PASSED
- Files exist: all seven modified files; commits 7e4fdcdc, 5f36ad3e and ec7efbb3 are on develop-rust.
- Acceptance: TrayAction in lib.rs (1), set_exit_label in unsupported.rs (1), "Quit" in tray.rs (0), pub struct Closing (1), CLOSE_TIMER in service.rs (4), FEATURES phrase (1).

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
