---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 29
subsystem: docs
tags: [measurements, meas-01, release-build, same-gpu, rtx-3050]
requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: the 02-27 status fix and the 02-23..02-25 read-back, location watch and re-place-all behavior, the 2026-11-23 same-GPU rows (02-21, 02-22)
provides:
  - one docs/MEASUREMENTS.md row for the final M1 build (release, one monitor, RTX 3050, 300 s after a 60 s warm-up)
  - a comparison sentence in the "reading the numbers" paragraph against the first same-GPU Rust row
affects: [phase verification, MEAS-01, MEAS-02 sign-off in Phase 8]
actuals:
  tokens: 900
  tasks: 3
  commits: 2
plan_head_before: 2fc55b0498c9f7c6ce153966541c6a917ee12ddf
plan_head_after: fc49656dc18c9157f0af9403d7ad878beb87a33c
tech-stack:
  added: []
  patterns:
    - "A measurement row that cannot be paired with the older C# result (LUIDs change at every boot) is Rust-only, says why, and points at the same-GPU C# reference row"
key-files:
  created: []
  modified:
    - docs/MEASUREMENTS.md
key-decisions:
  - "Rust-only row for the final M1 build: --row refused the C# pair (equal adapter names, different LUIDs after a reboot), so the C# column is empty and the result files were not edited"
requirements-completed: [MEAS-01]
coverage:
  - id: D1
    description: "The final M1 build (checked placement read-back, location watch, re-place-all, 02-27 fix) was measured on a release build and recorded as one MEASUREMENTS row"
    requirement: "MEAS-01"
    verification:
      - kind: other
        ref: "cargo xtask measure --side rust --out target/measure/rust-final-build.json (300 s after a 60 s warm-up, one renderer on the NVIDIA GeForce RTX 3050 Laptop GPU); the plan's grep and CRLF checks on docs/MEASUREMENTS.md"
        status: pass
    human_judgment: false
  - id: D2
    description: "The reading paragraph compares the final build with the first same-GPU Rust numbers and says the watch and read-backs run only at a display change or an explorer move"
    requirement: "MEAS-01"
    verification: []
    human_judgment: true
    rationale: "Prose comparison of measured values; adequacy of the wording is a reading judgment, the plan's grep checks only pin the text and the line endings"
duration: 10 min
completed: 2026-11-23
status: complete
---

# Phase 2 Plan 29: Measure the final M1 build Summary

**The final M1 release build measured 299 MB working set, 1034 MB private, 0.43 % CPU and 16.4 % GPU on the RTX 3050 with one monitor, level with the first same-GPU Rust row (301 MB, 1038 MB, 0.40 %, 16.4 %), recorded as a Rust-only MEASUREMENTS row because the machine rebooted since the C# run.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-11-23T19:03:23Z
- **Completed:** 2026-11-23T19:13:31Z
- **Tasks:** 3 (1 checkpoint, 1 tracer, 1 auto)
- **Files modified:** 1

## Accomplishments

- Task 1 (checkpoint:human-action): the maintainer replied "ready"; the precondition checks printed 0 C# Sucrose processes and 0 sucrose.exe.
- Release build of commit `2fc55b04` (contains the 02-27 fix: `grep -c 'fn slot_assigned(' apps/sucrose/src/service.rs` prints 1) started on the desktop with `SUCROSE_DATA_DIR` = target/scratch-data and measured with `cargo xtask measure --side rust` at the default log level. The started sucrose.exe processes were stopped with `taskkill.exe /IM sucrose.exe /F`; none is left.
- One row and one comparison sentence added to docs/MEASUREMENTS.md, CRLF kept on every line, no existing row changed.

### Log lines of the 15 s check (step 3)

- Service: `selected theme theme="PEDoEdUHdEKlmxJFYH2Q1zk7T" kind=Video source="s_cyber-samurai-princess-live-wallpaper.mp4"`
- Service: `renderers started count=1 monitors=["\\\\.\\DISPLAY1"] primary=Some("\\\\.\\DISPLAY1")` (only one renderer; Renderer-1 log has no line since the start)
- Renderer-0: `surface attached layout=Raised strategy=LayeredChild target=RectPx { left: 0, top: 0, right: 1920, bottom: 1080 } actual=RectPx { left: 0, top: 0, right: 1920, bottom: 1080 } ... verdict=Some(Shown) order="SHELLDLL_DefView > SucroseDesktopSurface > WorkerW"`
- Neither location-watch warning appears in the Service or Renderer-0 log after the start (no WARN or ERROR line since 19:04 UTC).

### Measurement (result file target/measure/rust-final-build.json, untracked)

| Process | WS avg (peak) | Private avg | CPU avg (peak) | GPU avg | GPU memory | Adapter |
|---|---|---|---|---|---|---|
| renderer (pid 34100) | 282.3 MB (345.2) | 1028.5 MB | 0.43 % (1.46 %) | 16.4 % | 719.1 MB dedicated / 70.7 MB shared | NVIDIA GeForce RTX 3050 Laptop GPU, LUID 0x00000000_0x0000FBD2 |
| service (pid 42304) | 17.0 MB | 5.9 MB | 0.00 % | 0.0 % | 0 | none |
| total | 299.4 MB (362.3) | 1034.4 MB | 0.43 % (1.46 %) | 16.4 % | 719.1 MB / 70.7 MB | |

dwm.exe: 4.6 % on the Intel UHD (information only).

### --row result

`--row` with target/measure/csharp-same-gpu.json **refused** the pair: "the adapter names are equal but the LUIDs differ; LUIDs change at every boot" (C# LUID 0x0000FAEB from 2026-11-23 17:xx, Rust LUID 0x0000FBD2 now; the machine rebooted in between). As the plan's step 6 says, no result file was edited and the check was not bypassed; the row was written by hand in the number format of the existing Rust row, with "–" in the C# column and notes that give the build, the adapter, the reason the C# column is empty and the pointer to the first 2026-11-23 row as the same-GPU C# reference. Printed (committed) row:

```
| 2026-11-23 | M1 video wallpaper, 1 monitor, final M1 build (Rust) | – | service + 1 renderer: RAM 299 MB (peak 362), private 1034 MB, CPU 0.43% (peak 1.5%), GPU 16.4%, GPU memory 719 MB dedicated / 71 MB shared | final M1 build 2fc55b04: checked placement read-back, location watch, re-place-all after display changes, 02-27 status fix; steady state, no display change during the sample; video: HEVC 3840x2160, 30.04 fps, ~12.25 Mbps, AAC 2 ch 48 kHz; release build; hardware decoding on; Windows 10.0.26300; GPU = max engine per process; GPU: NVIDIA GeForce RTX 3050 Laptop GPU (renderer LUID 0x00000000_0x0000FBD2). The C# column is empty because `--row` refused the pair: the machine rebooted since the C# run, so its LUID (0x0000FAEB) differs from this boot's (the adapter names are equal, the LUIDs are not), and C# was not re-measured; the same-GPU C# reference is the first 2026-11-23 row. dwm.exe GPU: 4.6 % on the Intel UHD (information only). |
```

## Task Commits

1. **Task 1: Maintainer exits the C# app and leaves one display active** - no commit (checkpoint, no files)
2. **Task 2: The final release build runs on the desktop, is measured, and its row lands in MEASUREMENTS.md** - `96ee07f1` (docs)
3. **Task 3: The reading paragraph compares the final build with the first same-GPU Rust row** - `fc49656d` (docs)

**Plan metadata:** the docs(02-29) completion commit that follows this file.

## Files Created/Modified

- `docs/MEASUREMENTS.md` - the final-M1-build row and the comparison sentence
- `target/measure/rust-final-build.json` - result file, untracked under the gitignored target/

## Decisions Made

Rust-only row instead of a same-adapter pair, because the LUID check correctly refused the older C# result after a reboot. Re-measuring C# was out of scope (the plan changes no code and treats the 2026-11-23 C# numbers as not re-measured).

## Deviations from Plan

None - plan executed exactly as written (the step 6 fallback for a refused `--row` is a planned path, not a deviation). The task 2 and task 3 edits to the same file were committed separately by writing the row first and the paragraph sentence second.

## Issues Encountered

None. The measurement is inside the plan's bounds: 299 MB working set and 0.43 % CPU are below the C# MpvPlayer + Backgroundog subtotal (505 MB, 0.69 %), and below +10 % of the 2026-11-23 Rust values (301 MB, 0.40 %; CPU is +0.03 percentage points, +7 %). Peak CPU 1.5 % against 1.1 % is noted in the paragraph. MEAS-02 is judged at sign-off in Phase 8.

Note: `grep -c 'fn slot_assigned' apps/sucrose/src/service.rs` prints 2 (a test name contains the substring); the definition is one (`fn slot_assigned(` prints 1), as explained by plan 02-27.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

Verification gap 2 (MEAS-01 numbers for the final behavior) is closed. 02-UAT.md was not edited (the plan lists no gap id). The phase's plans are all executed after this one.

## Self-Check: PASSED

- docs/MEASUREMENTS.md: FOUND; 6 rows starting with "| 20", 2 occurrences of "final M1 build", 1 of "0.40 % and 16.4 %", 0 lines without CR
- target/measure/rust-final-build.json: FOUND, one renderer on the NVIDIA GeForce RTX 3050 Laptop GPU
- Commits 96ee07f1 and fc49656d: FOUND on develop-rust
- No sucrose.exe left running; no C# Sucrose process was touched

The C# app and the virtual display can be turned back on now.
