---
phase: 03-m2-service-ipc-and-portal-connected
plan: 22
subsystem: service
tags: [library, move, rollback, ipc, portal, dialog, worker-thread]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "PortalRequest::MoveLibrary and PortalEvent::MoveResult(MoveOutcome), RuntimeState.library_moving, MAX_PATH_TEXT (plan 06); UseRefusal::Moving and the fleet start path (plan 09); copy_tree and the library-ops worker (plan 21); the Library Location row with the Move checkbox (plan 20)"
provides:
  - "apps/sucrose/src/library_ops.rs: check_destination, check_location, move_library, finish_move, rollback_move, MoveOps (RealOps), MovePlanDone, MoveFailure, MoveStats, FinishStats"
  - "Service: MoveLibrary handling (LibraryMove state machine: close renderers, move on the library-ops worker, write Library.Location, remove the old copy, restart the wallpaper, answer), RuntimeState.library_moving, log line 'library moved'"
  - "Portal: folder picker with quick checks, dialogs/move_library.rs confirmation, progress state, Use disabled while moving, library reload from the new folder"
  - "docs/FEATURES.md: library location change and move row (D-07)"
affects: [03-23, library, settings, service]

actuals:
  tokens: 24500
  tasks: 2
  commits: 5
plan_head_before: 2e97abe06fef4e4dc7844d0fb5ca3e7327f6c096
plan_head_after: 25d98e9dc495b29514933d5573913f0a31069e4e

tech-stack:
  added: []
  patterns:
    - "A multi-step service job is a small state machine owned by Fleet (LibraryMove with MoveStage) that the main loop drives after every event batch; each blocking step runs on a named library-ops thread and returns as Event::LibraryMove"
    - "File operations sit behind a trait (MoveOps) so tests inject rename and copy failures; the real implementation is a unit struct"
    - "Closing the renderers for the move reuses RuleClosed (close, ended, want_back), so the exit of the old renderer is seen before a new one starts"

key-files:
  created:
    - apps/sucrose-portal/src/dialogs/move_library.rs
  modified:
    - apps/sucrose/src/library_ops.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/portal_link.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose-portal/src/settings/general.rs
    - apps/sucrose-portal/src/dialogs/mod.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/library/card.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/locales/sucrose.en.yml
    - docs/FEATURES.md

key-decisions:
  - "With Move off the destination may hold files (legacy: the empty-folder rule applies only when Move is on); check_location only needs an existing, writable folder that is not the library"
  - "Library.Location is written on the main thread only after the worker verified the content; the old copy is removed by a second worker step after the write, and a failed write rolls the content back before the Portal is told"
  - "The answer (Moved or Changed) goes out only when the whole move is over, including the wallpaper restart, so the old folder is already gone when the Portal hears Moved"
  - "Renderers are closed through RuleClosed and a 4 s timer ends the survivors, so a renderer that ignores Close cannot keep library_moving true forever"
  - "The Portal applies the new location itself from the destination it sent (no SettingChange exists for the location), and library.update_settings reloads on the changed location"

patterns-established:
  - "Pattern: while a move runs the monitor-change apply, shell-restart recovery, fullscreen tick, tray Exit and the exit rule all wait (guards on fleet.library_move)"
  - "Pattern: a top-level link in the library is never followed; rename moves it, the copy path leaves it in the old folder"

requirements-completed: [LIB-09]

coverage:
  - id: D1
    description: "A library moves to an empty folder through the service: every top-level entry renamed or copied and verified, Library.Location written after verification, old content removed, wallpaper restarted"
    requirement: "LIB-09"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/library_ops.rs#on_one_volume_every_entry_is_renamed_and_nothing_is_copied"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/library_ops.rs#when_rename_fails_every_entry_is_copied_and_verified_and_the_source_waits_for_finish"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_library_moves_to_an_empty_folder_and_the_location_follows"
        status: pass
    human_judgment: false
  - id: D2
    description: "A failure puts everything back: a copy failure or a failed settings write leaves the old folder whole and the destination empty"
    requirement: "LIB-09"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/library_ops.rs#a_copy_that_fails_on_the_third_entry_removes_the_copies_and_leaves_the_source_whole"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/library_ops.rs#entries_renamed_before_a_failure_are_renamed_back"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/library_ops.rs#a_rollback_after_a_finished_copy_removes_the_copies"
        status: pass
    human_judgment: false
  - id: D3
    description: "Not empty, not writable, the same folder, nested either way, missing and relative destinations are refused and nothing changes"
    requirement: "LIB-09"
    verification:
      - kind: unit
        ref: "apps/sucrose/src/library_ops.rs#a_folder_inside_the_library_or_around_it_is_nested"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_destination_that_is_not_empty_is_refused_and_nothing_changes"
        status: pass
    human_judgment: false
  - id: D4
    description: "With Move off no confirmation is shown and only the location changes; the themes stay in the old folder"
    requirement: "LIB-09"
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#with_move_off_only_the_location_changes"
        status: pass
    human_judgment: false
  - id: D5
    description: "RuntimeState.library_moving is true for the whole move and Use is refused with Moving; the Portal disables the cards and the row"
    requirement: "LIB-09"
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/card.rs#a_library_move_disables_use_before_any_other_reason_but_the_connection"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/portal_link.rs#the_runtime_state_says_when_the_library_moves"
        status: pass
    human_judgment: false
  - id: D6
    description: "The destructive confirmation, the folder picker, the inline sentences and the progress text look and behave as the UI-SPEC says"
    requirement: "LIB-09"
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/dialogs/move_library.rs#a_classic_install_adds_that_it_will_see_an_empty_library"
        status: pass
    human_judgment: true
    rationale: "The folder picker and the dialog need real input and a look at the window; the plan's human-check runs at the end of the phase on a synthetic library"

duration: 33min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 22: Library move Summary

**A library move that renames or copies-and-verifies every top-level entry, writes the new location only after verification, removes the old copy afterwards and rolls everything back on failure, driven by a closed-renderers state machine in the service and a destructive confirmation in the Portal**

## Performance

- **Duration:** 33 min
- **Started:** 2026-11-24T10:53:30Z
- **Completed:** 2026-11-24T11:26:00Z
- **Tasks:** 2
- **Files modified:** 12 (1 created)

## Accomplishments

- `library_ops` gets the D-07 move: `check_destination` (resolved paths: missing, same, nested both ways, not empty, not writable), `move_library` (rename each entry, copy and compare file count and size when rename fails, full rollback of renamed and copied entries), `finish_move` and `rollback_move`. 12 new unit tests with injected rename and copy failures, read-only files and links.
- The service handles `MoveLibrary`: it closes the renderers (and ends the ones that ignore Close after 4 s), moves on the library-ops worker, writes `Library.Location`, removes the old copy on the worker, restarts the wallpaper from the library the settings name and only then answers `Moved`. With Move off only the location is written (`Changed`). `library_moving` is published for the whole move and Use is refused with `Moving`.
- The Portal: a folder picker with background quick checks and inline sentences, the destructive Move Library confirmation (with the classic-app sentence when that install's `Setting` folder exists), the progress text with the row and the library cards disabled, a reload from the new folder, and an error toast on failure. FEATURES records the parity and the improvements.

## Task Commits

1. **Task 1 (tracer, TDD):** RED `75c74f2a` (test, move tests with stubs), GREEN `7fb7f372` (feat, library_ops), RED `3ebbf41e` (test, service tests), GREEN `24f2754d` (feat, service wiring)
2. **Task 2: Folder picker, confirmation dialog and progress state** - `25d98e9d` (feat)

**Plan metadata:** the docs commit that carries this summary (docs: complete plan)

## Files Created/Modified

- `apps/sucrose/src/library_ops.rs` - the move, checks, rollback and file-operation trait
- `apps/sucrose/src/service.rs` - `LibraryMove` state machine, `MoveEnv`, `resume_wallpaper`, guards while moving, `MOVE_TIMER`
- `apps/sucrose/src/portal_link.rs` - `runtime_state` takes `library_moving`
- `apps/sucrose/tests/portal.rs` - three service tests
- `apps/sucrose-portal/src/settings/general.rs` - picker, prechecks, note, progress, move result handling
- `apps/sucrose-portal/src/dialogs/move_library.rs` - confirmation (new)
- `apps/sucrose-portal/src/app.rs`, `library/card.rs`, `i18n.rs`, `locales/sucrose.en.yml`, `dialogs/mod.rs` - wiring, `DisabledReason::Moving`, five new keys
- `docs/FEATURES.md` - the move row (CRLF kept, no final newline)

## Decisions Made

See `key-decisions`. The one that goes beyond the plan text: Move off accepts a folder that already holds files, like the legacy app, so `check_location` exists next to `check_destination`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Guards that keep a move from being cut short or racing it**
- **Found during:** Task 1 (service wiring)
- **Issue:** Nothing stopped a monitor change, a shell restart, a fullscreen tick or the exit rule from starting a renderer on the old folder, or the service from exiting, in the middle of a move; a renderer ignoring Close would keep `library_moving` true forever.
- **Fix:** Those paths wait while `fleet.library_move` is set, the tray Exit is ignored meanwhile, and `MOVE_TIMER` (4 s) ends the renderers that did not close.
- **Files modified:** apps/sucrose/src/service.rs
- **Committed in:** 24f2754d

**2. [Rule 3 - Blocking] `runtime_state` and `current_runtime` gained a `library_moving` argument**
- **Found during:** Task 1
- **Issue:** The plan 06/09 interfaces hard-coded `library_moving: false`.
- **Fix:** The flag is read from `Fleet.library_move`; the two portal_link tests were updated and one added.
- **Files modified:** apps/sucrose/src/portal_link.rs, apps/sucrose/src/service.rs
- **Committed in:** 24f2754d

**3. [Rule 1 - Bug] Test helper looped forever on Memory events**
- **Found during:** Task 1 (RED run of the service tests)
- **Issue:** `move_library` in the integration test read events without a deadline; the service sends a Memory event every second, so a missing answer hung the run.
- **Fix:** A 10 s deadline like `wait_for_tray`.
- **Files modified:** apps/sucrose/tests/portal.rs
- **Committed in:** 3ebbf41e

**4. [Rule 2 - Missing Critical] New localization keys**
- **Found during:** Task 2
- **Issue:** The legacy "Access" text reads "you dont have access"; the UI-SPEC wants "don't". Nested, missing, busy and failed-move sentences did not exist.
- **Fix:** Five keys in `sucrose.en.yml` and `REQUIRED_KEYS` (NoAccess, Nested, Missing, Busy, MoveFailed); the legacy Access key stays unused.
- **Files modified:** apps/sucrose-portal/locales/sucrose.en.yml, apps/sucrose-portal/src/i18n.rs
- **Committed in:** 25d98e9d

---

**Total deviations:** 4 auto-fixed (2 missing critical, 1 blocking, 1 bug)
**Impact on plan:** All needed for correctness; no scope creep, no protocol change.

## Issues Encountered

- A crash-restart that the supervisor scheduled just before a move could still start a renderer on the old folder (rename then fails on Windows and the copy path is used, which leaves the old copy for a later cleanup). Left as is; nothing is lost.
- A link nested inside a copied theme folder is removed with that folder by `finish_move` (links are never copied). A top-level link stays in the old folder.
- `LoopEvent::Quit` (the OS or the exit rule firing before a move began) is not delayed; only the tray Exit and the exit rule are.

## Known Stubs

None.

## Threat Flags

None. The new surface is the MoveLibrary path text, already in the plan's threat model (T-03-51, T-03-52, T-03-53): absolute, within `MAX_PATH_TEXT`, no `..`, resolved for the nesting check.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- LIB-09 is done in code. The plan's human-check (move a synthetic library from the release Portal, then a non-empty folder) is left for the end of the phase; the maintainer's real library is never used.
- The Portal's picker and dialog were not looked at in a window in this plan (unit tests and clippy only).

## Self-Check: PASSED

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
