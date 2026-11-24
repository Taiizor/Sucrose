---
phase: 03-m2-service-ipc-and-portal-connected
plan: 24
subsystem: testing
tags: [measurement, xtask, portal, memory, sysinfo, d-25, ui-07]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "The release Portal and service (plans 10-23), the 500-theme synthetic library and prepared settings (plan 05), LibraryPreview keys (plan 19)"
provides:
  - "cargo xtask measure --side portal (per-pid time series of working set and private bytes), --portal-summary (first page, round end and peak, peak, final) and --portal-row (MEASUREMENTS.md row)"
  - "Side::Portal, portal_summary, PortalSummary in xtask/src/measure.rs"
  - "Untracked sample files target/measure/portal-real.json (240 s) and target/measure/portal-synth.json (400 s) with their summaries"
  - "Finding: hover previews do not animate in the release Portal, so neither scenario measured the preview cost"
affects: [03-25, ui-07, meas-02]

actuals:
  tokens: 8000
  tasks: 3
  commits: 2
plan_head_before: 410be6d951c49c386573f6dd1b8450b17c7ded74
plan_head_after: a560014a2d0d89584f4d028f1dc5a205510c4138

tech-stack:
  added: []
  patterns:
    - "Measurement automation lives in xtask: the tool only samples and splits time, the maintainer pages and hovers by hand (no synthesized input)"

key-files:
  created:
    - target/measure/portal-real.json
    - target/measure/portal-real-summary.txt
    - target/measure/portal-synth.json
    - target/measure/portal-synth-summary.txt
  modified:
    - xtask/src/measure.rs
    - xtask/src/main.rs

key-decisions:
  - "Both scenarios must be repeated after the hover-preview defect is fixed; the numbers recorded here are valid for first page, peak and final, not for the preview cost"
  - "The scenario 2 per-round values are not comparable with the planned windows (see Deviations); only first-page, peak and final values are used"

patterns-established:
  - "Sample files and summaries stay under target/measure (untracked); plan 25 records the judged numbers in docs/MEASUREMENTS.md"

requirements-completed: [UI-07]

coverage:
  - id: D1
    description: "cargo xtask measure samples the Portal process every 500 ms and summarizes it in the D-25 shape (first page, round end/peak, peak, final)"
    requirement: "UI-07"
    verification:
      - kind: unit
        ref: "xtask/src/measure.rs tests (cargo test -p xtask measure; classify Side::Portal, parse_args portal flags, portal_summary on a synthetic series)"
        status: pass
      - kind: integration
        ref: "smoke run on target/synth-small: measure --side portal then --portal-summary (target/measure/portal-smoke.json)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Scenario 1: the real library (90 themes, 30 per page) paged by the maintainer for 3 rounds in a release build, sampled for 240 s"
    requirement: "UI-07"
    verification:
      - kind: other
        ref: "target/measure/portal-real.json and portal-real-summary.txt"
        status: pass
    human_judgment: true
    rationale: "Hover previews did not animate during the run, so the preview cost is not measured; the maintainer's manual paging cannot be asserted by a test and the run must be repeated after the fix"
  - id: D3
    description: "Scenario 2: the 500-theme synthetic library paged by the maintainer for 3 rounds in a release build, sampled for 400 s"
    requirement: "UI-07"
    verification:
      - kind: other
        ref: "target/measure/portal-synth.json and portal-synth-summary.txt"
        status: pass
    human_judgment: true
    rationale: "Hover previews did not animate and the maintainer's rounds did not line up with the fixed windows; the run must be repeated after the fix"

duration: 90min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 24: Service, IPC and Portal connected, Portal measurement Summary

**`cargo xtask measure` gained a Portal mode (per-pid working set and private bytes every 500 ms, D-25 summary and MEASUREMENTS row), and both D-25 scenarios were sampled in a release build with the maintainer paging by hand; the hover previews did not animate, so both scenarios must be repeated after that defect is fixed.**

## Performance

- **Duration:** about 90 min including the two maintainer runs (Task 1 RED at 14:59 +0300, scenario 2 sampler ended 15:26)
- **Started:** 2026-11-24T11:55Z (approx.)
- **Completed:** 2026-11-24T12:31Z
- **Tasks:** 3 (1 tracer, 2 maintainer checkpoints)
- **Files modified:** 2 (tracked); 4 untracked data files

## Accomplishments

- `Side::Portal`, the sampling loop, `--portal-summary` and `--portal-row` in xtask, tested first (RED 39374759, GREEN a560014a); `cargo xtask ci` passes.
- Scenario 1 (real library, 90 themes, release build, 240 s, 480 samples), `--portal-summary --warmup 20 --rounds 3 --idle 20`, MB:
  - working set: first page 172.8, round end 214 / 206 / 206, round peak 221 / 229 / 206, peak 228.9, final 205.5
  - private: first page 163.1, round end 204 / 196 / 196, round peak 211 / 219 / 197, peak 219.3, final 196.0
  - The working set rises while paging (about 173 to 229 MB over the first 100 s) and then sits at a flat 205-206 MB for the rest of the run, including the idle window.
- Scenario 2 (500-theme synthetic library, release build, 400 s, 800 samples), same summary command, MB:
  - working set: first page 130.8, round end 206 / 206 / 270, round peak 221 / 207 / 287, peak 286.6, final 269.4
  - private: first page 123.5, round end 199 / 199 / 262, round peak 214 / 200 / 279, peak 279.3, final 261.8
  - Real timeline (20 s windows, working set): 131 MB on page 1 until about 40 s, rising to about 221 MB by 120 s (round 1), flat at 206-207 MB from 140 s to 280 s (maintainer idle on page 1), then rising again to 287 MB at about 360 s (rounds 2 and 3) and 269 MB in the last idle window. This growth after the flat stretch is the main number for plan 25 to judge against the UI-07 bound; it was reached with previews not animating.
- Both summaries are kept next to the data: `target/measure/portal-real-summary.txt`, `target/measure/portal-synth-summary.txt`. The apps were stopped (the service had already exited by itself after the Portal closed, D-13); nothing is left running.

## Task Commits

1. **Task 1: Portal memory sampled and summarized (tracer, TDD)** - `39374759` (test, RED), `a560014a` (feat, GREEN)
2. **Task 2: Scenario 1, real library (maintainer-run)** - no commit, data untracked under `target/measure/`
3. **Task 3: Scenario 2, synthetic library (maintainer-run)** - no commit, data untracked under `target/measure/`

**Plan metadata:** the docs commit after this summary.

## Files Created/Modified

- `xtask/src/measure.rs` - Portal side, per-pid sampler, `portal_summary`/`PortalSummary`, `--portal-row`, tests
- `xtask/src/main.rs` - USAGE text for the Portal flags

## Decisions Made

- Both scenarios must be repeated after the preview defect is fixed (below); the present numbers stand only for first page, peak and final of a Portal whose previews show one frame.
- The scenario 2 per-round numbers are not used (timing deviation below).

## Deviations from Plan

### Maintainer-run findings (not auto-fixed)

**1. Hover previews do not animate (found in both scenarios, by the maintainer)**
- **Found during:** Task 2 (reported at its checkpoint) and confirmed in Task 3
- **Issue:** hovering a card shows only the first frame, so neither scenario measures the preview cost that D-25 and finding F1 target.
- **Leads (code of plan 19, not changed here):** `apps/sucrose-portal/src/library/card.rs:282-283` (`img(image)` of a multi-frame RenderImage may paint only frame 0) and `apps/sucrose-portal/src/library/mod.rs:541-562` (`hover_card` bails out silently).
- **Action:** out of scope for this plan (the code belongs to 03-19); both scenarios must be repeated after the fix.

**2. Scenario 2 timing did not match the fixed three-round windows**
- **Found during:** Task 3
- **Issue:** sampler T0 was 15:19:24 local (400 s). The checkpoint reached the maintainer only at about 15:19:44, round 1 went faster than planned and ended at about 15:21, the maintainer then stayed idle on page 1 from about 15:21 to 15:23, did the remaining two rounds from about 15:23 to 15:25:44 and stayed on page 1 until 15:26:04. The fixed windows of `--portal-summary` (warm-up 20, 3 rounds, idle 20) therefore do not line up with the real rounds: the per-round end and peak values (206 / 206 / 270 and 221 / 207 / 287) are skewed. First-page, overall peak and final values stay valid.
- **Action:** use the 20 s timeline above for the shape; repeat the run with a tighter cue.

**Total deviations:** 2 maintainer-run findings, none auto-fixed (out of scope or a run-timing issue). **Impact:** the tool is done and verified; the D-25 numbers exist but must be repeated before plan 25 can judge the preview cost.

## Issues Encountered

- The release builds ran while the maintainer's real data was only read; scratch folders `target/measure-real` and `target/synth-500` were used and the preview switch was set only there.

## Known Stubs

None in the code of this plan.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 25 can record the numbers (`--portal-row target/measure/portal-real.json target/measure/portal-synth.json`) but should state that previews did not animate and the scenario 2 round windows are skewed; repeating both scenarios after the hover-preview fix (plan 19 code) is needed for a valid UI-07 judgment.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*

## Self-Check: PASSED
