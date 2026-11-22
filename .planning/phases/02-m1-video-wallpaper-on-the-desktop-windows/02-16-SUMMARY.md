---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 16
subsystem: platform
tags: [win32, winevent, progman, workerw, message-loop, shell-restart, gap-closure]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "EventLoop, DesktopSurface and the service's TaskbarCreated / DisplayChanged handling (plans 02-04 to 02-14)"
provides:
  - "EventLoop hands sent display and shell messages to the handler within milliseconds (WM_DRAIN)"
  - "layout::raised_order and a Progman child order check in verify()"
  - "A raised attach that waits up to 10 s for the shell's wallpaper WorkerW and an event-driven WinEvent order guard"
  - "FEATURES note and render.rs comments that state how shell restart recovery works"
affects: [02-17, 02-18, phase-2-verification, uat-test-2]

actuals:
  tokens: 10600
  tasks: 3
  commits: 5
plan_head_before: 4b768f292f8fc90f5738e5c276e32a5bfff8fa54
plan_head_after: ad602d4fed4fbe2a8ee05146c900f09e7dc35b11

tech-stack:
  added: ["windows feature Win32_UI_Accessibility (no new crate)"]
  patterns:
    - "A window procedure that queues an event from a sent message posts one private drain message so GetMessageW returns"
    - "The Progman child order is plain data (SiblingKind) checked by a pure function and kept by a WinEvent hook limited to the shell process"

key-files:
  created: []
  modified:
    - crates/sucrose-platform/src/os_windows/event_loop.rs
    - crates/sucrose-platform/src/layout.rs
    - crates/sucrose-platform/tests/layout.rs
    - crates/sucrose-platform/src/os_windows/surface.rs
    - crates/sucrose-platform/Cargo.toml
    - apps/sucrose/src/render.rs
    - docs/FEATURES.md

key-decisions:
  - "The 500 ms restart delay after TaskbarCreated (FIRST_BACKOFF) stays unchanged; the attach now waits for the WorkerW itself instead of depending on the start time"
  - "The order guard lives for the surface's lifetime, not only after a restart, because explorer can recreate its WorkerW at any time; capped at 10 restores per 60 s"
  - "A 0x052C timeout always fails the attempt, including the last-resort one (a hung Progman is not attached to)"

requirements-completed: [DESK-07, DESK-01]

coverage:
  - id: D1
    description: "A sent WM_DISPLAYCHANGE, WM_SETTINGCHANGE (SPI_SETWORKAREA) or TaskbarCreated reaches the EventLoop handler within 500 ms without any other message (G-02-2d)"
    requirement: DESK-07
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-platform sent_ (4 tests)"
        status: pass
    human_judgment: true
    rationale: "The non-primary monitor unplug and re-plug within about 3 s needs the maintainer's second display; harvested at end of phase"
  - id: D2
    description: "layout::raised_order and verify() reject an attach whose Progman child order is not SHELLDLL_DefView, the surface, then every WorkerW"
    requirement: DESK-01
    verification:
      - kind: unit
        ref: "cargo test -p sucrose-platform --test layout raised_order (6 tests)"
        status: pass
    human_judgment: false
  - id: D3
    description: "The raised attach waits for the wallpaper WorkerW (resending 0x052C each attempt), and the WinEvent order guard keeps the surface above a WorkerW created later"
    requirement: DESK-01
    verification:
      - kind: other
        ref: "release run: Renderer log 'surface attached ... order=\"SHELLDLL_DefView > SucroseDesktopSurface > WorkerW\"'"
        status: pass
    human_judgment: true
    rationale: "Two explorer.exe restarts in a row on the maintainer's machine must show the video back on every monitor; harvested at end of phase"
  - id: D4
    description: "FEATURES.md and render.rs comments record that a killed explorer.exe sends no WM_DESTROY and recovery relies on TaskbarCreated"
    requirement: DESK-01
    verification:
      - kind: other
        ref: "grep -c 'does not send WM_DESTROY' docs/FEATURES.md prints 1; render.rs diff is comments only"
        status: pass
    human_judgment: false

duration: 15min
completed: 2026-11-23
status: complete
---

# Phase 02 Plan 16: Video wallpaper on the desktop (Windows), gap closure G-02-2d and G-02-2a Summary

**Sent display and shell messages reach the service at once through a WM_DRAIN post, and the raised attach waits for the shell's wallpaper WorkerW, checks the Progman child order and keeps it with a WinEvent guard.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-11-22T22:09:52Z
- **Completed:** 2026-11-22T22:25:00Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- G-02-2d: `loop_proc` posts a private `WM_DRAIN` (once per burst) after queuing an event from a sent message, so a non-primary monitor unplug and TaskbarCreated no longer wait for an unrelated posted message. Four `sent_` tests fail on the old code (about 2.9 s, the watchdog quit) and pass in about 0.1 s.
- G-02-2a: `layout::raised_order` (pure, six tests) is used by `verify()` for the LayeredChild strategy; every attempt without a WorkerW resends 0x052C and fails with "Progman has not created its wallpaper WorkerW yet" until the 10 s deadline, then one last-resort attach logs a warning.
- A WinEvent hook (CREATE..REORDER, limited to Progman's process, 10 restores per 60 s) puts the surface back under the icons when the shell creates, shows or reorders its WorkerW later; the "surface attached" line now shows the order.
- A release run on the maintainer's machine (scratch data dir) attached both monitors with `SHELLDLL_DefView > SucroseDesktopSurface > WorkerW` and playback started.
- FEATURES row and render.rs comments state the observed fact (no WM_DESTROY on a killed explorer.exe, no cause claimed) and that recovery relies on TaskbarCreated.

## Task Commits

1. **Task 1: sent messages reach the handler (G-02-2d)** - `6c145a74` (test, RED), `39dc8666` (fix)
2. **Task 2: WorkerW wait and Progman order check (G-02-2a)** - `33fe5c46` (test, RED), `714a7120` (feat)
3. **Task 3: order guard, FEATURES and render.rs comments** - `ad602d4f` (feat)

**Plan metadata:** the docs commit that adds this summary.

## Files Created/Modified

- `crates/sucrose-platform/src/os_windows/event_loop.rs` - WM_DRAIN, DRAIN_POSTED, sent_ tests
- `crates/sucrose-platform/src/layout.rs` - SiblingKind, RaisedOrder, raised_order, WORKERW_CLASS
- `crates/sucrose-platform/tests/layout.rs` - six raised_order tests
- `crates/sucrose-platform/src/os_windows/surface.rs` - WorkerW wait, order check in verify(), order guard
- `crates/sucrose-platform/Cargo.toml` - windows feature Win32_UI_Accessibility
- `apps/sucrose/src/render.rs` - comments only
- `docs/FEATURES.md` - shell restart row notes

## Decisions Made

See key-decisions. FIRST_BACKOFF and service.rs were deliberately left unchanged, as the plan states.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] The guard also accepts OBJID_CLIENT for a REORDER on Progman**
- **Found during:** Task 3
- **Issue:** The plan filters on OBJID_WINDOW only; for EVENT_OBJECT_REORDER the object id may be the client area, which would make the guard miss a reorder.
- **Fix:** REORDER on Progman is accepted with OBJID_WINDOW or OBJID_CLIENT; CREATE and SHOW still need OBJID_WINDOW.
- **Files modified:** crates/sucrose-platform/src/os_windows/surface.rs
- **Committed in:** ad602d4f

**2. [Rule 3 - Blocking] Test-only drain counter and a single-thread burst test**
- **Found during:** Task 1
- **Issue:** "At most a few drain posts" is not deterministic with a cross-thread sender (a fast loop can drain between messages).
- **Fix:** A `#[cfg(test)]` thread-local `DRAIN_POSTS` counts posts, and the burst test sends 20 messages with SendMessageW on the loop thread and asserts exactly one post and 20 events.
- **Files modified:** crates/sucrose-platform/src/os_windows/event_loop.rs
- **Committed in:** 6c145a74, 39dc8666

**3. [Rule 1 - Bug] The surface.rs doc comment of destroyed_externally claimed an explorer restart destroys the surface**
- **Found during:** Task 3
- **Issue:** It contradicts the observed fact recorded in FEATURES and render.rs.
- **Fix:** Reworded to the observation only.
- **Files modified:** crates/sucrose-platform/src/os_windows/surface.rs
- **Committed in:** ad602d4f

---

**Total deviations:** 3 auto-fixed (1 Rule 1, 1 Rule 2, 1 Rule 3)
**Impact on plan:** No scope creep; all keep the plan's intent.

## Issues Encountered

- On this WSL setup `cargo fmt` rewrites a changed file with LF endings; the edited files were converted back to CRLF in the working tree after each format run (git stores LF, `core.autocrlf` is on).
- The first `taskkill` after the 14 s release run reported one process already gone; no sucrose.exe was left running.

## User Setup Required

None - no external service configuration required.

## Human Checks Harvested for End of Phase

- Non-primary monitor unplug and re-plug applied within about 3 s with FullScreenPerformance at Resume (Task 1).
- Two explorer.exe restarts in a row bring the video back on every monitor, with the Service log showing the restart within a few seconds and the DefView > surface > WorkerW order or an "order restored" line (Task 3).

## Next Phase Readiness

- Plans 02-17 and later can build on a loop that delivers display and shell events at once.
- The behavior on a real explorer restart is not yet confirmed on the machine; it is in the end-of-phase UAT.

## Self-Check: PASSED

FOUND:6c145a74
FOUND:39dc8666
FOUND:33fe5c46
FOUND:714a7120
FOUND:ad602d4f
FOUND:apps/sucrose/src/render.rs
FOUND:crates/sucrose-platform/Cargo.toml
FOUND:crates/sucrose-platform/src/layout.rs
FOUND:crates/sucrose-platform/src/os_windows/event_loop.rs
FOUND:crates/sucrose-platform/src/os_windows/surface.rs
FOUND:crates/sucrose-platform/tests/layout.rs
FOUND:docs/FEATURES.md

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-23*
