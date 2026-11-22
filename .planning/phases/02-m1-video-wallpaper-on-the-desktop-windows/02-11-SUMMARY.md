---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 11
subsystem: engine-playback
tags: [libmpv, mpv-options, hwdec, ipc, engine-json, d-09, d-10]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "plan 04 LegacySettings.engine and legacy_mpv_cache_dir, plan 07 Player and the renderer, plan 10 supervisor"
provides:
  - "sucrose_engine_mpv::config::{embedding_options, generated_options, shader_cache_options, user_config_path, ALLOWED_OPTIONS}"
  - "sucrose_engine_mpv::player::{OpenOptions, Player::set_paused, Player::set_volume, Player::get_property_string}"
  - "sucrose_ipc::protocol::{PlaybackSettings, Assignment.settings, ToRenderer::Pause, ToRenderer::SetVolume}, PROTOCOL_VERSION = 2"
  - "Engine.json volume, loop, stretch, hardware acceleration and screen area reach libmpv; user uMpvPlayer.config is honored read-only"
affects: [02-13, 02-14, 02-15]

actuals:
  tokens: 7500
  tasks: 2
  commits: 2

plan_head_before: 7f90a3a0230ce59852c8664c9032e06cfd23d8a6
plan_head_after: 0ef3691bd549d7aaa6e265a579007e426ee5bb46

tech-stack:
  added: []
  patterns:
    - "Option names come from constants and values from typed settings; an allow-list constant is tested against everything the generator emits"
    - "Option order is user config or generated look first, embedding and hardening last, so a user config cannot weaken them"
    - "Ignored integration tests drive the real libmpv headless (vo=null, ao=null, no wid, lavfi test source)"

key-files:
  created:
    - crates/sucrose-engine-mpv/src/config.rs
    - crates/sucrose-engine-mpv/tests/options.rs
  modified:
    - crates/sucrose-core/src/settings.rs
    - crates/sucrose-ipc/src/protocol.rs
    - crates/sucrose-ipc/Cargo.toml
    - crates/sucrose-engine-mpv/Cargo.toml
    - crates/sucrose-engine-mpv/src/lib.rs
    - crates/sucrose-engine-mpv/src/player.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - Cargo.lock

key-decisions:
  - "Embedding and hardening options are applied after the user's mpv config (the plan's threat model had them before), so the config cannot turn scripts, ytdl or config loading back on"
  - "The shader cache options are always added to the embedding set as well, so a user config cannot send the cache into the C# app's folder"
  - "PlaybackSettings lives in sucrose-ipc as the plan says; sucrose-engine-mpv therefore depends on sucrose-ipc and sucrose-core"

requirements-completed: [ENG-01]

coverage:
  - id: D1
    description: "Engine.json volume, loop, stretch, hardware acceleration and screen area become libmpv options and a surface rectangle"
    requirement: ENG-01
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-mpv/src/config.rs#each_stretch_value_emits_exactly_its_legacy_option"
        status: pass
      - kind: integration
        ref: "crates/sucrose-engine-mpv/tests/options.rs#every_generated_option_is_accepted_by_the_pinned_libmpv"
        status: pass
    human_judgment: false
  - id: D2
    description: "The maintainer's 4K HEVC theme decodes on the GPU"
    requirement: ENG-01
    verification:
      - kind: other
        ref: "target/debug/sucrose.exe run with SUCROSE_DATA_DIR scratch: 'playback started hwdec=d3d11va'"
        status: pass
    human_judgment: false
  - id: D3
    description: "Pause, volume and loop work on a real stream"
    requirement: ENG-01
    verification:
      - kind: integration
        ref: "crates/sucrose-engine-mpv/tests/options.rs#pause_stops_the_clock_and_resume_restarts_it"
        status: pass
      - kind: integration
        ref: "crates/sucrose-engine-mpv/tests/options.rs#loop_file_inf_restarts_the_stream_at_its_end"
        status: pass
      - kind: integration
        ref: "crates/sucrose-engine-mpv/tests/options.rs#set_volume_changes_the_volume_property"
        status: pass
    human_judgment: false
  - id: D4
    description: "A user uMpvPlayer.config is loaded read-only and replaces the generated look"
    requirement: ENG-01
    verification: []
    human_judgment: true
    rationale: "No uMpvPlayer.config exists on the maintainer's machine, so the load path ran only as far as user_config_path unit tests; a real config file was not exercised end to end"

duration: 8min
completed: 2026-11-22
status: complete
---

# Phase 2 Plan 11: Video wallpaper settings and controls Summary

**Engine.json volume, loop, stretch, hardware decoding and screen area now reach libmpv through a generated, allow-listed option set (hwdec d3d11va on the maintainer's 4K HEVC theme), with the user's mpv config honored read-only and Pause/SetVolume controls proven on the real libmpv.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-11-22T15:37:00Z
- **Completed:** 2026-11-22T15:45:00Z
- **Tasks:** 2
- **Files modified:** 11 (2 created)

## Accomplishments

- `config.rs` generates the option set from `PlaybackSettings` (stretch mapping: UniformToFill panscan=1.0, Fill keepaspect=no, None video-unscaled=yes, Uniform mpv default); names are checked against `ALLOWED_OPTIONS`.
- The service builds `PlaybackSettings` from Engine.json and picks the surface rectangle by ScreenType (monitor rect or work area); protocol version is 2.
- The renderer logs `playback started hwdec=d3d11va` on the real run; no software fallback, so `hwdec-codecs=all` (A19) was not needed and no CPU comparison was taken.
- Four ignored integration tests (all headless) prove option validity on mpv v0.42.0-38, loop, pause/resume and volume; with the plan 01 load test, 5 ignored tests pass.
- `cargo xtask ci` (fmt, clippy -D warnings, tests) is green.

## Task Commits

1. **Task 1: Engine.json settings reach the player end to end** - `2c0395ae` (feat)
2. **Task 2: Pause/volume controls and options proven on the pinned libmpv** - `0ef3691b` (test)

## Files Created/Modified

- `crates/sucrose-engine-mpv/src/config.rs` - option generation, shader cache options, user config path, allow-list
- `crates/sucrose-engine-mpv/src/player.rs` - `OpenOptions`, config loading, set_paused/set_volume/get_property_string, hwdec in the start log; `embedded_options` removed
- `crates/sucrose-engine-mpv/tests/options.rs` - ignored headless integration tests
- `crates/sucrose-ipc/src/protocol.rs` - PlaybackSettings, Assignment.settings, Pause, SetVolume, version 2
- `crates/sucrose-core/src/settings.rs` - serde derives on the legacy enums
- `apps/sucrose/src/service.rs`, `apps/sucrose/src/render.rs` - settings and rectangle selection; option assembly, shared player for control messages

## Decisions Made

See key-decisions above. The embedding-after-user-config order and the always-redirected shader cache are stronger than the plan's threat model (T-02-25) and match the must-have that the embedding options stay.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Embedding options applied after the user config, shader cache always redirected**
- **Found during:** Task 1
- **Issue:** Applying embedding options before a user config would let it override hardening (scripts, ytdl, osc); a legacy config may also name its own `gpu-shader-cache-dir` in the C# folder, breaking the read-only rule (D-08).
- **Fix:** `Player::open` order is user config or generated options, then embedding options; the renderer appends `shader_cache_options` to the embedding set.
- **Files modified:** player.rs, config.rs, render.rs
- **Committed in:** 2c0395ae

**2. [Rule 3 - Blocking] Player API shape**
- **Found during:** Task 1 and 2
- **Issue:** mpv_load_config_file needs the handle before initialize, and the tests need a window-less player on a non-path source (a lavfi URL); `Player::open` required an absolute path and a wid.
- **Fix:** `Player::open(mpv, wid, file, &OpenOptions)` for the renderer; `#[doc(hidden)] Player::open_source(mpv, Option<wid>, source, &OpenOptions)` for tests; a failing user config logs a warning and the generated options apply.
- **Files modified:** player.rs
- **Committed in:** 2c0395ae

**3. [Rule 3 - Blocking] Pause/SetVolume landed in the Task 1 commit**
- **Found during:** Task 1
- **Issue:** The renderer's exhaustive match on `ToRenderer` and the player controls had to exist for Task 1 to compile and for the renderer to forward them.
- **Fix:** The protocol variants, `set_paused`, `set_volume` and the IPC forwarding were committed with Task 1; Task 2 added the tests.
- **Committed in:** 2c0395ae

**4. [Rule 1 - Bug risk] hwdec log moved to PLAYBACK_RESTART**
- **Issue:** hwdec-current is not set at FILE_LOADED (decoder not created yet); "playback started" is now logged at the first PLAYBACK_RESTART.
- **Committed in:** 2c0395ae

---

**Total deviations:** 4 auto-fixed (1 missing critical, 2 blocking, 1 bug risk)
**Impact on plan:** No scope change; all within the planned files plus `config.rs` extra helper `shader_cache_options`.

## TDD Gate Compliance

Task 2 is `tdd="true"`, but its implementation (Pause/SetVolume, Player controls) was already committed with Task 1, so there is no RED commit that failed before the implementation. The tests were written after the code and passed on first run; the option-validity test includes a negative control (a nonexistent option must be refused) so it cannot pass vacuously. This is a process gap, recorded here rather than hidden.

## Issues Encountered

- libmpv logged "Audio/Video desynchronisation detected" warnings for the 4K HEVC theme in the debug build (unoptimized renderer, hardware decode on). Not investigated here; release-build CPU/RAM measurements belong to the M1 measurement step.
- `MpvPlayer.config` (the C# generated config) exists in the maintainer's folder but there is no `uMpvPlayer.config`, so the user-config path was not exercised in a real run.

## Known Stubs

None.

## Threat Flags

None. The new surface (user mpv config load, option generation) is covered by T-02-24 and T-02-25; `open_source` is `#[doc(hidden)]` and takes only fixed test sources.

## Next Phase Readiness

- Plan 13 (fullscreen rule) can send `ToRenderer::Pause`; plan 14 (sync) can send `SetVolume`.
- Plan 15 records StayAwake and the ScreenType/option mapping in FEATURES.md.

## Self-Check: PASSED

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-22*
