---
phase: 03-m2-service-ipc-and-portal-connected
plan: 34
subsystem: engine
tags: [rust, libmpv, player, gap-closure, wr-08, tdd]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: Player::load_source and the D-26 LoadSource transition (03-11), the renderer's Started-driven Playing report and clock anchor (D-030)
provides:
  - "SourceSwitch and Restart in sucrose-engine-mpv player.rs: a pure state machine that takes a playback restart for the new file's first frame only after the FILE_LOADED event of the latest load_source request"
  - "Shared.load_generation: counts load_source requests, bumped before loadfile is issued"
  - "Ignored real-libmpv test load_source_reports_started_for_the_new_file"
affects: [03-verification, arch-01, arch-05]
actuals:
  tokens: 3200
  tasks: 2
  commits: 3
plan_head_before: a34a078334b03b3ce2c815e87a41bd35cd3bfdeb
plan_head_after: eecceea205395f907a97d6467d637a179102147c
tech-stack:
  added: []
  patterns:
    - "Event-thread decisions that depend on event order are a pure state machine fed with generations, unit-tested with event sequences"
key-files:
  created: []
  modified:
    - crates/sucrose-engine-mpv/src/player.rs
    - crates/sucrose-engine-mpv/tests/options.rs
key-decisions:
  - "The new file is attributed to a load request by the generation read at its START_FILE event, because mpv_command does not return the playlist entry id of loadfile"
  - "A restart that arrives before the new file loaded is an ordinary restart of the old file (note_restart), and has_started stays false until the new file shows its first frame"
requirements-completed: [ARCH-01, ARCH-05]
coverage:
  - id: D1
    description: "A restart of the old file that mpv queued before a load_source (loop wrap, finished seek) is never taken for the new source's first frame"
    requirement: ARCH-05
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-mpv/src/player.rs#source_switch_ignores_a_restart_queued_before_the_new_file_loaded"
        status: pass
    human_judgment: false
  - id: D2
    description: "Loading the same path twice gives one Started per load; two quick loads give one Started, for the last file"
    requirement: ARCH-01
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-mpv/src/player.rs#source_switch_reports_one_start_per_load_of_the_same_path"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-mpv/src/player.rs#source_switch_starts_only_the_last_of_two_quick_loads"
        status: pass
    human_judgment: false
  - id: D3
    description: "The real libmpv still reports Started for a file loaded with load_source, and the service's Use tests are unchanged"
    requirement: ARCH-05
    verification:
      - kind: integration
        ref: "crates/sucrose-engine-mpv/tests/options.rs#load_source_reports_started_for_the_new_file"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/service.rs#of_the_selected_theme_that_plays_reloads_nothing"
        status: pass
    human_judgment: false
duration: 7min
completed: 2026-11-24
status: complete
---

# Phase 03 Plan 34: Source start tied to the new file Summary

**WR-08 fixed: a SourceSwitch state machine makes the player report Started for a load_source only on the first playback restart after that request's own FILE_LOADED event, so a queued loop-wrap restart of the old video no longer passes for the new wallpaper's first frame.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-11-24T15:15:00Z
- **Completed:** 2026-11-24T15:22:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- `SourceSwitch` (pure, `requested` / `file_loaded` / `restart`) decides `Restart::Old` or `Restart::NewStart`; four sequence tests cover a restart queued before the load, a same-path reload, two quick loads and no pending load.
- `event_loop` reads `Shared.load_generation` before every event, records the generation at `START_FILE`, feeds `FILE_LOADED` to the switch and asks it at every `PLAYBACK_RESTART`. `NewStart` clears `loading`, takes a fresh anchor and sends `Started`; `Old` goes through `note_restart`.
- Real libmpv: the ignored suite (11 tests) passes with a new test that loads a scratch image through `load_source` and checks that `Started` arrives for it and the `path` property names it.
- ARCH-05 idempotency unchanged: `of_the_selected_theme_that_plays_reloads_nothing` and `..._does_not_play_starts_it` pass. `cargo xtask ci` passes.

## Task Commits

1. **Task 1: SourceSwitch to event loop to has_started (TDD)**
   - RED: `98ea095d` (test) - a stub that keeps today's behavior; three of the four tests fail on their assertions (`[NewStart, Old]` instead of `[Old, NewStart]`), the no-pending test passes by design
   - GREEN: `1c312195` (fix) - the real state machine and the event-loop wiring
2. **Task 2: real-libmpv coverage** - `eecceea2` (test)

**Plan metadata:** the docs commit that carries this file.

## Files Created/Modified

- `crates/sucrose-engine-mpv/src/player.rs` - `SourceSwitch`, `Restart`, `Shared.load_generation`, event loop wiring, the four `source_switch_*` tests
- `crates/sucrose-engine-mpv/tests/options.rs` - ignored `load_source_reports_started_for_the_new_file` (a 42-byte GIF written to the temp folder)

## Decisions Made

- The load request a loaded file belongs to is the generation read at its `START_FILE`: `mpv_command` does not return the playlist entry id that `loadfile` creates and no binding for `mpv_command_ret` exists. The generation is bumped before the command, so it is never older than the request. The remaining race is documented in the event loop: when another `load_source` lands between a `loadfile` and the event thread reading its `START_FILE`, the first file's loaded event counts for the newer request.
- The `!started` branch (the very first frame) also clears `loading` when the switch says the restart is the new file's, so a `load_source` made before the first frame cannot leave `has_started` false forever.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] load_source before the first frame left loading set**
- **Found during:** Task 1 (wiring the event loop)
- **Issue:** the unchanged `!started` branch never cleared `loading`, so a `load_source` issued before the first-ever playback restart kept `has_started` false for good.
- **Fix:** that branch asks the switch and clears `loading` on `NewStart`; otherwise it is as before.
- **Files modified:** crates/sucrose-engine-mpv/src/player.rs
- **Committed in:** 1c312195

**2. [Rule 3 - Blocking] The real-libmpv test lives in tests/options.rs**
- **Found during:** Task 2
- **Issue:** the plan lists only player.rs, but the crate's ignored real-libmpv tests (and their helpers: `start`, `wait_until`, the fetched DLL loader) are in `tests/options.rs`, and none called `load_source`. `load_source` accepts only absolute file paths, and the repository has no small video fixture, so the test writes a 1x1 GIF to the temp folder instead of using a fixture video.
- **Files modified:** crates/sucrose-engine-mpv/tests/options.rs
- **Committed in:** eecceea2

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** no scope creep; both stay inside the sucrose-engine-mpv crate.

## Issues Encountered

- Edits made through the edit tool turned player.rs and options.rs from CRLF to LF once; the working-tree CRLF was restored with sed before each commit (`git ls-files --eol` shows `i/lf w/crlf` for both, and the diffs contain only the intended lines).
- The RED evidence is a plain `cargo test` run: the format-based classifier does not support Rust test output (a known gap in the TDD reference), so the semantic assessment was done by hand: the target test ran and failed on the planned assertion.
- The race described under Decisions is not closed by a test; a real-libmpv test of a loop wrap racing a load needs a timing-dependent fixture and is not part of this plan.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- WR-08 is fixed; the maintainer's manual check of Use between Video themes (03-11) is the remaining confirmation on a real library.

## Self-Check: PASSED

- FOUND: crates/sucrose-engine-mpv/src/player.rs, crates/sucrose-engine-mpv/tests/options.rs
- FOUND commits: 98ea095d, 1c312195, eecceea2 (ancestors of HEAD)
- Acceptance: `MPV_EVENT_FILE_LOADED` appears once, `struct SourceSwitch` once; four source_switch tests ok; ignored libmpv run 11 passed; of_the_selected_theme tests ok; clippy clean; cargo xtask ci exit 0
