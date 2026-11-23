---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 28
subsystem: docs
tags: [decisions, parity-record, explorer, monitor-change, accepted-limitation]
requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: the 02-26 checkpoint reply "wait" and the spike numbers (02-26-SUMMARY.md), the location watch of 02-23..02-25
provides:
  - D-031 in docs/DECISIONS.md, the maintainer's decision to wait for explorer when a display change leaves a monitor outside its desktop window
  - FEATURES row 'Monitor changes, shell restart recovery (CrashExplorer)' states the observed behavior, the spike numbers and the accepted limitation instead of a guessed cause (WR-20)
  - install_location_watch doc comment names the completed desktop check (IN-28, surface part)
affects: [02-29, docs/FEATURES.md, docs/DECISIONS.md, phase verification]
actuals:
  tokens: 1100
  tasks: 2
  commits: 2
plan_head_before: 9dedb60c0b29ebf698ee79ce9147f22f80e18907
plan_head_after: d7b1c04937ced73daa959485cde43ea217c9316e
tech-stack:
  added: []
  patterns:
    - "An accepted limitation is recorded as an Accepted decision entry plus the parity row, while the UAT gap stays open"
key-files:
  created: []
  modified:
    - docs/DECISIONS.md
    - docs/FEATURES.md
    - crates/sucrose-platform/src/os_windows/surface.rs
key-decisions:
  - "D-031: wait for explorer after a display change; no fallback window; G-02-7 is an accepted limitation and stays open"
requirements-completed: [DESK-07, DESK-01]
coverage:
  - id: D1
    description: "The parity record tells what a monitor shows right after a display change and that the maintainer accepted it (D-031, FEATURES row)"
    requirement: "DESK-07"
    verification: []
    human_judgment: true
    rationale: "Documentation of a maintainer decision; adequacy of the wording is a reading judgment, no test asserts it (the plan's grep checks only pin the text and the line endings)"
  - id: D2
    description: "The install_location_watch doc comment no longer points at a pending desktop check"
    requirement: "DESK-07"
    verification:
      - kind: other
        ref: "grep -cF 'checked on a real desktop' crates/sucrose-platform/src/os_windows/surface.rs (1), cargo fmt --all -- --check, cargo clippy -p sucrose-platform --all-targets -- -D warnings"
        status: pass
    human_judgment: false
duration: 3 min
completed: 2026-11-23
status: complete
---

# Phase 2 Plan 28: Record the accepted G-02-7 limitation Summary

**D-031 records the maintainer's "wait" for explorer after display changes (fallback window rejected, 8 of 8 moves caught, episodes 1.73-2.11 s), the FEATURES row states the behavior instead of a guessed cause, and the location watch's doc comment names the completed desktop check.**

## Performance

- **Duration:** 3 min
- **Started:** 2026-11-23T18:55:20Z
- **Completed:** 2026-11-23T18:58:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- WR-20 addressed: docs/FEATURES.md row 'Monitor changes, shell restart recovery (CrashExplorer)' ends with the observed behavior (a monitor, or the part of it, outside explorer's desktop window shows the static Windows wallpaper and no icons until explorer moves that window), the 02-26 spike numbers, the not-recurred 90 s UAT episode with its cause stated as not established, and "Accepted limitation (G-02-7, D-031)". The status cell keeps 🟨.
- D-031 added after D-030 (append-only, no existing line changed): decision, why (spike numbers), the rejected fallback window, consequences.
- IN-28 (surface part) addressed: the doc comment of `install_location_watch` says hook delivery "was checked on a real desktop (UAT G-02-7, D-031)"; no code changed.
- G-02-7 stays open: 02-UAT.md is untouched and still lists G-02-7 with status failed. It is an accepted limitation, not a closed gap.

## Requirements

- DESK-07: met with an accepted limitation: G-02-7 open (02-UAT.md, status failed), D-031
- DESK-01: met with an accepted limitation: G-02-7 open (02-UAT.md, status failed), D-031

The update_requirements step marks both Complete in .planning/REQUIREMENTS.md. That records the maintainer's acceptance (D-031), not a closed G-02-7; phase verification judges both requirements with G-02-7 open.

## Task Commits

1. **Task 1: D-031 and FEATURES row 30** - `baea16a5` (docs)
2. **Task 2: install_location_watch doc comment** - `d7b1c049` (docs)

## Files Created/Modified

- `docs/DECISIONS.md` - D-031 entry (LF)
- `docs/FEATURES.md` - last sentence of the monitor-change row replaced (CRLF kept)
- `crates/sucrose-platform/src/os_windows/surface.rs` - doc comment only (CRLF kept)

## Decisions Made

D-031 (see docs/DECISIONS.md): wait for explorer, no fallback window; recorded from the maintainer's verbatim reply "wait" at the 02-26 checkpoint.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Task 1 verify clause "every FEATURES.md line ends with CR" cannot hold**
- **Found during:** Task 1
- **Issue:** `grep -vc $'\r$' docs/FEATURES.md` must print 0, but the file's last line ("... at threshold 0.") has no line ending at all, at HEAD as well, so exactly one line never ends with CR. My edit also briefly lost every CR because the REPL file reader normalizes CRLF; I restored CRLF on all terminated lines with a byte-level rewrite before committing.
- **Fix:** Kept the file's original shape (CRLF on every terminated line, unterminated last line); ran the rest of the verify chain unchanged and the CR clause as "at most 1 line without CR".
- **Files modified:** docs/FEATURES.md
- **Verification:** all other Task 1 checks pass; the diff against HEAD touches only row 30
- **Committed in:** baea16a5

---

**Total deviations:** 1 auto-fixed (1 blocking, verify-clause only)
**Impact on plan:** None on content; the plan's CR check should say "no CR-less line except an unterminated last line".

## Issues Encountered

None beyond the deviation above.

## Known Stubs

None.

## Threat Flags

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 02-29. G-02-7 remains open in 02-UAT.md as an accepted limitation (D-031) and is visible to phase verification.

## Self-Check: PASSED

- docs/DECISIONS.md, docs/FEATURES.md, surface.rs exist; commits baea16a5 and d7b1c049 are ancestors of HEAD.
