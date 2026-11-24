---
phase: 03-m2-service-ipc-and-portal-connected
plan: 39
subsystem: measurement
tags: [measurement, d-25, ui-07, lib-06, review-disposition, gap-closure, docs]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: the D-25 reruns (03-37, 03-38), the growth attribution (03-40), the atlas fix (03-27), animating previews (03-28), the review fixes 03-29..03-36
provides:
  - "docs/MEASUREMENTS.md: two rows 'after the atlas fix' (real library, 500 synthetic themes), both Rule: pass, and the paragraph 'M2 Portal library paging, repeat after the atlas fix'"
  - "03-REVIEW-DISPOSITION.md: 10 findings fixed, 12 deferred with reasons, none open"
  - "LIB-06 and UI-07 Complete in .planning/REQUIREMENTS.md"
affects: [03-verification]
actuals:
  tokens: 6500
  tasks: 3
  commits: 3
plan_head_before: 01a46d65424ea7262fa0e6483497e9547cf8bb38
plan_head_after: 3860b872965a30ae0f3715d02a428fa6dadbcf54
tech-stack:
  added: []
  patterns:
    - "A D-25 verdict is applied as written to the tool's numbers; a caveat the rule does not cover is recorded next to the verdict, not folded into it"
key-files:
  created: []
  modified:
    - docs/MEASUREMENTS.md
    - docs/GPUI.md
    - docs/FEATURES.md
    - .planning/phases/03-m2-service-ipc-and-portal-connected/03-REVIEW-DISPOSITION.md
    - .planning/REQUIREMENTS.md
key-decisions:
  - "Both D-25 scenarios pass the rule as written (final at most first page + 48 MiB; round 3 end at most round 1 end x 1.05; working set and private bytes), so the atlas release of 03-27 is recorded as the confirmed source of the earlier 500-theme growth and no follow-up F-1 to F-4 is selected"
  - "The real library's 'unattributed' attribution (private +57.4 MB at the round 3 end) is recorded as found; the rule judges the final value (+35 MB) and the round ratio, so it is a caveat, not a failure"
requirements-completed: [LIB-06, UI-07]
coverage:
  - id: D1
    description: "The D-25 reruns are judged with the rule as written and recorded with their attribution verdicts in docs/MEASUREMENTS.md, docs/GPUI.md and docs/FEATURES.md"
    requirement: UI-07
    verification:
      - kind: other
        ref: "cargo xtask measure --portal-summary target/measure/portal-real-2.json / portal-synth-2.json --rounds 3 (exit 0); grep -c 'after the atlas fix' docs/MEASUREMENTS.md = 3"
        status: pass
    human_judgment: false
  - id: D2
    description: "Every review finding has a final disposition (CR-01, CR-02, WR-01..WR-08 fixed; IN-01..IN-12 deferred with reasons)"
    verification:
      - kind: other
        ref: "Task 2 automated check: 10 fixed, 12 deferred, 0 open in frontmatter and table"
        status: pass
    human_judgment: false
  - id: D3
    description: "LIB-06 and UI-07 read Complete on evidence (maintainer hover confirmation in 03-28, both reruns pass)"
    requirement: UI-07
    verification:
      - kind: other
        ref: "grep of the four LIB-06 / UI-07 lines in .planning/REQUIREMENTS.md"
        status: pass
    human_judgment: true
    rationale: "Memory numbers come from one hand-driven run per scenario; verify-work may still want the maintainer's view on the real-library caveat"
duration: 7min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 39: Record the D-25 reruns, close the review ledger, mark LIB-06 and UI-07 Summary

**Both D-25 reruns pass the rule as written (500-theme final working set 165 MB against 269 MB before the atlas fix), the atlas release of 03-27 is recorded as the confirmed cause of the earlier growth, the review ledger holds 10 fixed and 12 deferred findings with none open, and LIB-06 and UI-07 are Complete.**

## Performance

- **Duration:** 7 min
- **Tasks:** 3
- **Files modified:** 5

## Rule verdicts (one line each)

- Real library (90 themes, 3 pages): working set first page 143.1, final 178.4 (limit 193.4), round 3 end 201 against 194 x 1.05 = 203.7; private first page 133.7, final 168.7 (limit 184.0), round 3 end 191 against 184 x 1.05 = 193.2. **Rule: pass.** Attribution: unattributed (private +57.4 MB first page to round 3 end; gpu atlas +20.0, held heap +9.0, retention +1.0, thumbnails +9.6). No follow-up selected, because the rule passes.
- 500 synthetic themes (17 pages): working set first page 134.1, final 165.0 (limit 184.4), round 3 end 166 against 180 x 1.05 = 189; private first page 125.1, final 155.8 (limit 175.4), round 3 end 156 against 170 x 1.05 = 178.5. **Rule: pass.** Attribution: within the cap (private +31.3 MB; gpu +20.0, held heap +10.0, retention +1.0, thumbnails +9.5). Atlas cause confirmed: it failed before the fix (final 269 MB) and passes after.
- Previews animated in both runs (63 and 272 `preview playing` lines) and the maintainer confirmed animation, strip hiding and a quiet fast sweep in the 03-28 hover run.

## Accomplishments

- Two MEASUREMENTS rows and a reading paragraph; the earlier FAIL rows and paragraph are unchanged history. GPUI.md's paging-memory bullet and FEATURES.md's hover preview row state the result.
- 03-REVIEW-DISPOSITION.md: every fixing plan's SUMMARY was checked first (all complete, tests and `cargo xtask ci` passing). IN-03 is deferred with the part 03-29 fixed named (one `decode_jpeg_scaled`, confirmed in 03-29's SUMMARY) and the bin_name half left.
- LIB-06 and UI-07 set to Complete (checkbox and Traceability row); no other requirement touched.

## Task Commits

1. **Task 1: reruns judged and recorded** - `1b8927b5` (docs)
2. **Task 2: review ledger closed** - `e2109d20` (docs)
3. **Task 3: LIB-06 and UI-07 Complete** - `3860b872` (docs)

## Files Created/Modified

- `docs/MEASUREMENTS.md` (CRLF kept) - two rows and the repeat paragraph
- `docs/GPUI.md` (CRLF kept) - paging-memory bullet states the rerun result
- `docs/FEATURES.md` (CRLF and missing final newline kept) - hover preview row memory note
- `.planning/phases/03-m2-service-ipc-and-portal-connected/03-REVIEW-DISPOSITION.md` (LF) - final dispositions
- `.planning/REQUIREMENTS.md` (LF) - LIB-06, UI-07

## Decisions Made

See key-decisions: the rule is applied as written; the real-library attribution caveat is recorded, not tuned away.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] FEATURES.md has no separate "library memory" row**
- **Found during:** Task 1
- **Issue:** the plan names a hover preview row and a library memory row; only the hover preview row exists in docs/FEATURES.md.
- **Fix:** the memory result went into the hover preview row's "Memory effect" sentence; no new row was invented.
- **Files modified:** docs/FEATURES.md
- **Commit:** 1b8927b5

**Total deviations:** 1 (documentation placement). **Impact:** none on verdicts.

## Issues Encountered

- The real-library run's round 3 end (191 MB private) is above first page + 50 MB and its attribution is "unattributed"; the rule judges the final value and the round 3 / round 1 ratio, both of which pass. Recorded in MEASUREMENTS.md as found. Runs were by hand, one per scenario, with uneven round lengths.
- The Portal logs six "invalid window handle" ERROR lines at window close (seen in 03-37 and 03-38); not part of this plan, left for triage.

## Human-pending

- SC3 "Exit / Close Applications closes every Sucrose process" (TRAY-05; 03-16 tray check; WR-02 fixed by 03-32) and SC4 "with start with the OS on, Sucrose starts after sign-in and resumes the wallpaper" (TRAY-03; 03-23 sign-in check; WR-05 fixed by 03-35) stay in 03-VERIFICATION.md behavior_unverified_items for /gsd-verify-work. TRAY-03 and TRAY-05 are not touched.

## Known Stubs

None.

## Threat Flags

None. No app was started; no real data touched.

## Next Phase Readiness

Gap-closure round 3 plans 03-27..03-40 are complete; the phase is ready for /gsd-verify-work (gap 1 and 2 on record, review ledger closed).

## Self-Check: PASSED

- All five modified files exist; commits 1b8927b5, e2109d20, 3860b872 are ancestors of HEAD; `cargo xtask ci` exit 0.
