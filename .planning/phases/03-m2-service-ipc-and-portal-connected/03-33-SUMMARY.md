---
phase: 03-m2-service-ipc-and-portal-connected
plan: 33
subsystem: ipc
tags: [rust, ipc, named-pipe, portal, gap-closure, wr-04, win32]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: Portal registry, PortalLink writer and serve_portal reader (plans 03-06, 03-32)
provides:
  - "Connection::disconnect (sucrose-ipc): ends a connection for every handle of it; a reader blocked in recv on another handle returns and the client's next recv fails. Other OSes return IpcError::Unsupported"
  - "disconnect_pipe (sucrose-platform, Windows): DisconnectNamedPipe in one unsafe block with a SAFETY comment"
  - "PortalLink::with_closer/close; PortalRegistry::send ends the connection of a Portal it gives up for a full queue"
  - "portal_gone_effect: every ended Portal connection re-checks the exit rule, only the registered one is told to the closing sequence"
affects: [03-verification, arch-02]
actuals:
  tokens: 4738
  tasks: 2
  commits: 4
plan_head_before: 92172af225d6b4e9964af7d44e1f7006805561f5
plan_head_after: 4bda19184966c9f1581b8ef33de8bbe29e33d789
tech-stack:
  added: []
  patterns:
    - "A link carries an optional closer built from one more clone of its connection, so the owner can end the connection without the reader thread's cooperation"
key-files:
  created: []
  modified:
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-platform/src/os_windows.rs
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-ipc/src/lib.rs
    - crates/sucrose-ipc/tests/socket.rs
    - apps/sucrose/src/portal_link.rs
    - apps/sucrose/src/service.rs
key-decisions:
  - "disconnect lives on Connection as a method that matches the Windows named-pipe variant of interprocess's Stream; no unsafe in sucrose-ipc (D-011)"
  - "IpcError gained an Unsupported variant for OSes without a disconnect, rather than faking success"
  - "Ipc tests for disconnect sit in crates/sucrose-ipc/tests/socket.rs next to the other real-socket tests, not in the lib.rs test module"
requirements-completed: [ARCH-02]
duration: 13 min
completed: 2026-11-24
status: complete
coverage:
  - id: D1
    description: "Connection::disconnect wakes a reader blocked on another handle and the client sees the connection end"
    requirement: "ARCH-02"
    verification:
      - kind: integration
        ref: "crates/sucrose-ipc/tests/socket.rs#a_disconnected_connection_wakes_a_blocked_reader"
        status: pass
      - kind: integration
        ref: "crates/sucrose-ipc/tests/socket.rs#disconnecting_twice_or_after_the_peer_left_is_harmless"
        status: pass
    human_judgment: false
  - id: D2
    description: "A Portal dropped for a full queue is disconnected once; with no Portal registered nothing is disconnected; the slot is free for one new Portal"
    requirement: "ARCH-02"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/portal_link.rs#a_portal_dropped_for_a_full_queue_is_disconnected"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/portal_link.rs#sending_with_no_portal_disconnects_nothing"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/portal_link.rs#a_dropped_portal_frees_the_slot_for_one_new_portal"
        status: pass
    human_judgment: false
  - id: D3
    description: "Any Portal connection that ends sets check_exit, so a service kept alive only by a dropped Portal exits by the D-13 rule"
    requirement: "ARCH-02"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#a_portal_that_leaves_always_rechecks_the_exit_rule"
        status: pass
    human_judgment: true
    rationale: "The unit test proves the decision helper; the end-to-end case (a stuck real Portal dropped by the service, then the service exiting) was not run against a live Portal and the main-loop wiring is covered only by the 26 existing portal.rs integration tests staying green"
---

# Phase 03 Plan 33: Disconnect a dropped Portal Summary

**Connection::disconnect on a DisconnectNamedPipe platform helper, so PortalRegistry::send ends the connection of a Portal it gives up on and every ended Portal connection re-checks the service exit rule (WR-04)**

## Performance

- **Duration:** 13 min
- **Started:** 2026-11-24T16:52:00Z
- **Completed:** 2026-11-24T17:06:00Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- `disconnect_pipe` in sucrose-platform wraps `DisconnectNamedPipe` in one unsafe block with a SAFETY comment; `Win32_System_Pipes` added to the `windows` features.
- `Connection::disconnect` ends the connection for every handle of it. Proven with a real server/client pair: a thread blocked in `recv` on a clone returns an error and the client's next `recv` fails. On macOS and Linux it returns `IpcError::Unsupported`.
- `PortalLink` keeps a closer built from one more `try_clone` of its connection. `PortalRegistry::send` calls it on `Offer::Failed` before dropping the registration, so the Portal sees its connection end and reconnects, and the service reader thread for it ends.
- The `Event::PortalGone` arm sets `check_exit` for every connection that ends and tells the closing sequence only when the connection was the registered one (`portal_gone_effect`).

## Task Commits

1. **Task 1 (tracer, TDD): disconnect wakes a blocked reader**
   - RED: `c8ee4f00` (test) - the target test failed on the Unsupported stub, not on a build error
   - GREEN: `9d74b6ac` (feat)
2. **Task 2 (TDD): drop a Portal for a full queue disconnects it**
   - RED: `cccbf62d` (test) - two target tests failed on their assertions
   - GREEN: `4bda1918` (fix)

**Plan metadata:** the docs commit that follows this file.

## Files Created/Modified

- `crates/sucrose-platform/src/os_windows.rs`, `src/lib.rs`, `Cargo.toml` - `disconnect_pipe` and its Windows-only re-export
- `crates/sucrose-ipc/src/lib.rs` - `Connection::disconnect`, `IpcError::Unsupported`
- `crates/sucrose-ipc/tests/socket.rs` - the two disconnect tests
- `apps/sucrose/src/portal_link.rs` - closer on `PortalLink`, `PortalRegistry::send` closes a dropped link, tests
- `apps/sucrose/src/service.rs` - `portal_gone_effect`, the `PortalGone` arm, test

## Decisions Made

See `key-decisions` above. The tracer feedback gate: the tracer's `<verify>` is automated only and passed (`cargo test -p sucrose-ipc disconnect`, `cargo test -p sucrose-platform`, clippy), so execution expanded to Task 2.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Disconnect tests placed in tests/socket.rs, and a stub for a valid RED**
- **Found during:** Task 1
- **Issue:** The plan says "sucrose-ipc's test module", but the crate's real-socket tests with the unique-endpoint helper live in `tests/socket.rs`. A RED commit that does not compile is an INVALID_RED, so the RED commit carries an `Unsupported` stub of `disconnect`.
- **Fix:** Tests added next to the other real-socket tests; the stub made the target test fail on its `disconnect().expect`.
- **Files modified:** crates/sucrose-ipc/tests/socket.rs
- **Committed in:** c8ee4f00

**2. [Rule 2 - Missing] Added `IpcError::Unsupported`**
- **Found during:** Task 1
- **Issue:** The plan needs an `IpcError` that says disconnecting is not supported on other OSes; no such variant existed.
- **Fix:** New variant; no exhaustive match on `IpcError` elsewhere needed a change.
- **Commit:** c8ee4f00

**3. [Rule 2 - Missing] Extra test a_dropped_portal_frees_the_slot_for_one_new_portal**
- **Found during:** Task 2
- **Issue:** The ARCH-02 adjacency truth (a dropped Portal and a new one never both registered) had no test among the plan's three.
- **Fix:** Added it.
- **Commit:** cccbf62d

---

**Total deviations:** 3 auto-fixed (1 blocking, 2 missing)
**Impact on plan:** None on scope.

## Issues Encountered

`cargo fmt` on the Windows toolchain rewrote the CRLF working-tree files as LF; the CRLF files touched here were converted back before each commit, so the line endings stay as they were. `git status` keeps listing `crates/sucrose-ipc/tests/socket.rs` as modified with an empty diff: a line-ending artifact, not committed.

## User Setup Required

None - no external service configuration required.

## Threat Flags

None. The one new unsafe call is the planned T-03-76 mitigation (sucrose-platform only, borrowed handle, tested with a real pipe pair).

## Next Phase Readiness

WR-04 is fixed in code. A live check (a stuck Portal dropped by the service, the Portal reconnecting) was not run; `cargo xtask ci` (fmt, clippy, tests) passes.

## Self-Check: PASSED
