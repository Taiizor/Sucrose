---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 19
subsystem: service
tags: [master-clock, multi-monitor, libmpv, election, ipc]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: master clock, role_changes and per-slot renderers (plans 02-06 and 02-18)
provides:
  - PlayerEvent::Started and Player::has_started (first frame shown)
  - ToService::Playing, the clock and drift correction wait for the first frame
  - Fleet::playing set and an election among playing renderers only
  - choose_master, a sticky master policy with tests (D-030)
affects: [02-20 follower frame alignment, 02-21 measure]

actuals:
  tokens: 15000
  tasks: 3
  commits: 3
plan_head_before: 54fdea8b0efc7c11eb6775b5c85c6d874f2d1328
commits: 3
plan_head_after: 0dce7b88886b38146b0b8f6c4520f91533ca518a

tech-stack:
  added: []
  patterns:
    - "Election inputs are facts reported by verified connections (slot from the connection, never the message)"
    - "Pure policy function (choose_master) with the Fleet passing the present master in"

key-files:
  created: []
  modified:
    - crates/sucrose-engine-mpv/src/player.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/src/service.rs
    - docs/FEATURES.md
    - docs/DECISIONS.md

key-decisions:
  - "D-030: sticky master. A renderer counts only once its first frame shows; the first playing renderer, primary monitor first at start, keeps the master role while its monitor exists and it plays. Departs from the 02-CONTEXT rule 'the primary monitor, with another renderer promoted when the master's monitor goes away'."

requirements-completed: [DESK-07, DESK-01]

coverage:
  - id: D1
    description: "ToService::Playing, the published clock and drift following wait for the first frame (PlayerEvent::Started)"
    requirement: DESK-07
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-engine-mpv"
        status: pass
      - kind: other
        ref: "release run 12 s: Renderer log shows 'playback started' before 'role=Master' (Task 1 gate)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Election only among playing renderers (no master with nobody playing)"
    requirement: DESK-07
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#a_renderer_that_does_not_play_yet_is_never_the_master, with_no_renderer_playing_there_is_no_master"
        status: pass
    human_judgment: false
  - id: D3
    description: "Sticky master: plugging a monitor, even as the new primary, keeps the playing master; policy recorded in D-030 and FEATURES.md"
    requirement: DESK-01
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#plugging_a_monitor_keeps_the_playing_master, a_sticky_master_survives_a_new_primary_through_the_fleet"
        status: pass
      - kind: other
        ref: "release run 12 s: slot 1 played first and became master; slot 0 played 195 ms later and a single 'master clock elected' line was logged"
        status: pass
    human_judgment: true
    rationale: "The real plug of the virtual display (G-02-2c) needs the maintainer's desktop; verified at end of phase"

duration: continuation after the Task 2 decision, about 4 min of execution
completed: 2026-11-23
status: complete
---

# Phase 2 Plan 19: Master election waits for the first frame and stays sticky Summary

**A renderer is elected master only after its first frame, the clock and drift correction wait for it, and the master stays put when a monitor is plugged (sticky policy D-030, chosen by the maintainer).**

## Performance

- **Tasks:** 3 (Task 2 was the maintainer decision)
- **Files modified:** 5

## Accomplishments
- `PlayerEvent::Started` / `Player::has_started`; the renderer reports `Playing` from its clock thread on the first frame; `publish_clock` and `follow_clock` do nothing before it.
- `Event::Renderer { slot, message }` carries the slot of the verified connection; `Fleet.playing` drives the election, so a renderer that is not playing is never master and with nobody playing there is no master.
- `choose_master(current, playing)` replaces the stateless `elect_master`: the present master is kept while its monitor plays, otherwise the primary among the playing, else the lowest device name. No protocol change.
- Maintainer choice at the checkpoint: **sticky**. This departs from the 02-CONTEXT rule "the primary monitor, with another renderer promoted when the master's monitor goes away": after a primary-monitor change the master can sit on a non-primary monitor until its monitor goes away or it stops. D-030 records this explicitly; the FEATURES note reads "Master election (02-19)".
- Release evidence (12 s, scratch data): before Task 1 slot 1 played first, became master, and the stateless rule moved the master to slot 0 14 ms later; with sticky one `master clock elected old=None new=Some(1)` line appears and slot 0 (195 ms later) does not take the role.

## Task Commits

1. **Task 1 (RED): failing master tests** - `254e8aa9` (test)
2. **Task 1 (GREEN): first frame gates Playing, clock and election** - `88a4999f` (feat)
3. **Task 2: master policy decision (sticky)** - no commit (decision checkpoint)
4. **Task 3: choose_master, tests, D-030, FEATURES note** - `0dce7b88` (feat)

## Files Created/Modified
- `crates/sucrose-engine-mpv/src/player.rs` - PlayerEvent::Started, has_started
- `apps/sucrose/src/render.rs` - Playing and clock after the first frame
- `apps/sucrose/src/service.rs` - playing set, choose_master, tests
- `docs/DECISIONS.md` - D-030
- `docs/FEATURES.md` - Master election (02-19) note

## Decisions Made
D-030 (sticky master), see above.

## Deviations from Plan

### Auto-fixed Issues

**1. [Process] Task 3 tests and implementation in one commit**
- **Found during:** Task 3
- **Issue:** The plan marks Task 3 tdd="true"; the pure function and its tests were written together, so there is no separate RED commit for it. The failing-first discipline was kept for Task 1, whose RED commit exists.
- **Files modified:** apps/sucrose/src/service.rs
- **Committed in:** 0dce7b88

---

**Total deviations:** 1 (process only)
**Impact on plan:** none on behavior.

## Issues Encountered
None. `docs/DECISIONS.md` and `docs/FEATURES.md` are LF in the working tree (the planning note said CRLF); their line endings were kept as found. `service.rs` stays CRLF.

## Human check outstanding
The plan's end-of-phase human check (disable and re-enable the virtual display, confirm the monitor that kept playing does not seek back to the start and the service log shows no master move) needs the maintainer's desktop and is left to phase verification. It was not run here.

## Known Stubs
None.

## Next Phase Readiness
Ready for 02-20 (a follower that reaches the same frame). Master moves now happen only when the master's monitor goes away or its renderer stops.

## Self-Check: PASSED
- Commits 254e8aa9, 88a4999f, 0dce7b88 are on the branch; `cargo xtask ci` exits 0; `grep` counts: choose_master 1, Master election (02-19) 1, Master clock election 1, departs from the Phase 2 context rule 1.
