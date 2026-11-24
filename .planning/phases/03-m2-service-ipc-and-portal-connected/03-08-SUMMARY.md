---
phase: 03-m2-service-ipc-and-portal-connected
plan: 08
subsystem: portal
tags: [rust, gpui, ipc, portal, service-link, reconnect, banner, single-instance]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: Portal protocol and the service endpoint (plan 06), settings host (plan 07), localized strings (plan 03), synth-library (plan 05)
provides:
  - "sucrose --no-portal: Mode::Service { open_portal }, how a Portal-started service is told not to open another Portal (D-12)"
  - "sucrose_platform::spawn_detached: starts a sibling executable without a console window (Windows CREATE_NO_WINDOW, plain spawn elsewhere)"
  - "apps/sucrose-portal ipc: ServiceLink::{start, send, retry}, LinkEvent, ConnectionState, next_state, backoff, first_events"
  - "apps/sucrose-portal app: PortalApp with connection state, settings view, runtime state and page; on_link_event handles Welcome, Changed, Runtime, AlreadyOpen, Refused, Focus and Close"
  - "apps/sucrose-portal banner: connection banner (connecting, lost, failed with Try Again)"
  - "Portal arguments: only --no-focus, GeneralSetting and SystemSetting; anything else exits 64"
affects: [03-09, 03-10, 03-13, 03-15, 03-16, 03-17, 03-20, 03-21, 03-22, 03-23]
actuals:
  tokens: 10600
  tasks: 2
  commits: 3
plan_head_before: 82b7066e220a9dfe183a311e55de9b696da9cd7b
plan_head_after: 4a32469ccd3ddd2518633799ee590d7bc46d6575
tech-stack:
  added: [async-channel 2.5]
  patterns: [one reader thread and one writer thread per Portal connection with events reaching GPUI through an async channel awaited in cx.spawn_in, pure next_state and backoff functions behind the link, hello before the window so a second Portal exits without a window]
key-files:
  created:
    - apps/sucrose-portal/src/ipc.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/banner.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - apps/sucrose/src/main.rs
    - apps/sucrose-portal/Cargo.toml
    - apps/sucrose-portal/src/main.rs
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - crates/sucrose-platform/src/os_linux.rs
    - crates/sucrose-platform/src/os_macos.rs
    - crates/sucrose-platform/src/unsupported.rs
key-decisions:
  - "The Portal says PortalHello before it opens a window and waits up to 2 s for the service's first word, so a second Portal ends on AlreadyOpen with code 0 and no window flashes; when the service has to be started the window opens at once with the connecting banner"
  - "A connection that broke before Welcome does not reset the reconnect attempts, so a service that dies right after accepting cannot keep the Portal retrying forever"
  - "The hello names a page only when the user asked for one (GeneralSetting, SystemSetting), so a plain launch never changes the page of an open Portal"
requirements-completed: [ARCH-02, UI-03]
coverage:
  - id: D1
    description: "Opening the Portal with no service running starts sucrose --no-portal, connects and registers; the started service opens no other Portal"
    requirement: ARCH-02
    verification:
      - kind: integration
        ref: "Task 1 verify: Service log 'portal registered' and Portal log 'connected to the service' after starting sucrose-portal alone on a synth-library scratch folder; service log 'open_portal=false'"
        status: pass
    human_judgment: false
  - id: D2
    description: "A second Portal exits with code 0 after AlreadyOpen without showing a window"
    requirement: ARCH-02
    verification:
      - kind: integration
        ref: "Task 2 verify: second sucrose-portal exits 0 within 10 s, Portal log 'already open'"
        status: pass
    human_judgment: false
  - id: D3
    description: "A killed service is started again and the Portal registers again; after 3 failed attempts the failed banner with Try Again shows"
    requirement: ARCH-02
    verification:
      - kind: integration
        ref: "Task 2 verify: two 'portal registered' lines after taskkill of the service; Portal copied without a sibling sucrose.exe reaches the failed banner (captured)"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/ipc.rs#connecting_becomes_connected_and_a_lost_connection_counts_its_attempts, a_lost_event_after_the_last_attempt_is_a_failure, try_again_starts_over_and_a_late_connection_still_counts, a_refusal_ends_in_failed_and_other_service_messages_change_nothing"
        status: pass
    human_judgment: false
  - id: D4
    description: "The Portal accepts only --no-focus, GeneralSetting and SystemSetting (both open General); anything else is logged and exits with 64"
    requirement: UI-03
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/main.rs#no_argument_opens_the_library_and_takes_focus, general_and_system_setting_both_open_general, no_focus_keeps_both_flags_in_either_order, anything_else_is_refused"
        status: pass
    human_judgment: false
  - id: D5
    description: "The banner shows Connecting, Lost and Failed (error look and Try Again only for Failed) and its text wraps"
    requirement: UI-03
    verification:
      - kind: automated_ui
        ref: "PrintWindow captures of the connecting and failed banners read by the executor"
        status: pass
    human_judgment: true
    rationale: "Look and wrapping of localized text across 25 languages needs a human eye; only the English connecting and failed states were captured"
  - id: D6
    description: "No I/O or IPC runs on the GPUI thread: one thread reads, one writes, events reach the UI through an async channel awaited in cx.spawn_in (A11)"
    requirement: ARCH-02
    verification:
      - kind: integration
        ref: "Welcome reaches the window (Task 1 capture shows the connected window and the log shows registration); no wake problem seen"
        status: pass
    human_judgment: false
duration: 17min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 08: Service, IPC and Portal connected Summary

**The Portal starts `sucrose --no-portal` when no service runs, says PortalHello over the one endpoint on its own reader and writer threads, shows connecting, lost and failed banners with Try Again, reconnects 3 times, and a second Portal exits with code 0 on AlreadyOpen before any window exists.**

## Performance

- **Duration:** 17 min
- **Started:** 2026-11-24T05:41:40Z
- **Completed:** 2026-11-24T05:58:00Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- The empty GPUI window became `PortalApp`: connection state, the settings view and runtime state from Welcome, Changed and Runtime, and the page it was opened on. Later plans add children to it.
- `ServiceLink` (apps/sucrose-portal/src/ipc.rs) connects, starts the sibling `sucrose --no-portal` with an argument vector when no endpoint answers (T-03-19), retries the connect with 50 ms doubling to 500 ms for 10 s, sends PortalHello, and forwards every PortalEvent with `send_blocking`. A lost connection is retried 3 times 1 s apart, each attempt starting the service again; Try Again starts a new cycle.
- A second Portal ends before a window exists: `first_events` waits up to 2 s for the first event, AlreadyOpen exits with 0 and logs "already open". Focus activates (and so restores) the window, Close quits, Refused ends in the failed state.
- `parse_args` accepts only `--no-focus`, `GeneralSetting` and `SystemSetting` (each at most once); anything else logs and exits with 64 before a window or connection exists (T-03-21).
- The banner uses the Alert component between the title bar and the content; wrapping text, the error look only for the failed state, Try Again button only there.

## Task Commits

1. **Task 1: The Portal starts the service and connects** - `71249fcb` (feat)
2. **Task 2: Banners, reconnect, single-instance exit, strict arguments** - `73294cbd` (test, RED: 5 of 16 tests failed on assertions), `4a32469c` (feat, GREEN: 16 passed)

**Plan metadata:** docs commit follows this summary.

## Files Created/Modified

- `apps/sucrose-portal/src/ipc.rs` - ServiceLink, LinkEvent, ConnectionState, next_state, backoff, first_events, service start
- `apps/sucrose-portal/src/app.rs` - PortalApp root entity and link event handling
- `apps/sucrose-portal/src/banner.rs` - connection banner
- `apps/sucrose-portal/src/main.rs` - parse_args, early hello, window with PortalApp
- `apps/sucrose/src/main.rs` - `--no-portal`, `Mode::Service { open_portal }`
- `crates/sucrose-platform/src/{lib,os_windows,unsupported,os_linux,os_macos}.rs` - `spawn_detached`
- `Cargo.toml`, `Cargo.lock`, `apps/sucrose-portal/Cargo.toml` - async-channel 2.5 and the Portal's new dependencies

## Decisions Made

See key-decisions. The service only logs `open_portal`; plan 15 makes it act on the flag.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The crate-root GPUI glob shadows the built-in test attribute**
- **Found during:** Task 2 (RED tests)
- **Issue:** `use super::*` in the Portal's test module also imports GPUI's `test` macro, which failed with "recursion limit reached while expanding #[test]"
- **Fix:** the test module imports `Args` and `parse_args` explicitly, with a comment saying why
- **Files modified:** apps/sucrose-portal/src/main.rs
- **Committed in:** 73294cbd

**2. [Rule 2 - Missing critical] Reconnect attempts must not reset for a connection that never registered**
- **Found during:** Task 2 (designing the retry cycle)
- **Issue:** resetting the 3 attempts on every reconnect would retry forever against a service that accepts and dies at once
- **Fix:** `Served::Lost { welcomed }`; only a connection that reached Welcome earns fresh attempts
- **Files modified:** apps/sucrose-portal/src/ipc.rs
- **Committed in:** 4a32469c

**3. [Process] TDD RED evidence**
- cargo test output is not a format `gsd_run check tdd-red-evidence` reads, and the plan is not `type: tdd`, so the RED commit is backed by the failing `cargo test -p sucrose-portal` run (5 failed on assertions: parse_args stub, next_state without attempt counting or refusal) rather than a classifier verdict.

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 missing critical), 1 process note
**Impact on plan:** none on scope.

## Issues Encountered

- `LinkEvent::Lost` was never constructed at the Task 1 commit (a dead-code warning); Task 2 constructs it and `cargo clippy -D warnings` is clean.
- The editing tools write LF; the touched files were converted back to CRLF after each `cargo fmt` so the working copy keeps its endings (git normalizes to LF in the index).
- Pre-existing modifications in crates/sucrose-config and crates/sucrose-core (shown by git status from the start) are line-ending artifacts and were not touched or committed.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None. `PortalApp` fields and `ServiceLink::send` are unread until plans 10 to 23 and carry `allow(dead_code)` with the reason.

## Verification Run

- Task 1: Service log shows `portal registered`, Portal log `connected to the service`, capture shows the 900x600 window with the title bar and no banner.
- Task 2: second Portal exit code 0 with "already open"; after `taskkill` of the service a second `portal registered` follows within about 1 s. Banner captures: connecting (info look) and, from a Portal copied away from `sucrose.exe`, failed (error look and Try Again).
- `cargo test -p sucrose-portal` 16 passed, `cargo test -p sucrose` passed, `cargo xtask ci` passed.

## Next Phase Readiness

- Plans 10 onward can add the library grid under the banner and use `PortalApp.link.send`.
- Bringing the first Portal to the front may only flash the taskbar button until plan 15 adds the foreground handoff (A4).

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*

## Self-Check: PASSED

- Files found: ipc.rs, app.rs, banner.rs; commits found: 71249fcb, 73294cbd, 4a32469c.
