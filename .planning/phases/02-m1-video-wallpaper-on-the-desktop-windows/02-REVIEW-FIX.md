---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
fixed_at: 2026-11-23T04:14:34.949Z
review_path: .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-REVIEW.md
iteration: 1
findings_in_scope: 26
fixed: 25
skipped: 1
status: partial
---

# Phase 2: Code Review Fix Report

**Fixed at:** 2026-11-23T04:14:34.949Z
**Source review:** .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 26
- Fixed: 25
- Skipped: 1

**Verification environment:** the fixer ran in an isolated git worktree (`.claude/worktrees/rf-02-4242-...`) with its **own** `target/` directory (not shared with the main checkout), so the stale-test-binary problem of the earlier run does not apply. In the worktree: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` passed after the last commit, and the ignored real-libmpv tests (`cargo test -p sucrose-engine-mpv --test options -- --ignored`, 10 tests, using a copy of `target/engines/x86_64/libmpv-2.dll`) passed. No GUI or app was run. After the fast-forward the worktree, its temp branch and the recovery sentinel were removed. The orchestrator should still re-run `cargo test --workspace` in the main checkout (the worktree numbers are not reproducible from the main checkout after teardown). No fix commit touched `src/` (legacy C#).

Fixes marked "requires human verification" cannot be proven without a GUI run or a real shell/pipe failure (WR-08, WR-09, WR-10, WR-11, WR-13, WR-14, IN-11); their logic and plumbing are unit-tested where Windows allows it.

## Fixed Issues

### WR-05: Hello trusts the peer's claimed slot, and a second Hello replaces the registered writer

**Files modified:** `apps/sucrose/src/service.rs`, `apps/sucrose/src/supervisor.rs`
**Commit:** c6095c63
**Status:** fixed
**Applied fix:** The supervisor keeps pid -> slot; `accept_hello` requires the claimed slot to be the one the pid was started for and refuses a slot that already has a connection; `register_writer` never replaces a registered connection (atomic `Entry::Vacant`). The optional `GetNamedPipeClientProcessId` check and a first-read deadline were not added (no deadline support in the blocking IPC crate).

### WR-06: mpv hardening does not neutralise scripts, IPC server, log file or includes when a user config is loaded

**Files modified:** `crates/sucrose-engine-mpv/src/config.rs`, `crates/sucrose-engine-mpv/tests/options.rs`, `docs/FEATURES.md`
**Commit:** 109cc849
**Status:** fixed
**Applied fix:** Kept loading the user config (D-10, maintainer decision untouched). `embedding_options` now sets `scripts`, `input-ipc-server`, `input-conf`, `log-file` to empty and `use-filedir-conf` / `save-position-on-quit` to no after the user config; the names are on `ALLOWED_OPTIONS`. `include` needs no neutral value (it only pulls in more options, and the neutral values come after it); the doc says so. A new ignored real-libmpv test reads the effective values back after `mpv_initialize` and was run against the fetched v0.42 libmpv (it fails when `scripts` is not neutralised, checked by mutation).

### WR-07: libmpv is also loaded from user-writable `PATH` folders

**Files modified:** `crates/sucrose-engine-mpv/src/loader.rs`, `docs/FEATURES.md`
**Commit:** 449c19c6
**Status:** fixed
**Applied fix:** Stayed inside D-028: the search order is unchanged. A library found outside the exe folder is logged at warn level with its full path; `PATH` entries that equal the current folder or are UNC/network folders are skipped (the exe folder itself never is). Dropping the PATH step in release builds was NOT done: it would supersede D-028 and is a maintainer decision.

### WR-08: `DesktopSurface::attach` sleeps up to 10 s on a thread that owns two windows and pumps nothing

**Files modified:** `crates/sucrose-platform/src/os_windows/surface.rs`, `crates/sucrose-platform/src/os_windows/event_loop.rs`
**Commit:** d86e2e02
**Status:** fixed: requires human verification
**Applied fix:** The retry wait is now `wait_pumping` (`MsgWaitForMultipleObjects` plus a `PeekMessageW` drain; a `WM_QUIT` is re-posted and ends attach with an error). `EventLoop::run` also drains queued events before it first blocks. Unit tests cover dispatching and the quit path. The explorer-kill "not responding" check needs a GUI run.

### WR-09: A failing `GetMessageW` ends the loop like a normal quit and the renderer exits with code 0

**Files modified:** `apps/sucrose/src/service.rs`, `apps/sucrose/src/supervisor.rs`, `apps/sucrose/src/render.rs`, `crates/sucrose-platform/src/lib.rs`, `crates/sucrose-platform/src/os_windows/event_loop.rs`, `crates/sucrose-platform/src/unsupported.rs`
**Commit:** e6a11b51
**Status:** fixed: requires human verification
**Applied fix:** `EventLoop::run` returns `LoopEnd::{Quit, Failed}` (the stub returns `Quit`). The renderer maps `Failed` to the new crash-class exit code `LOOP_FAILED` (5, restarted within the budget); the service returns an error. A failing `GetMessageW` cannot be provoked in a test, so only the exit-code classification is unit-tested.

### WR-10: Loop wrap is never detected, `loop_gen` stays 0 for an ordinary loop

**Files modified:** `crates/sucrose-engine-mpv/src/player.rs`, `crates/sucrose-engine-mpv/tests/options.rs`
**Commit:** 56abd80e
**Status:** fixed: requires human verification
**Applied fix:** The event thread keeps an anchor (time, position, rate: speed, 0 while paused; re-anchored on restarts and pause changes). `classify_restart` calls it a wrap when the position is below 1 s and the expected position reached the end (margin 1 s + 10 % of the duration); without a duration it falls back to a jump back to the start after progress. Eight unit tests, and the real-libmpv loop test now asserts `loop_gen >= 1` (run and passing).

### WR-11: Tray/log status stays "no wallpaper" after the renderer recovers

**Files modified:** `apps/sucrose/src/service.rs`
**Commit:** b284811e
**Status:** fixed: requires human verification
**Applied fix:** `Playing` now replaces `Starting`, `Failed` and `RendererStopped` (not `NoWallpaper`), and `restarting()` maps `Failed` to `Starting`; both take the selected theme title. This is the single-status variant the review offers, not a per-slot status (a stopped monitor's notice is replaced when another renderer plays). New tests cover Failed -> restarting -> Playing.

### WR-12: Unsynchronised writes on the reader's pipe handle can interleave frames

**Files modified:** `apps/sucrose/src/service.rs`, `crates/sucrose-ipc/src/frame.rs`
**Commit:** 80b7b97b
**Status:** fixed
**Applied fix:** `write_frame` builds one buffer and issues one `write_all`; the catch-up `SetRect`/`Close` goes through `send_to` (the registered writer) with the assignments lock released first. A counting-writer test proves one write per frame.

### WR-13: Blocking pipe writes while holding the global `Renderers` mutex can freeze the service loop

**Files modified:** `apps/sucrose/src/service.rs`, `docs/ARCHITECTURE.md`
**Commit:** 88b2c050
**Status:** fixed: requires human verification
**Applied fix:** Each connection has a `Link`: a bounded (64) queue drained by its own writer thread; sends use `try_send` (a clock reading is dropped when full, any other message is a failure) so nothing blocks under the lock. Shutdown, `send_to`, `send_role`, `elect` and `relay_clock` use it. Unit tests with a blocked writer prove the sender never waits, drops only clock readings, keeps order, and fails after a broken connection. Hanging a real follower was not tried.

### WR-14: The order guard that holds back never looks again, so a surface left under the wallpaper stays hidden

**Files modified:** `crates/sucrose-platform/src/os_windows/surface.rs`, `docs/FEATURES.md`
**Commit:** e1fb1870
**Status:** fixed: requires human verification
**Applied fix:** The budget is consulted without counting; a restore is counted only after `SetWindowPos` succeeded. When the budget is spent the guard arms one thread timer (`SetTimer` with a callback) for when the oldest restore leaves the window (+100 ms); the callback kills itself and runs `restore_order` again; `remove_guard` kills a pending timer. Unit tests cover the budget and a real timer firing through `wait_pumping`. The explorer-restart plus wallpaper-slideshow burst needs a GUI run.

### IN-01: Playing is only reported after the first frame, with no path for a file that never starts

**Files modified:** `apps/sucrose/src/render.rs`, `crates/sucrose-engine-mpv/src/player.rs`, `crates/sucrose-engine-mpv/tests/options.rs`, `docs/FEATURES.md`
**Commit:** adc43e67
**Status:** fixed
**Applied fix:** An `END_FILE` with an error before the first frame becomes `PlayerEvent::StartFailed` and `Player::start_failure()`; the clock thread reports `Failed` to the service and stops the loop, and the renderer exits with the start-failure code 4. A real-libmpv test with a missing file passes.

### IN-02: Stale comments after the 02-19/02-20/UAT changes and after the WR-03 fix

**Files modified:** `crates/sucrose-engine-mpv/src/player.rs`, `apps/sucrose/src/render.rs`, `apps/sucrose/src/supervisor.rs`, `apps/sucrose/src/service.rs`
**Commit:** 40ba2e05
**Status:** fixed
**Applied fix:** The three comment groups were corrected as suggested.

### IN-03: The GPU preference overwrites a choice made in Windows Settings, on every start

**Files modified:** `docs/FEATURES.md`
**Commit:** 1b839cc5
**Status:** fixed
**Applied fix:** Only the documentation option was applied: the FEATURES row now states the override as parity behavior and that honoring the user's Settings choice is a maintainer decision. Behavior is unchanged.

### IN-04: The renderer creates a Direct3D device just to log the adapter

**Files modified:** `apps/sucrose/src/render.rs`, `docs/FEATURES.md`, `docs/MEASUREMENTS.md`
**Commit:** 102d96f9
**Status:** fixed
**Applied fix:** `log_video_adapter` runs (and logs at debug) only when debug logging is enabled; FEATURES and MEASUREMENTS say the line needs `SUCROSE_LOG=debug`.

### IN-05: Smaller inconsistencies in error codes and measurement notes

**Files modified:** `xtask/src/measure.rs`, `crates/sucrose-platform/src/os_windows/event_loop.rs`, `crates/sucrose-platform/src/os_windows/surface.rs`
**Commit:** abb66254
**Status:** fixed
**Applied fix:** (a) `checked_row` trims trailing punctuation and joins with ". " (tested for a note with, without and empty). (b) The two `create_window` errors use `HRESULT::from_win32`; `attach` now returns `PlatformError::Failed` with the real reason instead of `Os { code: 0 }`.

### IN-06: Unused public timer methods on `EventLoop`

**Files modified:** `crates/sucrose-platform/src/os_windows/event_loop.rs`, `crates/sucrose-platform/src/unsupported.rs`, `crates/sucrose-platform/src/lib.rs`
**Commit:** 507caf33
**Status:** fixed
**Applied fix:** `set_timer`/`kill_timer` and the stubs were removed; the docs point at `EventLoop::timers`.

### IN-08: Untrusted theme text reaches the log and the tooltip unsanitised

**Files modified:** `apps/sucrose/src/service.rs`
**Commit:** 612ef5ce
**Status:** fixed
**Applied fix:** `printable` replaces control characters with U+FFFD in the job title, `limit()` and the gate rejection text (which carries the folder name); the selected theme and its source are logged with `?`. Tested.

### IN-09: Backslash-escaped backticks in a crate doc comment

**Files modified:** `crates/sucrose-core/src/theme.rs`
**Commit:** f8137dd5
**Status:** fixed
**Applied fix:** Plain backticks.

### IN-11: `reparent` sets the child style after `SetParent`

**Files modified:** `crates/sucrose-platform/src/os_windows/surface.rs`
**Commit:** b7753020
**Status:** fixed: requires human verification
**Applied fix:** The style bits are set before `SetParent` and restored if it fails; `verify` stays as the safety net. Needs a GUI run on the raised and classic layouts.

### IN-12: Log message and behaviour disagree for a control message before the assignment

**Files modified:** `apps/sucrose/src/render.rs`
**Commit:** cfa8f17b
**Status:** fixed
**Applied fix:** The renderer now keeps waiting for the `Assign` after a stray control message, as the log says.

### IN-13: The shader-cache comments claim a separation that the default configuration does not have

**Files modified:** `crates/sucrose-engine-mpv/src/config.rs`, `apps/sucrose/src/render.rs`
**Commit:** 94e65c47
**Status:** fixed
**Applied fix:** The comments say the cache stays out of the C# `MpvPlayer` folder and that the data folder itself is shared without `SUCROSE_DATA_DIR`.

### IN-14: `Player::open_source` is public and takes any mpv source string

**Files modified:** `crates/sucrose-engine-mpv/Cargo.toml`, `crates/sucrose-engine-mpv/src/player.rs`, `crates/sucrose-engine-mpv/tests/options.rs`, `Cargo.lock`
**Commit:** ae2bff30
**Status:** fixed
**Applied fix:** `open_source` is private; the integration tests call `open_source_for_tests`, which exists only with the `test-support` feature, enabled by a self dev-dependency (Cargo.lock gains that edge).

### IN-15: Any unrecognised command line starts the service

**Files modified:** `apps/sucrose/src/main.rs`
**Commit:** 1aa6a25b
**Status:** fixed
**Applied fix:** `parse_mode` accepts no argument or `--render <slot>` only; anything else is logged and exits with code 64. Tested.

### IN-16: A late `Playing` is only dropped by connection presence, because reports carry no renderer identity

**Files modified:** `apps/sucrose/src/service.rs`, `apps/sucrose/src/supervisor.rs`
**Commit:** 3cad7ded
**Status:** fixed
**Applied fix:** Reports carry the Hello-verified pid, `Link` records it, and `Exited` carries the exiting pid. A report from a replaced renderer is ignored, `Playing` counts only for the current link, an old connection is removed only by its own pid, and an exit while a different process runs for the slot is ignored. Tested through two pure helpers.

### IN-17: The "all monitors" measurement row repeats the one-monitor numbers

**Files modified:** `docs/MEASUREMENTS.md`
**Commit:** 215f6b91
**Status:** fixed
**Applied fix:** The row is renamed "1 active display (repeat, Rust)" and the note says it is not a multi-monitor measurement. A real two-monitor row still needs the maintainer's second display.

## Skipped Issues

### IN-07: Generality without instantiation: affected-monitor scope and `PauseKind` plumbing

**File:** `apps/sucrose/src/fullscreen.rs:94-105`
**Reason:** Not applied. The finding says to keep the code if a later phase consumes it, and Accepted phase decision D-05 requires M1 to implement both action scopes with legacy "all monitors" as the fixed default; `.planning/ROADMAP.md`/`REQUIREMENTS.md` (PERF-01) and 02-CONTEXT.md schedule the setting and its UI for Phase 3/4. Removing it would contradict D-05 and is a scope decision for the maintainer.
**Original issue:** `ActionScope::AffectedMonitor` (with `Rule::acted`, `acts_on`'s second arm and an `#[allow(dead_code)]`) is reachable only through the constant `ACTION_SCOPE = AllMonitors`; `PauseKind` is read from settings and only logged. Necessity finding: runs and is tested but changes no behavior in M1.

---

_Fixed: 2026-11-23T04:14:34.949Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
