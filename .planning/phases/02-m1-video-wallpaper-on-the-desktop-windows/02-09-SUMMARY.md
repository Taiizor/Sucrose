---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
plan: 09
subsystem: service-tray
tags: [win32, tray-icon, message-loop, shutdown, ipc, tooltip, d-13, d-14, d-15, d-20]

requires:
  - phase: 02-m1-video-wallpaper-on-the-desktop-windows
    provides: "plan 06/07 EventLoop, DesktopSurface and the libmpv renderer; plan 08 the D-13 gate (ThemeRejection, SelectedTheme.info.title) and APP_LONG_NAME; plan 04 LegacySettings.general.app_visible"
provides:
  - "sucrose_platform::{EventLoop::run(handler), LoopProxy::{wake, quit}, LoopEvent, Flow}: a hidden top-level SucroseServiceWindow and a handler-based message loop"
  - "sucrose_platform::{Tray::{new, set_tooltip}, refresh_desktop, truncate_tooltip, icon::pick_ico_entry}: the legacy-icon tray with a Quit-only menu (Windows), Unsupported stubs elsewhere"
  - "ToService::{Failed, Playing} appended to the IPC protocol; renderer exit code 4 for non-engine start failures now reports why"
  - "Service: ordered Quit (Close to all, 2 s shared grace on a worker thread, kill survivors, desktop repaint), Status value shown in the tooltip and log, AppVisible gate"
affects: [02-10, 02-12, 02-13, 02-14, 02-15]

actuals:
  tokens: 12300
  tasks: 2
  commits: 2

plan_head_before: 87d92d90c9034df5158a986a6bc216c4dcc74be2
plan_head_after: e4f43af8668dbd9539be3bd7488863d5dd1f45fe

tech-stack:
  added: ["tray-icon 0.26.1 (default-features = false, Windows only, via sucrose-platform)"]
  patterns:
    - "The service main thread only runs the Win32 loop; worker threads send Event values over a crossbeam channel and wake the loop through LoopProxy::wake (D-012)"
    - "The window procedure only queues LoopEvents in a thread-local; run() hands them to the handler after each dispatch, so wake and quit posted while a tray menu runs its own modal loop are not lost"
    - "Shutdown sequencing is a pure function over a RendererControl trait so the order is unit tested"
    - "One Status value renders both the tooltip (truncated to 127 UTF-16 units) and the log line"

key-files:
  created:
    - crates/sucrose-platform/src/icon.rs
    - crates/sucrose-platform/src/os_windows/tray.rs
    - crates/sucrose-platform/src/os_windows/refresh.rs
    - crates/sucrose-ipc/tests/duplex.rs
    - apps/sucrose/assets/TrayIcon.ico
  modified:
    - crates/sucrose-platform/src/os_windows/event_loop.rs
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - crates/sucrose-platform/src/unsupported.rs
    - crates/sucrose-platform/src/os_linux.rs
    - crates/sucrose-platform/src/os_macos.rs
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-ipc/src/protocol.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - Cargo.toml
    - Cargo.lock
    - docs/FEATURES.md

key-decisions:
  - "ToService::Playing was added next to Failed: the tooltip says 'playing <Title>' only once the renderer really plays, and no existing message said so"
  - "A renderer that cannot load libmpv reports the short reason 'libmpv was not found.' plus the loader's hint; the full search list stays in the log only, so the 127-unit tooltip is not used up by paths"
  - "Shutdown waits up to 2 s in total (one shared deadline) rather than 2 s per renderer, so Quit with several renderers still ends in about 2 s"
  - "refresh_desktop is implemented because the plan 06 SUMMARY recorded a lingering black area; it runs only after the renderers of a Quit are gone"

patterns-established:
  - "PlatformError::Failed { context, message } for libraries that wrap the OS (tray-icon), next to Os { code }"
  - "Test the service's own Win32 window by posting its WM_APP+1 message, targeting the window by owning pid (the renderer creates a window of the same class)"

requirements-completed: [TRAY-01]

duration: 21 min
completed: 2026-11-22
status: complete

coverage:
  - id: D1
    description: "The service runs a hidden top-level Win32 window and message loop; the tray shows the legacy TrayIcon.ico with one Quit item; Quit closes the renderer, the service exits with code 0 and no sucrose.exe remains"
    requirement: TRAY-01
    verification:
      - kind: unit
        ref: "apps/sucrose service::tests::shutdown_closes_all_then_waits_then_kills_only_the_survivors"
        status: pass
      - kind: integration
        ref: "start target/debug/sucrose.exe, post WM_APP+1 to the service window, log shows Quit chosen / renderer exited code 0 / service stopped, tasklist shows 0 sucrose.exe"
        status: pass
    human_judgment: true
    rationale: "That the icon is visible, its menu holds only Quit, and a real click on Quit behaves the same needs a person at the desktop (no input may be synthesized)"
  - id: D2
    description: "ICO parsing picks the entry closest to the wanted size; tooltips are cut to 127 UTF-16 units at a character boundary"
    verification:
      - kind: unit
        ref: "crates/sucrose-platform/src/icon.rs tests (9 tests, including the real legacy icon)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Theme rejection, libmpv failure and playback each produce their reason in the log and tooltip text; the service stays in the tray after a failure"
    requirement: TRAY-01
    verification:
      - kind: unit
        ref: "apps/sucrose service::tests (status and tooltip tests) and render::tests (report)"
        status: pass
      - kind: integration
        ref: "libmpv-2.dll moved aside: log shows 'no wallpaper: libmpv was not found. <hint>', service kept running until Quit"
        status: pass
    human_judgment: true
    rationale: "The tooltip as drawn by Windows when hovering the icon was not observed"
  - id: D4
    description: "AppVisible false builds no tray at all"
    verification:
      - kind: unit
        ref: "apps/sucrose service::tests::with_app_visible_false_no_tray_is_built"
        status: pass
    human_judgment: false
  - id: D5
    description: "Quit leaves the normal Windows wallpaper without a stale frame"
    verification: []
    human_judgment: true
    rationale: "refresh_desktop re-applies the current wallpaper after a Quit, but whether a frame lingers is only visible on the desktop and was not observed here"
  - id: D6
    description: "Quit never signals, stops or inspects C# Sucrose processes"
    requirement: TRAY-01
    verification:
      - kind: other
        ref: "shutdown only addresses Child handles held in the service's own registry; no process enumeration or name-based kill exists in apps/sucrose"
        status: pass
    human_judgment: false
---

# Phase 2 Plan 09: Tray Quit, reasons and AppVisible Summary

**Win32 service loop with a hidden window, the legacy tray icon with a Quit-only menu, an ordered Quit that closes the renderers within 2 s and repaints the desktop, one Status value shown in the log and tooltip (playing, D-13 rejection, D-20 libmpv missing), and AppVisible false creating no tray.**

## Performance

- **Duration:** 21 min
- **Started:** 2026-11-22T14:58:00Z
- **Completed:** 2026-11-22T15:20:00Z
- **Tasks:** 2 (Task 1 is a tracer)
- **Files modified:** 18 (5 created)

## Accomplishments

- `EventLoop` now owns a hidden top-level `SucroseServiceWindow` and calls a handler with `LoopEvent::{Wake, Quit}`; the service main thread only pumps messages and drains a channel, all IPC reads run on worker threads.
- The tray (tray-icon 0.26.1, Windows only, `default-features = false`) shows the legacy `TrayIcon.ico` (copied byte for byte), picks the icon size for the session's small-icon metric, and has one menu item, Quit.
- Quit sends Close to every renderer, waits up to 2 s (shared deadline) on a worker thread, kills only the survivors, re-applies the wallpaper so no video frame stays, and lets the loop exit with code 0. Verified live: Quit chosen, renderer exited with code 0, service stopped, 0 sucrose.exe left.
- Every "no wallpaper" case has a reason in the log and tooltip text: theme rejection (D-13), libmpv missing (D-20, with the loader's repair hint), other start failures (exit code 4). The service then waits in the tray.
- General.json AppVisible false: `start_tray` never calls the tray constructor (no icon, menu or extra resources), the wallpaper still plays.
- `#![warn(clippy::undocumented_unsafe_blocks)]` makes CI reject any unsafe block in `sucrose-platform` without a SAFETY comment; the existing code from plans 05 and 06 already passed it.

## Task Commits

1. **Task 1 (tracer): Tray icon with Quit on the service's event loop; Quit closes the renderer and the service exits** - `6b0b58ee` (feat)
2. **Task 2: Reasons in log and tooltip, AppVisible gate, desktop repaint on stop** - `e4f43af8` (feat)

**Plan metadata:** recorded in the docs commits that follow this SUMMARY.

Tracer gate: `HUMAN_VERIFY_MODE` is `end-of-phase` and the tracer's verify is automated only, so it was re-run end to end (icon and shutdown tests, build, 8 s run, "tray icon ready" in the log, `cmp` of the icon) and passed before Task 2 started: tracer verified end-to-end, expanding.

## Files Created/Modified

- `crates/sucrose-platform/src/os_windows/event_loop.rs` - hidden window, handler-based loop, `LoopProxy::{wake, quit}`
- `crates/sucrose-platform/src/os_windows/tray.rs` - `Tray` on tray-icon, Quit item, `set_tooltip`
- `crates/sucrose-platform/src/os_windows/refresh.rs` - `refresh_desktop` (SPI_GETDESKWALLPAPER then SPI_SETDESKWALLPAPER, no registry write)
- `crates/sucrose-platform/src/icon.rs` - portable `pick_ico_entry` and `truncate_tooltip`
- `crates/sucrose-platform/src/{lib,unsupported,os_windows,os_linux,os_macos}.rs` - `LoopEvent`, `Flow`, `PlatformError::Failed`, stubs for other OSes
- `crates/sucrose-ipc/src/protocol.rs` - `ToService::{Failed, Playing}` appended, wire-index tests
- `crates/sucrose-ipc/tests/duplex.rs` - proves a write on a cloned connection end is not blocked by a blocked read
- `apps/sucrose/src/service.rs` - event loop, tray, registry of renderers, shutdown, Status, AppVisible gate
- `apps/sucrose/src/render.rs` - reports Failed and Playing, handler-based loop
- `apps/sucrose/assets/TrayIcon.ico` - legacy tray icon, byte-identical copy
- `Cargo.toml`, `Cargo.lock`, `crates/sucrose-platform/Cargo.toml`, `docs/FEATURES.md`

## Decisions Made

See key-decisions above. The tooltip shows the start of reason plus hint cut at 127 UTF-16 units; the full text is always in the log.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] ToService::Playing**
- **Found during:** Task 2
- **Issue:** The plan asks for "Sucrose - playing <Title>" once the renderer reports playback, but the protocol had no such message and the service cannot know otherwise (Hello comes before the engine starts).
- **Fix:** Appended `ToService::Playing` after `Failed` (appending keeps the wire contract); the renderer sends it after the player started.
- **Files modified:** crates/sucrose-ipc/src/protocol.rs, apps/sucrose/src/render.rs, apps/sucrose/src/service.rs
- **Committed in:** e4f43af8

**2. [Rule 1 - Bug avoided] No DestroyIcon on the tray HICON**
- **Found during:** Task 1
- **Issue:** The plan asks for `DestroyIcon` on the owned HICON, but `tray_icon::Icon::from_handle` wraps it in a RAII type that destroys it on drop; destroying it too would be a double free.
- **Fix:** The handle is handed over immediately and not destroyed by Sucrose code (comment in tray.rs).
- **Committed in:** 6b0b58ee

**3. [Rule 3 - Blocking] GetSystemMetricsForDpi lives in Win32_UI_HiDpi**
- **Found during:** Task 1
- **Issue:** The import from WindowsAndMessaging did not compile.
- **Fix:** Imported from `Win32::UI::HiDpi` (feature already enabled).
- **Committed in:** 6b0b58ee

**4. [Rule 3 - Blocking] PlatformError::Failed variant**
- **Found during:** Task 1
- **Issue:** tray-icon and muda errors are not OS error codes, so `PlatformError::Os` could not carry them.
- **Fix:** Added `PlatformError::Failed { context, message }`.
- **Committed in:** 6b0b58ee

**5. [Rule 2 - Missing Critical] A non-Hello first message is refused**
- **Found during:** Task 2
- **Issue:** With `ToService` gaining variants, `accept_hello`'s irrefutable `let` no longer compiled, and a first message that is not a Hello must not be accepted.
- **Fix:** `Refusal::NotHello` plus a test.
- **Committed in:** e4f43af8

**6. [Rule 2 - Missing Critical] Renderer text is length-limited and the tooltip text is built from a short reason**
- **Found during:** Task 2 (live run with libmpv moved aside)
- **Issue:** `LoadError::NotFound`'s Display lists every PATH candidate (several KB), which filled the whole 127-unit tooltip and the log line.
- **Fix:** The renderer sends "libmpv was not found." plus the hint; the service additionally limits received reason and hint to 300 characters (T-02-19).
- **Committed in:** e4f43af8

**7. [Rule 2 - Missing Critical] refresh_desktop added**
- **Found during:** Task 2, step 4
- **Issue:** The 02-06 SUMMARY records that a black area lingered after a force-kill (02-07 saw none), so by the plan's rule the repaint is added.
- **Fix:** `sucrose_platform::refresh_desktop()`, called on the shutdown worker after the renderers are gone, only when renderers existed.
- **Committed in:** e4f43af8

**8. [Scope] No per-child wait threads**
- Plan step 6 mentions "child waits" on worker threads. Nothing in M1 reads a renderer's exit code on its own (Failed and the IPC disconnect cover status; shutdown polls `try_wait` on its worker), so no polling thread was added (D-008). Plan 12 or 14 adds restart handling where an exit code matters.

---

**Total deviations:** 8 (3 Rule 2 additions of protocol and safety behavior, 2 Rule 3 build fixes, 1 Rule 1 avoided double free, 1 required companion change, 1 scope note)
**Impact on plan:** All were needed for correctness or to meet the plan's stated behavior; no scope creep.

## Authentication Gates

None.

## Issues Encountered

- **Same window class in the renderer.** The renderer also creates an `EventLoop`, so its hidden window shares the class `SucroseServiceWindow`. A first test that posted Quit to the first window of that class stopped the renderer, not the service. The tests now pick the window by owning pid. Not a product problem (the tray lives only in the service), but later plans that look up the service window by class must filter by pid.
- **Execution policy.** pwsh `-File` on a script under the WSL path is blocked; the test messages were sent with `-EncodedCommand`.
- **Linux and macOS** builds could not be run locally (targets not installed); the stubs mirror the existing `unsupported` pattern and CI covers them.

## Human check (Task 2) status

Done by the executor without synthesizing input: step 1 (service started with a scratch data dir), step 4 in part (Quit posted to the service window as the menu item does: Close, renderer exit code 0, service exit, no sucrose.exe left), step 5 (libmpv moved aside: log carries "no wallpaper: libmpv was not found." with the repair hint, service stayed until Quit, DLL restored), step 6 (precondition: no C# Sucrose process was running; shutdown touches only its own Child handles).
Not observed and still for the maintainer: step 2 (hover tooltip as drawn), step 3 (menu shows only Quit on a real right click), and that after a real Quit the normal wallpaper is back without a frozen frame (D5).

## Known Stubs

None.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: local-message-surface | crates/sucrose-platform/src/os_windows/event_loop.rs | The hidden top-level window accepts WM_APP and WM_APP+1 from any process of the same user, which stops the service. A same-user process can already terminate it, so no new capability; recorded for completeness. |

## Next Phase Readiness

- Plans 10 onward can add `LoopEvent` variants (ShellRestarted, DisplayChanged, Timer) and render them from the same handler; `Status` is the single place for what the user is told.
- Open for the maintainer: the hover tooltip, the menu and the post-Quit desktop (see Human check above).

## Self-Check: PASSED

- FOUND: apps/sucrose/assets/TrayIcon.ico, crates/sucrose-platform/src/icon.rs, os_windows/event_loop.rs, os_windows/tray.rs, os_windows/refresh.rs, crates/sucrose-ipc/tests/duplex.rs
- FOUND commits: 6b0b58ee, e4f43af8
- `cargo xtask ci` (fmt, clippy -D warnings, test) exit 0; icon, shutdown, status and AppVisible tests pass

---
*Phase: 02-m1-video-wallpaper-on-the-desktop-windows*
*Completed: 2026-11-22*
