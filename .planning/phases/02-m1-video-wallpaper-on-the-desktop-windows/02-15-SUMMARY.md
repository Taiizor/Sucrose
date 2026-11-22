---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 15
subsystem: measurement
tags: [xtask, sysinfo, typeperf, gpu-engine, mpv, meas-01]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "the Rust service and per-monitor libmpv renderers measured here (plans 06-14)"
provides:
  - "cargo xtask measure --side rust|csharp and --row, reusable for MEAS-02"
  - "MEAS-01 rows in docs/MEASUREMENTS.md (C# PerDisplay vs Rust, one monitor; Rust all monitors)"
  - "FEATURES.md, ARCHITECTURE.md and MEASUREMENTS.md describe the M1 that actually ships"
affects: [phase-2-verification, MEAS-02, performance-tuning]

actuals:
  tokens: 15000
  tasks: 3
  commits: 2
plan_head_before: a78bfc9b6bf306d72f3474e223092a0e4771c9cb
plan_head_after: cd4bf222b69ede172763f6fb66d71285a79a8896

tech-stack:
  added: ["sysinfo 0.40.4 (default-features = false, features = [\"system\"]) in xtask"]
  patterns:
    - "GPU counters are read by running the Windows tool typeperf (no FFI); GPU = max over engine types per process; non-Windows prints n/a"
    - "Totals are summed per sample over present processes, then averaged; GPU samples are aligned to CPU samples by index; a changed process set is a warning in the result file"

key-files:
  created: [xtask/src/measure.rs]
  modified: [Cargo.toml, Cargo.lock, xtask/Cargo.toml, xtask/src/main.rs, docs/MEASUREMENTS.md, docs/FEATURES.md, docs/ARCHITECTURE.md]

key-decisions:
  - "GPU memory is also read from \\GPU Process Memory(*)\\Shared Usage (gpu_shared_avg_mb) because Dedicated Usage is always 0 on this integrated GPU"
  - "\"private (commit)\" is accurate: sysinfo virtual_memory() on Windows is PROCESS_MEMORY_COUNTERS_EX.PrivateUsage (A10 answered)"

requirements-completed: [MEAS-01]

coverage:
  - id: D1
    description: "cargo xtask measure samples processes by image name (warm-up, 300 s window, WS, private, CPU and GPU) and writes a JSON result; --row prints the MEASUREMENTS row"
    requirement: MEAS-01
    verification:
      - kind: unit
        ref: "cargo test -p xtask measure"
        status: pass
      - kind: integration
        ref: "cargo xtask measure --side rust and --side csharp on the maintainer's machine (target/measure/*.json)"
        status: pass
    human_judgment: false
  - id: D2
    description: "MEAS-01 rows recorded: C# PerDisplay vs Rust on one monitor, and Rust all monitors"
    requirement: MEAS-01
    verification:
      - kind: other
        ref: "grep -c 'M1 video wallpaper' docs/MEASUREMENTS.md prints 3"
        status: pass
    human_judgment: true
    rationale: "The numbers are measured, but reading them (GPU 93.7 % vs 16.5 %) and deciding whether it is acceptable for M1 is a maintainer judgment"
  - id: D3
    description: "FEATURES.md Video and Stay awake rows and the ARCHITECTURE.md status line match the shipped M1"
    verification:
      - kind: other
        ref: "grep -c 'except the M0 skeleton' docs/ARCHITECTURE.md prints 0; grep -c 'M1 does not read StayAwake' docs/FEATURES.md prints 1"
        status: pass
    human_judgment: false

duration: 40min
completed: 2026-11-22
status: complete
---

# Phase 2 Plan 15: Measurement and docs Summary

**`cargo xtask measure` (sysinfo plus typeperf GPU counters) measured the release Rust wallpaper against the C# MpvPlayer engine: RAM and CPU are on par or lower, but the Rust renderer holds the GPU 3D engine at 93.7 % against 16.5 % for C#.**

## Performance

- **Duration:** about 40 min wall clock for this continuation and the first executor's work (the two 6-minute samples dominate)
- **Started:** 2026-11-22T16:22Z
- **Completed:** 2026-11-22T17:03Z
- **Tasks:** 3 (Task 2 was the maintainer checkpoint)
- **Files modified:** 8

## Accomplishments

- `cargo xtask measure --side rust|csharp [--warmup 60] [--duration 300] [--out]` and `--row`, with unit tests for the CSV and formatting helpers; it only reads process statistics and never touches C# processes (T-02-33).
- MEAS-01 measured on one 4K HEVC theme, 300 s average after a 60 s warm-up, one app at a time, release builds, hardware decoding on, Windows 10.0.26300, 16 logical CPUs.
- Docs updated: MEASUREMENTS (two rows plus a reading note), FEATURES (Video, Stay awake), ARCHITECTURE (status line).

## Results (MEAS-01)

| | C# (Launcher + MpvPlayer + Backgroundog) | C# MpvPlayer + Backgroundog | Rust (service + 1 renderer) |
|---|---|---|---|
| Working set avg (peak) | 599 MB (663) | 509 MB (571) | 618 MB (618) |
| Private (commit) avg | 1220 MB | 1188 MB | 593 MB |
| CPU avg (peak), % of machine | 0.77 % (2.3) | 0.76 % (2.3) | 0.51 % (1.8) |
| GPU, max engine per process | 16.5 % | 16.5 % | 93.7 % |
| GPU memory | 733 MB dedicated / 81 MB shared | same | 0 dedicated / 564 MB shared |

Rust breakdown: service 22 MB WS / 6.4 MB private / 0.00 % CPU; renderer 596 MB WS / 587 MB private / 0.51 % CPU (peak 1.76 %).

**Performance finding to carry forward (not fixed here).** The Rust renderer keeps the GPU 3D engine at about 94 % (hardware decode works, VideoDecode about 6 %), where the C# MpvPlayer uses 16.5 %. RAM and CPU meet the "lower than C#" goal; GPU does not. Candidate causes are mpv `vo`, `gpu-api` and scaling options (not investigated here). This needs a decision at Phase 2 verification or in a later phase; it is recorded in docs/MEASUREMENTS.md and the FEATURES.md Video row.

## Task Commits

1. **Task 1: cargo xtask measure** - `e3397fb4` (feat)
2. **Task 2: maintainer starts the C# app** - checkpoint:human-action, no commit
3. **Task 3: C# measurement, MEAS-01 rows, FEATURES and ARCHITECTURE** - `cd4bf222` (docs)

**Plan metadata:** the docs(02-15) completion commit that follows this SUMMARY.

## Files Created/Modified

- `xtask/src/measure.rs` - sampling, typeperf parsing, result JSON, `--row` formatting, tests
- `Cargo.toml`, `Cargo.lock`, `xtask/Cargo.toml`, `xtask/src/main.rs` - sysinfo 0.40.4 dependency and the `measure` arm
- `docs/MEASUREMENTS.md`, `docs/FEATURES.md`, `docs/ARCHITECTURE.md` - as above

## Decisions Made

- Dedicated GPU memory reads 0 on this integrated GPU, so shared usage is reported next to it.
- The columns are labeled "private (commit)"; verified as PrivateUsage.
- The C# subtotal (MpvPlayer + Backgroundog) is reported next to the three-process total (D-27).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical] GPU shared memory counter**
- **Found during:** Task 1
- **Issue:** `\GPU Process Memory(*)\Dedicated Usage` is always 0 on this integrated GPU, so GPU memory would have read as zero for the Rust renderer.
- **Fix:** also sample `Shared Usage` (`gpu_shared_avg_mb`) and show both in the row.
- **Files modified:** xtask/src/measure.rs
- **Committed in:** e3397fb4

**2. [Docs] The FEATURES Video row note was rewritten more broadly than the plan's text**
- **Found during:** Task 3
- **Issue:** the existing row still said multi-monitor, pause and volume "follow in later M1 plans", which is no longer true after plans 11 to 14.
- **Fix:** kept the hardening and exit-code details, added the plan's note, the multi-monitor statement and the measurement caveat.
- **Files modified:** docs/FEATURES.md
- **Committed in:** cd4bf222

---

**Total deviations:** 2 (1 Rule 2, 1 documentation accuracy)
**Impact on plan:** none on scope.

## Issues Encountered

- `crates/sucrose-platform/src/os_windows/display.rs` and `xtask/src/measure.rs` show as modified in `git status` while `git diff` is empty: a CRLF/stat-cache artifact of `core.autocrlf`, left untouched and not committed.
- Nothing else; the C# processes were only read, and are still running.

## Verification scope

How each ROADMAP success criterion for Phase 2 was checked, from this phase's SUMMARYs. The machine has one display (`\\.\DISPLAY1`), so everything about more than one monitor rests on unit tests.

| # | Criterion | How it was checked |
|---|---|---|
| 1 | Selected video plays behind the icons on every monitor | **Verified on hardware for one monitor only** (plans 06, 07, 11: video behind the icons, icons clickable, `hwdec=d3d11va`). More than one monitor: `cargo test -p sucrose topology` (one renderer per monitor keyed by device name, diff on change). |
| 2 | Survives an explorer.exe restart and monitors being plugged or unplugged | **One monitor only, no real explorer kill or hot-plug on hardware** (plan 12 deliberately did not kill the maintainer's explorer; a synthetic TaskbarCreated run touching only Sucrose's windows was done). Covered by `cargo test -p sucrose topology`, `cargo test -p sucrose debounce`, `cargo test -p sucrose shell_restart`, and for the master moving when a monitor goes away `cargo test -p sucrose-engine-mpv sync` plus `cargo test -p sucrose elect_master` (plan 14). Open human checks: real explorer restart, display scale 125/150 %, second or virtual display hot-plug, cross-monitor sync on two screens. |
| 3 | Fullscreen app applies the configured action (Pause pauses and resumes) | Unit and integration evidence: `cargo test -p sucrose fullscreen`, `cargo test -p sucrose-platform`, and a run on the maintainer's real settings (FullScreenPerformance = Resume, rule off). Not observed with a real fullscreen app and Pause or Close: that needs the maintainer's own edit of Backgroundog.json (D-08). Open human check. |
| 4 | CPU and RAM measured against C# MpvPlayer + Backgroundog and recorded | Done in this plan: rows in docs/MEASUREMENTS.md from `cargo xtask measure` on release builds. GPU result flagged above. |
| 5 | Plays the theme selected in the C# app (Library.json, General.json unchanged), sharp at each monitor's DPI, Quit stops it | Selection read read-only from the maintainer's real files (plans 03, 04, 08); video plays (plan 07); Quit posted to the service window ends renderers and service with exit code 0 (plan 09). Open human checks: sharpness at a non-100 % scale, tray tooltip and menu as drawn on a real right click, desktop repaint after a real Quit. |

`/gsd-verify-work` should carry the open human checks (rows 2, 3 and 5) and the GPU finding into 02-VERIFICATION.md.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `cargo xtask measure` is ready for MEAS-02 (Phase 8 and later phases).
- Decide at verification whether the GPU usage of the Rust renderer is acceptable for M1 or needs an mpv option pass.

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-22*

## Self-Check: PASSED

Found: xtask/src/measure.rs, docs/MEASUREMENTS.md, docs/FEATURES.md, docs/ARCHITECTURE.md; commits e3397fb4 and cd4bf222 are ancestors of HEAD; `cargo xtask ci` exit 0.
