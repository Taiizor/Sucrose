---
phase: 03-m2-service-ipc-and-portal-connected
plan: 01
subsystem: settings
tags: [rust, serde, serde_json, atomic-write, migration, settings]
requires:
  - phase: 02-m1-service-and-renderer
    provides: legacy Setting reader (sucrose-config legacy.rs), service plan_wallpapers, data-folder paths
provides:
  - versioned Setting/Sucrose.json model (sucrose_config::model::Settings, SETTINGS_VERSION)
  - one-time migration from the real legacy Setting folder (sucrose_config::migrate::migrate)
  - atomic settings store with .bak, newer-version guard and stale-temp cleanup (sucrose_config::store)
  - service that plays from Setting/Sucrose.json instead of the legacy files
  - D-032 in docs/DECISIONS.md
affects: [03-04, 03-05, 03-06, 03-07, 03-26]
actuals:
  tokens: 13500
  tasks: 2
  commits: 4
plan_head_before: bbd3e320105aa4e5d3144d83d669b7036bcf6a3a
plan_head_after: 1a77e9ec76cc6b8185ab0257a3a50ff23fe3c091
tech-stack:
  added: []
  patterns: [temporary file + sync_all + rename atomic write, lenient section-by-section settings reading with unknown keys kept, kill-the-child-process test using the test binary itself]
key-files:
  created:
    - crates/sucrose-config/src/model.rs
    - crates/sucrose-config/src/migrate.rs
    - crates/sucrose-config/src/store.rs
    - crates/sucrose-config/tests/store.rs
  modified:
    - crates/sucrose-config/Cargo.toml
    - crates/sucrose-config/src/lib.rs
    - apps/sucrose/src/service.rs
    - docs/DECISIONS.md
    - Cargo.lock
key-decisions:
  - "Setting/Sucrose.json (D-01, settled by the maintainer) with sections and keys named like the legacy files (D-032)"
  - "Default library location is stored as an empty string meaning the legacy default folder, so the file never pins a path nobody chose"
  - "Invalid values read as legacy defaults section by section instead of failing the whole file; only non-JSON or a non-object root counts as corrupt"
  - "The first settings write at open does not create a .bak; the .bak is the previous good file copied at each update"
requirements-completed: [ARCH-04, FMT-01, FMT-06]
duration: 10 min
completed: 2026-11-24
status: complete
coverage:
  - id: D1
    description: "The service migrates the legacy settings once into Setting/Sucrose.json and plays the selected theme from it; the next start loads the file"
    requirement: FMT-01
    verification:
      - kind: e2e
        ref: "cargo build -p sucrose, sucrose.exe on a scratch SUCROSE_DATA_DIR: 'settings loaded outcome=Migrated' then 'renderer connected', second start 'outcome=Loaded'"
        status: pass
      - kind: integration
        ref: "crates/sucrose-config/tests/store.rs#migration_runs_once_and_never_writes_a_legacy_file"
        status: pass
    human_judgment: false
  - id: D2
    description: "Legacy Setting/*.json files are byte-identical before and after the runs"
    requirement: FMT-01
    verification:
      - kind: e2e
        ref: "sha256sum -c target/legacy-setting.sha256 (14 real legacy files, before the first run and after both)"
        status: pass
    human_judgment: false
  - id: D3
    description: "A kill during a write never leaves a corrupt settings file; stale temporaries are removed at open"
    requirement: ARCH-04
    verification:
      - kind: integration
        ref: "crates/sucrose-config/tests/store.rs#kill_during_write_never_leaves_a_corrupt_file"
        status: pass
      - kind: integration
        ref: "crates/sucrose-config/tests/store.rs#stale_temporary_files_are_removed_at_open"
        status: pass
    human_judgment: false
  - id: D4
    description: "A missing or corrupt Sucrose.json next to a good .bak is restored; a newer Version is read but never overwritten; unknown keys survive"
    requirement: ARCH-04
    verification:
      - kind: integration
        ref: "crates/sucrose-config/tests/store.rs (a_corrupt_file_with_a_good_backup_is_restored_not_migrated, a_missing_file_with_a_good_backup_is_restored, a_newer_version_is_read_but_never_overwritten, unknown_keys_survive_load_update_load, an_update_keeps_the_previous_file_as_the_backup)"
        status: pass
    human_judgment: false
  - id: D5
    description: "A data folder without Setting/ gets that folder created and the app adds only Sucrose.json (and later Sucrose.json.bak)"
    requirement: FMT-06
    verification:
      - kind: integration
        ref: "crates/sucrose-config/tests/store.rs#nothing_at_all_gives_the_defaults_in_a_new_setting_folder"
        status: pass
    human_judgment: false
---

# Phase 3 Plan 01: Service plays from a migrated Setting/Sucrose.json Summary

**Versioned Setting/Sucrose.json (legacy section and key names, lenient reading, unknown keys kept) with a temp-file/sync_all/rename atomic store, a .bak restore path, a one-time read-only migration from the C# Setting folder, and a service that now plays from the new file**

## Performance

- **Duration:** 10 min
- **Started:** 2026-11-24T03:54:59Z
- **Completed:** 2026-11-24T04:06:00Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- `sucrose-config` gained `model` (Settings with Library, General, Engine, Backgroundog sections, legacy keys and defaults, unknown keys kept), `migrate` (reuses the Phase 2 legacy reader, read-only) and `store` (load order file, .bak, migrate; atomic write; backup copy; newer-version guard; stale temp cleanup).
- The service opens the store in `plan_wallpapers` and plans from `Settings`; if the folder cannot be opened it migrates in memory and still plays. The store lives for the whole `run()`.
- Scratch run: first start logged `settings loaded outcome=Migrated` and `renderer connected`, wrote a 478-byte pretty-printed Sucrose.json; the second start logged `outcome=Loaded`. SHA-256 of all 14 real legacy files matched before and after.
- Kill test: 60 rounds per run, the test binary re-runs itself as the writer and is killed 0-40 ms after its loop starts; 47 to 53 of 60 kills left a temporary file behind in three runs and every round still had a parsing Sucrose.json and a clean open afterwards.
- D-032 recorded.

## Task Commits

1. **Task 1: Service plays the selected theme from a migrated Setting/Sucrose.json (tracer)** - `5f947865` (feat)
2. **Task 2: Store hardening (TDD)**
   - RED - `cc80f145` (test): 6 of 11 store tests fail on the planned assertions
   - GREEN - `cb115d69` (feat)
   - D-032 - `1a77e9ec` (docs)

**Plan metadata:** committed after this summary (docs: complete plan).

## TDD Gate Compliance

RED: `cargo test -p sucrose-config --test store` exited non-zero with 5 passed, 6 failed, every failure on the planned assertion (restore from .bak returned Migrated or Defaults, no .bak after two updates, an update over a Version 2 file returned Ok, stale `Sucrose.json.*.tmp` files survived open, the kill test found a stale temporary file after open); the tests compile, so no build, load or zero-test failure. The `gsd_run check tdd-red-evidence` classifier has no cargo-test report adapter and workflow.tdd_mode is off, so the evidence is this semantic assessment, not a classifier verdict. GREEN: all 11 pass; no REFACTOR commit.

## Files Created/Modified

- `crates/sucrose-config/src/model.rs` - versioned settings model with lenient reading and unknown keys kept
- `crates/sucrose-config/src/migrate.rs` - one-time migration from the legacy folder
- `crates/sucrose-config/src/store.rs` - atomic store, .bak, newer-version guard, stale cleanup
- `crates/sucrose-config/tests/store.rs` - load-order, backup, guard, unknown-key, migration and kill tests
- `crates/sucrose-config/src/lib.rs`, `Cargo.toml`, `Cargo.lock` - modules, serde dependency (already in the workspace)
- `apps/sucrose/src/service.rs` - opens the store, plans from Settings, ignored real-install test uses migrate
- `docs/DECISIONS.md` - D-032

## Decisions Made

See key-decisions. D-032 carries the settled one-way path decision; nothing needed another name or folder, so no stop was required.

## Deviations from Plan

None - plan executed exactly as written. Two small adaptations inside plan scope: `StoreError::NewerVersion` and the full `LoadOutcome` were defined in Task 1 so the Task 2 RED tests compile and fail on assertions; the Task 2 clippy run (`collapsible_if`, `large_enum_variant`) was fixed in the GREEN commit.

## Issues Encountered

- Editing and `cargo fmt` flipped service.rs and model.rs to LF in the working copy; both were converted back to CRLF (the working-copy convention). Committed content is normalized by git either way.

## Known Stubs

None.

## Threat Flags

None. Mitigations T-03-01 (kill test, temp + sync_all + rename, .bak), T-03-02 (read-only migration, SHA-256 check) and T-03-03 (unknown keys kept, newer Version guard, invalid values read as defaults) are implemented; T-03-SC: no crate added.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 04 can replace the body of `migrate` with the per-file migration of every key; plan 07 can move the `Store` (kept alive in `run()`) into the settings host. The maintainer's own runs of the Rust service now create %APPDATA%\Sucrose\Setting\Sucrose.json by design.

## Self-Check: PASSED

Files model.rs, migrate.rs, store.rs, tests/store.rs exist; commits 5f947865, cc80f145, cb115d69, 1a77e9ec are ancestors of HEAD; `cargo xtask ci` exits 0.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
