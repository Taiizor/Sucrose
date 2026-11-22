---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 12
subsystem: desktop-surface
tags: [windows, multi-monitor, hot-plug, debounce, taskbarcreated, explorer-restart, win32, ipc]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "plan 06 Platform::monitors and DesktopSurface, plan 09 EventLoop and tray, plan 10 Supervisor and RestartBudget, plan 11 Assignment and ScreenArea"
provides:
  - "apps/sucrose topology::{TopologyAction, diff, SlotMap, with_area, primary, Debounce, TimerCommand, TOPOLOGY_DEBOUNCE, TOPOLOGY_SETTLE}"
  - "sucrose_platform::{LoopEvent::{DisplayChanged, ShellRestarted, Timer}, EventLoop::{set_timer, kill_timer, timers}, LoopTimers, DesktopSurface::{set_rect, destroyed_externally}}"
  - "sucrose_ipc::protocol::ToRenderer::SetRect([i32; 4]) (wire index 4)"
  - "supervisor::{ShellRestart, decide_exit, SHELL_RESTART_WINDOW}"
  - "One renderer per connected monitor, monitor changes applied as debounced diffs, event-driven recovery after explorer.exe restarts"
affects: [02-13, 02-14, 02-15]

actuals:
  tokens: 17100
  tasks: 3
  commits: 3

plan_head_before: c36729531e1344611746e57f00e9cee1a42c4049
plan_head_after: a260ae0eb5ea9a953955b36f6ebc348355c2c970

tech-stack:
  added: []
  patterns:
    - "Pure state machines with no timers or OS calls (Debounce, ShellRestart, SlotMap) say what to do; the service performs it, so hot-plug and shell-restart rules are unit-tested with fake events"
    - "Slots are keyed by monitor device name and a removed monitor's slot stays reserved until its renderer has really ended"
    - "A surface destroyed by someone else records itself (thread-local handle set) and posts quit, so the renderer exits with its own code instead of being polled"

key-files:
  created:
    - apps/sucrose/src/topology.rs
  modified:
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/supervisor.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/src/main.rs
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - crates/sucrose-platform/src/os_windows/event_loop.rs
    - crates/sucrose-platform/src/os_windows/surface.rs
    - crates/sucrose-platform/src/unsupported.rs
    - crates/sucrose-platform/src/os_linux.rs
    - crates/sucrose-platform/src/os_macos.rs
    - crates/sucrose-ipc/src/protocol.rs
    - docs/FEATURES.md

key-decisions:
  - "Timers are changed through a LoopTimers handle (EventLoop::timers) because EventLoop::run consumes the loop; EventLoop::set_timer and kill_timer exist as the contract says but cannot be called from inside the handler"
  - "The service ignores an empty monitor snapshot and does not re-arm the debounce for it: the next display event brings the monitors back, so no timer runs while nothing is pending"
  - "Shell restarts are acted on at most 5 times per 60 s (T-02-26), a lost surface inside the 30 s restart window is free, outside it the lost-surface cap of 5 per 60 s applies"
  - "A renderer that is closed for a shell restart is started again whatever its exit code once it ended, exactly once per slot; one that has no connection yet is left alone"

patterns-established:
  - "Respawn after Close runs through the same exit path as a crash (decide_exit) so the budgets and the supervisor stay the single owner of restarts"
  - "Supervisor::spawn refuses a slot that already runs, which makes every late or duplicate restart harmless"

requirements-completed: [DESK-01, DESK-06, DESK-07]

coverage:
  - id: D1
    description: "One renderer per monitor, keyed by device name; enumeration order, adjacency, empty and duplicate snapshots handled"
    requirement: DESK-01
    verification:
      - kind: unit
        ref: "apps/sucrose/src/topology.rs (cargo test -p sucrose topology)"
        status: pass
      - kind: integration
        ref: "sucrose.exe on the one-display maintainer machine: 'renderers started count=1 monitors=[DISPLAY1]'"
        status: pass
    human_judgment: false
  - id: D2
    description: "Monitor, resolution, DPI and work-area changes are debounced 2000 ms + 500 ms and applied as diffs with SetRect, without restarting playback"
    requirement: DESK-06
    verification:
      - kind: unit
        ref: "apps/sucrose/src/topology.rs (cargo test -p sucrose debounce)"
        status: pass
      - kind: integration
        ref: "five WM_DISPLAYCHANGE posts to the service window gave exactly one 'monitors did not change' application about 2.5 s after the last"
        status: pass
    human_judgment: false
  - id: D3
    description: "TaskbarCreated closes the running renderers and starts one per monitor again, lost surfaces are free inside the restart window, no polling"
    requirement: DESK-07
    verification:
      - kind: unit
        ref: "apps/sucrose/src/supervisor.rs (cargo test -p sucrose shell_restart)"
        status: pass
      - kind: integration
        ref: "a TaskbarCreated posted to the Sucrose windows gave 'the shell restarted, replacing the renderers closing=[0]', exit 0, Restart(500ms), a new renderer that attached again"
        status: pass
    human_judgment: false
  - id: D4
    description: "Real explorer.exe restart, display scale change and monitor hot-plug on real hardware"
    requirement: DESK-07
    verification: []
    human_judgment: true
    rationale: "Killing explorer.exe and changing the display scale on the maintainer's live desktop were not done by the executor, and the machine has one display; the surface-destroyed-by-the-shell path (WM_DESTROY of a child of Progman) is only proven by reading the code and the synthetic TaskbarCreated run"

duration: 14min
completed: 2026-11-22
status: complete
---

# Phase 2 Plan 12: Video wallpaper on every monitor Summary

**One libmpv renderer per connected monitor keyed by device name, monitor and DPI changes applied as debounced diffs through SetRect, and explorer.exe restarts recovered by the TaskbarCreated broadcast with lost surfaces exempt from the crash budget**

## Performance

- **Duration:** 14 min
- **Started:** 2026-11-22T15:47:25Z
- **Completed:** 2026-11-22T16:01:43Z
- **Tasks:** 3
- **Files modified:** 14

## Accomplishments

- `topology::diff` turns two monitor snapshots into closes, rect changes and spawns by device name (reversed enumeration, adjacent monitors, empty and duplicate snapshots covered by 13 tests), and `SlotMap` keeps a slot with its monitor and reserves a removed monitor's slot until its renderer ended.
- The service watches `WM_DISPLAYCHANGE`, `WM_DPICHANGED` and work-area `WM_SETTINGCHANGE` on its top-level window, debounces 2000 ms then checks again after 500 ms, applies a diff only when both looks agree, and kills each timer as soon as it did its job. Moved or rescaled monitors get `ToRenderer::SetRect`, the renderer moves its surface on its own thread and mpv follows the size, so playback is not restarted.
- `TaskbarCreated` is registered (and allowed through the UIPI filter); the service closes renderers that still run, starts one per monitor once each has ended, spawns a fresh one for any monitor without a running renderer, and counts a lost surface inside the 30 s window as neither a crash nor a lost-surface strike. A renderer whose surface the shell destroyed records it, posts quit and exits with code 3.

## Task Commits

1. **Task 1: One renderer per monitor from the monitor snapshot, keyed by device name (tracer)** - `494b4c54` (feat)
2. **Task 2: Monitor plug/unplug/move/DPI changes applied as debounced diffs** - `25334efa` (feat)
3. **Task 3: Explorer.exe restart recovered through TaskbarCreated** - `a260ae0e` (feat)

Tracer gate (end-of-phase mode, automated-only verify): the verify command passed end to end (13 topology tests, one renderer for the one connected monitor), so expansion continued.

## Files Created/Modified

- `apps/sucrose/src/topology.rs` - pure diff, slot map, work-area selection, debounce state machine
- `apps/sucrose/src/service.rs` - Fleet (slots, shared assignments, budgets), timer and shell events, Close/SetRect delivery
- `apps/sucrose/src/supervisor.rs` - ShellRestart, decide_exit, one-renderer-per-slot guard
- `apps/sucrose/src/render.rs` - SetRect handling on the surface thread, exit with the surface-lost code
- `crates/sucrose-platform/src/os_windows/event_loop.rs` - new loop events, timers, TaskbarCreated registration
- `crates/sucrose-platform/src/os_windows/surface.rs` - set_rect, externally destroyed detection
- `crates/sucrose-ipc/src/protocol.rs` - `ToRenderer::SetRect` appended
- `docs/FEATURES.md` - surface, per-display, per-monitor DPI and shell restart rows

## Decisions Made

See key-decisions. The timer handle, the empty-snapshot rule and the shell restart rate limit are the ones later plans should know about.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] EventLoop::run consumes the loop, so set_timer/kill_timer cannot be called from the handler**
- **Found during:** Task 2
- **Issue:** The contract gives `EventLoop::set_timer(&self, ...)`, but the service's handler runs inside `run(self, ...)` and has no loop to call.
- **Fix:** Added `EventLoop::timers() -> LoopTimers` (Copy handle with `set`/`kill`); the contract methods remain and delegate to it. Plans 13 and 14 should use `timers()` inside handlers.
- **Files modified:** crates/sucrose-platform/src/os_windows/event_loop.rs, unsupported.rs, lib.rs
- **Committed in:** 25334efa

**2. [Rule 2 - Missing critical] Flood and duplicate protection (T-02-26, T-02-27)**
- **Found during:** Tasks 2 and 3
- **Issue:** TaskbarCreated is a broadcast any process can send, and a late restart could double a monitor's renderer.
- **Fix:** ShellRestart acts on at most 5 restarts per 60 s; `Supervisor::spawn` refuses a running slot and `restart_after` skips one; the Hello path catches up a SetRect (or Close) that raced the registration of the connection.
- **Files modified:** apps/sucrose/src/supervisor.rs, apps/sucrose/src/service.rs
- **Committed in:** 25334efa, a260ae0e

**3. [Rule 3 - Blocking] Edge: removed monitor whose renderer never exits**
- **Found during:** Task 2
- **Issue:** A reserved slot would leak if the renderer had already ended for good (engine missing, budget spent) and no exit event could free it.
- **Fix:** The fleet remembers slots that ended for good and releases them at once when their monitor goes.
- **Committed in:** 25334efa

### Other deviations

- Task file split: `surface.rs` and `render.rs` changes for both the move and the surface-lost exit are in the Task 2 commit, because `set_rect` and the destroyed flag live in the same file and the renderer's loop change is one block.
- The executor did not run the human-check steps that change the maintainer's live desktop (display scale change, `taskkill explorer.exe`) and the machine has one display; they are left for the end-of-phase human verification. Synthetic substitutes that touch only Sucrose's own windows were run instead (see coverage D2, D3).
- `cargo fmt` rewrote `surface.rs` (CRLF in the working tree) to LF once; the CRLF endings were restored.

---

**Total deviations:** 3 auto-fixed (1 blocking contract gap, 1 missing critical, 1 blocking edge), 2 process notes
**Impact on plan:** No scope creep; the extras are protections the threat model asks for.

## Issues Encountered

- A first attempt to post synthetic messages from PowerShell failed because a null string argument became an empty string, so no window matched; fixed with `[NullString]::Value`.
- Verification limits (stated as the plan requires): success criteria 1 and 2 were checked on ONE monitor only. Multi-monitor and hot-plug logic is covered by the unit-test filters `topology` (13 tests), `debounce` (5), `supervisor` (16) and `shell_restart` (8); the real explorer.exe restart, scale change and hot-plug need the human check.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 13 and 14 can use `LoopEvent::Timer`, `EventLoop::timers()` and the fleet's connection map to send pause and volume to every monitor's renderer.
- Human verification still owed: scale change, explorer.exe restart (video and tray icon back), second display plug and unplug.

## Self-Check: PASSED

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-22*
