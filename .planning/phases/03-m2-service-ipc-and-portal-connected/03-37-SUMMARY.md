---
phase: 03-m2-service-ipc-and-portal-connected
plan: 37
subsystem: measurement
tags: [measurement, d-25, scenario-1, real-library, portal, memory, gap-closure]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: the page-log summary (03-36), the growth attribution fields (03-40), the atlas and library fixes (03-27..03-35)
provides:
  - "target/measure/portal-real-2.json, portal-real-2.log and portal-real-2-summary.txt (untracked): D-25 scenario 1 on the maintainer's real library, release build, three log-split rounds with hover previews, with the 03-40 attribution line"
affects: [03-39]
actuals:
  tokens: 0
  tasks: 3
  commits: 0
plan_head_before: 544ffded1aa8037b677ee771e5c517116a6831d3
plan_head_after: 544ffded1aa8037b677ee771e5c517116a6831d3
tech-stack:
  added: []
  patterns:
    - "A timed maintainer run starts only after the ready gate; the continuation starts the Portal and the sampler itself and returns T0 at once"
key-files:
  created: []
  modified: []
key-decisions:
  - "No cap or code was changed after the run; the summary was produced from the page log exactly as the plan says, and judging the numbers is left to 03-39"
requirements-completed: []
coverage:
  - id: D1
    description: "D-25 scenario 1 re-sampled on the final release build: real library (90 themes, 3 pages of 30), three complete rounds paged and hovered by hand, rounds taken from the Portal's page log"
    requirement: UI-07
    verification:
      - kind: manual
        ref: "target/measure/portal-real-2-summary.txt (cargo xtask measure --portal-summary ... --portal-log ... --rounds 3, exit 0)"
        status: pass
    human_judgment: true
  - id: D2
    description: "Hover previews animated during the run and the page log carries the heap and video memory fields with an attribution line"
    requirement: UI-07
    verification:
      - kind: manual
        ref: "grep -c 'preview playing' target/measure/portal-real-2.log = 63; every 'page shown' line has heap_allocated_mb, heap_committed_mb, gpu_local_mb, gpu_non_local_mb"
        status: pass
    human_judgment: true
duration: ~1h (including the aborted first attempt)
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 37: D-25 scenario 1, real library, final build Summary

**The release Portal was sampled for 318 s on the maintainer's real library (90 themes, 3 pages of 30) while the maintainer paged three full rounds and hovered cards; the page-log summary gives a private working set of 133.7 MB on the first page, 184 / 169 / 191 MB at the round ends, a peak of 232.3 MB and a final 168.7 MB, and the 03-40 attribution line is "unattributed" (gpu atlas the largest part at +20.0 MB of +57.4 MB private growth).** Judging these numbers is 03-39's job; this plan records them.

## Performance

- **Duration:** about 1 h including the aborted first attempt (see Deviations)
- **Completed:** 2026-11-24
- **Tasks:** 3 (Task 1 tracer prep, Task 2 ready gate, Task 3 the maintainer's run and the summary)
- **Files modified:** 0 tracked files (measurement data only, under target/)

## Accomplishments

- Task 1: release build; scratch data folder `target/measure-real-2` (the Setting folder copied read-only from the maintainer's real settings, Library Preview on only in the copy); unattended smoke passed: `library loaded themes=90 corrupt=16 ignored=2`, a `page shown page=1 pages=3` line with the heap and video memory fields, 30 samples (`target/measure/portal-real-2-smoke.json`); the smoke Log folder was removed before the real run; nothing was left running.
- Task 2: ready gate. The maintainer's reply, verbatim: "yeni geldim başına" ("I just got to the PC"), taken as "start".
- Task 3: the timed run, the Portal started with focus.

### The run

- **T0** = 21:57:14 local (18:57:14Z); the maintainer was told to stay on page 1 until 21:57:34.
- **Sampler:** `sampling sucrose-portal.exe (pid 45860) every 500 ms for 900 s` ... `wrote target/measure/portal-real-2.json (637 samples)`; `the Portal (pid 45860) exited after 318 s; the series is saved with ended_by portal-exit`; exit code 0. Series started 2026-11-24T18:57:12.083Z.
- **Page log** (`target/measure-real-2/Log/Portal.2026-11-24.log`, copied to `target/measure/portal-real-2.log`): 13 "page shown" lines, three complete rounds. The Portal closed at about 19:02:31Z, more than 60 s of idle on page 1 after the last round.
- **Maintainer's report after the run, verbatim:** "done" (nothing odd reported).

### Summary output (`cargo xtask measure --portal-summary target/measure/portal-real-2.json --portal-log target/measure/portal-real-2.log --rounds 3`, exit 0, saved to `target/measure/portal-real-2-summary.txt`)

```
target/measure/portal-real-2.json: 637 samples every 500 ms, build release, started 2026-11-24T18:57:12.083Z, ended by portal-exit; rounds from the page log target/measure/portal-real-2.log (MB)
round times, local clock (UTC+03:00):
  round 1: 21:58:11.9 to 21:58:49.3 (37.5 s)
  round 2: 21:58:56.6 to 22:01:11.1 (134.4 s)
  round 3: 22:01:13.9 to 22:01:29.2 (15.3 s)
working set: first page 143.1, round end 194 / 179 / 201, round peak 214 / 210 / 216, peak 241.9, final 178.4
private    : first page 133.7, round end 184 / 169 / 191, round peak 204 / 200 / 207, peak 232.3, final 168.7
attribution: unattributed, largest part gpu atlas +20.0 MB (private +57.4 MB; gpu +20.0, held heap +9.0, retention +1.0, thumbnails +9.6 MB; start 21:58:11.9, end 22:01:29.2)
```

Facts for 03-39, not judged here:

- Round 2 lasted 134.4 s because the maintainer rested on page 2 for about two minutes (18:59:10.8Z to 19:01:11.1Z) before going back to page 1; rounds 1 and 3 took 37.5 s and 15.3 s. The round-2 peak therefore covers that dwell.
- Round ends (private) 184 / 169 / 191 MB: round 2 ended lower than round 1, round 3 higher than both. First page 133.7 to final 168.7 MB after the idle on page 1.
- The attribution window starts at the first page-leaving event, which already follows the first load of page 2. The raw "page shown" fields in the log: `gpu_local_mb` 30 (page 1 first shown), 59 (page 2 first shown), 75 (page 3), 79 (round 1 end), 83 (peak, rounds 1 and 2 on pages 2 and 3), 79 (round 2 end), 71 (page 3 in round 3), 79 (final page 1); `heap_allocated_mb` 38 at first page, 49 after page 2, 57 after page 3, then 58 constant to the end; `heap_committed_mb` 40 first, 52 / 61 after pages 2 / 3, 80 once (page 2, return in round 1), 62 afterwards (67 once); `gpu_non_local_mb` 0 throughout; `thumbs` 0 / 30 / 60 / 64 (thumb_bytes 18974880 to 19126080 once all four were cached).
- The unattributed remainder: the three named parts plus thumbnails come to about +30 MB of the +57.4 MB private growth.
- **Previews:** `grep -c 'preview playing' target/measure/portal-real-2.log` = 63 (first at 18:57:09.2Z, last at 19:01:35.9Z); `grep -c 'preview decoded'` = 77; `grep -c 'hover preview not started'` = 0. Every "page shown" line has `preview_playing=false` (it is read at the page change, not during the hover). The maintainer did not mention a problem with the previews.
- **Portal close:** the log ends with six `ERROR : Geçersiz pencere işleci. (0x80070578)` / `(0x80040102)` lines ("invalid window handle") between 19:02:30.997Z and 19:02:31.011Z, at the moment the maintainer closed the window. The earlier real-library log (`target/measure/portal-real.log`, 03-24) has none. They did not affect the measurement; they are listed under Issues for a follow-up decision.
- **Cleanup:** the service and its two renderers outlived the Portal (the tray keeps the service alive) and were stopped by the orchestrator with `taskkill.exe /IM sucrose.exe /F`. `tasklist.exe` afterwards, checked again by this continuation: no Sucrose, sucrose-portal or xtask process (only unrelated cargo.exe processes of another tool, left alone).

## Task Commits

None. This plan changes no tracked file (`files_modified: []`); the measurement data lives untracked under `target/`.

## Files Created/Modified

Untracked data only: `target/measure/portal-real-2.json`, `target/measure/portal-real-2.log`, `target/measure/portal-real-2-summary.txt`, `target/measure/portal-real-2-smoke.json`, `target/measure/portal-real-2-sampler.out`, `target/measure/portal-real-2-sampler.err`, `target/measure/portal-real-2-aborted-1846/`, scratch folder `target/measure-real-2`.

## Decisions Made

None beyond the plan; no cap, threshold or code was tuned after seeing the numbers.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The first continuation stalled and its run was discarded and restarted**
- **Found during:** Task 3
- **Issue:** the first continuation started the Portal and the sampler at 21:46:08 local, then stalled (no progress for 600 s) without returning T0, so the maintainer never got the instructions. In those 10 minutes the maintainer stayed on page 1 and hovered a few cards (preview playing at 18:46:49Z and 18:48:33Z), with no page change.
- **Fix:** the orchestrator stopped that run with taskkill.exe (the Portal, the service, the renderers and the sampler's cargo/xtask tree only; unrelated cargo.exe processes of another tool were left alone) and moved its files to `target/measure/portal-real-2-aborted-1846/` (Log/ and the sampler .out/.err; no series JSON was written). It then restarted cleanly with Task 3's exact commands: release Portal with focus, `SUCROSE_DATA_DIR=target/measure-real-2`, `SUCROSE_LOG="info,sucrose_portal::library=debug"`, `WSLENV=SUCROSE_DATA_DIR:SUCROSE_LOG`, and `cargo xtask measure --side portal --seconds 900 --interval-ms 500 --out target/measure/portal-real-2.json` with stdout and stderr in `portal-real-2-sampler.out` / `.err`. Only the second run is a measurement; the aborted one has no series.
- **Files modified:** none tracked
- **Commit:** none

**Total deviations:** 1 (blocking, no code impact). **Impact:** the clock of the measured run started after the restart (T0 21:57:14 local), so the plan's rules (T0 returned at once, rounds from the page log) hold for it.

## Issues Encountered

- The Portal logs six "invalid window handle" ERROR lines (Windows error codes 0x80070578 and 0x80040102) as the window closes. Not caused by this plan and not part of the memory question; for the maintainer or a later plan to triage (recorded here, not fixed, per the scope boundary).

## Authentication Gates

None.

## Known Stubs

None (no tracked file touched).

## Threat Flags

None. T-03-82 held: scratch `SUCROSE_DATA_DIR`, the preview switch edited only in the scratch Sucrose.json, nothing in the real %APPDATA%\Sucrose or library written, no input synthesized, every started app stopped and checked with tasklist.

## Next Phase Readiness

03-39 can record the row from `target/measure/portal-real-2-summary.txt` (and `--portal-row` with the 03-38 synthetic run) and judge the attribution: the verdict is "unattributed", so the GPU atlas cause is neither confirmed nor ruled out by this run alone.

## Verification

- `test -s target/measure/portal-real-2.json`: yes (637 samples). Summary holds three rounds and an `attribution:` line. `grep -c 'preview playing'` = 63 (at least 1). `tasklist.exe`: no Rust Sucrose process.
- `cargo xtask ci` (fmt, clippy -D warnings, tests): exit 0; `git status` clean afterwards.

## Self-Check: PASSED

- target/measure/portal-real-2.json, portal-real-2.log and portal-real-2-summary.txt exist and are non-empty.
- No task commits exist for this plan by design (`commits: 0`, no tracked change; HEAD is still 544ffded).
