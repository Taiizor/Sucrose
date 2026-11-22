---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 13
subsystem: performance-rules
tags: [windows, fullscreen, win32, dwm, debounce, ipc, pause, close]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "plan 04 Backgroundog settings, plan 11 ToRenderer::Pause, plan 12 EventLoop timers, Fleet and SlotMap"
provides:
  - "sucrose_platform::{ForegroundWindow, Platform::foreground_window} (default None; Windows implementation; Capabilities.fullscreen_detection = true on Windows)"
  - "apps/sucrose fullscreen::{covered_monitors, Debounce, ActionScope, RuleAction, plan_action, Rule, FULLSCREEN_TIMER}"
  - "The legacy fullscreen rule: Pause and Close actions, 5/3 debounce, 1 s timer that exists only while the rule is on"
affects: [02-14, 02-15]

actuals:
  tokens: 10600
  tasks: 2
  commits: 3

plan_head_before: 00628090e787b57927d1b43f2409624fee5a2499
plan_head_after: cae210724f18ded9b43a1782245bf966f65f644b

tech-stack:
  added: []
  patterns:
    - "The rule is a pure state machine (Debounce, Rule, plan_action) that says what to do; the service performs it"
    - "A renderer closed by the rule is tracked in RuleClosed and only started again after its exit event was seen"

key-files:
  created:
    - crates/sucrose-platform/src/os_windows/foreground.rs
    - apps/sucrose/src/fullscreen.rs
  modified:
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - apps/sucrose/src/main.rs
    - apps/sucrose/src/service.rs
    - docs/FEATURES.md

key-decisions:
  - "The predicate compares the foreground rectangle with the full monitor rectangles from Platform::monitors(), not with the surface rectangles of the fleet, so a maximized window never counts when ScreenType is WorkingArea"
  - "Recovery undoes the action on the monitors it was done to (Rule keeps them), also when the fullscreen window moved to another monitor meanwhile"
  - "A renderer that connects (restart) or a monitor that appears while the rule is on is paused or kept closed, so nothing plays behind a fullscreen app"
  - "Closed slots are started again only after the old renderer's exit event was processed; the respawn is retried on every tick"

patterns-established:
  - "Timers that belong to an optional feature are armed only when the feature is on (FULLSCREEN_TIMER, id 3)"

requirements-completed: [PERF-05]

coverage:
  - id: D1
    description: "Fullscreen predicate: covers a monitor on all four sides, skips desktop, taskbar, own, minimized, invisible and cloaked windows"
    requirement: PERF-05
    verification:
      - kind: unit
        ref: "apps/sucrose/src/fullscreen.rs (cargo test -p sucrose fullscreen)"
        status: pass
    human_judgment: false
  - id: D2
    description: "5 positive samples trigger, 3 negative recover; Pause, Close and Resume on both scopes; Rule undoes what it did"
    requirement: PERF-05
    verification:
      - kind: unit
        ref: "apps/sucrose/src/fullscreen.rs (21 tests) and RuleClosed tests in service.rs"
        status: pass
    human_judgment: false
  - id: D3
    description: "With FullScreenPerformance missing (Resume) no timer exists and the service logs 'fullscreen rule off'"
    requirement: PERF-05
    verification:
      - kind: integration
        ref: "sucrose.exe run with scratch SUCROSE_DATA_DIR on the maintainer's real settings: 'fullscreen rule off (FullScreenPerformance = Resume)'"
        status: pass
    human_judgment: false
  - id: D4
    description: "A real fullscreen app pauses the wallpaper after about 5 s and it plays again after about 3 s; Close stops and restarts the renderer"
    requirement: PERF-05
    verification: []
    human_judgment: true
    rationale: "Needs FullScreenPerformance set to Pause or Close in the maintainer's real Backgroundog.json (read-only for Sucrose, D-08) and a real fullscreen app; the executor did neither"

duration: 5min
completed: 2026-11-22
status: complete
---

# Phase 2 Plan 13: Video wallpaper gets out of the way of fullscreen apps Summary

**Legacy fullscreen rule on Windows: a foreground-window snapshot (no titles) feeds a pure predicate and a 5/3 debounce, and the service pauses, resumes, closes or respawns the renderers over IPC with a 1 s timer that only exists while FullScreenPerformance is Pause or Close**

## Performance

- **Duration:** 5 min
- **Started:** 2026-11-22T16:05:42Z
- **Completed:** 2026-11-22T16:10:52Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- `ForegroundWindow` and `Platform::foreground_window` read rect, class, pid, visible, minimized and DWM-cloaked state; the window title API is never called (T-02-28) and every unsafe block has a SAFETY comment.
- `covered_monitors`, `Debounce` (5 to trigger, 3 to recover), `plan_action`, `ActionScope` and `Rule` are pure and covered by 21 tests plus 3 for the closed-slot bookkeeping.
- The service arms `FULLSCREEN_TIMER` (1000 ms) only for Pause or Close; with Resume it logs "fullscreen rule off" and holds no timer and no state. Heavy pause is logged once as handled like Light.
- Close stops the renderers without counting a crash and starts them again with their last assignment once the fullscreen app is gone.

## Task Commits

1. **Task 1: Fullscreen foreground pauses the wallpaper and leaving it resumes (tracer)** - `74eb18cb` (feat)
2. **Task 2: Close with respawn, both scopes, Heavy as Light, FEATURES notes** - `c8e4c020` (test, RED: 5 tests failing on a stub) then `cae21072` (feat, GREEN)

Tracer gate (end-of-phase mode, automated-only verify): `cargo test -p sucrose fullscreen`, `cargo test -p sucrose-platform` and `cargo build -p sucrose` passed end to end, so expansion continued.

**Plan metadata:** committed with this summary (docs).

## Files Created/Modified

- `crates/sucrose-platform/src/os_windows/foreground.rs` - foreground window snapshot
- `crates/sucrose-platform/src/lib.rs`, `os_windows.rs`, `Cargo.toml` - struct, trait method, capability, Win32_Graphics_Dwm feature
- `apps/sucrose/src/fullscreen.rs` - predicate, debounce, action planning, Rule
- `apps/sucrose/src/service.rs` - timer, tick, Close/Respawn bookkeeping, Hello catch-up
- `docs/FEATURES.md` - fullscreen rule notes and the new action scope row

## Decisions Made

See key-decisions.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Maximized window would count as fullscreen with ScreenType = WorkingArea**
- **Found during:** Task 1
- **Issue:** The fleet's monitor rectangles are the surface areas, which are the work areas when Engine.json ScreenType is WorkingArea; a maximized window covers those.
- **Fix:** The tick compares against `Platform::monitors()` (full rectangles) and uses the fleet's list only for the action targets.
- **Committed in:** 74eb18cb

**2. [Rule 2 - Missing critical] Renderers that connect or monitors that appear while the rule is on**
- **Found during:** Task 2
- **Issue:** A restarted renderer or a hot-plugged monitor would play behind a fullscreen app (or a closed slot would be respawned by a shell restart).
- **Fix:** Hello catch-up sends Pause or Close, a new monitor under Close waits in RuleClosed, and the shell-restart path skips slots closed by the rule.
- **Committed in:** cae21072

---

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical)
**Impact on plan:** Both are correctness requirements of D-04 and D-06; no scope creep.

## Issues Encountered

- A maximized window with an auto-hidden taskbar covers the whole monitor and is therefore treated as fullscreen, as the legacy rule does; not changed.
- The scope switch is unreachable in M1 by design, so `ActionScope::AffectedMonitor` carries an `allow(dead_code)` with the reason.
- The human check (Pause/Close with a real fullscreen app) was not run: it needs the maintainer's own edit of Backgroundog.json (D-08). Requirement PERF-05 is marked complete on the unit and integration evidence above; criterion 3 as worded in ROADMAP still needs that check at end-of-phase verification.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 14 can use `ForegroundWindow`, `Rule::pauses` and the fleet's pause bookkeeping; a resumed monitor re-aligning to the master is plan 14's.
- `cargo xtask ci` is green.

## Self-Check: PASSED
