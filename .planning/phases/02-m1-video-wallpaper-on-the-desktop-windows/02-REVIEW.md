---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
reviewed: 2026-11-23T04:00:00Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - apps/sucrose/src/fullscreen.rs
  - apps/sucrose/src/main.rs
  - apps/sucrose/src/render.rs
  - apps/sucrose/src/service.rs
  - apps/sucrose/src/supervisor.rs
  - crates/sucrose-core/src/paths.rs
  - crates/sucrose-core/src/theme.rs
  - crates/sucrose-engine-mpv/src/config.rs
  - crates/sucrose-engine-mpv/src/loader.rs
  - crates/sucrose-engine-mpv/src/player.rs
  - crates/sucrose-engine-mpv/src/sync.rs
  - crates/sucrose-ipc/src/frame.rs
  - crates/sucrose-ipc/src/protocol.rs
  - crates/sucrose-platform/src/os_windows/event_loop.rs
  - crates/sucrose-platform/src/os_windows/gpu.rs
  - crates/sucrose-platform/src/os_windows/surface.rs
  - crates/sucrose-platform/src/unsupported.rs
  - docs/FEATURES.md
  - docs/MEASUREMENTS.md
  - xtask/src/measure.rs
findings:
  critical: 0
  warning: 10
  info: 16
  total: 26
  necessity: 1
status: issues_found
---

# Phase 2: Code Review Report (consolidated: fixes of a4f7d73b plus carry-forward)

**Reviewed:** 2026-11-23
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

One consolidated review of phase 02 with two parts. Nothing was built or run (Windows toolchain only, no GUI runs); the four fix commits were read as a diff (`git diff a4f7d73b..HEAD -- apps crates docs`) and every touched function was read whole.

**Incremental part (fix commits fb1265e2, 398b1601, e71834ab, 982bff57).**
- WR-01 (late Playing after Exited, fb1265e2): the `connections.contains_key` guard is correct for every ordering the code can produce, because `register_writer` always runs before the reader thread sends the first `Playing`, and `Exited` removes the entry. The window named in the fix report (a replacement renderer registers before the dead one's late `Playing` is handled) exists in principle but needs the dead renderer's reader thread to be starved for longer than the 500 ms minimum restart pause plus the replacement's start-up. It is recorded as IN-16, not as a warning.
- WR-02 (speed recorded before set_speed succeeded, 398b1601): correct and complete. `speed_failed` restores `last_speed` and the change counter, `reset` starts at the real speed, both unit tests exercise it. No new defect.
- WR-03 (frame period cached once, e71834ab): correct. Only a `container-fps` value is cached, estimates are read again and logged on a change above 1 %. One stale doc comment is left behind (listed under IN-02).
- WR-04 (order guard removed itself, 982bff57): the budget logic is correct and no longer removes the hook, but the held-back state has no way to end by itself: a guard that refused the last event of a burst never looks again, so the video can stay hidden. That is a new defect in the fix, WR-14.

**Carry-forward part.** All carried findings of `a4f7d73b` (IN-01..IN-05) and `ae3b8c76` (WR-01..WR-09, IN-01..IN-10) were re-checked against the current code. Only prior-review-1 IN-10 is resolved (see the last section); everything else still applies, with updated locations. Prior WR-01..WR-04 of the second review stay fixed and are not reopened. Prior-review-1 WR-01..WR-04 appear as WR-10..WR-13 and IN-01..IN-05 as IN-11..IN-15.

No security vulnerability that crosses a privilege boundary was found; no Critical issue. The gate in `theme.rs` (traversal, links, UNC, drive and stream prefixes) was re-read and holds.

## Warnings

### WR-05: Hello trusts the peer's claimed slot, and a second Hello replaces the registered writer

**Origin:** prior-review-1 WR-05
**File:** `apps/sucrose/src/service.rs:887-906` (`accept_hello`), `:666-710` (`read_connection`), `:759-776` (`register_writer`)
**Issue:** `accept_hello` checks that the pid is one of the renderers the supervisor started, and then returns the slot the peer wrote into its own Hello. The pid is never matched to the slot it was started for, and `register_writer` does `state.connections.insert(slot, writer)`, which silently replaces the connection of a renderer that is already registered. A same-user process that knows any renderer pid (a process list is enough) can take over another monitor's slot: it gets that monitor's source path and rectangle in the `Assign`, receives Close, Pause and the clock relay in place of the real renderer, and can claim the master slot. The first read of a connection also has no deadline and every accepted connection gets a thread. The pipe DACL keeps other users out, so this is a same-user robustness and integrity gap, not a privilege escalation.
**Fix:** Keep `pid -> slot` in `Supervisor::State` (it already has both at `spawn`), make `accept_hello` require `slot == slot_of(pid)`, and refuse a Hello for a slot that already has a registered connection. Optionally confirm the peer pid with `GetNamedPipeClientProcessId` (platform crate) and give the first `recv` a short deadline. Needs no protocol change.

### WR-06: mpv hardening does not neutralise scripts, IPC server, log file or includes when a user config is loaded

**Origin:** prior-review-1 WR-06
**File:** `crates/sucrose-engine-mpv/src/config.rs:67-93`, `crates/sucrose-engine-mpv/src/player.rs:208-228`, `apps/sucrose/src/render.rs:310`
**Issue:** The renderer loads `uMpvPlayer.config` from the C# app's folder through `mpv_load_config_file` and then applies `embedding_options`, documented (T-02-14, FEATURES Video row) as hardening the user config cannot turn off. `load-scripts=no` only stops auto-loading from the scripts directory; explicit `script=`/`scripts=`, `include=`, `input-ipc-server=`, `input-conf=`, `log-file=`, `use-filedir-conf` and `save-position-on-quit` from that config still take effect, and `config=no` applied afterwards does not undo a file that was already loaded. The file sits in a user-writable folder shared with the C# app, so whatever can write it gets code or a control pipe inside every renderer. Same user, so no privilege boundary is crossed, but the stated guarantee is false. (Confidence medium: confirm the neutral values with `mpv --list-options` of the pinned v0.42.0-38.)
**Fix:** Either stop loading the user config in M1 (the generated options then apply), or add explicit neutral values to `embedding_options` for `script`/`scripts` (empty), `input-ipc-server` (empty), `input-conf`, `log-file` (empty), `use-filedir-conf=no`, `save-position-on-quit=no` and `include`, add the names to `ALLOWED_OPTIONS`, and extend the ignored real-libmpv test (`tests/options.rs`) to read the effective values back after `mpv_initialize`. Reading values back needs a real libmpv: mark that test part as requiring `cargo xtask fetch-engines`. Keeping or dropping the user config is a maintainer decision (D-10 says it is honored).

### WR-07: libmpv is also loaded from user-writable `PATH` folders

**Origin:** prior-review-1 WR-07
**File:** `crates/sucrose-engine-mpv/src/loader.rs:128-142`, `:187-197`, `:253`
**Issue:** `load_default` searches the exe folder first, then every absolute `PATH` entry. `PATH` often holds folders a standard user (and on shared machines other users) can write; a planted `libmpv-2.dll` there is loaded by a long-lived background process, and `LOAD_WITH_ALTERED_SEARCH_PATH` resolves that DLL's own dependencies from the planted folder too. This is the narrowed form of the hole T-02-02 closes for the current folder. **Accepted decision D-028 (which narrows D-018 and states "searched next to `sucrose.exe`, then on absolute `PATH` entries") prescribes exactly this order, so the order itself must not be changed by the fixer.**
**Fix:** Stay inside D-028: keep the search order, and (a) log at `warn` level, with the full path, whenever the library is loaded from a folder other than the exe folder, so the use of the fallback is visible; (b) in `resolve`, skip a `PATH` entry that is also the current folder or a UNC path (network DLL loading). Removing the `PATH` step in release builds (`cfg(debug_assertions)` only) would contradict D-028 as written: that is a maintainer decision to supersede D-028, not a fixer change.

### WR-08: `DesktopSurface::attach` sleeps up to 10 s on a thread that owns two windows and pumps nothing

**Origin:** prior-review-1 WR-08
**File:** `crates/sucrose-platform/src/os_windows/surface.rs:418-448` (called from `apps/sucrose/src/render.rs:103-104`)
**Issue:** The renderer creates the loop window (`EventLoop::new`) and the surface and then retries `try_attach` with `thread::sleep` (0.1 s doubling to 1 s) for up to `ATTACH_TIMEOUT` = 10 s, with no message dispatched. Windows can mark the process "not responding", `SendMessage` callers and the broadcast of `TaskbarCreated`/display changes wait on it, and the IPC reader thread is not started yet, so a Close from the service is not seen until attach and engine start end. The path is exactly the explorer-restart recovery, where the wait is longest.
**Fix:** Replace `thread::sleep(delay)` by a wait that keeps messages flowing, for example `MsgWaitForMultipleObjects` with the delay as timeout followed by a `PeekMessageW`/`DispatchMessageW` drain, or drive the retries from `EventLoop` with a timer. Needs a GUI run to confirm (kill explorer.exe, watch for the "not responding" ghosting); the change itself is testable by review.

### WR-09: A failing `GetMessageW` ends the loop like a normal quit and the renderer exits with code 0

**Origin:** prior-review-1 WR-09
**File:** `crates/sucrose-platform/src/os_windows/event_loop.rs:224-251` (`run`), `apps/sucrose/src/render.rs:191-224`, `crates/sucrose-platform/src/unsupported.rs:53`
**Issue:** `run` returns `()` both for `WM_QUIT` and for `GetMessageW` = -1 (only logged at error level). The renderer then falls through to `ExitCode::SUCCESS`, which the supervisor classifies as `Clean`: no restart, the slot stays empty, the status does not change, and the wallpaper vanishes without a word to the user. In the service the same error returns from `run()` as if Quit had been chosen.
**Fix:** Return an enum or `Result` from `EventLoop::run` (`Quit | Failed`; the unsupported stub returns `Quit`), map `Failed` to a non-zero crash-class exit code in the renderer (so the supervisor restarts it within the budget) and to an error return in the service.

### WR-10: Loop wrap is never detected, `loop_gen` stays 0 for an ordinary loop

**Origin:** prior-review-1 WR-01
**File:** `crates/sucrose-engine-mpv/src/player.rs:530-543`, `:559-572` (`note_restart`), `crates/sucrose-ipc/src/protocol.rs:82-83`
**Issue:** `note_restart` compares the new `time-pos` with `shared.last_position`, which is written only at restarts (first one and each later one), never while playback advances. For a looping file the previous restart was near 0.0 s and so is the wrap restart: `delta` is tiny, `classify_jump` returns `Jump::None`, and `loop_gen` never increments. The unit tests feed `classify_jump` a hand-made "29.8 -> 0.05" pair that the event path cannot produce. `ClockTick::loop_gen` is documented as "how many times the master looped" and is sent on the wire, but no follower reads it (grep: only `render.rs:552` writes it), so today the effect is a wrong wire field, a wrong doc and a misleading test. It stays a warning because the field is a protocol contract that M2 work will rely on.
**Fix:** Track where playback should be: keep `(Instant, position, speed)` at each restart and on the next restart compute `expected = position + elapsed * speed`; call it a wrap when the new position is below 1 s and `expected` is beyond `duration - 1 s` (or beyond `duration`). Add a test of that extrapolation function with a looping scenario. If no consumer is planned, remove `loop_gen` from the protocol instead (needs a `PROTOCOL_VERSION` decision, so confirm with the maintainer).

### WR-11: Tray/log status stays "no wallpaper" after the renderer recovers

**Origin:** prior-review-1 WR-02
**File:** `apps/sucrose/src/service.rs:181-202` (`restarting`, `after`), `:353-363`, `:400-410`
**Issue:** `Status::after(Playing)` only moves `Starting -> Playing`, and `Status::restarting()` only maps `Playing -> Starting`. A renderer that reports `Failed` (for example the surface could not attach while explorer restarts, exit code 4) is restarted by `decide_exit`, but the status is `Failed`, so `restarting()` is `None`, and when the new renderer sends `Playing`, `after` sees `Failed` and returns `None`. The tooltip says "no wallpaper: ..." for the whole session while the wallpaper plays. The same holds after `RendererStopped` of one monitor while the others play, and one monitor's `Failed` hides the `Playing` of the rest. The unit test `a_playing_report_changes_nothing_when_no_renderer_was_expected` only pins the `NoWallpaper` case.
**Fix:** Let `Playing` replace `Starting`, `Failed` and `RendererStopped` (but not `NoWallpaper`, where no renderer was wanted), and map `Failed` to `Starting` in `restarting()`. Better: keep one status per slot and derive the tooltip from the set (any failure shown, otherwise playing). Add tests for Failed -> restarting -> Playing.

### WR-12: Unsynchronised writes on the reader's pipe handle can interleave frames

**Origin:** prior-review-1 WR-03
**File:** `apps/sucrose/src/service.rs:714-732` (catch-up `SetRect`/`Close` through `connection`), `crates/sucrose-ipc/src/frame.rs:22-24`
**Issue:** After `register_writer` stored a clone under the `Renderers` lock, `read_connection` still sends the catch-up `SetRect` (line 721) and `Close` (line 727) through its own handle, without that lock. Every other writer (`send_to`, `send_role`, `elect`, the master's `relay_clock`) writes through the registered clone under the lock. `write_frame` issues two `write_all` calls (4-byte length, then payload), so a frame from the main thread (a `SetRole`, `Pause`, `SetRect` from `move_to`) can land between header and payload of the other and the renderer decodes garbage or drops the connection with `FrameTooLarge`/`Decode`. The window is small but it is exactly where hot-plug and fullscreen changes race a starting renderer.
**Fix:** Do the catch-up through the shared writer (`send_to(renderers, slot, ...)`), and make `write_frame` build one buffer (`length || payload`) and issue a single `write_all` so a frame is never split between two writers. Both parts are testable without a GUI.

### WR-13: Blocking pipe writes while holding the global `Renderers` mutex can freeze the service loop

**Origin:** prior-review-1 WR-04
**File:** `apps/sucrose/src/service.rs:807-824` (`relay_clock`), `:952-963` (`send_to`), `:1198-1215` (`elect`), `:780-793` (`send_role`); lock taken by the main thread at `:342`, `:366`, `:1299`
**Issue:** Every send is a blocking synchronous named-pipe write with the single `Mutex<Renderers>` held (the IPC crate has no overlapped I/O or timeouts: grep of `crates/sucrose-ipc/src` finds none). The master's reader thread relays a tick to every follower once per second under that lock. A follower that stops reading (suspended, stuck in a driver call, under a debugger) eventually fills its pipe buffer and the write blocks with the lock held; the main thread, which pumps the tray and display messages, then blocks in `lock(&renderers)` (Exited, `send_to`, `elect`, the `Playing` guard added by fb1265e2), so Quit, hot-plug and the fullscreen rule stall. `start_shutdown` takes the same lock.
**Fix:** Give each renderer a bounded outgoing queue drained by a writer thread of its own (drop `Clock` ticks when full, never block on them), or at least clone the target connection out from under the lock and write outside it with a per-connection mutex. Hanging a follower to prove the freeze needs a real run; the queue itself is unit-testable.

### WR-14: The order guard that holds back never looks again, so a surface left under the wallpaper stays hidden

**Origin:** new, a defect in the fix of prior-review-2 WR-04 (commit 982bff57)
**File:** `crates/sucrose-platform/src/os_windows/surface.rs:695-728` (`restore_order`), doc at `:640-644`
**Issue:** After ten restores in 60 s the guard returns `Budget::Exhausted` and ignores the event; the doc and the FEATURES row promise that it "holds back until the oldest restore leaves the window". But the guard is event-driven only: the next restore happens at the next relevant WinEvent after the window has cleared. If the event that exhausted the budget was the last one of the burst (a shell restart or slideshow burst ends with the final `WorkerW` create/show/reorder), the surface stays under the shell's `WorkerW` (video invisible, `raised_order` wrong) until the shell happens to reorder again, which can be never. The fix replaced "gives up for good" by "gives up until an unrelated event", so the user-visible failure is the same in the burst-end case. Also, the budget is spent before `find_child`/`SetWindowPos` run, so a failed restore uses it up too.
**Fix:** When the guard returns `Exhausted`, arm a one-shot re-check for when the oldest restore leaves the window: a thread-local `SetTimer(None, 0, remaining_ms, Some(callback))` (the surface's thread pumps messages), whose callback re-runs `restore_order` for that hook and kills itself; clear it in `remove_guard`/`Drop`. Count a restore only after `SetWindowPos` succeeded. Needs a GUI run (explorer restart, then a wallpaper-slideshow burst) to confirm.

## Info

### IN-01: Playing is only reported after the first frame, with no path for a file that never starts

**Origin:** prior-review-2 IN-01
**File:** `apps/sucrose/src/render.rs:568-593`, `crates/sucrose-engine-mpv/src/player.rs:552`, `:589-599`
**Issue:** `Started` is set only by the first `PLAYBACK_RESTART`, and `END_FILE` is only logged (`log_end_file`). If mpv cannot open or decode the file (loadfile is asynchronous: `Player::open` returned Ok) the renderer stays alive, no `Playing` is sent, no master is elected and the tray stays on the "starting" text with no explanation. D-030 and FEATURES do not describe this state.
**Fix:** Treat an `END_FILE` with reason error before `Started` as a start failure: publish it as a `PlayerEvent`, have the clock thread call `report_failure` and make the renderer exit with the start-failure code (4). Or time out the first frame (for example 30 s) and report it. A real unplayable file is needed to see it (GUI run or the ignored libmpv tests).

### IN-02: Stale comments after the 02-19/02-20/UAT changes and after the WR-03 fix

**Origin:** prior-review-2 IN-02
**File:** `crates/sucrose-engine-mpv/src/player.rs:268`, `apps/sucrose/src/render.rs:219` and `:442-443`, `apps/sucrose/src/supervisor.rs:190-191`, `apps/sucrose/src/service.rs:507` and `:1282-1283`
**Issue:** (a) `set_speed` still documents "drift correction uses 0.98 and 1.02"; the controller sends any value within 5 % of the master's. (b) `FollowState::frame_s` says "read once the first frame is shown"; since e71834ab only a `container-fps` value is read once. (c) The renderer log says "the desktop surface was destroyed by the shell"; the supervisor and service docs say explorer.exe "took Progman and with it every renderer's surface" and "Progman and every surface in it are gone", while the constants' docs and FEATURES state that a killed explorer sends no `WM_DESTROY` and the cause is not established.
**Fix:** (a) "a speed within 5 % of the master's (see `sync.rs`)". (b) "the container's value is read once; an estimate is read again". (c) Say "Windows destroyed the desktop surface" and "its surfaces have to be replaced", without asserting the mechanism.

### IN-03: The GPU preference overwrites a choice made in Windows Settings, on every start

**Origin:** prior-review-2 IN-03
**File:** `apps/sucrose/src/service.rs:1408-1429`, `crates/sucrose-platform/src/os_windows/gpu.rs:88-109`, `docs/FEATURES.md:35`
**Issue:** With no `GraphicPreference` key the default is HighPerformance, so a user who set "Power saving" for `sucrose.exe` in Settings > Display > Graphics gets it replaced at each service start (`current_ordinal != 2`). That is the C# `EnsureHighPerformance` behavior, but FEATURES documents only "skipping an unchanged value", not the override.
**Fix:** Say so in the FEATURES row (parity behavior), or write only when the value is absent or Engine.json names the preference explicitly. Which one is a maintainer choice (behavior change vs parity).

### IN-04: The renderer creates a Direct3D device just to log the adapter

**Origin:** prior-review-2 IN-04
**File:** `apps/sucrose/src/render.rs:117`, `:286-296`, `crates/sucrose-platform/src/os_windows/gpu.rs:262-310`
**Issue:** `log_video_adapter` runs `D3D11CreateDevice` in every renderer on every start, before mpv creates its own device, only to emit one info line. The cost (driver load, a device on the dGPU) is paid by a process whose low resource use is a stated goal (CLAUDE.md non-negotiable 6). `cargo xtask measure` reads the adapter from per-process counters, not from this line.
**Fix:** Run it only when debug logging is enabled (`tracing::enabled!(tracing::Level::DEBUG)`) or behind the measurement run's env switch; keep the line at info only where it is free.

### IN-05: Smaller inconsistencies in error codes and measurement notes

**Origin:** prior-review-2 IN-05
**File:** `xtask/src/measure.rs:1012-1021` (`checked_row`), `crates/sucrose-platform/src/os_windows/gpu.rs:116-121`, `surface.rs:275-278`, `:436-439`, `event_loop.rs:137-140`
**Issue:** (a) `checked_row` appends ` GPU: ...` to `notes` with a plain space, so a note without a final period runs together with it. (b) `win32_error` stores an HRESULT (`HRESULT::from_win32`) in `PlatformError::Os::code`, while `create_window` (surface and event loop) store the raw Win32 code (`code.0 as i32`) and `attach` stores `0`, so callers cannot interpret `code` uniformly.
**Fix:** (a) `format!("{notes}; GPU: ...")` after trimming trailing punctuation. (b) Pick one representation (HRESULT through `HRESULT::from_win32`) and use it in all four places; give `attach` the last failure's code instead of 0.

### IN-06: Unused public timer methods on `EventLoop`

**Origin:** prior-review-1 IN-06
**File:** `crates/sucrose-platform/src/os_windows/event_loop.rs:200-209`, `crates/sucrose-platform/src/unsupported.rs:55-59`, `crates/sucrose-platform/src/lib.rs:60`
**Issue:** `EventLoop::set_timer` and `kill_timer` have no caller anywhere in the workspace (grep); the service uses `timers()` / `LoopTimers` directly. Dead public API that every OS stub mirrors; the doc of `LoopEvent::Timer` still points to `set_timer`.
**Fix:** Remove both (and the stubs) and point the docs at `EventLoop::timers`.

### IN-07: Generality without instantiation: affected-monitor scope and `PauseKind` plumbing

**Origin:** prior-review-1 IN-07
**Category:** necessity
**File:** `apps/sucrose/src/fullscreen.rs:94-105`, `:144-193`, `apps/sucrose/src/service.rs:120-122`, `:1094-1101`
**Issue:** `ActionScope::AffectedMonitor` (with `Rule::acted`, `acts_on`'s second arm and an `#[allow(dead_code)]`) is reachable only through the constant `ACTION_SCOPE = AllMonitors`; `PauseKind` is read from settings and passed to `enable_fullscreen` only to log "Heavy is handled like Light". The code runs and is tested but changes no behavior in M1. The comments say Phase 3 persists the switch, which is a judgment about the roadmap, not provable from the repository, hence Info.
**Fix:** Keep only if Phase 3 consumes it (check `.planning/ROADMAP.md` PERF-01); otherwise remove the unused scope branch and the `pause` parameter until needed. A scope decision for the maintainer.

### IN-08: Untrusted theme text reaches the log and the tooltip unsanitised

**Origin:** prior-review-1 IN-08
**File:** `apps/sucrose/src/service.rs:152-161`, `:1386-1393`, `:1435`; `crates/sucrose-core/src/theme.rs:270-300`
**Issue:** The theme `Title` (from `SucroseInfo.json`), the selected folder name and renderer-supplied reasons (only length-limited by `limit`) are interpolated into `Status::text`, the tray tooltip and log lines. A theme from a download can contain newlines or control characters (forged log lines, odd tooltips). Parse errors were deliberately kept free of file text; titles were not.
**Fix:** Strip or escape control characters (`char::is_control`) in `Job::title`, in `limit()` and in the folder name used by `ThemeRejection`, or log them with `{:?}`.

### IN-09: Backslash-escaped backticks in a crate doc comment

**Origin:** prior-review-1 IN-09
**File:** `crates/sucrose-core/src/theme.rs:1`
**Issue:** The first doc line contains `\`SucroseInfo.json\``, so rustdoc renders literal backslashes instead of code formatting.
**Fix:** Use plain backticks.

### IN-11: `reparent` sets the child style after `SetParent`

**Origin:** prior-review-1 IN-01
**File:** `crates/sucrose-platform/src/os_windows/surface.rs:316-322`
**Issue:** The Win32 documentation for `SetParent` asks to clear `WS_POPUP` and set `WS_CHILD` before the call when a top-level window becomes a child. The code does it afterwards. It works on the shells tested (`verify` follows), but relies on undocumented tolerance.
**Fix:** Change the style bits first, then call `SetParent`; keep `verify` as the safety net. Needs a GUI run on the raised and classic layouts to confirm nothing regresses.

### IN-12: Log message and behaviour disagree for a control message before the assignment

**Origin:** prior-review-1 IN-02
**File:** `apps/sucrose/src/render.rs:340-350`, `:91-93`
**Issue:** The warning says the message is ignored, but `wait_for_assignment` returns `Ok(None)` and the renderer exits with `ExitCode::SUCCESS`, which the supervisor reads as a clean exit (no restart, no status change). The protocol never produces this order, so it is only a mismatch.
**Fix:** Either keep waiting for the `Assign`, or log "closing" and return a start-failure error so the exit code is 4.

### IN-13: The shader-cache comments claim a separation that the default configuration does not have

**Origin:** prior-review-1 IN-03
**File:** `apps/sucrose/src/render.rs:303`, `:319-321`; `crates/sucrose-engine-mpv/src/config.rs:124-125`; `crates/sucrose-core/src/paths.rs:14-35`
**Issue:** Without `SUCROSE_DATA_DIR`, `cache_dir()` is `%APPDATA%\Sucrose\Cache`, which is the C# app's own data folder (`legacy_mpv_cache_dir` is `...\Cache\MpvPlayer` in it). The shader cache therefore lands in `...\Sucrose\Cache\mpv-shaders`, beside the C# `MpvPlayer` folder. That fits D-007/D-008 (the Rust app deliberately shares the folder), but the comments "never the C# app's" and "cannot send the cache into the C# app's folder" are not true by default.
**Fix:** Correct the comments to say "never into the C# `MpvPlayer` folder", or use a distinct subfolder if a clean separation is wanted.

### IN-14: `Player::open_source` is public and takes any mpv source string

**Origin:** prior-review-1 IN-04
**File:** `crates/sucrose-engine-mpv/src/player.rs:178-189`; only other caller `crates/sucrose-engine-mpv/tests/options.rs:95`
**Issue:** `#[doc(hidden)] pub fn open_source` skips the absolute-path/UTF-8 check of `open` and hands the text to `loadfile` (URLs, `edl://`, `av://`). Only a doc comment keeps theme text out of it, in a crate whose public API a later plan may call.
**Fix:** Make it `pub(crate)` and expose it to the integration test through a cargo feature (for example `test-support`), so a future caller cannot reach it by accident.

### IN-15: Any unrecognised command line starts the service

**Origin:** prior-review-1 IN-05
**File:** `apps/sucrose/src/main.rs:18-32`
**Issue:** Only the exact two-argument form `--render <slot>` selects renderer mode. `sucrose --render` (slot missing), `--render 1 extra` or a mistyped flag falls into the service branch; with the single-instance check it usually just exits, so a typo in a spawn argument would be hidden. In the release (GUI subsystem) build an `Err` from `main` is also invisible.
**Fix:** Reject unknown arguments with an error written to the log (`sucrose_log::init` first) before exiting non-zero; `--render` without a valid slot is an error, not a service start.

### IN-16: A late `Playing` is only dropped by connection presence, because reports carry no renderer identity

**Origin:** new, the window the fix report names for prior-review-2 WR-01 (commit fb1265e2)
**File:** `apps/sucrose/src/service.rs:337-352`, `:365-369`, `apps/sucrose/src/supervisor.rs:359-365`
**Issue:** `Event::Renderer` and `Event::Exited` identify a renderer by slot only. The guard `connections.contains_key(&slot)` is wrong only if a replacement renderer has registered its connection before the dead one's late `Playing` is handled. That needs the dead renderer's reader thread starved or blocked for longer than the minimum restart pause (500 ms, `FIRST_BACKOFF`) plus the replacement's start-up and Hello, so it is real in theory and not reachable with normal scheduling; the guard closes every ordering that matters in practice. A second, smaller instance of the same root cause: `Supervisor::wait_for` clears `running` (line 362) before it queues `Exited` (line 365), so `shell_restarted` can spawn the replacement into that gap and the `Exited` of the old one then removes the new one's connection and classifies the exit for the new slot owner; the gap is microseconds against a process start.
**Fix:** Tag both events with the pid verified at the Hello and the pid that exited, keep `fleet.pids: HashMap<u8, u32>`, and ignore a `Playing`/`Exited` whose pid is not the current one for the slot (the pid-to-slot map of WR-05 provides the other half). Low priority; no GUI run needed.

### IN-17: The "all monitors" measurement row repeats the one-monitor numbers

**Origin:** new (prior-review-1 IN-10 named the repeated row; the row is still there after the same-GPU rerun)
**File:** `docs/MEASUREMENTS.md` (second M1 row)
**Issue:** The row "M1 video wallpaper, all monitors (Rust)" is a copy of the one-monitor row and says so in its note ("all monitors = one", D-25). A table that MEAS-01 reads gets a row whose scenario name promises a measurement that was not made, and C# has no counterpart for it.
**Fix:** Remove the row, or keep it and rename the scenario to "1 active display (repeat)"; add a real two-monitor row when a second display is available (needs the maintainer's machine).

## Resolved Since Earlier Reviews

- **prior-review-1 IN-10** (recorded GPU regression, "first suspect is deband=yes and profile=low-latency"): resolved by evidence. `02-UAT.md` gap G-02-6 gives the root cause: the 93.7 % vs 16.5 % compared two GPUs (Rust on the Intel UHD because the GPU preference was not ported, C# on the RTX 3050). The preference is now written by the service (`service.rs:1408-1429`, `gpu.rs`), `cargo xtask measure --row` refuses two different adapters (`measure.rs:975-1001`, `same_adapter`), and the replaced same-GPU row in `docs/MEASUREMENTS.md` shows 16.4 % vs 16.1 % (R = 1.02), so `deband=yes` and `profile=low-latency` stay for parity (recorded in FEATURES). Nothing is left to change in `config.rs`.
- **prior-review-2 WR-01..WR-04**: fixed by fb1265e2, 398b1601, e71834ab, 982bff57 and checked above; their remaining edges are WR-14 and IN-16 (new ids), IN-02 (b).

---

_Reviewed: 2026-11-23_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
