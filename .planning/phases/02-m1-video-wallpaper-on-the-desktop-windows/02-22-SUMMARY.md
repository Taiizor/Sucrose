---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 22
subsystem: measurement
tags: [mpv, gpu, measurement, windows, meas-01]
requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "02-18 service-written GpuPreference, 02-21 same-adapter measure tool and the C# same-GPU reference"
provides:
  - "Same-GPU MEAS-01 rows (RTX 3050, Rust vs C# MpvPlayer) replacing the cross-GPU rows"
  - "On-machine proof that the renderer runs on the RTX 3050 through the service-written value (G-02-2b)"
  - "FEATURES Video row with the same-GPU result; mpv options confirmed for parity"
affects: [phase-2-verification, phase-4-performance]
actuals:
  tokens: 2500
  tasks: 3
  commits: 1
plan_head_before: dc847c46caf513abb0b44a93f79ca987be182128
plan_head_after: 0003b6efcf49b24100cd4d8573323af5354f2af0
tech-stack:
  added: []
  patterns: ["Adapter evidence comes from per-process GPU Engine counters, not DXGI order"]
key-files:
  created: []
  modified: [docs/MEASUREMENTS.md, docs/FEATURES.md]
key-decisions:
  - "mpv options deband=yes and profile=low-latency stay for parity: same-GPU R = 1.02 (<= 1.25), no DECISIONS entry"
requirements-completed: [MEAS-01, DESK-12]
coverage:
  - id: D1
    description: "Rust release renderer runs on the RTX 3050 through the service-written GpuPreference, proven by per-process GPU Engine counters matching the 'video GPU adapter' LUID"
    requirement: DESK-12
    verification:
      - kind: other
        ref: "cargo xtask measure --side rust (target/measure/rust-same-gpu.json, gpu_adapter LUID 0x00000000_0x0000FAEB)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Same-GPU MEAS-01 row and cross-GPU note in docs/MEASUREMENTS.md; FEATURES Video row corrected"
    requirement: MEAS-01
    verification:
      - kind: other
        ref: "cargo xtask measure --row (refuses cross-adapter rows) and grep acceptance checks on docs"
        status: pass
    human_judgment: false
duration: 12min
completed: 2026-11-23
status: complete
---

# Phase 2 Plan 22: Same-GPU measurement of the Rust renderer Summary

**On the same RTX 3050 the Rust renderer's GPU 3D engine averages 16.4 % against 16.1 % for the C# MpvPlayer (R = 1.02), with half the working set; the earlier 94 % vs 16.5 % claim was a cross-GPU artifact and is corrected.**

## Performance

- **Duration:** about 12 min (measurement itself 6 min)
- **Started:** 2026-11-23T02:22Z
- **Completed:** 2026-11-23T02:33Z
- **Tasks:** 3 (task 1 human action, task 2 measured and recorded, task 3 skipped by its condition)
- **Files modified:** 2

## Accomplishments

- Verified on the machine, with the C# app exited (tasklist count 0), that the release renderer runs on the NVIDIA adapter by itself.
- Measured Rust for 300 s after a 60 s warm-up and recorded the same-GPU row with `cargo xtask measure --row` (it accepts only same-adapter pairs).
- Replaced the cross-GPU rows in MEASUREMENTS.md, added the "compared two GPUs" note, rewrote the reading paragraph.
- Replaced the FEATURES Video row's 94 % vs 16.5 % claim with the same-GPU result.

## Machine evidence (G-02-2b)

- Service log: `selected theme theme=PEDoEdUHdEKlmxJFYH2Q1zk7T kind=Video source=s_cyber-samurai-princess-live-wallpaper.mp4`
- Service log: `GPU preference already set for the renderers preference=HighPerformance` (the value written by plan 02-18's run; the service rewrites it per exe path, no manual entry)
- Renderer-0 log: `video GPU adapter name=NVIDIA GeForce RTX 3050 Laptop GPU luid=0x00000000_0x0000FAEB dedicated_mb=3962`
- rust-same-gpu.json, renderer pid 67744: `gpu_adapter` = NVIDIA GeForce RTX 3050 Laptop GPU, LUID 0x00000000_0x0000FAEB (equals the log line); service pid 37680: no adapter (no GPU engine use).
- `reg.exe query ... UserGpuPreferences` for target\release\sucrose.exe: `GpuPreference=2;`
- No "desynchronisation" or "underrun" line in the scratch Renderer log.

## Results (same GPU, 1 monitor, 4K30 HEVC)

| | C# (Launcher + MpvPlayer + Backgroundog) | Rust (service + renderer) |
|---|---|---|
| RAM (working set avg) | 596 MB (MpvPlayer + Backgroundog 505) | 301 MB (peak 363) |
| Private | 1215 MB | 1038 MB |
| CPU avg | 0.70 % | 0.40 % |
| GPU 3D engine (max per process) | 16.1 % | 16.4 % |
| GPU memory | 733 MB dedicated / 80 MB shared | 719 MB dedicated / 70 MB shared |
| dwm.exe (Intel UHD, information only) | 4.8 % | 4.2 % |

**R = 16.4 / 16.1 = 1.02**, well under the 1.25 threshold.

## Task Commits

1. **Task 1: maintainer exits the C# app** - human action, no commit
2. **Task 2: verify, measure, record, correct FEATURES** - `0003b6ef` (docs)
3. **Task 3: deband/profile trial** - skipped: options kept for parity: R = 1.02 (no source change, no DECISIONS entry)

## Files Created/Modified

- `docs/MEASUREMENTS.md` - same-GPU rows, cross-GPU note, rewritten reading paragraph (CRLF kept)
- `docs/FEATURES.md` - Video row states the same-GPU result (file is LF in the working tree)

## Decisions Made

None - the mpv options stay as they are (parity with the C# MpvPlayer.config).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] target/release/sucrose.exe vanished after the first release build**
- **Found during:** Task 2 (starting the app)
- **Issue:** The first start failed ("Permission denied", then "file in use"), and shortly after the exe was gone from target/release (likely antivirus scanning or quarantine of the fresh unsigned binary; Defender history was not readable from here).
- **Fix:** Rebuilt the release binary (touched apps/sucrose/src/main.rs only for mtime, no content change) and started it through PowerShell Start-Process with SUCROSE_DATA_DIR pointing at the scratch folder. It then ran for the whole measurement.
- **Files modified:** none committed
- **Verification:** the exe ran 6 minutes; counters and logs above.

**2. [Plan accuracy] FEATURES.md has LF line endings**
- The plan said every edited file uses CRLF; docs/FEATURES.md is LF in the working tree (MEASUREMENTS.md is CRLF). Each file's existing line endings were preserved.

---

**Total deviations:** 1 auto-fixed blocking issue, 1 plan inaccuracy noted. **Impact:** none on results.

## Issues Encountered

None beyond the above. The measurement was run with the registry value left in place (by design; SVC-04 removes it).

## Known Stubs

None.

## Threat Flags

None - no source change; T-02-46 not triggered (config.rs untouched).

## Next Phase Readiness

G-02-6 and G-02-2b are closed by measurement; success criterion 4 and MEAS-01 now rest on a same-GPU comparison. Phase 2 verification can proceed. The C# app is still exited; the maintainer can restart it.

## Self-Check: PASSED

- docs/MEASUREMENTS.md and docs/FEATURES.md exist and changed; commit 0003b6ef is on HEAD; `cargo xtask ci` exit 0; acceptance greps: 'compared two GPUs' = 1, 'same GPU' = 2, 'open performance item' = 0.
