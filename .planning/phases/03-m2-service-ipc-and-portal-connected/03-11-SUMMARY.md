---
phase: 03-m2-service-ipc-and-portal-connected
plan: 11
subsystem: service
tags: [rust, mpv, ipc, use-transition, checkpoint]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: UseTransition::LoadSource, ToRenderer::LoadSource and the switching client test (plan 09)
provides:
  - "D-26 decision: Use between Video themes keeps UseTransition::LoadSource (no renderer restart), confirmed on the maintainer's machine"
  - "Evidence: ten switches on two monitors, ten source loads per renderer, no respawn, no Failed report"
  - "Memory evidence: 30 Uses between two 4K videos plateau at the playing video's level (docs/MEASUREMENTS.md)"
affects: [03-12]
actuals:
  tokens: 800
  tasks: 3
  commits: 2
plan_head_before: 0e950f60
plan_head_after: 2d07055aa06abc57f832818ca6f5776f5799a601
tech-stack:
  added: []
  patterns: [checkpoint run on a scratch data folder with the release build, decision recorded in the feature row]
key-files:
  created: []
  modified:
    - docs/FEATURES.md
    - docs/MEASUREMENTS.md
key-decisions:
  - "Keep UseTransition::LoadSource for Use between Video themes; no hybrid memory-threshold restart (maintainer, 2026-11-24, D-26)"
requirements-completed: [ARCH-05]
coverage:
  - id: D1
    description: "Use between two Video themes on the release build shows no flicker, resize jump or noticeable black frame, and icons stay visible (maintainer observation)"
    requirement: ARCH-05
    verification: []
    human_judgment: true
    rationale: "Visual smoothness of a switch on real monitors is only judged by the maintainer; the renderer logs show ten loads per renderer and no respawn but cannot show a frame"
  - id: D2
    description: "The renderers keep running across ten Uses (ten source loaded lines per renderer, two renderer spawns in total for two monitors, ten theme used lines with transition=LoadSource)"
    requirement: ARCH-05
    verification:
      - kind: other
        ref: "target/scratch-data-run2/Log (Service and Renderer logs) and target/scratch-data-run2/switch.txt"
        status: pass
    human_judgment: false
  - id: D3
    description: "Renderer memory does not grow across 30 LoadSource Uses between two 4K videos"
    verification:
      - kind: other
        ref: "docs/MEASUREMENTS.md, M2 Use between Video themes (target/run3/samples-30.csv)"
        status: pass
    human_judgment: false
duration: ~1h (approximate, checkpoint waits included)
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 11: Use transition check (D-26) Summary

**LoadSource stays the Use transition between Video themes: the maintainer watched ten release-build switches on two monitors and saw no flicker, no resize jump and no noticeable black frame, with the renderers never restarted.**

## Performance

- **Duration:** about 1 h (approximate; the run, a failed first run and a memory measurement are included)
- **Completed:** 2026-11-24
- **Tasks:** 3
- **Files modified:** 2 (docs only; no code changed)

## Accomplishments

- Task 1: the release build `sucrose.exe --no-portal` played from `target/scratch-data-run2`; the switching client alternated two Video themes ten times, 10 s apart (06:52:24Z to 06:53:54Z, all ten results `Applied`).
- Task 2: the maintainer chose **loadsource** (no hybrid memory-threshold restart).
- Task 3: the decision and its evidence are in the Video row of `docs/FEATURES.md`; `USE_TRANSITION` is unchanged (`UseTransition::LoadSource`).

## Evidence

- **Maintainer observation (run 2):** clean, or a black frame too short to notice; icons visible; no flicker or resize jump. Decision reply: "loadsource".
- **Switch timestamps (switch.txt, UTC):** use 0 at 06:52:24.277, then every 10 s, use 9 at 06:53:54.429, done at 06:54:04.430.
- **Service log:** 10 `theme used ... transition=LoadSource renderers=2` lines; 2 `renderer spawned` lines (the initial start of two monitors; none after); no Failed report.
- **Renderer logs:** 10 `source loaded` lines in each of the two renderer logs (11 `playback started` in renderer 0: start plus ten loads); only libmpv's harmless `UDTA parsing failed retrying raw` warnings.
- **Why the first run was invalid:** run 1 was disturbed by the maintainer's own debug Portal on the real data folder, which started a second service. Run 2 is the valid run. DISPLAY9 is the maintainer's intentional virtual display.
- **The maintainer's RAM concern:** the C# app restarts on every Use to free memory. Measurement a67d49ef answered it: 30 Uses between two 4K videos show a plateau that depends on the playing video, within about 1 MB per Use, memory returns when switching back to the smaller video, and the level matches a fresh start. **Limitation:** two videos only; a library of many differently sized videos and the C# app were not measured. The D-25 measurement (plans 03-24 and 03-25) re-checks resource use.

## Task Commits

1. **Task 1: maintainer watches ten switches** - no commit (observation task; evidence untracked in `target/`)
2. **Task 2: decision (decision support: memory measurement)** - `a67d49ef` (docs)
3. **Task 3: record the decision** - `2d07055a` (docs)

**Plan metadata:** the docs(03-11) completion commit.

## Files Created/Modified

- `docs/FEATURES.md` - Video row note: LoadSource without a restart, checked 2026-11-24 (D-26) (CRLF, no final newline kept)
- `docs/MEASUREMENTS.md` - 30-Use renderer memory measurement (commit a67d49ef)

## Decisions Made

Keep LoadSource without a restart and without a hybrid memory-threshold restart; revisit only if the D-25 measurement shows a resource problem.

## Deviations from Plan

None - plan executed exactly as written. The extra memory measurement was run at the maintainer's request as decision support and is recorded in MEASUREMENTS.md.

## Issues Encountered

- Run 1 invalid (second service started by the maintainer's debug Portal on the real data folder); repeated as run 2.

## Known Stubs

None.

## Threat Flags

None.

## Next Phase Readiness

Plan 12 (Use from a card) can start; the transition is decided. No sucrose.exe was left running (`tasklist.exe` shows none).

## Self-Check: PASSED

- docs/FEATURES.md contains D-26; `const USE_TRANSITION` present in apps/sucrose/src/service.rs; `cargo test -p sucrose decide_use` passes (13 tests).
- Commits a67d49ef and 2d07055a exist on develop-rust; no sucrose.exe running.
