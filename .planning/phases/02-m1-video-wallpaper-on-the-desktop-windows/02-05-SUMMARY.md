---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 05
subsystem: ipc
tags: [interprocess, postcard, named-pipe, single-instance, windows-sid, sddl]
requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "plans 03-04: selected-theme and legacy settings readers logged by the service"
provides:
  - "crates/sucrose-ipc: u32-LE length-prefixed postcard frames (MAX_FRAME 64 KiB), append-only ToService/ToRenderer protocol, Listener::bind with AlreadyRunning, blocking Connection with try_clone"
  - "apps/sucrose service/renderer modes: single instance (D-023), 'sucrose --render <slot>', Hello handshake with pid and protocol-version check"
  - "Platform::user_key and PlatformError; Windows account SID from the process token"
  - "per-user named pipe Sucrose.Service.<SID> with a protected DACL for that SID only"
affects: [02-06, 02-07, 02-09, 02-10, 02-11, 02-12, 02-13, 02-14]
actuals:
  tokens: 21000
  tasks: 3
  commits: 4
plan_head_before: 735b97e5b67a5c50fe0714430d37d89c79d7bb52
plan_head_after: d90d744775b0ccd53b4f9ed5b30e62f8bc389ecf
tech-stack:
  added: [interprocess 2.4.9, postcard 1.2.0 (use-std), crossbeam-channel 0.5.17, windows 0.62.2 (platform crate, Windows only), widestring 1.2.1 (ipc crate, Windows only)]
  patterns:
    - "one blocking reader thread per connection, messages forwarded to the main thread over a crossbeam channel (no async runtime, D-012)"
    - "pure accept_hello decision function separated from the reader thread"
    - "Platform trait method with an Unsupported default so macOS/Linux stubs need no change"
key-files:
  created:
    - crates/sucrose-ipc/Cargo.toml
    - crates/sucrose-ipc/src/lib.rs
    - crates/sucrose-ipc/src/frame.rs
    - crates/sucrose-ipc/src/protocol.rs
    - crates/sucrose-ipc/tests/socket.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - apps/sucrose/Cargo.toml
    - apps/sucrose/src/main.rs
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - docs/ARCHITECTURE.md
key-decisions:
  - "A second bind on a Windows pipe name fails with ErrorKind::PermissionDenied (OS error 5), not AddrInUse (assumption A7 resolved); bind maps both to AlreadyRunning"
  - "Owner SID is validated to letters S, digits and dashes before it is put into the SDDL text"
requirements-completed: [TYPE-01, DESK-01]
duration: 26 min
completed: 2026-11-22
status: complete
coverage:
  - id: D1
    description: "A second sucrose.exe logs that Sucrose is already running and exits with code 0"
    requirement: DESK-01
    verification:
      - kind: e2e
        ref: "Task 1 verify script on the real machine: second instance exit 0 and 'already running' in Service log"
        status: pass
    human_judgment: false
  - id: D2
    description: "The service spawns sucrose --render 0; the renderer sends Hello over the pipe and the service logs 'renderer connected'"
    requirement: TYPE-01
    verification:
      - kind: e2e
        ref: "Task 1 and Task 3 verify scripts: 'renderer connected' in the Service log, 'Hello sent' in Renderer-0 log"
        status: pass
    human_judgment: false
  - id: D3
    description: "Oversized frames are refused before allocation; garbage and refused peers drop only their own connection"
    verification:
      - kind: unit
        ref: "crates/sucrose-ipc/src/frame.rs tests; crates/sucrose-ipc/tests/socket.rs#a_peer_that_sends_garbage_does_not_stop_the_listener"
        status: pass
    human_judgment: false
  - id: D4
    description: "Hello from an unknown pid or another protocol version is refused"
    verification:
      - kind: unit
        ref: "apps/sucrose service::tests::accept_hello_*"
        status: pass
    human_judgment: false
  - id: D5
    description: "The Windows endpoint is a per-user pipe whose DACL grants access only to the current SID"
    verification:
      - kind: integration
        ref: "crates/sucrose-ipc/tests/socket.rs#a_listener_restricted_to_the_current_sid_still_serves_its_owner; Task 3 verify script (renderer connects through the restricted pipe)"
        status: pass
    human_judgment: true
    rationale: "Positive path is tested; that another account is actually denied needs a second Windows account and was not exercised"
---

# Phase 2 Plan 05: Service and renderer channel Summary

**One Sucrose per user: a postcard-over-named-pipe service/renderer channel with 64 KiB frames, a pid-checked Hello, and a pipe named and ACL-restricted by the account SID**

## Performance

- **Duration:** 26 min
- **Started:** 2026-11-22T13:48:16Z
- **Completed:** 2026-11-22T14:14:00Z
- **Tasks:** 3
- **Files modified:** 15

## Accomplishments
- `crates/sucrose-ipc`: frames, append-only protocol, `Listener::bind` (single instance), `connect`, `Connection` with `try_clone`; no unsafe.
- `apps/sucrose` is now the D-008 binary: the service owns the endpoint, spawns `sucrose --render 0`, and logs the renderer's Hello; a second start logs "already running" and exits 0.
- Hardening: frame limit before allocation, garbage and refused peers drop only their own reader thread, `accept_hello` refuses an unknown pid or wrong protocol version.
- `Platform::user_key` (Windows SID via token APIs, every unsafe block carries a SAFETY comment, confined to `sucrose-platform`) names the pipe `Sucrose.Service.<SID>` and drives a protected DACL for that SID only.

## Task Commits

1. **Task 1 (tracer): single instance and Hello over the socket** - `27d57e1d` (feat)
2. **Task 2: channel hardening (TDD)** - RED `840fe5f3` (test), GREEN `2ad56c1d` (feat)
3. **Task 3: per-user endpoint restricted to the SID** - `d90d7447` (feat)

**Plan metadata:** docs commit follows this summary.

## Files Created/Modified
See key-files above. `docs/ARCHITECTURE.md` status line now lists `sucrose-ipc`.

## TDD Notes (Task 2)
- RED: the three `accept_hello` refusal tests failed on assertion against an accept-everything stub (`left: Ok(0)`, `right: Err(UnknownPid(999))` and the protocol-version equivalents); `accept_hello_takes_a_renderer_the_service_started` passed by design.
- The frame and socket tests passed immediately: the tracer already implemented the limit, decode and AlreadyRunning paths, so they are characterization tests that pin the behavior (including A7). They are not RED evidence.
- GREEN: `accept_hello` implemented and wired into the reader path; all tests pass (sucrose-ipc 5 unit + 7 socket on Windows, sucrose 4 accept_hello, sucrose-platform 2).

## Decisions Made
- A7 resolved by test: a second pipe with the same name fails with `PermissionDenied` (OS error 5). `bind` maps `PermissionDenied` and `AddrInUse` to `AlreadyRunning`; this also covers another user squatting the name (T-02-09), which is reported the same way.
- The SID string is checked against `[S0-9-]` before it is formatted into SDDL.
- `accept_hello` takes the spawned-pid set under a lock that is also held across `Command::spawn`, so a fast renderer cannot say Hello before its pid is recorded.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Flaky hang in two socket tests (client gone before accept)**
- **Found during:** Task 2
- **Issue:** a Windows named-pipe client that connects and disconnects before `accept()` runs leaves a dead-on-arrival instance; `accept()` then waits for the next client. Two of my tests did exactly that, hanging about 7% of runs.
- **Fix:** the tests wait for the server to report `accepted` before the client drops. No production change: the service's accept loop just continues with the next renderer. 150 consecutive runs pass.
- **Files modified:** crates/sucrose-ipc/tests/socket.rs
- **Committed in:** 840fe5f3

**2. [Rule 2 - Missing critical] Owner SID validated before SDDL formatting**
- **Found during:** Task 3
- **Issue:** a string put into SDDL text could add ACEs.
- **Fix:** reject anything outside letters S, digits and dashes; test `an_owner_that_is_not_a_sid_is_refused`.
- **Committed in:** d90d7447

**3. Minor, not rules:** clippy `never_loop` in the renderer (one-variant `ToRenderer`) is allowed with a comment; `Refusal` implements `Display` by hand instead of adding `thiserror` to the binary; the verify scripts redirect the background process output so the shell does not wait on it; `sucrose-platform` is a dev-dependency of `sucrose-ipc` for the SID round-trip test; the extra full-duplex and closed-peer tests go beyond the 6 listed behaviors.

---

**Total deviations:** 2 auto-fixed (1 bug in tests, 1 missing critical), 0 architectural
**Impact on plan:** none on scope.

## Issues Encountered
- A first `cargo test` run hung until aborted (the flaky tests above); resolved as described.

## Known Stubs
None. `Close` is the only renderer message and nothing sends it yet by design (plan 09 adds Quit).

## Threat Flags
None beyond the plan's threat model. T-02-09 to T-02-12 are mitigated as planned; same-user processes stay inside the trust boundary (ASVS L1). Not exercised: denial for a second Windows account.

## User Setup Required
None.

## Next Phase Readiness
Plans 06-07 can add `ToRenderer`/`ToService` variants at the end of the enums and reuse `Connection::try_clone` (a send on one handle while another waits in `recv` is covered by a test). `Listener::accept` can block after a client that vanished before it ran; the next connect releases it.

## Self-Check: PASSED
Created files exist, the four commits are ancestors of HEAD, `cargo xtask ci` exits 0.
