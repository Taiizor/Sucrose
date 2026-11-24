---
phase: 03-m2-service-ipc-and-portal-connected
plan: 38
subsystem: measurement
tags: [measurement, d-25, scenario-2, synthetic-library, portal, memory, gap-closure]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: the page-log summary (03-36), the growth attribution fields (03-40), the atlas and library fixes (03-27..03-35)
provides:
  - "target/measure/portal-synth-2.json, portal-synth-2.log and portal-synth-2-summary.txt (untracked): D-25 scenario 2 on the 500-theme synthetic library, release build, three log-split rounds with hover previews, with the 03-40 attribution line"
affects: [03-39]
actuals:
  tokens: 0
  tasks: 3
  commits: 0
plan_head_before: 08efb41e5447dfe9e4a370d71761f805c7bfef9a
plan_head_after: 08efb41e5447dfe9e4a370d71761f805c7bfef9a
tech-stack:
  added: []
  patterns:
    - "A timed maintainer run starts only after the ready gate; the Portal and the sampler are started with Task 3's exact commands and T0 is returned at once"
key-files:
  created: []
  modified: []
key-decisions:
  - "No cap or code was changed after the run; the summary was produced from the page log exactly as the plan says, and judging the numbers is left to 03-39"
requirements-completed: []
coverage:
  - id: D1
    description: "D-25 scenario 2 re-sampled on the final release build: 500-theme synthetic library (17 pages of 30), three complete rounds paged and hovered by hand, rounds taken from the Portal's page log"
    requirement: UI-07
    verification:
      - kind: manual
        ref: "target/measure/portal-synth-2-summary.txt (cargo xtask measure --portal-summary ... --portal-log ... --rounds 3, exit 0)"
        status: pass
    human_judgment: true
  - id: D2
    description: "Hover previews animated during the run and the page log carries the heap and video memory fields with an attribution line"
    requirement: UI-07
    verification:
      - kind: manual
        ref: "grep -c 'preview playing' target/measure/portal-synth-2.log = 272; every 'page shown' line has heap_allocated_mb, heap_committed_mb, gpu_local_mb, gpu_non_local_mb"
        status: pass
    human_judgment: true
duration: ~25 min of active work (the timed run itself 301 s)
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 38: D-25 scenario 2, 500-theme synthetic library, final build Summary

**The release Portal was sampled for 301 s on the 500-theme synthetic library (17 pages of 30) while the maintainer paged three full rounds and hovered cards; the page-log summary gives a private working set of 125.1 MB on the first page, 170 / 166 / 156 MB at the round ends, a peak of 179.2 MB and a final 155.8 MB, and the 03-40 attribution line is "within the cap" (private +31.3 MB; gpu +20.0, held heap +10.0, retention +1.0, thumbnails +9.5 MB).** Judging these numbers is 03-39's job; this plan records them.

## Performance

- **Duration:** about 25 min of active work; the timed run was 301 s
- **Completed:** 2026-11-24
- **Tasks:** 3 (Task 1 tracer prep, Task 2 ready gate, Task 3 the maintainer's run and the summary)
- **Files modified:** 0 tracked files (measurement data only, under target/)

## Accomplishments

- Task 1: release build current (no source newer than target/release/sucrose-portal.exe); `cargo xtask synth-library --out target/synth-500 --count 500` run without SUCROSE_DATA_DIR: "up to date (500 themes, seed 1)"; scratch `target/synth-500/Setting/Sucrose.json` has `"LibraryPreview": true` and `"AppVisible": false`; unattended smoke passed with `target/measure/portal-synth-2-smoke.json` (30 samples, ended by time). The smoke log, kept as `target/measure/portal-synth-2-smoke.log`, has `library loaded themes=500 corrupt=2 ignored=1` and `page shown page=1 pages=17 thumbs=0 thumb_bytes=0 preview_playing=false heap_allocated_mb=39 heap_committed_mb=42 gpu_local_mb=33 gpu_non_local_mb=0`. The first smoke attempt lost its log because the executor's sampler shell repeated `rm -rf target/synth-500/Log` while the Portal ran; that failed series was removed and the pass repeated once (an executor command mistake, not an app problem). The real %APPDATA%\Sucrose was not touched.
- Task 2: ready gate. The maintainer's reply, verbatim: "start".
- Task 3: the timed run, the Portal started with focus.

### The run

- **T0** = 22:16:24 local (19:16:25Z); the maintainer was told to stay on page 1 until 22:16:44. Before the start, `tasklist.exe` showed no Sucrose or xtask process and `target/synth-500/Log` did not exist.
- **Sampler:** `sampling sucrose-portal.exe (pid 68112) every 500 ms for 1200 s` ... `wrote target/measure/portal-synth-2.json (602 samples)`; `the Portal (pid 68112) exited after 301 s; the series is saved with ended_by portal-exit`; exit 0. Series started 2026-11-24T19:16:22.467Z.
- **Page log** (`target/synth-500/Log/Portal.2026-11-24.log`, copied to `target/measure/portal-synth-2.log`): 93 "page shown" lines, three complete rounds. The last "page shown" (page 1) is at 19:20:54.6Z; the service logged "the service stopped" at 19:21:23.9Z as the Portal closed, so the idle on page 1 before the close was about 29 s (the plan asked for at least 30 s; the summary's final value is still read from the log).
- **Page sequence** (page= values in log order, read from the log): round 1 went 1 to 17 and back to 1 one page at a time; round 2 went 1, 2, 3, 4, 5, 6, then 17 (a jump from page 6 at 19:18:36.7Z), 16, 15, 14, 13, 12, then 17 again (a jump from page 12 at 19:18:51.3Z), 16 and on down, 2, 1; round 3 went 1 to 17 and back to 1 one page at a time. The maintainer did not comment on the two jumps to page 17; they are recorded as seen in the log, no cause is attributed.
- **Maintainer's report after the run, verbatim:** "done" (no comment on rounds or previews; the preview counts from the log answer whether previews played).

### Summary output (`cargo xtask measure --portal-summary target/measure/portal-synth-2.json --portal-log target/measure/portal-synth-2.log --rounds 3`, exit 0, saved to `target/measure/portal-synth-2-summary.txt`)

```
target/measure/portal-synth-2.json: 602 samples every 500 ms, build release, started 2026-11-24T19:16:22.467Z, ended by portal-exit; rounds from the page log target/measure/portal-synth-2.log (MB)
round times, local clock (UTC+03:00):
  round 1: 22:16:50.9 to 22:18:30.3 (99.4 s)
  round 2: 22:18:33.0 to 22:19:33.7 (60.7 s)
  round 3: 22:19:35.9 to 22:20:54.6 (78.7 s)
working set: first page 134.1, round end 180 / 176 / 166, round peak 187 / 188 / 188, peak 187.7, final 165.0
private    : first page 125.1, round end 170 / 166 / 156, round peak 178 / 178 / 179, peak 179.2, final 155.8
attribution: within the cap (private +31.3 MB; gpu +20.0, held heap +10.0, retention +1.0, thumbnails +9.5 MB; start 22:16:50.9, end 22:20:54.6)
```

Facts for 03-39, not judged here:

- Round ends (private) 170 / 166 / 156 MB: each round ended at or below the one before. First page 125.1 to final 155.8 MB.
- The attribution window starts at the first page-leaving event (page 2 shown, 22:16:50.9), which already follows the first load of page 2. For comparison, scenario 1 (03-37) printed "unattributed" with private +57.4 MB; here the verdict is "within the cap" with +31.3 MB.
- Raw "page shown" fields in the log: `gpu_local_mb` 33 (page 1 first shown), 58 (page 2), 74 (page 3), 78 from page 4 on (82 twice in round 3 on pages 5 and 6; 74 once on page 5 in round 2); `heap_allocated_mb` 39 at first page, 49 after page 2, 58 after page 3, 59 from page 4 to the end; `heap_committed_mb` 42 first, 52 / 61 after pages 2 / 3, then between 62 and 79 for the rest of the run; `gpu_non_local_mb` 0 throughout; `thumbs` 0 / 30 / 60 / 64 (thumb_bytes 18708480 once all 64 were cached; it stays at 64 across all 17 pages).
- **Previews:** `grep -c 'preview playing' target/measure/portal-synth-2.log` = 272 (first at 19:16:48.3Z, last at 19:20:56.6Z); `grep -c 'preview decoded'` = 298; `grep -c 'hover preview not started'` = 0. Every "page shown" line has `preview_playing=false` (it is read at the page change, not during a hover). The maintainer did not mention a problem with the previews.
- **Portal close:** the log ends with six `ERROR : Geçersiz pencere işleci` ("invalid window handle", 0x80070578 / 0x80040102) lines at 19:21:23.87Z, the same as 03-37 recorded for the real library. They did not affect the measurement.
- **Cleanup:** the service logged "nothing needs the service, exiting" and "the service stopped" at 19:21:23.9Z by itself (D-13); `tasklist.exe` afterwards, checked by this continuation: no Sucrose, sucrose-portal or xtask process (the unrelated cargo.exe processes of another tool were left alone).

## Task Commits

None. This plan changes no tracked file (`files_modified: []`); the measurement data lives untracked under `target/`.

## Files Created/Modified

Untracked data only: `target/measure/portal-synth-2.json`, `target/measure/portal-synth-2.log`, `target/measure/portal-synth-2-summary.txt`, `target/measure/portal-synth-2-summary.err`, `target/measure/portal-synth-2-smoke.json`, `target/measure/portal-synth-2-smoke.log`, `target/measure/portal-synth-2-sampler.out` / `.err`, scratch folder `target/synth-500`.

## Decisions Made

None beyond the plan; no cap, threshold or code was tuned after seeing the numbers.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Task 3's start was performed by the orchestrator, not a continuation agent**
- **Found during:** Task 3
- **Issue:** 03-37's start continuation stalled for 600 s and cost the maintainer ten minutes, so the orchestrator itself started the run.
- **Fix:** the commands were exactly Task 3's: the release Portal WITH focus, `SUCROSE_DATA_DIR="$(wslpath -w target/synth-500)"`, `SUCROSE_LOG="info,sucrose_portal::library=debug"`, `WSLENV=SUCROSE_DATA_DIR:SUCROSE_LOG`, then `cargo xtask measure --side portal --seconds 1200 --interval-ms 500 --out target/measure/portal-synth-2.json` with stdout and stderr in `portal-synth-2-sampler.out` / `.err`. T0 was returned at once and no run was discarded.
- **Files modified:** none tracked
- **Commit:** none

**2. [Executor mistake, no code impact] The first smoke attempt lost its log**
- **Found during:** Task 1
- **Issue:** the executor's sampler shell repeated `rm -rf target/synth-500/Log` while the Portal ran, so the first smoke had no log to check.
- **Fix:** that smoke series was removed and the pass repeated once; the second pass is the one recorded above. It was not a measurement.
- **Files modified:** none tracked
- **Commit:** none

**Total deviations:** 2 (neither touches code or the measured run). **Impact:** none on the measurement; the clock of the measured run started after "start" and T0 was returned at once.

## Issues Encountered

- The Portal logs six "invalid window handle" ERROR lines as the window closes (see Facts above; the same as 03-37). Not caused by this plan and not part of the memory question; for the maintainer or a later plan to triage (recorded here, not fixed, per the scope boundary).
- The idle on page 1 before closing was about 29 s, 1 s short of the plan's "at least 30 s"; it does not change how the summary reads the final value.

## Authentication Gates

None.

## Known Stubs

None (no tracked file touched).

## Threat Flags

None. T-03-83 held: scratch `SUCROSE_DATA_DIR` (`target/synth-500`), the preview switch edited only in the scratch Sucrose.json, nothing in the real %APPDATA%\Sucrose or library written, no input synthesized, every started app ended and checked with tasklist.

## Next Phase Readiness

03-39 can record the row from `target/measure/portal-synth-2-summary.txt` together with `target/measure/portal-real-2-summary.txt` and judge the attribution: scenario 2 reads "within the cap" (+31.3 MB private, gpu +20.0 MB the largest named part), scenario 1 read "unattributed" (+57.4 MB).

## Verification

- `test -s target/measure/portal-synth-2.json`: yes (602 samples). The summary holds three rounds and an `attribution:` line. `grep -c 'preview playing'` = 272 (at least 1). `tasklist.exe`: no Rust Sucrose process.
- `cargo xtask ci` (fmt, clippy -D warnings, tests): exit 0; `git status` clean afterwards.

## Self-Check: PASSED

- target/measure/portal-synth-2.json, portal-synth-2.log and portal-synth-2-summary.txt exist and are non-empty.
- No task commits exist for this plan by design (`commits: 0`, no tracked change; HEAD is still 08efb41e).
