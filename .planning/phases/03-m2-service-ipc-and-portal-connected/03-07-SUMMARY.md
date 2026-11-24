---
phase: 03-m2-service-ipc-and-portal-connected
plan: 07
subsystem: settings
tags: [rust, settings, ipc, portal, atomic-write, validation, fullscreen-scope]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: Store with atomic writes (plan 01), Portal protocol, PortalLink and the test harness (plan 06), Backgroundog.FullScreenScope (plan 26)
provides:
  - "apps/sucrose settings_host: SettingsHost::{open, unsaved, persists, settings, view, apply, update}, validate, describe, NOT_SAVED, the service's only owner of the settings store"
  - "PortalRequest::Change handled by the service: Changed(stored value) or Rejected, applied in arrival order on the main thread"
  - "Fullscreen rule scope read from Backgroundog.FullScreenScope (ACTION_SCOPE constant removed)"
  - "Test harness: TestService::restart and settings_file, PortalClient::request"
affects: [03-09, 03-13, 03-16, 03-17, 03-20, 03-21, 03-22, 03-23]
actuals:
  tokens: 10800
  tasks: 2
  commits: 3
plan_head_before: f8423dedf1b1774b5dbf9979b1441dfb24e87877
plan_head_after: c2d7cd5edea13caca7ff89b289f66a5e8de2219f
tech-stack:
  added: []
  patterns: [one writer of the settings file owned by the service main thread, validate-then-write returning the stored value so the Portal only shows broadcast values, log values never paths]
key-files:
  created:
    - apps/sucrose/src/settings_host.rs
  modified:
    - apps/sucrose/src/main.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/portal_link.rs
    - apps/sucrose/src/fullscreen.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose/tests/common/mod.rs
key-decisions:
  - "SettingsHost without a store (settings folder unopenable or no config folder) keeps the legacy settings in memory and refuses every change with the not-saved text, so the wallpaper still plays"
  - "BackgroundImage is logged as set or none, never as a path (D-08)"
requirements-completed: [ARCH-05, ARCH-04]
coverage:
  - id: D1
    description: "A SettingChange from the Portal is stored by the service, answered Changed and still there after the service restarts"
    requirement: ARCH-05
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_change_survives_a_restart"
        status: pass
    human_judgment: false
  - id: D2
    description: "Numbers are clamped to the legacy ranges and the clamped value is what Changed carries; a bad BackgroundImage is Rejected and nothing is stored"
    requirement: ARCH-04
    verification:
      - kind: unit
        ref: "apps/sucrose/src/settings_host.rs#validate_clamps_numbers_to_the_legacy_ranges, validate_accepts_no_image_and_an_absolute_image_path_in_any_case, validate_refuses_a_relative_path_an_unknown_extension_and_a_long_text, a_refused_value_changes_nothing"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#an_out_of_range_change_comes_back_clamped, a_refused_image_is_rejected_and_nothing_is_stored"
        status: pass
    human_judgment: false
  - id: D3
    description: "A failed write restores the value and answers Rejected with the setting-not-saved text; nothing is broadcast"
    requirement: ARCH-05
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_failed_write_is_rejected_with_the_not_saved_text_and_nothing_is_broadcast"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/settings_host.rs#a_host_that_cannot_save_answers_with_the_not_saved_text_and_keeps_the_value"
        status: pass
    human_judgment: false
  - id: D4
    description: "The same change twice leaves the same file bytes; two quick changes end with the last one in arrival order"
    requirement: ARCH-05
    verification:
      - kind: unit
        ref: "apps/sucrose/src/settings_host.rs#the_same_change_twice_leaves_the_same_file_bytes"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#changes_are_applied_in_arrival_order_and_the_last_one_wins"
        status: pass
    human_judgment: false
  - id: D5
    description: "The fullscreen rule's scope comes from Backgroundog.FullScreenScope"
    requirement: ARCH-04
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#the_plan_carries_the_fullscreen_scope_of_the_settings"
        status: pass
    human_judgment: false
duration: 9min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 07: Service, IPC and Portal connected Summary

**The service now owns the settings file through a SettingsHost: a Portal SettingChange is validated and clamped to the legacy ranges, written atomically, answered Changed with the stored value (or Rejected with the not-saved text) and is still there after a restart; the fullscreen rule's scope comes from Backgroundog.FullScreenScope.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-11-24T05:28:50Z
- **Completed:** 2026-11-24T05:37:30Z
- **Tasks:** 2
- **Files modified:** 7 (1 created, 6 modified)

## Accomplishments

- `SettingsHost` replaces the bare store in the service: `plan_wallpapers` and the Welcome read from it, `portal_view` moved into it as `view()`, and the main loop answers `PortalRequest::Change` one request at a time, so the last change wins and the Portal shows only broadcast values.
- `validate` clamps BackgroundOpacity and AdaptiveLayout to 0..=100, LibraryPagination to 1..=100, AdaptiveMargin to 5..=25, and accepts a BackgroundImage only when empty or an absolute path of at most MAX_PATH_TEXT bytes with a png, jpg, jpeg, tiff, webp or gif extension (case-insensitive).
- A failed write (the store restores the previous value) is answered `Rejected` with "Your setting couldn't be saved. Try again." and nothing is broadcast; a host without a store refuses every change the same way.
- `ACTION_SCOPE` is gone: `Plan.scope` is mapped from `Backgroundog.FullScreenScope` and handed to `enable_fullscreen`.
- The test harness can restart the service on the same folder and read the saved file, which every later settings plan can reuse.

## Task Commits

1. **Task 1: A Portal change survives a restart (tracer)** - `04bde0a2` (feat)
2. **Task 2: validation, clamping, Rejected, order, fullscreen scope** - `da711ca5` (test, RED), `c2d7cd5e` (feat, GREEN)

**Plan metadata:** the docs commits that follow this file.

Tracer verified end-to-end (the portal integration tests re-run green) before expansion.

## Files Created/Modified

- `apps/sucrose/src/settings_host.rs` - SettingsHost, validate, describe, view, unit tests
- `apps/sucrose/src/service.rs` - Change handling, host in place of the bare store, scope in the plan
- `apps/sucrose/src/portal_link.rs` - `portal_view` removed, `clip` shared
- `apps/sucrose/src/fullscreen.rs` - stale dead_code allowance on `AffectedMonitor` removed
- `apps/sucrose/src/main.rs` - `mod settings_host`
- `apps/sucrose/tests/portal.rs`, `apps/sucrose/tests/common/mod.rs` - restart, clamp, order, rejection tests and harness helpers

## Decisions Made

- Without a store the host keeps the in-memory legacy settings and refuses changes, instead of pretending to save.
- The log line `setting changed` carries the variant and value; BackgroundImage shows only set or none.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] fullscreen.rs touched**
- **Found during:** Task 2
- **Issue:** `ActionScope::AffectedMonitor` carried `#[allow(dead_code)]` saying no switch reaches it in M1; with the scope now read from the settings the reason is false.
- **Fix:** removed the allowance and updated the type's doc line.
- **Files modified:** apps/sucrose/src/fullscreen.rs
- **Committed in:** c2d7cd5e

**2. [TDD ordering] The fullscreen scope test arrived with its implementation**
- **Found during:** Task 2 RED
- **Issue:** a test on `Plan.scope` cannot fail on an assertion before the field exists, only fail to compile, which is not a valid RED.
- **Fix:** the RED commit holds the validation, rejection, idempotency and order tests (each failing on its assertion); the scope test is in the GREEN commit.
- **Committed in:** c2d7cd5e

---

**Total deviations:** 2 (1 blocking cleanup, 1 TDD ordering note)
**Impact on plan:** none on behavior or scope.

## Issues Encountered

- My first edit helper wrote LF over CRLF working files; I converted the touched files back to CRLF before committing (the index is LF either way, so history is unaffected).

## Known Stubs

None.

## Threat Flags

None - no new endpoint or trust boundary beyond the planned settings host (T-03-17 mitigated: typed changes, clamped numbers, path allow-list, restore on a failed write, one writer on one thread).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 09, 13, 16, 17, 20-23 can send `PortalRequest::Change`; service-side reactions (AppVisible plan 13, AppExit plan 16) hook into the `Change` arm of the main loop.
- `SettingsHost::update` is ready for the service's own writes (Library.Selected, Library.Location, Warehouse.Showcase, General.RunStartup).

## Self-Check: PASSED

- apps/sucrose/src/settings_host.rs FOUND; commits 04bde0a2, da711ca5, c2d7cd5e FOUND on the branch; `cargo xtask ci` exit 0.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
