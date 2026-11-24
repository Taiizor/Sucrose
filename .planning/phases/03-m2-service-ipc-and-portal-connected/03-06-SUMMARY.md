---
phase: 03-m2-service-ipc-and-portal-connected
plan: 06
subsystem: ipc
tags: [rust, ipc, portal, postcard, named-pipe, single-instance, d-033]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: Settings model and Store (plan 01), Warehouse.hint_tray_icon and Library.delete_corrupt (plan 04), synthetic library and prepared settings convention (plan 05)
provides:
  - "sucrose_ipc::portal: the whole Portal protocol (PortalRequest, PortalEvent, SettingChange, PortalSettings, RuntimeState, Launch, LaunchReply, outcomes and refusals), append-only, PORTAL_PROTOCOL_VERSION 1, MAX_PATH_TEXT 4096"
  - "ToService::PortalHello (index 4) and ToService::Launch (index 5); PROTOCOL_VERSION stays 3"
  - "sucrose_ipc::{Endpoint, service_endpoint, service_endpoint_for, endpoint_name_for}: endpoint named per user and, with SUCROSE_DATA_DIR, per data folder (FNV-1a hash)"
  - "apps/sucrose portal_link: PortalLink (bounded ordered writer), PortalRegistry (one Portal on the main thread), accept_portal, portal_view, runtime_state"
  - "apps/sucrose/tests/common: TestService and PortalClient, the harness that drives the real sucrose binary on a scratch folder"
  - "D-033 in docs/DECISIONS.md"
affects: [03-07, 03-08, 03-09, 03-15, 03-21, 03-22, 03-23]
actuals:
  tokens: 21577
  tasks: 2
  commits: 3
plan_head_before: 86268b848e9e12bad85c29046705b6db7e77ae9f
plan_head_after: 520baff8155a55d259be2bbd2892842cccb11ce9
tech-stack:
  added: []
  patterns: [first message routes the peer kind on one endpoint, registry decided on the main thread, bounded writer per peer where only readings are droppable, scratch-folder service started by the integration tests, connection id instead of a claimed pid]
key-files:
  created:
    - crates/sucrose-ipc/src/portal.rs
    - apps/sucrose/src/portal_link.rs
    - apps/sucrose/tests/common/mod.rs
    - apps/sucrose/tests/portal.rs
  modified:
    - crates/sucrose-ipc/Cargo.toml
    - crates/sucrose-ipc/src/lib.rs
    - crates/sucrose-ipc/src/protocol.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/src/main.rs
    - docs/DECISIONS.md
key-decisions:
  - "D-033: one endpoint for service, renderers and Portal, routed by the first message; one registered Portal; endpoint name carries a hash of SUCROSE_DATA_DIR"
  - "A Portal connection is identified by a service-assigned connection id, never by the pid it claims"
  - "A stuck Portal (full queue on a non-Memory event) is unregistered; the slot is free for its reconnect"
requirements-completed: [ARCH-02]
coverage:
  - id: D1
    description: "A Portal client connecting to the real service with PortalHello gets Welcome carrying the settings view and the runtime state"
    requirement: ARCH-02
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#welcome_carries_the_settings_view"
        status: pass
    human_judgment: false
  - id: D2
    description: "A second Portal gets AlreadyOpen with the first one's pid and the first gets Focus; a Portal that left frees the place"
    requirement: ARCH-02
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_second_portal_is_told_the_first_is_open_and_the_first_is_focused"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_portal_that_left_makes_room_for_the_next"
        status: pass
    human_judgment: false
  - id: D3
    description: "A garbage first frame or a first message that is no hello is dropped without affecting the next Portal"
    requirement: ARCH-02
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_connection_whose_first_frame_is_garbage_is_dropped_and_a_portal_still_gets_welcome"
        status: pass
    human_judgment: false
  - id: D4
    description: "Wire indexes, the largest Welcome far below MAX_FRAME, apply, from_run_startup and the endpoint name per data folder"
    requirement: ARCH-02
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-ipc: 15 portal tests, 5 endpoint tests, protocol::tests::portal_hello_and_launch_keep_their_wire_index_and_survive_the_wire"
        status: pass
    human_judgment: false
  - id: D5
    description: "Admission is a pure function and the writer drops only Memory events; a Portal connecting after shutdown gets Close"
    requirement: ARCH-02
    verification:
      - kind: unit
        ref: "apps/sucrose/src/portal_link.rs tests (accept_portal_*, a_portal_that_does_not_read_never_blocks_the_sender_and_loses_only_memory_events, a_portal_that_connects_while_the_service_stops_gets_close_not_welcome)"
        status: pass
    human_judgment: false
duration: 11min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 06: Service, IPC and Portal connected Summary

**One local endpoint now serves the service, its renderers and the Portal, routed by the first message: a Portal gets Welcome with the settings view and runtime state, a second Portal gets AlreadyOpen while the first is told Focus, and the real service binary is the test peer.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-11-24T05:01:30Z
- **Completed:** 2026-11-24T05:12:40Z
- **Tasks:** 2
- **Files modified:** 11 (4 created, 7 modified)

## Accomplishments

- The whole Portal protocol exists once and is append-only (`sucrose_ipc::portal`), with a wire-index test per enum and a size test: the largest possible Welcome (all three path texts at 4095 bytes of a 3-byte character) encodes below MAX_FRAME / 4.
- `service_endpoint()` moved into `sucrose-ipc`; with `SUCROSE_DATA_DIR` set the name carries the FNV-1a hash of the folder, so a scratch-folder service never meets the maintainer's own (and the folder text never reaches the OS-wide name). Without it the name is unchanged.
- `read_connection` routes the first message: `Hello` keeps the renderer path, `PortalHello` goes to the registry on the main thread (log line `portal registered`), `Launch` is dropped with a warning until plan 15, anything else is dropped without touching other peers.
- Each Portal has one bounded writer: Memory events are the only droppable ones; any other overflow unregisters that Portal.
- `apps/sucrose/tests/common` starts the real `sucrose` binary on a scratch folder under the target directory and exposes a Portal client; every later service plan can be tested the same way.

## Task Commits

1. **Task 1: A Portal client gets Welcome from the real service (tracer)** - `9f1fbc38` (feat)
2. **Task 2: full message set, single-instance Portal, ordered writer, tests, D-033** - `445f76c2` (test, RED), `520baff8` (feat, GREEN)

**Plan metadata:** the docs commit that follows this file.

## Files Created/Modified

- `crates/sucrose-ipc/src/portal.rs` - the Portal protocol types and their tests
- `crates/sucrose-ipc/src/lib.rs` - `Endpoint`, `service_endpoint`, `service_endpoint_for`, `endpoint_name_for`, `IpcError::Identity`
- `crates/sucrose-ipc/src/protocol.rs` - `PortalHello` and `Launch` appended to `ToService`
- `crates/sucrose-ipc/Cargo.toml` - `sucrose-platform` is a normal dependency now
- `apps/sucrose/src/portal_link.rs` - PortalLink, PortalRegistry, accept_portal, portal_view, runtime_state
- `apps/sucrose/src/service.rs` - routing of the first message, Portal events on the main thread, the local `Endpoint` removed
- `apps/sucrose/src/render.rs`, `apps/sucrose/src/main.rs` - use `sucrose_ipc::service_endpoint`, `mod portal_link`
- `apps/sucrose/tests/common/mod.rs`, `apps/sucrose/tests/portal.rs` - the harness and the integration tests
- `docs/DECISIONS.md` - D-033

## Decisions Made

- D-033 (see the file). Beyond it: the hash covers the lossy UTF-8 bytes of the folder text (the plan allowed UTF-16 or bytes), identical in the service and the tests because both pass the same string.
- Requests of the Portal are logged by kind only (never their content, which can hold a theme name or a path, D-08) until their plans handle them.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical] Connection id in the Portal events**
- **Found during:** Task 1
- **Issue:** The plan's `Event::Portal { pid, request }` and `PortalGone { pid }` identify a Portal by the pid it claims. Two connections can claim the same pid (the integration tests run in one process), and a peer can claim another's pid, so one connection ending could unregister the other.
- **Fix:** `PortalLink` carries a service-assigned connection id; `PortalConnected`, `Portal` and `PortalGone` use it (the pid stays in the link for AlreadyOpen and the log).
- **Files modified:** apps/sucrose/src/service.rs, apps/sucrose/src/portal_link.rs
- **Committed in:** 9f1fbc38

**2. [Rule 3 - Blocking] Small signature changes the plan did not list**
- **Found during:** Task 1
- **Issue:** The Welcome needs the settings the service loaded, and `PortalLink` reuses the renderer link's `Offer` and queue capacity.
- **Fix:** `plan_wallpapers` also returns the loaded `Settings` (the view falls back to it when the store could not be opened); `Offer` and `OUTGOING_CAPACITY` are `pub(crate)`; `IpcError` gets an `Identity` variant for a user that cannot be identified.
- **Committed in:** 9f1fbc38

**3. [Rule 1 - Process] rustfmt rewrote some files to LF**
- **Issue:** `cargo fmt` turned several CRLF working-copy files into LF; the repository convention keeps each file's endings.
- **Fix:** restored CRLF on every touched Rust file after each format run; `docs/DECISIONS.md` stays LF.

### TDD note (Task 2)

Task 1 defined the whole protocol and the admission/registry (the plan allows that), so the AlreadyOpen/Focus and garbage-frame integration tests added in the Task 2 RED commit passed on arrival. The RED that failed on its planned assertion was the writer unit test (`a_portal_that_does_not_read_never_blocks_the_sender_and_loses_only_memory_events`: left `Failed`, expected `Dropped`); GREEN made the writer skip only Memory events. No `gsd_run check tdd-red-evidence` record was produced (the plan is `type: execute`).

### Not testable as planned

The post-Quit `Close` answer has no integration test: the scratch service has no tray (AppVisible false) and so no way to choose Quit. It is covered by unit tests of `accept_portal` and the registry (`a_portal_that_connects_while_the_service_stops_gets_close_not_welcome`).

---

**Total deviations:** 3 auto-fixed (1 Rule 2, 1 Rule 3, 1 process). **Impact on plan:** none on scope; the connection id makes the single-instance rule hold when pids collide.

## Issues Encountered

None beyond the deviations. `sucrose.exe` processes started by the tests are killed on drop; no process was left running.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None. Portal requests are received and logged by kind; handling them is the job of plans 07, 09, 21, 22 and 23 by design.

## Next Phase Readiness

- Plan 07 can move the store into the settings host and answer `Change` with `Changed`/`Rejected`; plan 08 can start the sibling service from the Portal and connect with `PortalHello`; plan 15 answers `Launch`.
- The pre-existing LF/CRLF "modified" state of `crates/sucrose-core/src/lib.rs` (no content change) was left alone.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*

## Self-Check: PASSED

- Files exist: portal.rs, portal_link.rs, tests/common/mod.rs, tests/portal.rs
- Commits exist: 9f1fbc38, 445f76c2, 520baff8
- `cargo xtask ci` exits 0; `cargo test -p sucrose-ipc portal` 15 passed; `cargo test -p sucrose --test portal` 4 passed; `cargo test -p sucrose accept_portal` 4 passed
