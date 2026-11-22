---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 18
subsystem: service
tags: [windows, gpu-preference, registry, direct3d11, legacy-settings, libmpv]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: Platform::set_gpu_preference, gpu_adapters, default_gpu_adapter and GpuPreference (plan 02-17)
provides:
  - EngineSettings.gpu_preference read from legacy Engine.json GraphicPreference (name or ordinal, default HighPerformance)
  - The service writes UserGpuPreferences for its own sucrose.exe before the first renderer starts
  - Each renderer logs the "video GPU adapter" libmpv's default Direct3D 11 device gets, before libmpv starts
  - DESK-12 recorded as shipped on Windows in Phase 2
affects: [02-21 measure, 02-22 same-GPU re-measurement, SVC-04 uninstaller]

actuals:
  tokens: 9000
  tasks: 2
  commits: 2

tech-stack:
  added: []
  patterns:
    - "Service-side apply_gpu_preference: capability-gated, failure only warns, exe path never logged"
    - "Adapter evidence is the per-process GPU Engine counters, not the DXGI enumeration order"

key-files:
  created:
    - crates/sucrose-config/tests/fixtures/Engine.gpu.json
  modified:
    - crates/sucrose-config/src/legacy.rs
    - crates/sucrose-config/tests/legacy.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - docs/FEATURES.md
    - docs/ARCHITECTURE.md
    - .planning/REQUIREMENTS.md

key-decisions:
  - "Engine.json GraphicPreference is read with get_enum (names and ordinals 0..=2); anything else is HighPerformance (T-02-39)"
  - "The preference is written only when a wallpaper job exists, right before the first Fleet::apply"

requirements-completed: [DESK-12, ENG-01]

duration: 6 min
completed: 2026-11-23
status: complete
commits: 2
plan_head_before: 6b34bd0000c3319a25b14ae209e6dfa06256e68e
plan_head_after: fa57c62759de6042f917b866aa46d657749895e8

coverage:
  - id: D1
    description: "Engine.json GraphicPreference is read (default HighPerformance, name, ordinal, invalid falls back)"
    requirement: DESK-12
    verification:
      - kind: unit
        ref: "crates/sucrose-config/tests/legacy.rs#graphic_preference_defaults_to_high_performance, graphic_preference_is_read_by_name, graphic_preference_accepts_the_ordinal"
        status: pass
    human_judgment: false
  - id: D2
    description: "The service writes GpuPreference=2; for target/release/sucrose.exe before the first renderer, on a machine with no manual entry"
    requirement: DESK-12
    verification:
      - kind: integration
        ref: "release run: Service log 'GPU preference written for the renderers', reg.exe shows GpuPreference=2;"
        status: pass
    human_judgment: false
  - id: D3
    description: "The renderer's video GPU adapter line names the RTX 3050 and the same LUID the per-process GPU Engine counters show"
    requirement: ENG-01
    verification:
      - kind: integration
        ref: "typeperf GPU Engine counters vs Renderer-0 log, LUID 0x00000000_0x0000FAEB"
        status: pass
    human_judgment: false
  - id: D4
    description: "The 4K60 theme plays smoothly on the NVIDIA GPU without a manual setting (Task Manager GPU engine column, 2 minute watch)"
    verification: []
    human_judgment: true
    rationale: "Smoothness and Task Manager's display need a human at the machine; the harvested end-of-phase human-check covers it"
---

# Phase 2 Plan 18: GPU preference wiring Summary

**The service now writes the legacy Engine.json GraphicPreference into UserGpuPreferences for sucrose.exe before any renderer starts, and each renderer logs the adapter libmpv's Direct3D 11 device gets; on the hybrid laptop the renderers run on the RTX 3050 with no manual setting.**

## Performance

- **Duration:** 6 min
- **Tasks:** 2
- **Files modified:** 8 (1 created)

## Accomplishments

- `EngineSettings.gpu_preference` reads `GraphicPreference` (name or ordinal, invalid or missing gives HighPerformance), with three `graphic_preference` fixture tests and the new `Engine.gpu.json` fixture.
- `service.rs`: `Plan.gpu`, the "legacy settings" log gains `gpu_preference`, and `apply_gpu_preference` runs before the first `Fleet::apply` (capability-gated, warns on failure, never logs the exe path).
- `render.rs`: `log_video_adapter` logs name, LUID and dedicated memory from `default_gpu_adapter()` before libmpv starts.
- Docs: FEATURES row ported, ARCHITECTURE status and per-OS row, REQUIREMENTS DESK-12 complete and SVC-04 extended.

## Machine evidence (G-02-2b)

First run after the maintainer removed the manual entry (release build, `SUCROSE_DATA_DIR` scratch folder, C# app not running):

- Service log: `GPU preference written for the renderers preference=HighPerformance`; `reg.exe` then showed `sucrose.exe    REG_SZ    GpuPreference=2;`.
- Renderer-0 log: `video GPU adapter name=NVIDIA GeForce RTX 3050 Laptop GPU luid=0x00000000_0x0000FAEB dedicated_mb=3962`.
- Per-process GPU Engine counters (typeperf, 5 samples, summed over the three sucrose.exe pids): LUID `0x00000000_0x0000FAEB` = 330.222, LUID `0x00000000_0x0000F608` (the Intel adapter) = 0. The counter LUID and the log line agree.
- No "Audio/Video desynchronisation" or "underrun" line in the scratch Renderer logs.

The registry value stays after the run (by design, SVC-04 removes it).

## Task Commits

1. **Task 1: service applies GraphicPreference, renderers log their adapter** - `53581370` (feat)
2. **Task 2: docs say DESK-12 shipped in Phase 2 on Windows** - `fa57c627` (docs)

## Decisions Made

None beyond the plan. Task 1 is a single commit instead of RED/GREEN commits because the plan is `type: execute` and the tests and the change share files.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Shared scratch folder in the `engine()` test helper**
- **Found during:** Task 1
- **Issue:** every `engine()` call wrote Engine.json into the same `engine-inline` scratch folder, so tests running in parallel overwrote each other; `graphic_preference_accepts_the_ordinal` failed intermittently with the wrong value.
- **Fix:** one scratch folder per call (atomic counter in the helper name).
- **Files modified:** crates/sucrose-config/tests/legacy.rs
- **Verification:** `cargo test -p sucrose-config` green three times in a row, and in `cargo xtask ci`.
- **Committed in:** 53581370

**Total deviations:** 1 auto-fixed (1 bug in a test helper). **Impact:** none on product code.

## Issues Encountered

None. The first dispatch stopped at the Task 1 precondition (manual graphics entry still present); after the maintainer removed it and the C# app was confirmed closed, execution passed it.

## Known Stubs

None.

## User Setup Required

Done by the maintainer: removing the manual graphics preference entry for target\release\sucrose.exe. The entry Sucrose itself wrote now exists in Windows Settings > System > Display > Graphics; it is expected.

## Next Phase Readiness

- Human check pending (end of phase): Task Manager "GPU engine" column on the renderers, and 2 minutes of smooth 4K60 playback.
- Plans 02-21 and 02-22 re-measure on the same GPU (G-02-6).
- Open decision (not planned): mpv `d3d11-adapter` from EnumAdapterByGpuPreference.

## Self-Check: PASSED

- Created file exists: crates/sucrose-config/tests/fixtures/Engine.gpu.json
- Commits 53581370 and fa57c627 are ancestors of HEAD; `cargo xtask ci` exit 0.
- Acceptance greps: GraphicPreference 2, set_gpu_preference 1, "video GPU adapter" 1, default_gpu_adapter 2, FEATURES/REQUIREMENTS/ARCHITECTURE greps 1 each.

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-23*
