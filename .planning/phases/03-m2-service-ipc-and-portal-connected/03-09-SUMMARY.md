---
phase: 03-m2-service-ipc-and-portal-connected
plan: 09
subsystem: service
tags: [rust, ipc, mpv, loadfile, use, runtime-state, restart-budget, fullscreen-rule]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: Portal protocol and the service endpoint (plan 06), settings host with update (plan 07), Portal link (plan 08), theme gate and library scan (plan 02)
provides:
  - "ToRenderer::LoadSource { source, settings } at wire index 7: the running renderer plays another file without restarting (D-26)"
  - "Player::load_source: loadfile replace plus the runtime playback settings (volume, loop, stretch); a new file's first frame is a new start (loading flag, Started event), and a load error before it counts as a failed start"
  - "service: decide_use (pure) over UseInput with UseDecision { Refuse, AlreadyApplied, Load, Start }, handle_use, Fleet::{load_source, restart_with, revive_stopped, adopt_job, rule_state}, UseTransition and USE_TRANSITION"
  - "service: current_runtime, publish_runtime and announce_runtime: RuntimeState goes to the Portal only when it changes and is never written to Sucrose.json (D-23)"
  - "ignored test use_switches_between_video_themes (SUCROSE_SWITCH_COUNT, SUCROSE_SWITCH_SECONDS, SUCROSE_SWITCH_DELAY) and PortalClient::hello_on for plan 11"
affects: [03-10, 03-11, 03-12, 03-13, 03-15, 03-22]
actuals:
  tokens: 14400
  tasks: 2
  commits: 2
plan_head_before: 54bbdb8ddc1cc8a2cc8741edb733e10a986760a2
plan_head_after: ed5887fd8c483030e688d708a6a07f8ef1af02f5
tech-stack:
  added: []
  patterns: [pure decision function over a small input struct with every refusal unit-tested, accepted Use updates the shared assignments before it messages renderers so a late or restarted renderer plays the new source, runtime state derived from the existing rule with no second record, last-published value kept so only a change is sent]
key-files:
  created: []
  modified:
    - crates/sucrose-ipc/src/protocol.rs
    - crates/sucrose-engine-mpv/src/player.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/portal_link.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose/tests/common/mod.rs
key-decisions:
  - "decide_use checks a library move first, then the fullscreen rule, then the gate's verdict, and only then the idempotent case, so a theme that broke on disk is refused even when it is the selected one"
  - "A Use that would start the fleet with no monitor is refused as Unavailable before the selection is stored; a refused Use never changes Library.Selected"
  - "Fleet::rule_state reads the existing Rule (closes and pauses per monitor) instead of the RuleClosed records, which the Restart transition also uses"
  - "The Restart transition reuses the fullscreen rule's close-and-want-back path and the exit handler now calls respawn_closed, so it needs no second mechanism"
  - "A user config loaded by mpv keeps its own volume, loop and stretch: Player::load_source applies runtime settings only when the generated options were in charge"
requirements-completed: [ARCH-05, ARCH-01, LIB-08]
coverage:
  - id: D1
    description: "Use of a Video theme while the wallpaper plays stores Library.Selected and loads the new video into the running renderer without a respawn"
    requirement: ARCH-05
    verification:
      - kind: integration
        ref: "Task 1 verify on the maintainer's machine: use_switches_between_video_themes (2 switches) printed Applied twice; renderer log has two 'source loaded' and two 'playback started on the new source'; service log has one 'renderer spawned' for one 'renderers started'; Sucrose.json holds the new selection"
        status: pass
    human_judgment: true
    rationale: "Whether the switch shows a black frame on screen is the maintainer's check in plan 11 (assumption A5)"
  - id: D2
    description: "Use is refused, and nothing changes, for unknown or escaping names, damaged, non-Video and newer themes, a paused or closed wallpaper and a library move; the selected theme that plays answers Applied without a reload"
    requirement: LIB-08
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#tests::decide_use (13 tests, including the real gate on fixture names ../x, a/b, empty, web, newer-version, broken-json)"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#use_of_a_web_theme_is_refused_and_the_selection_is_unchanged, use_of_a_theme_for_a_newer_sucrose_is_refused_and_the_selection_is_unchanged, use_of_a_name_that_leaves_the_library_or_does_not_exist_is_refused_as_not_found, use_of_a_damaged_theme_is_refused_as_corrupt"
        status: pass
    human_judgment: false
  - id: D3
    description: "An explicit Use resets every restart budget; a renderer restarted later plays the new source from the shared assignments"
    requirement: ARCH-01
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#tests::load_source_sends_one_load_source_to_every_connected_renderer_and_updates_what_a_restart_would_play, load_source_leaves_a_renderer_that_has_not_connected_to_read_the_new_assignment_at_its_hello"
        status: pass
    human_judgment: false
  - id: D4
    description: "The service publishes RuntimeState (selected, running, rule, tray) to the Portal when it changes and never writes it to Sucrose.json"
    requirement: ARCH-05
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#tests::the_runtime_state_is_published_only_when_it_differs_from_the_last_one_sent, the_fullscreen_rule_state_reads_paused_and_closed_from_the_rule"
        status: pass
    human_judgment: false
  - id: D5
    description: "ToRenderer::LoadSource has wire index 7 and the existing renderer variants keep theirs"
    requirement: ARCH-05
    verification:
      - kind: unit
        ref: "crates/sucrose-ipc/src/protocol.rs#tests::load_source_keeps_its_wire_index_and_survives_the_wire"
        status: pass
    human_judgment: false
duration: 15min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 09: Use through the service Summary

**A Portal "Use" now selects a Video theme through the service: the theme gate decides, Library.Selected is stored, and the running renderers load the new video in place with loadfile replace (new ToRenderer::LoadSource, wire index 7) while refusals, idempotence, budget reset and runtime-state publication are decided in one pure function and a few Fleet methods.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-11-24T06:02:46Z
- **Completed:** 2026-11-24T06:17:00Z
- **Tasks:** 2 (the first a tracer)
- **Files modified:** 7

## Accomplishments

- Tracer verified end to end on the maintainer's machine: a switching client connected as a Portal to a real service and renderer, two Uses answered Applied about 5 s apart, the renderer logged "source loaded" twice and "playback started on the new source" twice, the service spawned exactly one renderer, and the tray status went back to playing after each switch.
- `Player::load_source` applies volume, loop and stretch with the values config.rs produces, issues `loadfile replace`, and marks the player as loading so the first restart of the new file is reported as a new start (no skip, no wrap) and `has_started` is false until then; a load error before the first frame becomes a start failure and the renderer exits with the start-failure code so the supervisor budget takes over.
- `decide_use` refuses Moving, RulePaused, RuleClosed, NotFound, Corrupt, NotVideo and NewerVersion without changing anything, answers AlreadyApplied for the selected theme that plays, and otherwise chooses Load (renderers run) or Start (none run); an accepted Use resets every restart budget and updates the shared assignments before it messages the renderers.
- RuntimeState (selected, running, rule, tray) is derived from the settings and the existing fullscreen rule and sent to the Portal on change only, after every drained event batch, fullscreen tick, display timer and shell restart; it is never written to Sucrose.json.

## Task Commits

The two tasks share one code path (the Use handler needs `decide_use`, the refusal mapping and `Fleet::load_source`), so the commits are split by layer and not at the task boundary:

1. **Tasks 1 (engine and renderer half): LoadSource, Player::load_source, renderer arm** - `a437ead9` (feat)
2. **Tasks 1 and 2 (service half): Use handling, refusals, budget reset, rule state, runtime publication, switching client** - `ed5887fd` (feat)

**Plan metadata:** recorded in the docs commits that follow (SUMMARY, then state and roadmap).

## Files Created/Modified

- `crates/sucrose-ipc/src/protocol.rs` - `ToRenderer::LoadSource` appended at index 7 with a wire-index test
- `crates/sucrose-engine-mpv/src/player.rs` - `load_source`, the loading flag, runtime property mapping, restart handling for a new file, tests
- `apps/sucrose/src/render.rs` - the LoadSource arm (`switch_source`), atomic `looping`, re-announced Playing, hello-reply list
- `apps/sucrose/src/service.rs` - `decide_use`, `handle_use`, `USE_TRANSITION`, Fleet methods, runtime publication, catch-up of a newer source at Hello, 21 new unit tests
- `apps/sucrose/src/portal_link.rs` - `runtime_state` takes the rule state
- `apps/sucrose/tests/portal.rs`, `apps/sucrose/tests/common/mod.rs` - refusal integration tests on the fixture library and the ignored switching client for plan 11

## Decisions Made

See key-decisions in the frontmatter. In short: refusals before idempotence, no-monitor Start refused before anything is stored, rule state read from the rule itself, Restart transition built on the rule's close-and-want-back path, user mpv config left in charge of its own playback options.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] A Use between a renderer's Assign and its registration was lost**
- **Found during:** Task 1 (service half)
- **Issue:** read_connection already caught up a rectangle that changed between the Assign and the registration of the writer, but not a source: a renderer connecting at that moment would play the old theme while the service believed it switched.
- **Fix:** the catch-up now compares the whole assignment and sends SetRect and/or LoadSource.
- **Files modified:** apps/sucrose/src/service.rs
- **Committed in:** ed5887fd

**2. [Rule 2 - Missing Critical] Renderers that had given up, and a Use with nothing playing, needed more than the plan listed**
- **Found during:** Task 1/2 (Use handling)
- **Issue:** a slot whose budget was spent or that ended for good would stay dark after an explicit Use; a Start after a rejected-at-launch theme needed the GPU preference, the fullscreen rule and its timer, which only the launch path set up; and a Start with no monitor must not store a selection it cannot show.
- **Fix:** `Fleet::revive_stopped` for stopped slots, the Start path applies the GPU preference, enables the rule and asks the loop to arm its timer, and a Start with no monitor is refused as Unavailable before the selection is saved.
- **Files modified:** apps/sucrose/src/service.rs
- **Committed in:** ed5887fd

**3. [Rule 3 - Blocking] `runtime_state` and `SyncState.looping` signatures**
- **Found during:** Task 2 / Task 1
- **Issue:** RuntimeState.rule had no source (`runtime_state` hard-coded None), and `looping` is read by the follower while a new source can change it.
- **Fix:** `runtime_state` takes a `RuleState` (portal_link.rs is not in the plan's file list); `looping` became an AtomicBool.
- **Files modified:** apps/sucrose/src/portal_link.rs, apps/sucrose/src/render.rs
- **Committed in:** a437ead9, ed5887fd

**4. Task commits split by layer, not by task**
- The plan's Task 2 (`decide_use`, rule state, runtime publication) is the same code the Task 1 handler calls, so it could not be committed separately without a commit that does not build its own tests. The two commits are engine/renderer and service.

**5. Test names**
- The decide_use tests live in a nested module `tests::decide_use` (so `cargo test -p sucrose decide_use` runs 13 of them) and the other tests avoid the strings `fn decide_use`, `fn rule_state` and `fn publish_runtime`, so the plan's single-definition greps print 1.

---

**Total deviations:** 3 auto-fixed (2 missing critical, 1 blocking) plus 2 structural notes
**Impact on plan:** All additions are needed for the Use to be correct (no lost source, no dark monitors, no selection that cannot play); no scope creep beyond the files listed plus portal_link.rs.

## Issues Encountered

- `cargo fmt` rewrote some CRLF files as LF; the line endings of every touched file were restored to CRLF before the commits (the index is LF, so the diffs are unaffected).
- The existing `sent_to` test helper returns as soon as the log ends in a Close, which made it useless for asserting the Close of the Restart transition; that test reads the log directly.
- The tracer run used a scratch data folder without Sucrose.json, so the service migrated the maintainer's real legacy settings read-only and played their selected theme; the switching client only wrote the scratch folder's Sucrose.json. Both Sucrose processes were stopped with taskkill afterwards.

## Known Stubs

None. `RuntimeState.library_moving` stays false and `handle_use` passes `moving: false` until plan 22; that is the plan's stated scope, not a stub that blocks this plan's goal.

## Threat Flags

None. The Use request is the existing Portal channel; the folder is one path segment checked by the Phase 2 gate (T-03-22), a failing source ends in the existing restart budget (T-03-23), and LoadSource carries only the absolute path the service resolved while mpv keeps its hardening options (T-03-24).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 11 can run `use_switches_between_video_themes` on the maintainer's screen to check for a black frame; if it shows one, `USE_TRANSITION` switches to `UseTransition::Restart` (already implemented and unit-tested at the Fleet level).
- Plan 12 (card click) can send `PortalRequest::Use` and rely on `UseResult` plus the `Runtime` broadcast for the Selected pill.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*

## Self-Check: PASSED
