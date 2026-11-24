---
phase: 03-m2-service-ipc-and-portal-connected
plan: 36
subsystem: tooling
tags: [rust, xtask, measurement, d-25, wr-06, gap-closure, tdd]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: the Portal's "page shown" debug event with page and pages fields (03-27, 03-24 measurement tooling)
provides:
  - "cargo xtask measure --portal-summary <json> --portal-log <log> --rounds <n> and --portal-row <real.json> <synth.json> --portal-logs <real.log> <synth.log>: rounds, first page and final value come from the Portal's own page log"
  - "page_events, rebase_events, split_rounds, portal_log_summary, PortalLogSummary, RoundSpan, portal_row_from_logs, parse_utc_ms and iso_utc_ms in xtask/src/measure.rs"
  - "PortalSeries.started with milliseconds and PortalSeries.ended_by (time or portal-exit)"
  - "refuses, resolve and strip_verbatim in xtask/src/synth_library.rs: a canonical guard against the real data folder and the data folder in effect (WR-06)"
affects: [03-37, 03-38, 03-39, 03-40]
actuals:
  tokens: 14175
  tasks: 2
  commits: 2
plan_head_before: 5f83117df5ccb5fad0bccdd5b85b11bbfe717d5e
plan_head_after: 82e9c3c510a3d2dd5faa13cac7bcccf5b9bc62a3
tech-stack:
  added: []
  patterns:
    - "A measurement summary splits time by the app's own timestamped log lines, so nothing depends on the maintainer matching a clock"
    - "A destructive tool resolves both paths component by component (links, junctions, 8.3 names, verbatim prefix) before comparing, and the guard has no force override"
key-files:
  created: []
  modified:
    - xtask/src/measure.rs
    - xtask/src/main.rs
    - xtask/src/synth_library.rs
key-decisions:
  - "A round starts at the first page-1-leaving event and ends at the first page-1 event after the last page; the round's peak is the maximum between its start and its end, its end value the median of the 5 s after the closing event"
  - "The 20 s idle minimum is measured from the last round's end to the series end (the maintainer idles 30 s in 03-37/03-38); the final value itself is the median of 10 s after the end to 1 s before the series end"
  - "A Portal closed by the maintainer is a normal end of the sampler (exit 0, ended_by portal-exit); only the page-log summary accepts such a series, the fixed-window summary still refuses it"
  - "Local times of the rounds use the OS offset (PowerShell on Windows, date elsewhere), falling back to UTC with a note, because the workspace has no time zone crate"
requirements-completed: [UI-07]
coverage:
  - id: D1
    description: "The summary splits rounds, first-page and final values by the Portal's page log, and refuses a log with too few rounds, a short warm-up or less than 20 s of idle"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "xtask/src/measure.rs#portal_log_rounds_end_when_page_one_follows_the_last_page"
        status: pass
      - kind: unit
        ref: "xtask/src/measure.rs#portal_log_summary_matches_a_synthetic_series"
        status: pass
      - kind: unit
        ref: "xtask/src/measure.rs#portal_log_summary_refuses_too_few_rounds_and_a_short_idle"
        status: pass
    human_judgment: false
  - id: D2
    description: "The series start keeps milliseconds, records ended_by, and older series files still parse; the same series and log give the same numbers twice"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "xtask/src/measure.rs#portal_series_start_keeps_milliseconds_and_old_files_still_parse"
        status: pass
      - kind: unit
        ref: "xtask/src/measure.rs#portal_log_summary_is_the_same_twice"
        status: pass
    human_judgment: false
  - id: D3
    description: "--portal-row with --portal-logs prints the MEASUREMENTS row and says the rounds come from the page log; the summary prints each round's local start and end"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "xtask/src/measure.rs#portal_log_row_says_the_rounds_come_from_the_page_log"
        status: pass
      - kind: unit
        ref: "xtask/src/measure.rs#portal_log_clock_text_and_offsets"
        status: pass
      - kind: unit
        ref: "xtask/src/measure.rs#portal_log_args_choose_the_log_split_and_refuse_fixed_window_flags"
        status: pass
    human_judgment: false
  - id: D4
    description: "synth-library refuses an --out that is, holds or lies inside the real data folder or the data folder in effect, however spelled; --force never bypasses it (WR-06)"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "xtask/src/synth_library.rs#synth_a_verbatim_spelling_of_a_guarded_folder_is_refused"
        status: pass
      - kind: unit
        ref: "xtask/src/synth_library.rs#synth_a_folder_that_holds_a_guarded_folder_is_refused"
        status: pass
      - kind: unit
        ref: "xtask/src/synth_library.rs#synth_a_link_into_a_guarded_folder_is_refused"
        status: pass
      - kind: unit
        ref: "xtask/src/synth_library.rs#synth_generate_refuses_the_folders_it_guards_even_with_force"
        status: pass
    human_judgment: false
duration: 13min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 36: Portal log-split measurement and canonical synth-library guard Summary

**D-25 rounds now come from the Portal's own "page shown" debug lines (first page, round ends and peaks, final idle) instead of fixed time windows, and synth-library resolves paths through links, junctions, 8.3 names and the verbatim prefix before it refuses the real data folder.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-11-24T15:50:00Z
- **Completed:** 2026-11-24T16:03:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- `--portal-summary ... --portal-log ... --rounds 3` and `--portal-row ... --portal-logs ...` split a series by the page log; the first-page value is the median of the 10 s before the first page change, a round's end value the median of the 5 s after its closing page-1 event, the final value the median from 10 s after the last round to 1 s before the series ends. It refuses, naming the shortfall, too few rounds, a warm-up under 10 s and under 20 s of idle.
- The series start is RFC 3339 UTC with milliseconds and `ended_by` says whether the time ran out or the Portal was closed; old series files still parse. The summary prints each round's local start and end for cross-checking with the maintainer's report.
- WR-06: `refuses` resolves both sides component by component and compares case-insensitively; a verbatim spelling, a parent folder and a directory symlink into the real data folder or the data folder in effect are refused, `--force` included.
- Smoke on `target/synth-small` (40 themes, release build, debug filter, 12 s): the Portal log carries `2026-11-24T15:58:05.570398Z DEBUG sucrose_portal::library: page shown page=1 pages=2 thumbs=0 thumb_bytes=0 preview_playing=false`, the series has `"ended_by": "time"` and `"started": "2026-11-24T15:58:10.526Z"`; `tasklist.exe` showed no Rust Sucrose process afterwards.

## Task Commits

1. **Task 1: Rounds come from the Portal's page log** - `695c3cc8` (feat)
2. **Task 2: synth-library cannot be pointed at the real data folder by any spelling (WR-06)** - `82e9c3c5` (feat)

**Plan metadata:** the docs commit that follows this file.

## Files Created/Modified

- `xtask/src/measure.rs` - page-log parser, round splitter, log summary, row from logs, time helpers, CLI flags, `ended_by`, tests
- `xtask/src/main.rs` - task list text names `--portal-log`
- `xtask/src/synth_library.rs` - `resolve`, `strip_verbatim`, `refuses`, `guarded_folders`, tests

## Decisions Made

See `key-decisions`. The 03-40 attribution work extends `PageEvent` (fields) and reads `PortalLogSummary.rounds` for round starts and ends.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical] Refuse a page log that does not fit the series**
- **Found during:** Task 1
- **Issue:** the plan named three refusals; a log with more rounds than `--rounds`, paging after the last round, or round times outside the series would silently contaminate the final idle window or pair the wrong log with a series.
- **Fix:** `portal_log_summary` also refuses more complete rounds than asked (naming the number to pass), a round left open after the last one, and a round end outside the series.
- **Files modified:** xtask/src/measure.rs
- **Verification:** `portal_log_summary_refuses_too_few_rounds_and_a_short_idle`
- **Committed in:** 695c3cc8

**2. [Rule 1 - Bug] The sampler reported a closed Portal as a failure**
- **Found during:** Task 1
- **Issue:** a page-log run ends by the maintainer closing the Portal, but `--side portal` returned an error (exit 1) when the process went away.
- **Fix:** it saves the series with `ended_by: portal-exit`, prints a note and exits 0; `complete` stays false so the fixed-window summary still refuses such a series.
- **Files modified:** xtask/src/measure.rs
- **Committed in:** 695c3cc8

---

**Total deviations:** 2 auto-fixed (1 missing critical, 1 bug)
**Impact on plan:** Both keep the D-25 numbers honest; no scope creep.

## Issues Encountered

- On Windows `std::path::absolute` removes `..` lexically (as Win32 does), so `link\..` is not resolved through the link there; the symlink test checks that case on Unix only and says so. The guard and the OS agree on both platforms.
- The guard also refuses `--out` equal to `SUCROSE_DATA_DIR`; run synth-library without that variable set (the help text says so, and 03-38 does).

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

03-37 and 03-38 can run their D-25 reruns against `--portal-log`; 03-40 adds the attribution fields to the same parser. The local round times are printed in the local offset (UTC with a note when the OS cannot be asked).

## Self-Check: PASSED

- xtask/src/measure.rs, xtask/src/main.rs and xtask/src/synth_library.rs exist; commits 695c3cc8 and 82e9c3c5 are on HEAD.
- `cargo xtask ci` (fmt, clippy -D warnings, test: 62 xtask tests) passed.
