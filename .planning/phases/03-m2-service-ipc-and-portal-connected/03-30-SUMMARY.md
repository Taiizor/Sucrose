---
phase: 03-m2-service-ipc-and-portal-connected
plan: 30
subsystem: settings-store
tags: [rust, settings, atomic-store, arch-04, cr-02, gap-closure]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: Store, SettingsHost and the legacy migration (plans 03-01..03-26)
provides:
  - "Read::Unreadable and read_retrying: an I/O error other than not-found is retried, then refuses instead of restoring or migrating"
  - "Store::open returns Err(StoreError::Io) for an unreadable Sucrose.json or Sucrose.json.bak and writes nothing"
  - "Locked-file, directory, backup-lock and Unix mode tests, plus a host test that proves no-save mode"
affects: [03-verification, arch-04]
actuals:
  tokens: 14000
  tasks: 2
  commits: 3
plan_head_before: 6c9d1131e193b1b30986845101f77b58284d8c4a
plan_head_after: 7f27a7910f8e05b2110f35497662a98c8b0135d9
tech-stack:
  added: []
  patterns:
    - "Read outcomes are Missing, Corrupt, Unreadable and Good; only Missing and Corrupt may lead to a restore or a migration"
key-files:
  created: []
  modified:
    - crates/sucrose-config/src/store.rs
    - crates/sucrose-config/tests/store.rs
    - apps/sucrose/src/settings_host.rs
key-decisions:
  - "An unreadable settings file or backup makes Store::open fail with StoreError::Io; the existing SettingsHost Err arm already gives in-memory, no-save mode, so the host needed no change"
  - "Store warnings for an unreadable file log the error kind only, never the message or path (T-03-69)"
requirements-completed: [ARCH-04]
coverage:
  - id: D1
    description: "A locked or unreadable Sucrose.json is retried and then left exactly as it was; the store refuses to open instead of restoring the backup or migrating"
    requirement: ARCH-04
    verification:
      - kind: integration
        ref: "crates/sucrose-config/tests/store.rs#an_unreadable_settings_file_is_never_overwritten"
        status: pass
      - kind: integration
        ref: "crates/sucrose-config/tests/store.rs#a_settings_file_that_cannot_be_read_as_a_file_is_unreadable_not_corrupt"
        status: pass
    human_judgment: false
  - id: D2
    description: "An unreadable backup next to a corrupt settings file never leads to a migration over it"
    requirement: ARCH-04
    verification:
      - kind: integration
        ref: "crates/sucrose-config/tests/store.rs#an_unreadable_backup_with_a_corrupt_settings_file_is_never_migrated_over"
        status: pass
    human_judgment: false
  - id: D3
    description: "A read that fails once with a sharing violation is retried and then loads normally"
    requirement: ARCH-04
    verification:
      - kind: unit
        ref: "crates/sucrose-config/src/store.rs#a_read_that_fails_once_with_a_sharing_violation_is_retried"
        status: pass
    human_judgment: false
  - id: D4
    description: "The service host stays in no-save mode and writes nothing when Sucrose.json cannot be read"
    requirement: ARCH-04
    verification:
      - kind: unit
        ref: "apps/sucrose/src/settings_host.rs#a_host_whose_settings_file_cannot_be_read_saves_nothing"
        status: pass
    human_judgment: false
duration: 6 min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 30: Unreadable settings file is never overwritten Summary

**`Read::Unreadable` with a 3x50 ms retry makes `Store::open` refuse (Err(StoreError::Io)) instead of restoring the older backup or migrating over a locked Sucrose.json, closing CR-02**

## Performance

- **Duration:** 6 min
- **Started:** 2026-11-24T14:57:00Z
- **Completed:** 2026-11-24T15:03:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- `read_file` tells Missing, Corrupt, Unreadable and Good apart; reads that fail for any reason but not-found are tried up to `ATTEMPTS` times, `RETRY_DELAY` apart, as `legacy::read` does.
- `Store::open` returns `Err(StoreError::Io)` when Sucrose.json, or the backup of a missing or corrupt Sucrose.json, is unreadable. Nothing is written and the legacy files are not read.
- The red run showed the bug: a locked Sucrose.json was replaced by the older backup (`RestoredFromBackup`).
- `SettingsHost::open`'s existing Err arm is proven to give no-save mode (`persists()` false, every change answered NOT_SAVED, nothing written to the Setting folder).

## Task Commits

1. **Task 1 (tracer): a locked Sucrose.json is never overwritten** - `8a8533b8` (test, RED: both new tests failed with RestoredFromBackup), `0bf50f70` (fix, GREEN)
2. **Task 2: unreadable backup, retry, host wiring** - `7f27a791` (test)

**Plan metadata:** the docs(03-30) commit that follows this summary.

## Files Created/Modified
- `crates/sucrose-config/src/store.rs` - `Read::Unreadable`, `read_retrying`, `open` refusing instead of restoring or migrating, retry unit tests
- `crates/sucrose-config/tests/store.rs` - locked settings file (Windows), directory in place of the file, locked backup with a corrupt file (Windows), mode 0o000 (Unix, skipped as root)
- `apps/sucrose/src/settings_host.rs` - `a_host_whose_settings_file_cannot_be_read_saves_nothing`

## Decisions Made
- No production change in `settings_host.rs`: the host test showed the existing Err arm already holds.
- The retry unit tests were written together with the retry in Task 1 (the helper is needed there), so they sit in the Task 1 GREEN commit rather than in Task 2's.

## TDD Gate Compliance
- RED (`8a8533b8`): `cargo test -p sucrose-config --test store unreadable` failed for both target tests on the planned assertion (`Ok(.., RestoredFromBackup)` instead of `Err(StoreError::Io)`); semantic assessment: they executed and failed for the intended reason. Not run through the `tdd-red-evidence` classifier (the plan is `type: execute`, tasks only carry `tdd="true"`).
- GREEN (`0bf50f70`): both tests pass. The Task 2 integration tests passed on first run because `open` already refused for an unreadable backup after Task 1; they pin that behavior.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Wrong file name order in my own RED test**
- **Found during:** Task 1 (GREEN run)
- **Issue:** `a_settings_file_that_cannot_be_read_as_a_file_is_unreadable_not_corrupt` expected `[BACKUP_FILE, SETTINGS_FILE]` but the sorted listing is `[SETTINGS_FILE, BACKUP_FILE]`.
- **Fix:** swapped the expected order.
- **Files modified:** crates/sucrose-config/tests/store.rs
- **Committed in:** 0bf50f70

---

**Total deviations:** 1 auto-fixed (1 test bug). **Impact:** none on scope.

## Issues Encountered
- `cargo fmt` rewrites `tests/store.rs` (and can others) to LF in the working copy; the three files were converted back to CRLF after each format run and `git ls-files --eol` shows w/crlf for all of them.
- The Unix test (`#[cfg(unix)]`) was not run: the toolchain here is Windows. It compiles only on Unix and returns early as root.

## Known Stubs
None.

## Threat Flags
None. T-03-68 and T-03-69 are mitigated as planned.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- ARCH-04's store gap (03-VERIFICATION gap 4, review CR-02) is closed; `cargo xtask ci` passes.
- The maintainer-side check that a locked Sucrose.json leaves the file alone with the real service is not needed: the Windows lock test covers it.

## Self-Check: PASSED
- Files exist: store.rs, tests/store.rs, settings_host.rs
- Commits present: 8a8533b8, 0bf50f70, 7f27a791
- `cargo xtask ci`, `cargo test -p sucrose --bin sucrose settings_host` and `cargo test -p sucrose --test portal a_change_survives_a_restart` pass

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
