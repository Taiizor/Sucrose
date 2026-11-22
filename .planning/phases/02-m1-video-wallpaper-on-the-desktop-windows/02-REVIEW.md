---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
reviewed: 2026-11-22T17:30:00Z
depth: standard
files_reviewed: 81
files_reviewed_list:
  - .claude/rules/cargo.md
  - .claude/rules/platform-ffi.md
  - .gitignore
  - CLAUDE.md
  - Cargo.lock
  - Cargo.toml
  - apps/sucrose/Cargo.toml
  - apps/sucrose/assets/TrayIcon.ico
  - apps/sucrose/src/fullscreen.rs
  - apps/sucrose/src/main.rs
  - apps/sucrose/src/render.rs
  - apps/sucrose/src/service.rs
  - apps/sucrose/src/supervisor.rs
  - apps/sucrose/src/topology.rs
  - crates/sucrose-config/Cargo.toml
  - crates/sucrose-config/src/legacy.rs
  - crates/sucrose-config/src/lib.rs
  - crates/sucrose-config/tests/fixtures/Backgroundog.json
  - crates/sucrose-config/tests/fixtures/Backgroundog.pause.json
  - crates/sucrose-config/tests/fixtures/Engine.invalid.json
  - crates/sucrose-config/tests/fixtures/Engine.json
  - crates/sucrose-config/tests/fixtures/General.json
  - crates/sucrose-config/tests/fixtures/Library.bom.json
  - crates/sucrose-config/tests/fixtures/Library.json
  - crates/sucrose-config/tests/legacy.rs
  - crates/sucrose-core/Cargo.toml
  - crates/sucrose-core/src/lib.rs
  - crates/sucrose-core/src/paths.rs
  - crates/sucrose-core/src/settings.rs
  - crates/sucrose-core/src/theme.rs
  - crates/sucrose-core/src/version.rs
  - crates/sucrose-core/tests/fixtures/SucroseInfo.full.json
  - crates/sucrose-core/tests/fixtures/SucroseInfo.object-version.json
  - crates/sucrose-core/tests/fixtures/SucroseInfo.video.json
  - crates/sucrose-core/tests/fixtures/broken_theme/SucroseInfo.json
  - crates/sucrose-core/tests/fixtures/video_theme/SucroseInfo.json
  - crates/sucrose-core/tests/theme.rs
  - crates/sucrose-engine-mpv/Cargo.toml
  - crates/sucrose-engine-mpv/src/config.rs
  - crates/sucrose-engine-mpv/src/lib.rs
  - crates/sucrose-engine-mpv/src/loader.rs
  - crates/sucrose-engine-mpv/src/player.rs
  - crates/sucrose-engine-mpv/src/sync.rs
  - crates/sucrose-engine-mpv/src/sys.rs
  - crates/sucrose-engine-mpv/tests/load.rs
  - crates/sucrose-engine-mpv/tests/options.rs
  - crates/sucrose-ipc/Cargo.toml
  - crates/sucrose-ipc/src/frame.rs
  - crates/sucrose-ipc/src/lib.rs
  - crates/sucrose-ipc/src/protocol.rs
  - crates/sucrose-ipc/tests/duplex.rs
  - crates/sucrose-ipc/tests/socket.rs
  - crates/sucrose-platform/Cargo.toml
  - crates/sucrose-platform/src/geometry.rs
  - crates/sucrose-platform/src/icon.rs
  - crates/sucrose-platform/src/layout.rs
  - crates/sucrose-platform/src/lib.rs
  - crates/sucrose-platform/src/os_linux.rs
  - crates/sucrose-platform/src/os_macos.rs
  - crates/sucrose-platform/src/os_windows.rs
  - crates/sucrose-platform/src/os_windows/display.rs
  - crates/sucrose-platform/src/os_windows/event_loop.rs
  - crates/sucrose-platform/src/os_windows/foreground.rs
  - crates/sucrose-platform/src/os_windows/job.rs
  - crates/sucrose-platform/src/os_windows/refresh.rs
  - crates/sucrose-platform/src/os_windows/surface.rs
  - crates/sucrose-platform/src/os_windows/tray.rs
  - crates/sucrose-platform/src/unsupported.rs
  - crates/sucrose-platform/tests/fixtures/desktop-26300.json
  - crates/sucrose-platform/tests/fixtures/desktop-classic.json
  - crates/sucrose-platform/tests/layout.rs
  - docs/ARCHITECTURE.md
  - docs/DECISIONS.md
  - docs/FEATURES.md
  - docs/MEASUREMENTS.md
  - xtask/Cargo.toml
  - xtask/engines.toml
  - xtask/src/bindgen_mpv.rs
  - xtask/src/fetch_engines.rs
  - xtask/src/main.rs
  - xtask/src/measure.rs
findings:
  critical: 0
  warning: 9
  info: 10
  total: 19
  necessity: 1
status: issues_found
---

# Phase 2: Code Review Report

**Reviewed:** 2026-11-22T17:30:00Z
**Depth:** standard
**Files Reviewed:** 81
**Status:** issues_found

## Summary

All Rust sources of the phase were read in full (platform, engine-mpv, ipc, core, config, the `sucrose` service/renderer, xtask). Fixtures, lockfile, icon and documentation were checked only for consistency with the code; the integration tests under `crates/*/tests` were skimmed for reliability, not reviewed line by line. No structural (fallow) findings were provided. The necessity rubric was read and applied (one Info finding).

The code is generally careful: every `unsafe` block has a SAFETY comment, the theme gate resolves links and rejects traversal, the libmpv loader refuses relative paths, the pipe is restricted by a protected DACL, frames are size-checked before allocation, and fetch-engines hashes the archive before opening it. No exploitable vulnerability or data-loss bug was proven, so there are no Critical findings.

The defects are in the cross-process coordination: a clock/loop-wrap detector that cannot fire, a tray status that gets stuck, unsynchronised and potentially blocking writes on the service side of the pipe, and a Hello handshake that trusts the peer's own claims. Reading the process-identity and hardening code also turned up a gap between what the docs claim for mpv hardening (T-02-14) and what the option list actually enforces.

## Warnings

### WR-01: Loop wrap is never detected, `loop_gen` stays 0 for an ordinary loop

**File:** `crates/sucrose-engine-mpv/src/player.rs:546-559` (state kept at `:523-533`)
**Issue:** `note_restart` compares the new `time-pos` with `shared.last_position`, but that value is only written at the previous *restart* (line 532 and the `swap` on line 552), never as playback advances. For a looping file the previous restart was at about 0.0 s (the first start) and the wrap restart is also at about 0.0 s, so `delta` is tiny and `classify_jump` returns `Jump::None`; `loop_gen` never increments. The unit tests only feed `classify_jump` a hand-made "29.8 -> 0.05" pair that the event loop cannot produce. The doc of `ClockTick::loop_gen` ("how many times the master looped") and the comment "playback advances by well under a second between two readings" are therefore wrong. Nothing consumes `loop_gen` yet, so the visible effect today is limited to a misleading field, but a later plan that relies on it will silently get 0.
**Fix:** Track where playback should be, not where the last restart was. For example remember `(Instant, position, speed)` at each restart and on the next restart compute `expected = position + elapsed * speed`; call it a wrap when the new position is < 1.0 s and `expected` is within about a second of (or beyond) `duration`. Add a test that drives the real event path (or at least the extrapolation function) with a looping scenario.

### WR-02: Tray/log status stays "no wallpaper" after the renderer recovers

**File:** `apps/sucrose/src/service.rs:179-200`, `:342-399`
**Issue:** `Status::after(Playing)` only moves `Starting -> Playing`, and `Status::restarting()` only maps `Playing -> Starting`. A renderer that reports `Failed` (for example the surface could not attach while explorer was restarting, exit code 4 = `START_FAILED`) is restarted by `decide_exit` (`Decision::Restart`), but the status is `Failed`, so `restarting()` returns `None`; when the new renderer then sends `Playing`, `after` sees `Failed` and returns `None`. The tooltip keeps saying "Sucrose - no wallpaper: ..." for the whole session although the wallpaper plays. The same applies to `RendererStopped` of one monitor while the others play, and a `Failed` of one monitor hides the `Playing` of the rest.
**Fix:** Let `Playing` replace `Failed` (and `Starting`) unless the status is `NoWallpaper`, and map `Failed` to `Starting` in `restarting()` when a restart is scheduled. Better, keep one status per slot and derive the tooltip from the set (any failure shown, otherwise playing).

### WR-03: Unsynchronised writes on the reader's pipe handle can interleave frames

**File:** `apps/sucrose/src/service.rs:678-700` (versus `:727-761`, `:775-792`, `:909-920`, `:1145-1162`)
**Issue:** After `register_writer` has stored a clone of the connection under the `Renderers` lock, `read_connection` keeps sending through its own handle without that lock (the catch-up `SetRect` at line 689 and the `Close` at line 695). Every other writer (main thread `send_to`/`elect`, the master's `relay_clock`) writes through the registered clone under the lock. `write_frame` issues two `write_all` calls (length header, then payload), so a concurrent frame from the main thread (for example a `SetRect` from `move_to` or a `Pause`) can land between the header and the payload of the other, and the renderer decodes garbage or drops the connection with `FrameTooLarge`/`Decode`. The window is small but it sits exactly where hot-plug and fullscreen changes race a starting renderer.
**Fix:** Perform the catch-up through the shared writer (`send_to(renderers, slot, ...)`) or build the whole frame in one buffer and write it with a single `write_all` in `frame::write_frame` (still serialise writers per connection, for example one `Mutex<Connection>` per renderer).

### WR-04: Blocking pipe writes while holding the global `Renderers` mutex can freeze the service loop

**File:** `apps/sucrose/src/service.rs:775-792` (`relay_clock`), `:909-920` (`send_to`), `:1145-1162` (`elect`); lock taken by the main thread at `:343`, `:1246`
**Issue:** Every send to a renderer is a blocking synchronous named-pipe write executed with the single `Mutex<Renderers>` held. The master's reader thread relays a tick to every follower once per second under that lock. If one follower stops reading (suspended, stuck in a GPU driver call, being debugged) its pipe buffer eventually fills, the write blocks, and the lock is never released. The main thread, which pumps the message loop and the tray, then blocks on `lock(&renderers)` (Exited handling, `send_to` for Pause/SetRect, `elect`), so Quit from the tray, hot-plug handling and the fullscreen rule all stall. Shutdown has the same exposure because `start_shutdown` takes the lock too.
**Fix:** Give each renderer a bounded outgoing queue drained by its own writer thread (drop `Clock` ticks when full, never block on them), or use overlapped writes with a timeout. At minimum, clone the target connection out from under the lock and write outside it.

### WR-05: Hello trusts the peer's claimed pid and slot

**File:** `apps/sucrose/src/service.rs:846-865` (`accept_hello`), `:634-681` (`read_connection`), `:727-744` (`register_writer`)
**Issue:** The renderer states its own `pid` and `slot`. `accept_hello` only checks that the pid is *some* renderer the supervisor started and returns the slot from the message, without checking that this pid was started for that slot. A renderer (or any same-user process that learns a renderer pid from the process list; the pipe DACL only separates users) can therefore claim another slot, receive that monitor's assignment (source path, rectangle), replace its writer entry (`state.connections.insert` silently overwrites the real one, so Close, Pause and clock relay go to the impostor), or claim the master slot and feed clock ticks to the followers. The first read of a new connection also has no timeout and every connection gets a thread, so an idle same-user client can pile up reader threads.
**Fix:** Have the supervisor keep `pid -> slot` and require `slot == slot_of(pid)`; refuse a Hello for a slot that already has a registered connection; on Windows confirm the peer with `GetNamedPipeClientProcessId` instead of the claimed pid; read the first frame under a short deadline and cap the number of unauthenticated connections.

### WR-06: mpv hardening does not cover scripts, IPC server and friends when a user config is loaded

**File:** `crates/sucrose-engine-mpv/src/config.rs:67-93`, `crates/sucrose-engine-mpv/src/player.rs:204-224`
**Issue:** The embedding options are documented as hardening for a headless instance (no scripts, no user config, T-02-14) and are applied after the user's `uMpvPlayer.config` so it "cannot turn them off". But `load-scripts=no` only disables auto-loading from the scripts directory; explicit `script=`/`scripts=`, `include=`, `input-ipc-server=`, `input-conf=`, `log-file=`, `save-position-on-quit`/watch-later writes and `use-filedir-conf` in that config still take effect because none of them is overridden. The file lives in a user-writable folder (`%APPDATA%\Sucrose\Cache\MpvPlayer`) shared with the C# app, so anything that can write it can run code in every renderer, and an `input-ipc-server` pipe would give other processes control over the player. The allow-list test only proves that the options this module emits are known, not that dangerous ones are neutralised. (Confidence: medium; confirm the exact reset syntax against `mpv --list-options` of the pinned build.)
**Fix:** Either do not load the user config at all in M1, or extend `embedding_options` with explicit neutral values for `scripts`/`script`, `input-ipc-server`, `input-conf`, `log-file`, `save-position-on-quit`, `use-filedir-conf` and `include`, add them to `ALLOWED_OPTIONS`, and extend the ignored real-libmpv test to read the effective values back after `mpv_initialize`.

### WR-07: libmpv is also loaded from `PATH` folders in release builds

**File:** `crates/sucrose-engine-mpv/src/loader.rs:128-142`, `:187-197`
**Issue:** `load_default` searches the exe folder and then every absolute `PATH` entry. `PATH` commonly contains folders a standard user can write (Python `Scripts`, per-user tool folders, `%LOCALAPPDATA%\...`), so a `libmpv-2.dll` planted there is loaded by every renderer, which is a persistent per-user background process, while the installed copy sits in a protected folder. The loader already avoids the current directory for this reason (T-02-02); the `PATH` fallback re-opens a smaller version of the same hole. `LOAD_WITH_ALTERED_SEARCH_PATH` then also resolves the DLL's own dependencies from the planted folder.
**Fix:** Search only the exe folder in release builds; keep the `PATH` fallback behind `cfg(debug_assertions)` or an explicit opt-in environment variable for development. If the `PATH` fallback is a recorded decision, record the trade-off next to it.

### WR-08: `DesktopSurface::attach` sleeps up to 10 s on a window-owning thread that is not pumping messages

**File:** `crates/sucrose-platform/src/os_windows/surface.rs:327-347` (called from `apps/sucrose/src/render.rs:101-102`)
**Issue:** The renderer creates the loop window and the surface (a top-level popup until it is re-parented) and then retries `try_attach` with `thread::sleep` (0.1 s doubling to 1 s) for up to `ATTACH_TIMEOUT` = 10 s. During that time no message is dispatched on a thread that owns two windows: Windows can mark the process "not responding", broadcast `SendMessage` callers and `WM_QUERYENDSESSION` wait on it, and a `TaskbarCreated` or display broadcast is only handled after the retries. The IPC reader is not started yet either, so a Close from the service during this phase is not seen until the attach and the engine start have finished.
**Fix:** Wait with `MsgWaitForMultipleObjects`/a small `PeekMessage` loop (or drive the retries from `EventLoop` with a timer) so messages keep flowing while the shell settles.

### WR-09: A failing `GetMessageW` ends the loop like a normal quit and the renderer exits with code 0

**File:** `crates/sucrose-platform/src/os_windows/event_loop.rs:172-199`, `apps/sucrose/src/render.rs:187-219`
**Issue:** `EventLoop::run` returns `()` for both `WM_QUIT` and a `GetMessageW` error (-1, only logged). The renderer then returns `ExitCode::SUCCESS`, which the supervisor classifies as `Clean`: no restart, slot left empty, no status change, so the wallpaper disappears without any message to the user. In the service the same error stops the service as if Quit had been chosen.
**Fix:** Return a `Result` (or an enum `Quit | Failed`) from `run` and map the failure to a non-zero exit code (a crash class that restarts within the budget) in the renderer, and to an error exit in the service.

## Info

### IN-01: `reparent` sets the child style after `SetParent`

**File:** `crates/sucrose-platform/src/os_windows/surface.rs:276-282`
**Issue:** The Win32 documentation for `SetParent` asks to clear `WS_POPUP` and set `WS_CHILD` *before* the call when a top-level window becomes a child. The code does it afterwards. It works on the tested shells (the attach is verified by `verify`), but it relies on undocumented tolerance.
**Fix:** Change the style bits first, then call `SetParent`; keep `verify` as the safety net.

### IN-02: Log message and behaviour disagree for a control message before the assignment

**File:** `apps/sucrose/src/render.rs:320-330`
**Issue:** The warning says the message is ignored, but `wait_for_assignment` returns `Ok(None)` and the renderer exits with success (see WR-09 for what the service makes of that). The protocol never produces this order, so this is only a mismatch.
**Fix:** Either keep waiting for the `Assign` or log "closing" and exit with a start-failure code.

### IN-03: Comment says the shader cache is never in the C# app's folder

**File:** `apps/sucrose/src/render.rs:283`, `:299-306`; `crates/sucrose-core/src/paths.rs:14-35`
**Issue:** Without `SUCROSE_DATA_DIR`, `cache_dir()` is `%APPDATA%\Sucrose\Cache`, which is the C# app's own data folder (the same one `legacy_mpv_cache_dir` points into). The shader cache therefore lands in `...\Sucrose\Cache\mpv-shaders`, beside the C# `MpvPlayer` folder. That may be acceptable under D-08 (writes go to the Rust app's dir, which deliberately defaults to the shared folder), but the comment and the user-config rationale ("cannot send the cache into the C# app's folder") are not true in the default configuration.
**Fix:** Correct the comments, or give the Rust app a distinct subfolder (for example `Cache\Rust`) if a clean separation is wanted.

### IN-04: `Player::open_source` is public and takes any mpv source string

**File:** `crates/sucrose-engine-mpv/src/player.rs:174-185`
**Issue:** `#[doc(hidden)] pub fn open_source` skips the absolute-path/UTF-8 check of `open` and passes the text straight to `loadfile` (URLs, `edl://`, `av://`). Only a doc comment keeps theme text out of it.
**Fix:** Make it `pub(crate)` and expose it to the integration test through a cargo feature (for example `test-support`), so a future caller cannot reach it by accident.

### IN-05: Any unrecognised command line starts the service

**File:** `apps/sucrose/src/main.rs:18-32`
**Issue:** Only the exact two-argument form `--render <slot>` selects renderer mode. `sucrose --render` (slot missing), `--render 1 extra` or a mistyped flag falls into the service branch; with the single-instance check it usually just exits, but a typo in a spawn argument would be hidden. In the release (GUI subsystem) build an `Err` returned from `main` is also invisible.
**Fix:** Reject unknown arguments with an error written to the log before exiting.

### IN-06: Unused public timer methods on `EventLoop`

**File:** `crates/sucrose-platform/src/os_windows/event_loop.rs:150-157`, `crates/sucrose-platform/src/unsupported.rs:56-59`
**Issue:** `EventLoop::set_timer` and `kill_timer` have no caller; the service uses `timers()` directly. They are dead public API that every OS stub must mirror.
**Fix:** Remove them (and the stubs) or use them.

### IN-07: Generality without instantiation: affected-monitor scope and `PauseKind` plumbing

**File:** `apps/sucrose/src/fullscreen.rs:94-105`, `apps/sucrose/src/service.rs:118-120`, `:1053-1055`
**Category:** necessity
**Issue:** `ActionScope::AffectedMonitor` (with `Rule.acted`/`scope`, `#[allow(dead_code)]`) is only reachable through the constant `ACTION_SCOPE = AllMonitors`; `PauseKind` is read from settings and carried in `Plan` only to log "Heavy is handled like Light". The code runs but guards nothing in M1. The comments say Phase 3 will wire both, which is a judgment about the roadmap and not provable from the repository, hence Info.
**Fix:** Keep only if Phase 3 really consumes it; otherwise drop the unused branch until it is needed.

### IN-08: Untrusted theme text reaches the log and the tooltip unsanitised

**File:** `apps/sucrose/src/service.rs:138-158`, `:1353`; `crates/sucrose-core/src/theme.rs:270-301`
**Issue:** The theme `Title` and the library folder name are interpolated into `Status::text`, the tray tooltip and log lines. A theme from a download can contain newlines or control characters (forged log lines, odd tooltips). Parse errors were deliberately kept free of file text; titles were not.
**Fix:** Strip or escape control characters when building `Job::title` and the folder name in rejection messages.

### IN-09: Backslash-escaped backticks in a crate doc comment

**File:** `crates/sucrose-core/src/theme.rs:1`
**Issue:** The first doc line contains `\`SucroseInfo.json\``, so rustdoc shows literal backslashes instead of code formatting.
**Fix:** Use plain backticks.

### IN-10: The recorded GPU regression has an obvious first suspect in the generated options

**File:** `crates/sucrose-engine-mpv/src/config.rs:98-122`; `docs/MEASUREMENTS.md`
**Issue:** Performance is outside this review, but the Rust renderer is recorded at 93.7 % GPU against 16.5 % for the C# player on the same 4K HEVC video, and the generated set enables `deband=yes` (a full-resolution shader pass) together with `profile=low-latency`. Both are carried over from the legacy config. The second measurement row ("all monitors") repeats the one-monitor numbers and is disclosed as such in its note.
**Fix:** Try `deband=no` and the default profile before the other options listed in the docs, and measure again.

---

_Reviewed: 2026-11-22T17:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
