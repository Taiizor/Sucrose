---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 07
subsystem: engine-playback
tags: [libmpv, mpv, wid, embedding, windows, hardening, exit-codes]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "plan 01 libmpv loader (load_default, LoadError::hint); plan 06 DesktopSurface::native_handle, Assignment.source, renderer message loop"
provides:
  - "sucrose_engine_mpv::player::{Player::open, PlayerError, embedded_options}: mpv embedded with wid, loadfile loop, event thread, ordered shutdown"
  - "Renderer plays the selected video behind the desktop icons on the primary monitor (DESK-01 for one monitor)"
  - "Renderer exit codes: 2 = libmpv missing or not loadable (logged with hint), 4 = other start failure (PlayerError or attach)"
  - "mpv hardening set pinned by tests (load-scripts, ytdl, config, input bindings, osc all off)"
affects: [02-08, 02-09, 02-10, 02-11, 02-14]

actuals:
  tokens: 4300
  tasks: 2
  commits: 3

plan_head_before: d490d361ea144e6f2f1269c6ee525c4cd2550a35
plan_head_after: fd8291adb12dadde357d3ffb9a3a09d9bfa83e54

tech-stack:
  added: []
  patterns:
    - "Only the renderer loads libmpv; the Player is opened on the thread that owns and pumps the wid parent window"
    - "Engine start failures are values (EngineError) with one log line and one exit code, never a panic (D-20)"
    - "Drop order: Player (event thread joined, mpv destroyed) before the DesktopSurface"

key-files:
  created:
    - crates/sucrose-engine-mpv/src/player.rs
  modified:
    - crates/sucrose-engine-mpv/src/lib.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/Cargo.toml
    - Cargo.lock
    - docs/FEATURES.md

key-decisions:
  - "Raised/LayeredChild surface from plan 06 composites a libmpv d3d11 video child as-is; no strategy swap needed on Windows 10.0.26300"
  - "embedded_options() holds the embedding and hardening set until plan 11 generates options from Engine.json"
  - "EngineError (Load -> exit 2, Player -> exit 4) is the single start-failure path in render.rs"

patterns-established:
  - "Hardening options are asserted by pure unit tests that were proven able to fail by mutation"

requirements-completed: [DESK-01, DESK-06, ENG-01, TYPE-01]

duration: "~13 min (checkpoint continuation; Task 1 ran in the earlier session)"
completed: 2026-11-22
status: complete

coverage:
  - id: D1
    description: "The selected theme's video plays behind the desktop icons on the primary monitor, icons clickable above it, no taskbar or Alt-Tab entry, audio at volume 100"
    requirement: DESK-01
    verification:
      - kind: manual_procedural
        ref: "Task 1 human check on the maintainer's desktop (approved); renderer log 'surface attached' then 'playback started'"
        status: pass
    human_judgment: true
    rationale: "Compositing behind the icons and audio can only be judged on the real desktop"
  - id: D2
    description: "libmpv is loaded only in the renderer process and played through Player embedded via wid"
    requirement: ENG-01
    verification:
      - kind: manual_procedural
        ref: "Task 1 automated run: surface attached + playback started in the renderer log; service code has no loader calls"
        status: pass
    human_judgment: false
  - id: D3
    description: "mpv hardening options (load-scripts, ytdl, config, input bindings, osc) are pinned; no duplicate option names; wid is not in the list"
    requirement: TYPE-01
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-mpv/src/player.rs#untrusted_theme_content_cannot_run_scripts_or_fetch_urls"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-mpv/src/player.rs#no_option_is_named_twice_and_wid_is_left_to_open"
        status: pass
    human_judgment: false
  - id: D4
    description: "A missing libmpv ends the renderer with code 2 and a logged hint; a PlayerError ends it with code 4; neither panics"
    requirement: ENG-01
    verification:
      - kind: unit
        ref: "apps/sucrose/src/render.rs#a_missing_engine_exits_with_code_2"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/render.rs#a_player_failure_exits_with_code_4"
        status: pass
      - kind: manual_procedural
        ref: "libmpv-2.dll moved aside, sucrose.exe started: Renderer log has one ERROR line 'libmpv cannot be loaded' with the hint; DLL moved back"
        status: pass
    human_judgment: false
  - id: D5
    description: "The video is as sharp and full-size at a non-100 % display scale as at 100 %"
    requirement: DESK-06
    verification: []
    human_judgment: true
    rationale: "Sharpness at 125/150 % needs the maintainer to change the Windows display scale; deferred to the end-of-phase verifier (human_verify_mode is end-of-phase)"
---

# Phase 2 Plan 07: Video wallpaper on the desktop Summary

**The selected video theme now plays looping behind the desktop icons on the primary monitor through the bundled libmpv embedded with wid in the renderer only, with scripts, ytdl, config and input hardened off by tests and a missing libmpv ending the renderer with exit code 2 and a hint.**

## Performance

- **Duration:** ~13 min for this continuation (Task 1 ran in the earlier session)
- **Completed:** 2026-11-22
- **Tasks:** 2 (Task 1 tracer, Task 2 TDD)
- **Files modified:** 6 (5 code and manifest files, docs/FEATURES.md)

## Accomplishments

- `Player::open` creates an mpv handle, sets `wid` and the hardening options before `mpv_initialize`, starts `loadfile` and runs a single event thread that logs "playback started", end-file errors and libmpv log lines; drop joins the thread and destroys the instance.
- The renderer loads libmpv, opens the player on the main thread that pumps the surface and plays `Assignment.source`; the service never maps libmpv-2.dll (D-17).
- Task 1 human check approved on the real desktop: video behind the icons covering the monitor, icons and labels clickable above it, no taskbar or Alt-Tab entry, audio at volume 100.
- Engine start failures run through one path (`EngineError`): LoadError exits 2, PlayerError exits 4, each logged once at error level, no panic.

## Task Commits

1. **Task 1: selected video plays on the surface behind the icons** - `a25f3a07` (feat)
2. **Task 2 RED: hardening tests and exit-code tests** - `bbdad7ca` (test)
3. **Task 2 GREEN: exit code 2 for a missing engine, 4 for a player failure** - `fd8291ad` (feat)

**Plan metadata:** the docs(02-07) commit that adds this file (FEATURES.md, STATE.md, ROADMAP.md, REQUIREMENTS.md).

## Files Created/Modified

- `crates/sucrose-engine-mpv/src/player.rs` - Player, PlayerError, embedded_options and their tests
- `crates/sucrose-engine-mpv/src/lib.rs` - exports the player module
- `apps/sucrose/src/render.rs` - start_engine, EngineError (exit code and log), ordered shutdown, exit-code tests
- `apps/sucrose/Cargo.toml`, `Cargo.lock` - dependency on sucrose-engine-mpv
- `docs/FEATURES.md` - Video and Surface-behind-icons rows now 🟨 with the Rust behavior

## Decisions Made

- Plan 06's Raised/LayeredChild strategy works unchanged with a libmpv d3d11 video child on Windows 10.0.26300; no swap of the strategy order.
- `embedded_options()` is the interim option set; plan 11 replaces its body with options from Engine.json and moves the hardening assertions to its config tests.
- A `LoadError` and a `PlayerError` are distinguished only by exit code (2 versus 4), as the plan specifies.

## Deviations from Plan

None - plan executed exactly as written. The RED step of Task 2 had two parts: the `embedded_options()` tests already passed against Task 1's implementation, so they were proven able to fail by temporarily changing `ytdl` to `yes` and adding a duplicate `osc` (both tests failed, file restored); the exit-code mapping test failed against a stub that returned 4 for both variants (expected 2), which is the genuine RED.

## Issues Encountered

- The service log shows only "a renderer disconnected" when the engine is missing; the exit code itself is not visible in any log (the service does not record child exit codes yet). Code 2 is pinned by the unit test and the error line with the hint was observed in the real run; plan 10 (restart budget) is the natural place to log child exit codes.

## Human-Verification Items

- **DESK-06 at a non-100 % display scale (open, for the end-of-phase verifier):** set the Windows display scale to 150 % (or 125 %), start `target/debug/sucrose.exe` with a scratch SUCROSE_DATA_DIR, check the video is as sharp and full-size as at 100 % and the renderer log shows the same physical rect; restore the scale. It needs the maintainer to change the display scale. It is also still pending from plan 06.
- **Desktop after stopping (for plan 09):** after `taskkill /IM sucrose.exe /F` in this plan's run the normal Windows wallpaper came back with no last frame and no black surface left. In plan 06 a black surface did linger after a force-kill. The two observations differ, so plan 09 should still decide on an explicit desktop refresh; the cause of the difference (video child versus plain surface, or timing) is not established.

## Measurements

Not measured in this plan (debug build, no RAM or CPU numbers recorded). `docs/MEASUREMENTS.md` is unchanged; the release-build measurement belongs to the end-of-phase work.

## Known Stubs

None.

## Threat Flags

None. The hardening mitigation of T-02-14 (options set before mpv_initialize, option errors fatal) is implemented and tested; T-02-15 is accepted as planned.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The Player contract (`Player::open(Arc<Mpv>, wid, &Path, &[(&str, String)])`, `embedded_options()`) is ready for plans 09 (surface refresh and pause), 11 (Engine.json options) and 14.
- Open: the 150 % scale sharpness check (DESK-06) and the desktop-refresh question.

## Self-Check: PASSED

- FOUND: crates/sucrose-engine-mpv/src/player.rs, apps/sucrose/src/render.rs
- FOUND commits: a25f3a07, bbdad7ca, fd8291ad
- `cargo xtask ci` (fmt, clippy -D warnings, test) exit 0
