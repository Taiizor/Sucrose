---
phase: 03-m2-service-ipc-and-portal-connected
plan: 21
subsystem: service
tags: [library, showcase, recycle-bin, trash, ipc, worker-thread]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "PortalRequest::MaintainLibrary / PortalEvent::LibraryMaintained(MaintainReport) (plan 06), SettingsHost::update and Notifier/Event loop (plans 07/09), library::scan with CorruptReason (plan 02), Portal library view, toasts and i18n keys (plans 14/19)"
provides:
  - "apps/sucrose/src/library_ops.rs: maintain, MaintainInput, copy_tree, CopyStats, Trash, record_names"
  - "library-ops worker thread and Event::LibraryOps in the service; log line 'library maintained'"
  - "sucrose_platform::move_to_trash and Capabilities.recycle_bin (trash 5.4.0 behind the wrapper)"
  - "Portal: MaintainLibrary once per connection after the first scan, the preparing line, re-scan after a change, the corrupt-themes toast with the bin name"
affects: [03-22, library, settings, platform]

actuals:
  tokens: 12000
  tasks: 2
  commits: 4
plan_head_before: 46c5df3d84ca3d457e5ce5162dcf93927b041556
plan_head_after: cc46cc9d0fec1c792525fd8805b2683c6bb7bbc3

tech-stack:
  added: ["trash 5.4.0 (workspace dependency, used only by sucrose-platform)"]
  patterns:
    - "Service jobs that touch the disk run on a named worker thread and come back as an Event through the Notifier; a panic still reports (empty report)"
    - "Side-effecting OS calls sit behind a small trait (Trash) so tests record calls with a fake instead of touching the real bin"
    - "Copy and trash functions take the file copy / bin as a parameter so failure paths are unit-tested"

key-files:
  created:
    - apps/sucrose/src/library_ops.rs
    - crates/sucrose-platform/src/trash.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - crates/sucrose-platform/src/os_linux.rs
    - crates/sucrose-platform/src/os_macos.rs
    - apps/sucrose/src/main.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/library/mod.rs
    - apps/sucrose-portal/src/library/grid.rs
    - apps/sucrose-portal/src/toast.rs
    - docs/FEATURES.md

key-decisions:
  - "A showcase folder that already exists in the library is recorded in Warehouse.Showcase without being written to and without counting as copied (legacy: it was already there)"
  - "A request that arrives while a maintenance runs is answered by the broadcast of the running one (one Portal is registered, so the broadcast reaches it)"
  - "The Portal also re-scans after maintenance when its first scan failed, because the maintenance creates a missing library folder"
  - "The preparing line shows only while no theme is on screen yet (first run); with themes the maintenance stays unseen"
  - "The bin wrapper maps trash errors to path-free sentences (D-08: logs never carry paths or theme names)"

patterns-established:
  - "Pattern: link/junction entries are never followed by the copy or the corrupt step (file_type of the entry itself)"

requirements-completed: [LIB-01, LIB-02]

coverage:
  - id: D1
    description: "Showcase folders not listed in Warehouse.Showcase are copied into the library once, never overwriting, and recorded only after the copy succeeded"
    requirement: LIB-01
    verification:
      - kind: unit
        ref: "apps/sucrose/src/library_ops.rs#copies_only_the_showcase_folders_that_are_not_listed"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#showcase_themes_are_copied_once_and_recorded_in_the_settings"
        status: pass
    human_judgment: false
  - id: D2
    description: "Corrupt themes go to the recycle bin or trash only when Corrupt Themes is on, decided by the service after its own scan, and their Cache/Properties file is deleted"
    requirement: LIB-02
    verification:
      - kind: unit
        ref: "apps/sucrose/src/library_ops.rs#corrupt_folders_go_to_the_bin_once_and_their_property_file_is_deleted"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/library_ops.rs#corrupt_error_from_the_bin_counts_as_skipped_and_keeps_the_folder"
        status: pass
      - kind: unit
        ref: "crates/sucrose-platform/src/trash.rs#a_temporary_folder_moves_to_the_real_bin (ignored, run once by hand)"
        status: pass
    human_judgment: false
  - id: D3
    description: "The Portal shows the preparing line on first run and the corrupt-themes toast with the right bin name; the Recycle Bin shows the moved synthetic folders and can restore them"
    verification: []
    human_judgment: true
    rationale: "Seeing the line, the toast and restoring from the Windows Recycle Bin are desktop checks (plan human-check at the end of the phase)"

duration: 24min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 21: Service, IPC and Portal connected (library maintenance) Summary

**The service now prepares the library on a library-ops worker thread: unlisted Showcase folders are copied in once and tracked in the new settings, and corrupt themes go to the OS recycle bin through a platform wrapper over the trash crate, with the Portal asking once per connection, showing the preparing line and toasting the count.**

## Performance

- **Duration:** 24 min
- **Started:** 2026-11-24T10:25:00Z
- **Completed:** 2026-11-24T10:49:00Z
- **Tasks:** 2 (both TDD: RED then GREEN commits)
- **Files modified:** 17 (2 created)

## Accomplishments
- `library_ops::maintain` creates the library folder, copies unlisted showcase folders without overwriting (links skipped, a failed copy removes what it created) and, with Corrupt Themes on, re-scans and moves Corrupt (never Unreadable, never linked) folders to the bin, then deletes their Cache/Properties file.
- The service starts the work on a `library-ops` thread (one at a time, panic-safe), records the copied names in `Warehouse.Showcase` on the main thread and answers with `LibraryMaintained`; it logs `library maintained` with the counts.
- `sucrose-platform` gained `move_to_trash` (trash 5.4.0, path-free error text) and `Capabilities.recycle_bin`.
- The Portal sends `MaintainLibrary` once per connection after its first scan, shows "Preparing your library…" under the skeleton grid while nothing is on screen, re-scans when themes were copied or moved (or the first scan failed) and toasts "{count} corrupt themes were moved to the {bin}" (Recycle Bin on Windows, Trash elsewhere).
- FEATURES.md records the bin as an improvement (D-09) and marks both rows 🟨.

## Task Commits

1. **Task 1: Showcase themes copied once (tracer)** - `4ae16200` (test), `cf985319` (feat)
2. **Task 2: Corrupt themes to the recycle bin** - `b6723dfa` (test), `cc46cc9d` (feat)

**Plan metadata:** the docs commit that carries this SUMMARY.

## Files Created/Modified
- `apps/sucrose/src/library_ops.rs` - maintenance logic and 12 unit tests with a recording fake bin
- `crates/sucrose-platform/src/trash.rs` - `move_to_trash` wrapper, one test plus one ignored real-bin test
- `apps/sucrose/src/service.rs` - `Event::LibraryOps`, `start_library_ops`, `OsBin`, the request handling
- `apps/sucrose/tests/portal.rs` - integration test of the showcase copy through the real service binary
- `apps/sucrose-portal/src/library/{mod,grid}.rs`, `app.rs`, `toast.rs` - request, preparing line, re-scan, toast
- `Cargo.toml`, `Cargo.lock`, platform crate files - the `trash` dependency and `recycle_bin` capability
- `docs/FEATURES.md` - Showcase and corrupt-theme rows

## Decisions Made
See key-decisions above. The tracer was verified end to end on a real Portal plus service run (below).

## Verification
- Tracer run: `target/synth-show` (a copy of the synthetic 40-theme library whose Location points at its own Library), Portal started with `--no-focus`: the showcase folder was copied, its name is in Sucrose.json, the log says `library maintained showcase_copied=1`; the second run logs `showcase_copied=0`.
- `cargo test -p sucrose library_ops` 12 passed; `cargo test -p sucrose corrupt` 5 passed; `cargo test -p sucrose-platform` 64 passed; `cargo clippy ... -D warnings` clean; `cargo xtask ci` exit 0.
- The ignored platform test was run once: it moved one temporary folder (`sucrose-trash-probe-<pid>`, a probe file inside) into the real Recycle Bin of this machine. It can be deleted from the bin.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Tracer verify command pointed at the wrong library**
- **Found during:** Task 1 verification
- **Issue:** the plan's command checks `target/synth-show/Library/...`, but the settings that `synth-library` writes name the absolute path of `target/synth-small/Library`, so the copy (correctly) landed there and the check failed; the first run also left one extra folder in `target/synth-small/Library`.
- **Fix:** removed that one generated folder from `target/synth-small`, then rewrote `Library.Location` of the `synth-show` copy to its own Library before running the same Portal steps; all checks then pass. No code change.
- **Files modified:** none (generated files under target/ only)
- **Verification:** `COPIED`, name recorded, `library maintained` logged, second run copies 0.

**2. [Rule 1 - Plan command] Acceptance command with two filters is not valid cargo**
- **Found during:** Task 2 verification
- **Issue:** `cargo test -p sucrose library_ops corrupt` is rejected by cargo (one filter before `--`).
- **Fix:** ran `cargo test -p sucrose corrupt` instead: 5 passed, including the test that asserts the corrupt folder still exists after a bin error.

**3. [Rule 2 - Missing critical] Path-free bin errors and link guard**
- **Found during:** Task 2
- **Issue:** trash's error text can carry a path, and the scan follows links, so a linked folder could be classified corrupt.
- **Fix:** the wrapper maps errors to fixed sentences (D-08), and the corrupt step skips link entries (counted as skipped).
- **Files modified:** crates/sucrose-platform/src/trash.rs, apps/sucrose/src/library_ops.rs

**4. [Rule 2 - Missing critical] Re-scan after a failed first scan**
- **Found during:** Task 1
- **Issue:** a missing library folder makes the first scan fail; the maintenance creates the folder but the plan only re-scanned on copied/moved themes, leaving the load-error state.
- **Fix:** the Portal also re-scans when its phase is Failed.
- **Files modified:** apps/sucrose-portal/src/library/mod.rs

---

**Total deviations:** 4 (2 verification-command, 2 small correctness additions)
**Impact on plan:** no scope creep; the code matches the plan's contract.

## Issues Encountered
- A temporary stub bin (`NoBin`) existed in the Task 2 RED commit only so the tests could compile; the GREEN commit replaced it with the real wrapper.

## Known Stubs
None. `library_ops::copy_tree` is public, tested and currently used only inside the module; it carries an `allow(dead_code)` with a reason because the library move of plan 22 uses it across volumes.

## Threat Flags
None beyond the plan's threat model (T-03-48..50, T-03-SC are mitigated as planned: bin instead of delete, own scan, links skipped, worker thread, `trash` pinned once in the workspace).

## User Setup Required
None - no external service configuration required.

## Human Check (end of phase)
On a fresh `cargo xtask synth-library --out target/synth-bin --count 40` folder with `"DeleteCorrupt": true` set by hand in its `Setting/Sucrose.json` (never a real folder), open the release Portal: a toast should say 2 corrupt themes were moved to the Recycle Bin, and the two synthetic folders should be restorable. Not run here: automated runs never turn Corrupt Themes on.

## Next Phase Readiness
- Plan 22 can use `library_ops::copy_tree`, `Event::LibraryOps` style worker jobs and `Capabilities.recycle_bin`.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*

## Self-Check: PASSED
