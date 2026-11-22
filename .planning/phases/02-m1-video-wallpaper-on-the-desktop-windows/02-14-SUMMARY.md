---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 14
subsystem: engine
tags: [libmpv, ipc, master-clock, drift-correction, audio, multi-monitor]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: one renderer per monitor and TopologyAction::Close (plan 12), pause and volume messages and the Player wrapper (plan 11), action scopes (plan 13)
provides:
  - "sucrose_platform::monotonic_ns (QueryPerformanceCounter scaled to ns)"
  - "IPC protocol version 3: ToService::Clock, ToRenderer::SetRole and ToRenderer::Clock, ClockTick and Role"
  - "Player::{set_speed, seek_exact, set_audio, time_pos, duration, speed, is_paused, loop_gen, take_events}"
  - "sucrose_engine_mpv::sync::{correct, Follower, SyncAction}: pure follower drift controller"
  - "service master election (primary monitor, else lowest device name), master-only clock relay, promotion on monitor loss"
affects: [02-15, phase-4-audio-rules, phase-4-engine-choice]

actuals:
  tokens: 11000
  tasks: 2
  commits: 3

tech-stack:
  added: []
  patterns:
    - "roles and the master slot live under the same mutex as the renderer connections, so a role is never sent from a stale election"
    - "pure decision functions (correct, Follower::step, elect_master, role_changes, clock_targets) with the IO around them"

key-files:
  created:
    - crates/sucrose-engine-mpv/src/sync.rs
  modified:
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - crates/sucrose-platform/src/os_linux.rs
    - crates/sucrose-platform/src/os_macos.rs
    - crates/sucrose-platform/src/unsupported.rs
    - crates/sucrose-ipc/src/protocol.rs
    - crates/sucrose-engine-mpv/src/lib.rs
    - crates/sucrose-engine-mpv/src/player.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - docs/FEATURES.md

key-decisions:
  - "ToService::Clock is appended after Playing (wire index 3), not after Failed as the plan text said, because variants are only ever appended (Playing was added by plan 09)"
  - "Wrap and seek detection uses MPV_EVENT_PLAYBACK_RESTART after the first one, not an observed time-pos, so the event thread does not wake 60 times a second per renderer (low resource use)"
  - "Follower wraps correct() with memory: returns to the master's speed once inside a frame, repeats no identical nudge, and waits 2 s between exact seeks"
  - "A promoted master restores speed 1.0 before it publishes"

patterns-established:
  - "Clock path: master renderer clock thread -> service reader thread of that slot -> followers' connections; the relay runs on the reader thread and never wakes the main loop"

requirements-completed: [DESK-01]

duration: 9 min
completed: 2026-11-22
status: complete
plan_head_before: 5bc25617e298ec7247e7da9f4868fa52f737b3c4
plan_head_after: cd19d5fd04af3216e3650983507d978dc5663454
commits: 3

coverage:
  - id: D1
    description: "Followers compute drift against the master's extrapolated clock and leave it, nudge speed by 2 % or seek exactly, with circular distance across the loop end"
    requirement: DESK-01
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-mpv/src/sync.rs#tests (13 tests)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The service elects the primary monitor's renderer as master, relays only its clock, and promotes the next renderer with SetRole(Master) and SetVolume when the master's monitor goes away"
    requirement: DESK-01
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#tests elect_master_*, a_promoted_renderer_*, a_demoted_master_*, only_the_masters_clock_*"
        status: pass
    human_judgment: false
  - id: D3
    description: "Only the master plays audio, on one monitor the single renderer is the audible master"
    requirement: DESK-01
    verification:
      - kind: other
        ref: "target/debug/sucrose.exe run with a scratch data dir: Renderer log 'role role=Master', service log 'master clock elected old=None new=Some(0)'"
        status: pass
    human_judgment: true
    rationale: "Whether sound is actually heard once, and whether two monitors show the same frame, needs ears, eyes and a second display"
---

# Phase 2 Plan 14: Master clock and follower drift correction Summary

**Master/follower playback sync over IPC: the primary monitor's renderer is the QPC-stamped clock and the only audio source, followers correct drift with a pure 2 % speed or exact-seek controller, and the service promotes the next renderer when the master's monitor goes away.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-11-22T16:12:50Z
- **Completed:** 2026-11-22T16:21:51Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- Protocol version 3 with `ClockTick`, `Role`, `ToService::Clock`, `ToRenderer::SetRole` and `ToRenderer::Clock`, all appended, with wire-index tests.
- `monotonic_ns` in sucrose-platform (Windows: QueryPerformanceCounter, cached frequency; other OSes: documented wall-clock fallback).
- `Player` gained speed, exact seek, audio on/off (aid), position and duration reads, an observed pause flag, a loop-wrap counter and a channel of `PlayerEvent`s from the event thread.
- `sync::correct` (None inside one frame, Speed 0.98 or 1.02 up to 0.5 s, Seek beyond, circular distance in a loop, paused master gives nothing) and `sync::Follower` (memory around it).
- Service: `elect_master`, role sent after every Assign (under the connection lock), clock relayed only from the master's verified connection (T-02-30), `Fleet::elect` after every applied snapshot sending `role_changes` (T-02-31: ticks about 1 Hz, seeks only above 0.5 s and 2 s apart).
- Renderer: opens with `aid=no`; Master turns audio on at the configured volume, restores speed 1.0 and publishes a tick every second and at once after seek, wrap, pause, resume and promotion; Follower corrects through `Follower::step` and skips while paused locally.
- FEATURES.md documents the rule and the below-one-frame residual between refresh rates.

## Task Commits

1. **Task 1: roles, Clock relay, audio only on the master (tracer)** - `176b60f6` (feat)
2. **Task 2: drift cases, promotion, pause and re-align, documented bound (TDD)**
   - RED `70d95598` (test): 7 sync tests and 2 service tests failed on assertions against stubs
   - GREEN `cd19d5fd` (feat)

**Plan metadata:** added by the docs commit that follows this summary.

## Files Created/Modified

See the frontmatter key-files. `crates/sucrose-engine-mpv/src/sync.rs` is new; the rest extend existing modules.

## Decisions Made

See key-decisions. In short: Clock appended after Playing, restart events instead of observing time-pos, a Follower with memory around the pure `correct`, speed restored on promotion.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test written wrong in the RED commit**
- **Found during:** Task 2 (GREEN)
- **Issue:** `a_second_seek_within_two_seconds_waits` used a follower position that equalled the master's expected position at the second check, so it could never seek
- **Fix:** the follower is at 15.0 s for the late check
- **Files modified:** crates/sucrose-engine-mpv/src/sync.rs
- **Committed in:** cd19d5fd

### Plan text adjustments (not bugs)

- `ToService::Clock` has wire index 3 (after Playing), see key-decisions.
- The plan's `correct` returns `Speed(1.0)` once inside the frame window but gives it no local speed to know it was nudged; that return-to-normal lives in `Follower::step`, `correct` itself returns None inside the window as the behavior list requires.
- `Player` observes only `pause`; position jumps come from playback-restart events (resource use).

---

**Total deviations:** 1 auto-fixed (1 bug in a test), 3 plan-text adjustments
**Impact on plan:** none on scope.

## Issues Encountered

- On the one-display machine a debug build logs "Audio/Video desynchronisation detected" about three seconds after the master switches audio on. Audio is enabled mid-stream (aid=no to auto), which is the likely cause; it needs a listen in the plan 07 human check and a release-build look. Not investigated further here.
- mpv also logs "client removed during hook handling" at start; not caused by this plan as far as can be told, not investigated.

## Human check (Task 2): not verifiable: one display

This machine has one display (`\\.\DISPLAY1`), so steps 2 to 4 (same frame on two monitors for 5 minutes, one audio stream, audio moves on unplugging the primary) were not run (RESEARCH Pitfall 12). The thresholds (A5) and the QPC cross-process assumption (A4) stay assumed. One-display run: service logs `master clock elected old=None new=Some(0)`, renderer logs `role role=Master`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 15 can build on the roles. A second display or a virtual display driver is needed to close A4 and A5 and the promotion human check. The pre-existing uncommitted change to `crates/sucrose-platform/src/os_windows/display.rs` was present before this plan and was left alone.

## Known Stubs

None.

## Threat Flags

None - no surface beyond the plan's threat model (the Clock relay checks the sender's slot against the master).

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-22*

## Self-Check: PASSED

- FOUND: crates/sucrose-engine-mpv/src/sync.rs
- FOUND: 176b60f6, 70d95598, cd19d5fd (ancestors of HEAD)
- `cargo xtask ci` green; `grep -c 'M1 sync (D-02)' docs/FEATURES.md` prints 1; PROTOCOL_VERSION is 3
