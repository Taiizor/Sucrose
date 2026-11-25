---
phase: 03-m2-service-ipc-and-portal-connected
plan: 44
subsystem: ui
tags: [uat, gap-closure, measurement, d-25, thumbnails, skipped]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "In-view thumbnail requests with wanted thumbnails kept in the cache (03-42), checked by hand in 03-43"
provides:
  - "The 500-theme synthetic library (target/synth-500) and a 30-sample smoke series, prepared for a later D-25 scenario 2 run (untracked, under target/)"
  - "The maintainer's decision to skip the D-25 scenario 2 rerun for now (UAT test 10 stays open)"
affects: [gap-closure, UAT test 10]

actuals:
  tokens: 2500
  tasks: 1
  commits: 0
plan_head_before: 328f40aaf458d475505f609d3b683c5f132dbb7a
plan_head_after: 328f40aaf458d475505f609d3b683c5f132dbb7a

tech-stack:
  added: []
  patterns: []

key-files:
  created: []
  modified: []

key-decisions:
  - "The maintainer skipped the D-25 scenario 2 rerun (reply: \"bu testi geçelim.\"); no measurement was taken and docs/MEASUREMENTS.md is unchanged"
  - "UI-07 is not marked Complete on this plan"

requirements-completed: []

coverage:
  - id: D1
    description: "D-25 scenario 2 on the release build after 03-42: final at most first page + 48 MiB and round 3 end at most round 1 end x 1.05, on working set and private bytes (UAT test 10)"
    requirement: UI-07
    verification: []
    human_judgment: true
    rationale: "Not measured: the maintainer skipped the run at the ready checkpoint, so there is no evidence either way"

duration: ~15min
completed: 2026-11-25
status: halted
---

# Phase 3 Plan 44: D-25 scenario 2 rerun after the in-view thumbnail requests Summary

**The plan did not reach its goal: the data and the pipeline were prepared and proven on a 15 s smoke pass, but the maintainer answered the ready checkpoint with "bu testi geçelim." (skip), so the measured D-25 rerun was never taken and UAT test 10 stays open.**

## Outcome, stated plainly

- The plan's success criterion is **NOT met**: UAT test 10 is **not** unblocked and the D-25 rule has **not** been checked on the 03-42 thumbnail path. There is no pass and no FAIL.
- UAT test 10 remains **open**, skipped by the maintainer.
- `docs/MEASUREMENTS.md` has **no new row and no new paragraph**. No tracked file changed (`files_modified` of the plan, `docs/MEASUREMENTS.md`, is untouched).
- Task 3 was not run. The Portal, the service and the sampler were not started after the checkpoint.

## Performance

- **Duration:** about 15 min across the first executor and this continuation (excluding the wait for the maintainer)
- **Completed:** 2026-11-25
- **Tasks:** 1 of 3 done (Task 2 ended by the maintainer's skip, Task 3 not run)
- **Files modified:** 0 tracked (scratch only under `target/`, gitignored)

## Task results

| Task | Name | Result |
|------|------|--------|
| 1 | Prepare scenario 2 and prove the pipeline on a short unattended pass | Done, no commit (only gitignored data under `target/`) |
| 2 | Ready gate: the maintainer says "start" | Ended with the maintainer's verbatim reply "bu testi geçelim." (skip, 2026-11-25); the clock was never started |
| 3 | The maintainer pages, scrolls and hovers while the release Portal is sampled; record in docs/MEASUREMENTS.md | Not run, by the maintainer's decision |

## Task 1 evidence

- Release build current: `cargo build --release -p sucrose -p sucrose-portal` finished in 1.00 s; git clean at HEAD 328f40aa.
- `cargo xtask synth-library --out target/synth-500 --count 500` printed "up to date (500 themes, seed 1)" (run without `SUCROSE_DATA_DIR`).
- Scratch `target/synth-500/Setting/Sucrose.json`: `"LibraryPreview": true`, `"DeleteCorrupt": false`, `"Backdrop": "None"`.
- Smoke: Portal `--no-focus` for 15 s, 30 samples; both apps stopped, `tasklist.exe` empty, the smoke Log folder removed. Series kept at `target/measure/portal-synth-3-smoke.json` (untracked).
- Smoke log lines:

```
library loaded themes=500 corrupt=2 ignored=1
page shown page=1 pages=17 thumbs=0 thumb_bytes=0 preview_playing=false heap_allocated_mb=39 heap_committed_mb=42 gpu_local_mb=33 gpu_non_local_mb=0
thumbnails requested first=0 last=11 wanted=12 queued=12
```

- The Portal started the release `sucrose.exe` itself (no wallpaper, `open_portal=false`) and it registered the Portal pid, so the pipeline is proven up to the page log.

## Deviations from Plan

None in the sense of the deviation rules: no code or doc was changed. The plan itself did not complete because the maintainer decided at the Task 2 checkpoint to skip the test; that is a decision, not an auto-fix.

## Issues Encountered

- UAT test 10 (D-25 rule on the thumbnail path after 03-42) is still unverified. Until it is run, the claim "scrolling a large library keeps memory bounded after the G-03-6 / G-03-8 fix" rests only on the last measured row (the one before 03-42) and the maintainer's hand check in 03-43, not on a new measurement.

## Prepared for a later run

- `target/synth-500` (500 synthetic themes, 17 pages of 30, scratch settings as above) is ready.
- `target/measure/portal-synth-3-smoke.json` is the smoke series.
- Not existing, by decision: `target/measure/portal-synth-3.json`, `portal-synth-3.log` and `portal-synth-3-summary.txt`.
- To run it later: re-execute Tasks 2 and 3 of this plan (ready gate, clock starts only after "start", three rounds, judged by the rows 16-17 rule).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 3 gap closure still has UAT test 10 open. The remaining open gaps recorded in 03-43 (Mica/Tabbed backdrop, grid scrollbar not visible) are unchanged by this plan.
- No requirement is marked Complete by this plan (UI-07 stays as it was).

## Self-Check: FAILED (honest, by decision)

- The plan's own success criterion is not met, and the checks below show what exists and what does not:
- FOUND: `target/measure/portal-synth-3-smoke.json`, `target/synth-500` (prepared data).
- MISSING by decision: `target/measure/portal-synth-3.json`, `target/measure/portal-synth-3-summary.txt` (the run was skipped).
- NOT DONE by decision: the new row and reading paragraph in `docs/MEASUREMENTS.md` (the acceptance criterion `contains: "after the in-view thumbnail requests"` is not satisfied).
- No task commits exist (`commits: 0`, measured: `git rev-list --count 328f40aa..HEAD` is 0 before the SUMMARY commit); nothing is uncommitted: the working tree was clean.
- No sucrose.exe or sucrose-portal.exe is running (`tasklist.exe` checked).

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-25 (halted: skipped by the maintainer)*
