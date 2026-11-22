---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 17
subsystem: platform
tags: [windows, registry, dxgi, direct3d11, gpu-preference, windows-rs]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: Platform trait, Capabilities and the Windows os_windows modules (plans 02-01 to 02-16)
provides:
  - GpuPreference setting enum (Engine.json GraphicPreference) with legacy_ordinal
  - Platform::set_gpu_preference (HKCU UserGpuPreferences write with the legacy merge semantics)
  - Platform::gpu_adapters (DXGI adapters with the typeperf LUID text)
  - Platform::default_gpu_adapter (adapter read back from a default Direct3D 11 device)
  - Capabilities.gpu_preference (true on Windows, false on macOS and Linux)
affects: [02-18 GPU preference wiring, 02-21 cargo xtask measure, DESK-12]

actuals:
  tokens: 18000
  tasks: 2
  commits: 2

tech-stack:
  added: []
  patterns:
    - "Legacy registry value merge as a pure function (merged_value) tested separately from the Win32 write"
    - "Scratch HKCU subkey with a Drop guard for registry tests, never the real DirectX key"

key-files:
  created:
    - crates/sucrose-platform/src/os_windows/gpu.rs
  modified:
    - crates/sucrose-core/src/settings.rs
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - Cargo.lock

key-decisions:
  - "default_gpu_adapter creates a D3D11 device with no adapter and reads IDXGIDevice::GetAdapter instead of taking an entry of the EnumAdapters1 list, because the enumeration order is not evidence of the adapter a device gets"
  - "sucrose-platform re-exports sucrose_core::settings::GpuPreference so callers of the Platform trait need no second import"

patterns-established:
  - "GPU adapter LUIDs are reported as 0x{HighPart:08X}_0x{LowPart:08X}, the form typeperf uses in GPU instance names"

requirements-completed: [DESK-12]

plan_head_before: 4bee2c8acbdbb8583b09e9434d6639b136e0f20e
plan_head_after: 42da95c4e83405501ce60ec2cc785f09c34b69f0
commits: 2
duration: 8 min
completed: 2026-11-23
status: complete
---

# Phase 2 Plan 17: GPU preference platform half Summary

**Windows UserGpuPreferences write with the exact C# merge rules, a DXGI adapter list in typeperf LUID form, and the adapter a default Direct3D 11 device really gets, all behind the Platform trait**

## Performance

- **Duration:** 8 min
- **Started:** 2026-11-22T22:27:42Z
- **Completed:** 2026-11-22T22:35:30Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- `GpuPreference` (Default, PowerSaving, HighPerformance; default HighPerformance) with `legacy_ordinal` 0/1/2 in sucrose-core.
- `Platform::set_gpu_preference(exe, preference)` writes `GpuPreference=<n>;` under `HKCU\Software\Microsoft\DirectX\UserGpuPreferences` named by the exe path. It keeps other pairs, matches keys case-insensitively, treats a missing or out-of-range value as Default, skips an unchanged value and touches only that one value (T-02-38).
- `Platform::gpu_adapters()` and `Platform::default_gpu_adapter()` give plan 02-18 and `cargo xtask measure` the means to name the GPU a renderer really uses. `Capabilities.gpu_preference` is true on Windows and false elsewhere (D-002).

## Task Commits

1. **Task 1: set an exe's GPU preference with the legacy value semantics** - `ef5c809e` (feat)
2. **Task 2: GPU adapter list and the default D3D11 device's adapter** - `42da95c4` (feat)

**Plan metadata:** the docs commit that carries this SUMMARY.

## Files Created/Modified
- `crates/sucrose-platform/src/os_windows/gpu.rs` - merge rules, registry write, DXGI list, default-device adapter, 14 tests
- `crates/sucrose-core/src/settings.rs` - `GpuPreference` enum and tests
- `crates/sucrose-platform/src/lib.rs` - `Capabilities.gpu_preference`, `GpuPreferenceChange`, `GpuAdapter`, three trait methods with unsupported defaults
- `crates/sucrose-platform/src/os_windows.rs` - capability and delegation to the gpu module
- `crates/sucrose-platform/Cargo.toml`, `Cargo.lock` - `sucrose-core` workspace dependency and the Registry, Dxgi, Direct3D and Direct3D11 features of the pinned windows crate (no new crate)

## Decisions Made
- The default adapter is read back from a device, never derived from the enumeration order (plan requirement, documented on both trait methods).
- `GpuPreference` is re-exported from `sucrose-platform`.

## Deviations from Plan

None - plan executed exactly as written. Each TDD task was committed once with its tests (pure merge tests and registry tests were written together with the implementation); the tests are independent of Win32 state except the scratch-key ones.

## Issues Encountered
- `cargo fmt` rewrote edited files to LF; CRLF was restored with `sed` before each commit so the working tree keeps its line endings.
- The registry tests delete their own scratch subtree but leave the empty parent `HKCU\Software\SucroseTest`: the two tests run in parallel and deleting the parent would race. It holds no values.

## Verification
- `cargo test -p sucrose-core settings`: 6 passed; `cargo test -p sucrose-platform gpu`: 14 passed (plan minimums 7 and 10).
- `cargo clippy -p sucrose-core -p sucrose-platform --all-targets -- -D warnings` and `cargo xtask ci`: exit 0.
- All acceptance greps meet their expectations.

## Known Stubs
None.

## Threat Flags
None beyond the plan's threat model (T-02-38 mitigated as specified; T-02-SC: no crate added).

## Next Phase Readiness
Plan 02-18 can call `set_gpu_preference` before the first renderer starts, log `default_gpu_adapter` per renderer, and mark DESK-12 shipped. `docs/FEATURES.md` was not touched here because 02-18 records the feature as shipped.

## Self-Check: PASSED
- FOUND: crates/sucrose-platform/src/os_windows/gpu.rs
- FOUND commits ef5c809e and 42da95c4 (ancestors of HEAD)

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-23*
