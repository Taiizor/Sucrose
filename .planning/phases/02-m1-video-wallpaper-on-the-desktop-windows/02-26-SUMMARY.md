---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 26
subsystem: service
tags: [spike, uat, display-change, progman, location-watch, g-02-7]
requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "Checked surface placement (02-23), the location watch (02-24) and the re-place-all rule (02-25), all running in the instrumented release build"
provides:
  - "A maintainer-run spike of UAT test 7(c) and an origin-only change with SUCROSE_LOG=debug, with the evidence classified (a) only"
  - "The spike entry 'Spike (02-26)' in the debug session real-screen-static-wallpaper-after-primary-switch (on disk only, .planning/debug/ is gitignored)"
  - "The maintainer's decision 'wait': G-02-7 is an accepted limitation, not a closed gap"
affects: [G-02-7, docs/FEATURES.md]
actuals:
  tokens: 2600
  tasks: 3
  commits: 0
plan_head_before: 032bfc32fd6b572eb0bb6f44a71551c8baf40cd2
plan_head_after: 032bfc32fd6b572eb0bb6f44a71551c8baf40cd2
tech-stack:
  added: []
  patterns: []
key-files:
  created: []
  modified:
    - .planning/debug/real-screen-static-wallpaper-after-primary-switch.md
key-decisions:
  - "G-02-7: wait for explorer and accept the gap (option A); the maintainer's reply 'wait' is the sign-off. No watch fix, because the location watch missed no parent move (class c not seen)."
requirements-completed: [DESK-07, DESK-01]
coverage:
  - id: D1
    description: "The instrumented release build ran through UAT test 7(c) and an origin-only change; every display change has its applied layout, per-surface Progman rectangle, verdict, re-place and Hidden-episode length"
    requirement: DESK-07
    verification:
      - kind: manual_procedural
        ref: "Spike (02-26) entry in .planning/debug/real-screen-static-wallpaper-after-primary-switch.md (4 display changes, 8 of 8 Progman moves caught by the watch)"
        status: pass
    human_judgment: true
    rationale: "What the monitors showed during each change is the maintainer's observation; the timeline comes from the debug log"
  - id: D2
    description: "The evidence is classified (a) explorer keeps up; (b), (c) and (d) did not occur"
    requirement: DESK-07
    verification:
      - kind: manual_procedural
        ref: "Spike (02-26) classification; every outside episode ended within 2.11 s"
        status: pass
    human_judgment: true
    rationale: "Classification (a) versus (b) rests on a 3 s threshold and the maintainer's judgment that the gap is normal"
  - id: D3
    description: "The maintainer's decision on G-02-7 is recorded as an accepted gap with a verbatim sign-off"
    requirement: DESK-01
    verification: []
    human_judgment: true
    rationale: "Accepting a limitation is the maintainer's call"
duration: "interactive (run window 17:20Z to 17:30Z)"
completed: 2026-11-23
status: complete
---

# Phase 2 Plan 26: Spike the real-screen display-change behaviour Summary

**A maintainer-run spike of UAT test 7(c) and an origin-only change on the instrumented release build showed explorer catches up within about 2 s after every display change (class (a) only), and the maintainer chose to wait for explorer and accept the gap.**

Decision: wait
Watch missed a parent move (class c): no
G-02-7 after 02-26: open
G-02-7 sign-off: "wait" (the maintainer's verbatim reply choosing option A, whose text states that choosing it is the sign-off on the limitation)
Next step: `/gsd-plan-phase 2 --gaps` writes the plan for the FEATURES limitation note (no watch fix, since class (c) was not seen); that plan lists no gap_ids.

G-02-7 stays open as an accepted gap. It is not closed here, and 02-UAT.md is not edited.

## Performance

- **Run window:** 17:20:50Z (start) to 17:30:36Z (app stopped); the maintainer's clock is UTC+3
- **Tasks:** 3 (1 maintainer run, 1 evidence entry, 1 decision)
- **Files modified:** 1 on disk (the debug file, not tracked)
- **Code commits:** 0 (no production code in this plan)

## Accomplishments

- The release build with 02-23, 02-24 and 02-25 ran with SUCROSE_LOG=debug on the maintainer's desktop through UAT test 7(c) and an origin-only change; the executor changed no display setting.
- The evidence is classified (a) only: every outside episode ended within 2.11 s; (b) no; (c) no, the location watch caught 8 of 8 Progman moves; (d) no.
- The spike entry answers the debug session's blind spot on Progman's rectangle: explorer moves Progman on a primary switch and on an origin-only change, about 1.7 to 2.1 s after the change.
- The maintainer decided "wait" with that evidence.

## Timeline (UTC)

Start 17:20:50: both slots Shown, Progman [0,0,3840,1080]; slot 1 (DISPLAY9) elected master at 17:20:53.

| # | Time | Change | Progman after | Episode | Ended |
|---|------|--------|---------------|---------|-------|
| 1 | 17:23:15.76 | Scenario A: DISPLAY9 made main | [-1920,0,1920,1080] | slot 1 Hidden 1.82 s | 17:23:17.58 |
| 2 | 17:25:32.00 | DISPLAY1 made main again | [0,0,3840,1080] | slot 0 Hidden 1.73 s | 17:25:33.73 |
| 3 | 17:26:49.54 | Scenario B: virtual display dragged left | [-1920,0,1920,1080] | slot 1 Hidden 2.11 s | 17:26:51.64 |
| 4 | 17:27:56.26 | Dragged back right | [0,0,3840,1080] | slot 1 Hidden 2.05 s | 17:27:58.30 |

In each change the service applied the layout 1.7 to 2.1 s after explorer's move and re-placed both surfaces; every slot ended Shown.

## Maintainer's observations

Video on both monitors, in sync. On the primary switch and at every origin change the video on the virtual display (or perhaps one screen) went away and came back, judged normal; layout restored; overall pass. The maintainer gave qualitative observations only.

## UAT test 7(c) observations

- Master elected once at start (slot 1, DISPLAY9); no second election after the primary switch; slot 1 kept playing as master through its Hidden episode.
- No seek on either renderer: 19 drift summaries, seeks=0.
- Follower max drift 1.99 to 4.33 frames (about 72 ms) in the 17:23:15 to 17:25:32 window.
- Largest drift of the run: 15.02 frames (about 250 ms) at 17:21:46.99, before any display change, with no seek.

## Caveat

In this run Windows applied one layout per display change. The UAT's two-step B-then-C sequence did not occur, so the inferred class (b) link (explorer keeping Progman at B during C) was neither confirmed nor refuted. The run confirmed that explorer moves Progman on an origin-only change.

## Task Commits

No task produced a commit: Task 1 was resolved by the maintainer's run, Task 2's entry lives in .planning/debug/ (ignored by the `[Dd]ebug/` rule in .gitignore, so it is on disk only and was not force-added), and Task 3 was resolved by the maintainer's reply.

**Plan metadata:** the docs(02-26) commit that carries this SUMMARY and the tracking files.

## Files Created/Modified

- `.planning/debug/real-screen-static-wallpaper-after-primary-switch.md` - the "Spike (02-26)" entry with timeline and classification (untracked by design)

## Decisions Made

- G-02-7: option A, wait for explorer and accept the gap. The maintainer's verbatim reply "wait" is the sign-off. The fallback window and the watch fix are not built.

## Deviations from Plan

None - plan executed exactly as written.

## Authentication Gates

None.

## Issues Encountered

None. The B-then-C two-step sequence did not occur (see Caveat).

## Known Stubs

None.

## Next Phase Readiness

`/gsd-plan-phase 2 --gaps` writes the plan for the FEATURES limitation note. Checked: `gap_ids: []` in 02-23, 02-24, 02-25 and 02-26 (four times).

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-23*
