---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 20
subsystem: engine-sync
tags: [follower-sync, pi-controller, libmpv, multi-monitor, drift]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: master election and first-frame gating (plan 02-19), follower clock correction (02-06)
provides:
  - frame_period(fps): the follower band is one frame of the video actually playing
  - Follower PI controller (proportional speed, integral rate trim, hysteresis, anti-windup)
  - DriftSummary / Follower::take_summary and a periodic "follower drift summary" log
  - drift_s, the signed drift for logging
affects: [02-21 measure, phase 02 UAT re-run G-02-2c]

actuals:
  tokens: 11000
  tasks: 2
  commits: 2
plan_head_before: 14166e79440b51f88c703c0dcd72dff8ba815b15
commits: 2
plan_head_after: d81b37f615f215e9f7f4f3c510d80e7884d28e9c

tech-stack:
  added: []
  patterns:
    - "Pure controller with a 1 Hz simulation harness in the tests (master rate, follower rate, start offset)"
    - "Integral anti-windup by zone, saturation and a closing-drift guard"

key-files:
  created: []
  modified:
    - crates/sucrose-engine-mpv/src/sync.rs
    - apps/sucrose/src/render.rs
    - docs/FEATURES.md

key-decisions:
  - "Integral gain 0.1 (plan start value 0.03) plus a closing-drift guard: with 0.03 and the plain zone/saturation anti-windup the simulations overshoot the one-frame band (the integral accumulated while the proportional term closes in drives the follower to the band edge and beyond)"
  - "Followers assume a new player runs at speed 1.0 for the first speed-change comparison"

requirements-completed: [DESK-01]

duration: 8min
completed: 2026-11-23
status: complete
---

# Phase 2 Plan 20: Follower convergence (G-02-2c, offset half) Summary

**Followers hold the same frame as the master to within one frame of the actual video: PI speed controller with an fps-derived band, release at a quarter frame, an integral rate trim and a 30-tick drift summary log.**

## Performance

- **Duration:** about 8 min
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- `sync.rs`: `frame_period(fps)` (1/fps for 5..=240, else 1/30), a `Follower` with correcting hysteresis (enter above one frame, leave at a quarter), proportional gain 0.5/s limited to +/-5 %, an integral trim (gain 0.1, zone 0.1 s, anti-windup) and 0.1 % speed-change suppression; seeks beyond 0.5 s (2 s gap) clear the integral; a paused master returns the master's speed and clears the integral.
- Simulation tests (1 Hz ticks, 60 fps): 0.3 s behind converges within one frame from tick 10 and stays for 60 ticks without a seek; 0.45 s ahead from tick 12; master 1.5 % slow from tick 20; follower 3 % slow from tick 40; master slow plus offset from tick 25; a 25 ms drift is corrected on the first tick and within a quarter frame by tick 5; also checked at 30 fps. Speed never leaves 0.95..1.05 of the master's.
- `render.rs`: the frame rate is read once the first frame is shown (`container-fps`, else `estimated-vf-fps`) and logged ("frame period"); `follow_clock` passes that period to `Follower::step`, logs speed changes at debug (speed, drift) and seeks at info, and every 30 ticks logs "follower drift summary" (mean and max drift in frames, speed changes, seeks). A new role resets the follower.
- `docs/FEATURES.md`: the correction numbers are replaced by the "Follower correction (02-20)" text; master election (02-19) and the refresh-rate sentence are kept.

## Task Commits

1. **Task 1: follower controller (fps band, PI speed, release, drift summary)** - `641c7e67` (feat)
2. **Task 2: renderer wiring, drift summary log, FEATURES** - `d81b37f6` (feat)

## Files Created/Modified

- `crates/sucrose-engine-mpv/src/sync.rs` - controller, frame_period, DriftSummary, drift_s, 29 sync tests
- `apps/sucrose/src/render.rs` - FollowState, video_frame_period, summary logging
- `docs/FEATURES.md` - follower correction note

## Decisions Made

See key-decisions above. Gains are named constants with doc comments in `sync.rs`; the simulation tests are the contract, not the planner's starting gains.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Planner gains did not converge within one frame**
- **Found during:** Task 1 (prototype simulation before writing the Rust)
- **Issue:** With INTEGRAL_GAIN 0.03 and the specified anti-windup (zone + saturation only) the integral winds up while the proportional term closes a 0.3 s offset; after release the follower drifts to the band edge and overshoots it (0.0172 s at 60 fps), failing the "stays within one frame" tests.
- **Fix:** INTEGRAL_GAIN = 0.1 and CLOSING_FRACTION = 0.75: the integral does not accumulate while the drift shrinks faster than 25 % per tick (the controller closing in by itself). Checked in simulation with up to 3 ms measurement noise at 60, 30 and 24 fps.
- **Files modified:** crates/sucrose-engine-mpv/src/sync.rs
- **Commit:** 641c7e67

**2. [Rule 3 - Blocking] Added `pub fn drift_s`**
- **Found during:** Task 2 (the plan asks to log speed changes with the drift, which `step` does not return)
- **Fix:** small public helper in sync.rs (tested) used by the debug log.
- **Commit:** d81b37f6

### Other notes

- TDD: tests and implementation for Task 1 were written together and committed as one `feat` commit, not a separate RED commit (the plan task is `type="auto"`; the simulation tests were first validated against a JS prototype of the controller).
- `SPEED_STEP` was removed; `correct` now uses `MAX_SPEED_STEP` for its direction.
- Line endings: `cargo fmt` rewrites CRLF to LF in the working tree; CRLF was restored on both Rust files. `docs/FEATURES.md` is LF in the working tree (the plan said CRLF) and was left LF.
- Residual: at 120 fps the one-tick speed granularity (0.1 %) exceeds an 8.3 ms band by about 5 % in simulation; 60 fps and below stay within the band.

## Issues Encountered

None.

## Human check pending

The plan's `<human-check>` (plug a virtual display, compare both monitors, read the "frame period" and "follower drift summary" log lines) was not run: it needs the maintainer's desktop and virtual display, and human_verify_mode is end-of-phase. It is left for the phase UAT.

## Known Stubs

None.

## Threat Flags

None. T-02-43 mitigations applied: speed bounded to +/-5 % of the master's, changes below 0.1 % suppressed, seeks only beyond 0.5 s with a 2 s gap, anti-windup on the integral.

## Self-Check: PASSED

- sync.rs, render.rs, FEATURES.md changed as listed; commits 641c7e67 and d81b37f6 are on develop-rust.
- `cargo test -p sucrose-engine-mpv sync` 29 passed, `cargo test -p sucrose` passed, `cargo xtask ci` exit 0; acceptance greps print 1 each.
