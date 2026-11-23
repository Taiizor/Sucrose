---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 21
subsystem: measurement
tags: [xtask, measure, typeperf, gpu-adapter, luid, dwm, g-02-6]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: Platform::gpu_adapters() and the renderer adapter logging (plans 02-17, 02-18)
provides:
  - cargo xtask measure records the GPU adapter (LUID and name) every measured process used
  - dwm.exe GPU sample, shown next to each side and never added to a total
  - cargo xtask measure --row refuses a cross-adapter or adapter-less comparison (G-02-6)
  - target/measure/csharp-same-gpu.json, the C# reference measured on the RTX 3050
affects: [02-22 Rust measurement and MEASUREMENTS.md rows]

actuals:
  tokens: 9000
  tasks: 3
  commits: 1
plan_head_before: abf0b360ead4f79ee2ac41ecd65492f6bae79414
commits: 1
plan_head_after: bbda1f27f3a36d2ef5f4b367cb47ecc438671cd4

tech-stack:
  added: []
  patterns:
    - "A measurement row is only produced when both sides' per-pid counter LUID is the same adapter"

key-files:
  created: []
  modified:
    - xtask/src/measure.rs
    - xtask/Cargo.toml
    - Cargo.lock

key-decisions:
  - "The adapter evidence is the per-pid typeperf LUID with the largest summed utilization over the run; Platform::gpu_adapters() is only a LUID-to-name lookup"
  - "Reports written before this change deserialize with no adapter and are refused by --row"

requirements-completed: [MEAS-01]

coverage:
  - id: D1
    description: "measure records each process's GPU adapter, dwm.exe sample, and --row refuses a cross-adapter or adapter-less pair"
    requirement: MEAS-01
    verification:
      - kind: unit
        ref: "xtask/src/measure.rs#measure_same_adapter_accepts_one_luid_and_refuses_the_rest"
        status: pass
      - kind: unit
        ref: "cargo test -p xtask measure (17 passed)"
        status: pass
      - kind: other
        ref: "cargo xtask measure --row on the old 02-15 csharp.json/rust.json exits 1 with the no-adapter message"
        status: pass
    human_judgment: false
  - id: D2
    description: "The C# MpvPlayer reference measured on the RTX 3050 with the 4K30 theme on one monitor"
    requirement: MEAS-01
    verification:
      - kind: other
        ref: "target/measure/csharp-same-gpu.json lists Sucrose.Live.MpvPlayer on NVIDIA GeForce RTX 3050 Laptop GPU"
        status: pass
    human_judgment: false

duration: ~40min (including the 6 min sampling run)
completed: 2026-11-23
status: complete
---

# Phase 2 Plan 21: Same-GPU measurement tool and C# reference Summary

**cargo xtask measure now records each process's GPU adapter from the per-pid counter LUID, samples dwm.exe, and refuses --row for a cross-GPU pair; the C# MpvPlayer reference was re-measured on the RTX 3050 at 16.1 % GPU.**

## Performance

- **Tasks:** 3 (1 auto/TDD, 1 human-action checkpoint, 1 measurement)
- **Files modified:** 3 (xtask/src/measure.rs, xtask/Cargo.toml, Cargo.lock)
- **Plan-level verification:** `cargo test -p xtask measure` 17 passed; `cargo xtask ci` exit 0

## Accomplishments

- `ProcessReport.gpu_adapter` (LUID plus name) and `Report.dwm` are recorded, printed and kept in the JSON; both are `serde(default)` so older results still read.
- `same_adapter(csharp, rust)` gates `--row`: different LUIDs or a missing adapter exit non-zero with a clear message and no row. Verified on the old 02-15 files: "the C# result has no GPU adapter for mpv-player pid 44364 ... the comparison is not valid, measure again".
- The C# side was measured under controlled conditions (see below), so plan 02-22 can measure the Rust side against it.

## Task Commits

1. **Task 1: measure records each process's GPU adapter, samples dwm.exe and refuses a cross-adapter row** - `bbda1f27` (feat)
2. **Task 2: maintainer prepares the C# run** - human action, no commit
3. **Task 3: measure the C# side** - no source change; result written to `target/measure/csharp-same-gpu.json` (not committed, under target/)

## C# measurement (Task 3)

Conditions: one active monitor (internal panel, virtual display disabled), Cyber Samurai Princess (4K HEVC, 30 fps), PerDisplay, hardware acceleration on, no Rust sucrose.exe running. 60 s warm-up, 300 s sampled, 16 logical CPUs. Started 2026-11-23T02:12:03Z. No C# process was started, stopped or changed.

| role | pid | WS avg | private avg | CPU avg | GPU avg | GPU mem | GPU adapter |
|---|---|---|---|---|---|---|---|
| launcher | 16184 | 91.3 MB | 33.1 MB | 0.01 % | 0.0 % | 0.0 MB | none |
| mpv-player | 43632 | 417.7 MB | 1141.9 MB | 0.50 % | 16.1 % | 733.1 MB (+79.7 MB shared) | NVIDIA GeForce RTX 3050 Laptop GPU (LUID 0x00000000_0x0000FAEB) |
| backgroundog | 66212 | 87.3 MB | 40.2 MB | 0.19 % | 0.0 % | 0.0 MB | none |
| total | | 596.3 MB | 1215.2 MB | 0.70 % | 16.1 % | 733.1 MB | |
| subtotal (MpvPlayer + Backgroundog) | | 505.0 MB | 1182.1 MB | 0.69 % | 16.1 % | 733.1 MB | |

dwm.exe GPU avg: **4.8 % on Intel(R) UHD Graphics** (LUID 0x00000000_0x0000F608), information only. The internal panel is driven by the Intel adapter while the C# video renders on the NVIDIA adapter, so the desktop compositor runs on a different GPU than the video; plan 02-22 should compare the Rust side's dwm.exe value against this.

The result file is `target/measure/csharp-same-gpu.json` (adapter recorded, MpvPlayer on NVIDIA as expected).

## Decisions Made

- Adapter evidence is the per-pid counter LUID with the largest summed utilization; `gpu_adapters()` is only used to look the name up.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. The tool needed no fix after the real run (the adapter was present for the MpvPlayer; the launcher and backgroundog correctly have none because they use no GPU).

## Authentication Gates

None. Task 2 was the planned human-action checkpoint (maintainer set up the C# run).

## Known Stubs

None.

## Threat Flags

None. The tool only reads process statistics and counters (T-02-44); no crate was added (T-02-SC).

## Next Phase Readiness

- Plan 02-22 can measure the Rust side with the same tool and record rows. The C# app is still running on purpose; 02-22 asks the maintainer to exit it before the Rust measurement.
- Open point for 02-22: the display is attached to the Intel adapter (dwm.exe on Intel) while the C# video is on NVIDIA; check where the Rust renderer runs and compare dwm.exe on both sides.

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-23*

## Self-Check: PASSED

- xtask/src/measure.rs, xtask/Cargo.toml: FOUND; commit bbda1f27 is an ancestor of HEAD
- target/measure/csharp-same-gpu.json: FOUND, MpvPlayer adapter contains "NVIDIA"
- cargo test -p xtask measure and cargo xtask ci: exit 0
