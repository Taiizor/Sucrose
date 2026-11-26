---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 05
subsystem: engines
tags: [engine-trait, mpv, cef, surface-pump, assignment-engine, pause-level]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-04: EngineId, choose_engine, Availability, PauseLevel and sucrose_engine::sync; 04-02: the CEF host (CefHost, CefStart)"
provides:
  - "sucrose_engine::engine::{Engine, EngineClock, SurfacePump, Started, EngineError}: the seam every engine plugs into"
  - "sucrose_engine_mpv::engine::{MpvEngine, start}: the Player behind the trait, clock reads included"
  - "sucrose_engine_cef::engine::{CefEngine, CefPump, SurfaceHooks, CEF_PUMP_TIMER, start, file_url}: CEF behind the trait, commands queued for the surface thread"
  - "Assignment.engine (protocol version 5) and PlaybackSettings.pause: the service names the engine and the pause level"
  - "service log line 'engine chosen' (kind, configured, engine, fell_back) once per job"
affects: [04-06, 04-09, 04-12, 04-13, 04-15, 04-20]

actuals:
  tokens: 12900   # chars/4 over the added lines of the realized diff (51520 bytes, 17 files)
  tasks: 2
  commits: 2
plan_head_before: 451877d78f7bf2c44588f31c062b15f320ba7e78
plan_head_after: db798f2481b4064d94da03a28783609b40c5689f

tech-stack:
  added: []
  patterns:
    - "One renderer loop for every engine: thread-safe Engine handle for the IPC and clock threads, optional SurfacePump for the thread that owns the surface"
    - "Commands for a thread-bound engine go through a bounded crossbeam channel plus a LoopProxy wake; a full queue drops and logs, never blocks (T-04-13)"
    - "The renderer maps the Assignment's legacy engine name with EngineId::from_legacy and refuses an engine it does not contain (T-04-12)"

key-files:
  created:
    - crates/sucrose-engine/src/engine.rs
    - crates/sucrose-engine-mpv/src/engine.rs
    - crates/sucrose-engine-cef/src/engine.rs
  modified:
    - apps/sucrose/src/render.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/Cargo.toml
    - crates/sucrose-ipc/src/protocol.rs
    - crates/sucrose-engine/src/lib.rs
    - crates/sucrose-engine/Cargo.toml
    - crates/sucrose-engine-mpv/src/player.rs
    - crates/sucrose-engine-mpv/src/lib.rs
    - crates/sucrose-engine-cef/src/lib.rs
    - crates/sucrose-engine-cef/Cargo.toml
    - docs/ARCHITECTURE.md

key-decisions:
  - "EngineClock carries what the follower and the clock thread really read from the player: position, duration, frame period (with its final flag), speed, set_speed, seek_exact, paused, has_started, start_failure, loop_gen and wait_event(timeout) (the player's event channel behind a timeout wait). Engine adds name() and set_audio (default no-op) beyond the contract's list, because apply_role and the log need them"
  - "The pause level travels in PlaybackSettings.pause (the migrated PausePerformanceType), so a LoadSource also refreshes it; until Round D every engine treats Heavy like Light"
  - "CEF is offered by the service's static availability report only for Web themes; its generated pages for GIF and video come in a later round, so a user who configured CEF for videos keeps mpv"
  - "CefEngine::load_source is Unsupported and decide_use restarts the renderer whenever the running or the new wallpaper is a Web theme (Web to Web included), as the plan says"
  - "The renderer's start factory takes the surface hooks as a closure so only the CEF branch builds them"

patterns-established:
  - "Engine crates own their start-up: libmpv loading, option assembly and shader cache folder moved from render.rs into MpvEngine::start; CEF's file URL and failure mapping moved into the CEF engine"

requirements-completed: []

coverage:
  - id: D1
    description: "mpv starts and plays through the Engine trait with no behaviour change: the service chooses the engine, the Assignment carries it, the renderer builds MpvEngine"
    requirement: ENG-03
    verification:
      - kind: unit
        ref: "cargo test -p sucrose (render and service tests, tests/portal.rs unchanged: 26 passed)"
        status: pass
      - kind: integration
        ref: "scratch run: Service log 'engine chosen kind=Video configured=MpvPlayerLive engine=Mpv', Renderer log 'engine started engine=Mpv', status 'playing'"
        status: pass
    human_judgment: false
  - id: D2
    description: "CEF plays the hello fixture through the trait; its commands reach the surface thread through a bounded queue and no CEF call runs on another thread"
    requirement: ENG-03
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-cef/src/engine.rs#tests (queue order, wake per command, full queue drops without blocking, load_source unsupported)"
        status: pass
      - kind: integration
        ref: "scratch run on target/web-lib: 'engine chosen kind=Web engine=Cef', 'engine started engine=Cef', 'page loaded ... hello/index.html status=200'"
        status: pass
    human_judgment: false
  - id: D3
    description: "Pause reaches every engine as set_paused(paused, level) with the migrated PausePerformanceType as the level; no pause path suspends a process or attaches a debugger"
    requirement: PLAY-05
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-cef/src/engine.rs#commands_reach_the_queue_in_order_and_each_wakes_the_surface_thread"
        status: pass
      - kind: other
        ref: "grep NtSuspendProcess|DebugActiveProcess|SuspendThread over crates and apps prints 0 hits"
        status: pass
    human_judgment: true
    rationale: "Heavy is handled like Light until Round D and CEF only records the pause (page calls come with 04-09), so the user-visible pause behaviour of a Web wallpaper is not proven here; the verifier of the phase judges it"
  - id: D4
    description: "Loop and stretch reach the engine through the trait: mpv applies them live, CEF keeps them for the compatible keys of 04-09"
    requirement: PLAY-01
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-cef/src/engine.rs#the_engine_keeps_the_newest_value_of_each_setting"
        status: pass
    human_judgment: true
    rationale: "MpvEngine::set_loop and set_stretch are new live setters with no unit or run check of their own (no libmpv in the unit tests and no setting change is driven in this plan); CEF applies nothing yet"

# Metrics
duration: 19min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 05: Engine trait with mpv and CEF Summary

**One engine-agnostic renderer: mpv and CEF both play behind the Engine trait, the service names the engine in the Assignment through choose_engine, and every CEF call stays on the surface thread behind a bounded command queue**

## Performance

- **Duration:** 19 min
- **Started:** 2026-11-26T02:12:40Z
- **Completed:** 2026-11-26T02:31:00Z
- **Tasks:** 2
- **Files modified:** 17 (3 created)

## Accomplishments

- The renderer starts every wallpaper through `sucrose_engine::engine`: `start_engine` maps `Assignment.engine` with `EngineId::from_legacy`, builds mpv or CEF, and refuses an engine its build does not contain with a hint pointing to Wallpaper settings (T-04-12). The reader thread, the clock thread and the follower use `Arc<dyn Engine>` and `EngineClock`; the old CEF-only loop is gone, one message loop drives both engines.
- `MpvEngine` owns what render.rs did with the player (library loading, option assembly, shader cache folder, frame period reads, the event wait) and mpv behaves as in Phase 3: the apps/sucrose tests pass unchanged and a scratch run plays the maintainer's Video theme.
- `CefEngine` (Send + Sync) queues pause, volume, loop and stretch commands in a bounded crossbeam channel and wakes the loop; `CefPump` owns the browser, drains the queue on `LoopEvent::Wake`, pumps CEF and re-arms its timer. The hello fixture loads in a scratch run.
- The service fills each job's engine with `choose_engine` over a static report of this build (mpv, and CEF for Web themes), logs `engine chosen` once per job, and refuses a theme no engine can play. The pause level (`PausePerformanceType`) and the engine are carried to the renderer (protocol version 5).

## Task Commits

1. **Task 1: mpv plays through the Engine trait** - `6541e28c` (feat)
2. **Task 2: CEF plays behind the trait; IPC-thread commands reach the CEF UI thread through a queue** - `db798f24` (feat)

**Plan metadata:** the docs commit after this summary.

## Files Created/Modified

- `crates/sucrose-engine/src/engine.rs` - Engine, EngineClock, SurfacePump, Started, EngineError
- `crates/sucrose-engine-mpv/src/engine.rs` - MpvEngine and the mpv start function
- `crates/sucrose-engine-cef/src/engine.rs` - CefEngine, CefPump, SurfaceHooks, start, file_url
- `apps/sucrose/src/render.rs` - one engine-agnostic renderer (factory, reader, clock thread, loop with pump)
- `apps/sucrose/src/service.rs` - Job.engine via choose_engine, built_availability, pause level, Web restarts on Use
- `crates/sucrose-ipc/src/protocol.rs` - Assignment.engine, PlaybackSettings.pause, protocol version 5
- `crates/sucrose-engine-mpv/src/player.rs` - set_looping and set_stretch live setters
- `docs/ARCHITECTURE.md` - the Engine seam

## Decisions Made

See key-decisions. Neither the trait contract nor any Accepted decision was reopened; EngineClock and Engine grew by the members the renderer's real calls need.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] CEF offered for GIF and video themes would have regressed users who configured it**
- **Found during:** Task 1 (service.rs, choose_engine call)
- **Issue:** The plan reports CEF Installed whenever it is compiled in. For a Video theme whose Engine.Video setting is CefSharpLive the choice would then pick CEF, which has no generated page for a video yet and would open the raw file; before this plan every Video theme played in mpv.
- **Fix:** `built_availability(kind)` reports CEF Installed only for Web themes; for other types it is NotInThisBuild, so choose_engine falls back to mpv and logs fell_back=true.
- **Files modified:** apps/sucrose/src/service.rs
- **Verification:** scratch run shows kind=Video configured=MpvPlayerLive engine=Mpv; kind=Web engine=Cef
- **Committed in:** 6541e28c

**2. [Rule 3 - Blocking] rustfmt rewrote touched Rust files and Cargo.lock to LF**
- **Found during:** Task 1 (cargo fmt)
- **Issue:** The working tree keeps these files CRLF (autocrlf); the formatter wrote LF.
- **Fix:** Converted every touched file back to CRLF before each commit; `cargo fmt --all --check` passes on the CRLF files.
- **Committed in:** 6541e28c, db798f24

### Notes

- The first `cargo xtask ci` run failed two timing-based tests in `apps/sucrose/tests/portal.rs` (`a_service_with_the_tray_hidden_and_nothing_to_play_exits_when_its_only_portal_leaves`, `quick_toggles_leave_the_state_of_the_last_value`) while the machine was building in parallel (the portal test binary took 6 s instead of 2.3 s). The tests passed in three further standalone runs and in the second full `cargo xtask ci` run (exit 0). They look load-sensitive and were not touched by this plan.

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** No scope change; the first keeps Phase 3 behaviour for video themes.

## Issues Encountered

None beyond the notes above.

## Known Stubs

- `CefPump::drain` in crates/sucrose-engine-cef/src/engine.rs only logs the queued pause, volume, loop and stretch commands at debug level; applying them to the page is plan 04-09 (page calls). The engine keeps the latest values and exposes them through getters.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Later engines (WebView2, built-in GIF, libVLC, Media Foundation, the application embedder) plug into `start_engine` and implement `Engine`; 04-06 replaces `file_url` with the virtual origin behind `sucrose_engine_cef::engine::start`; 04-09 applies the queued commands to the page; 04-12 replaces `built_availability` with detection.
- ENG-03 stays partial (completes in Round B); PLAY-05, PLAY-01 and DESK-05 stay partial until Round E, so `requirements-completed` is empty.

## Self-Check: PASSED

Created files exist, both task commits are ancestors of HEAD, `pub trait Engine` present, `Player::open` count in render.rs is 0, `choose_engine` is used in service.rs, `impl Engine for CefEngine` and `sucrose_engine_cef::engine::start` are present, and no suspend or debug-attach call exists in the code.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*
