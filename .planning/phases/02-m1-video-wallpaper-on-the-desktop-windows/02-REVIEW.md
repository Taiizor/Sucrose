---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
reviewed: 2026-11-23T00:00:00Z
depth: standard
files_reviewed: 23
files_reviewed_list:
  - Cargo.lock
  - apps/sucrose/src/render.rs
  - apps/sucrose/src/service.rs
  - crates/sucrose-config/src/legacy.rs
  - crates/sucrose-config/tests/fixtures/Engine.gpu.json
  - crates/sucrose-config/tests/legacy.rs
  - crates/sucrose-core/src/settings.rs
  - crates/sucrose-engine-mpv/src/player.rs
  - crates/sucrose-engine-mpv/src/sync.rs
  - crates/sucrose-platform/Cargo.toml
  - crates/sucrose-platform/src/layout.rs
  - crates/sucrose-platform/src/lib.rs
  - crates/sucrose-platform/src/os_windows.rs
  - crates/sucrose-platform/src/os_windows/event_loop.rs
  - crates/sucrose-platform/src/os_windows/gpu.rs
  - crates/sucrose-platform/src/os_windows/surface.rs
  - crates/sucrose-platform/tests/layout.rs
  - docs/ARCHITECTURE.md
  - docs/DECISIONS.md
  - docs/FEATURES.md
  - docs/MEASUREMENTS.md
  - xtask/Cargo.toml
  - xtask/src/measure.rs
findings:
  critical: 0
  warning: 4
  info: 5
  total: 9
  necessity: 0
status: issues_found
---

# Phase 2: Code Review Report (gap-closure plans 02-16..02-22)

**Reviewed:** 2026-11-23
**Depth:** standard
**Files Reviewed:** 23
**Status:** issues_found

## Summary

Incremental review of what changed since ae3b8c76 (GPU preference and adapter reporting, sticky master election with
the Playing-after-first-frame rule, the proportional+integral follower controller, the order guard and sent-message
drain in the Windows platform crate, and the same-adapter check in `cargo xtask measure`). Each file was read whole
for context. Nothing was built or run (Windows toolchain only, no GUI runs permitted).

No security problem or crash was found. The registry code in `gpu.rs` is bounded (one HKCU value, buffers sized
from the reported length with a spare terminator, every unsafe block carries a SAFETY comment), the hook callback
only touches thread-local state and only moves the renderer's own surface, and the controller math in `sync.rs`
holds up on a trace of the anti-windup branches, the paused-master path and the loop-wrap path. The findings below
are ordering races and robustness gaps in the new election/controller wiring, plus stale documentation.
No necessity findings were produced (the necessity reference was not needed for any item).

## Warnings

### WR-01: A Playing report that is processed after its renderer's Exit leaves a stale "playing" slot

**File:** `apps/sucrose/src/service.rs:338-366` (insert at 338, remove at 362), `apps/sucrose/src/service.rs:731-737`
**Issue:** `Event::Renderer { Playing }` is produced by the connection reader thread and `Event::Exited` by the
supervisor's wait thread. Both feed one channel with no ordering guarantee between them. A renderer that is closed
(monitor unplugged, fullscreen Close, shell restart) or crashes right after its first frame can have its `Exited`
handled first: `fleet.playing.remove(slot)` finds nothing, then the late `Playing` runs `fleet.playing.insert(slot)`
for a dead process and calls `elect`. Nothing removes that entry afterwards. Consequences: (a) `master_slot` can
elect a slot with no connection (no master clock, no audio until something else changes the election), and (b) once
`slots.release` frees the number and a new monitor reuses it, the brand-new renderer is treated as already playing
and can be elected master before its first frame, which is the G-02-2c failure that D-030 was written to prevent.
The window is narrow, but the code gives no protection against it.
**Fix:** Make the report identify the incarnation it came from and drop reports from dead ones. For example, have
`read_connection` send `Event::Renderer` together with the pid verified at the Hello and let `Event::Exited` carry
the pid, then ignore a `Playing` whose pid is not the one currently registered for that slot (keep
`fleet.pids: HashMap<u8, u32>`, set at Hello, cleared at Exited). A cheaper variant: in the `Playing` arm, also
require `lock(&renderers).connections.contains_key(&slot)` and `supervisor.slots().contains(&slot)` before inserting.

### WR-02: The follower controller records a speed as sent before `set_speed` succeeded

**File:** `crates/sucrose-engine-mpv/src/sync.rs:300-307` (`speed_if_changed`), `apps/sucrose/src/render.rs:480-500`
**Issue:** `speed_if_changed` updates `last_speed` and the speed-change counter before the caller applies the
action. When `player.set_speed(speed)` fails, `follow_clock` only logs a warning, but the controller now believes mpv
runs at `speed` and will not send it again until the wanted speed differs from it by 0.1 %. A follower can therefore
sit at the wrong speed (including a leftover corrected speed after the master paused) for many ticks while the
summary reports the change as made. `FollowState::reset` has the same blind spot: it sets `last_speed` to 1.0 without
looking at the player's real speed.
**Fix:** Let the caller confirm the action: add `Follower::speed_applied(&mut self, ok: bool)`, or make `step`
store the wanted speed in a `pending_speed` that `follow_clock` commits only after `set_speed` returns `Ok`. In
`reset`, read the real speed from the player (`speed` property) or call `set_speed(1.0)` so both sides agree.

### WR-03: The video's frame period is cached once, including the fallback default

**File:** `apps/sucrose/src/render.rs:457-467` and `apps/sucrose/src/render.rs:481`
**Issue:** `state.frame_s.get_or_insert_with(|| video_frame_period(player))` runs on the first tick after
`has_started`. `estimated-vf-fps` is usually not yet meaningful at the first frame, so when `container-fps` is
missing (some streams, some containers) the value is either the 1/30 s default or a transient estimate (anywhere
inside 5..=240 fps) and it is then used for the whole life of the renderer, in the band, the release threshold, the
integral logic and the logged "drift in frames" numbers. A wrong frame period silently changes how tightly the
follower holds sync (the 02-20 goal), and it is only logged once.
**Fix:** Cache the value only when it came from `container-fps`; otherwise recompute on the next ticks (for example
until `estimated-vf-fps` is stable or after N ticks) and log when it changes:
```rust
fn video_frame_period(player: &Player) -> (f64, bool /* final */) { /* container-fps => final */ }
```

### WR-04: The order guard removes itself permanently after ten restores in a minute

**File:** `crates/sucrose-platform/src/os_windows/surface.rs:696-706`
**Issue:** After `GUARD_MAX_RESTORES` (10) in `GUARD_WINDOW` (60 s) `remove_guard` unhooks and never re-arms. From
then on a shell `WorkerW` that is created above the surface hides the video until the renderer restarts, with a
single warn line as the only trace. A burst of `WorkerW` CREATE/SHOW events after a shell restart or a wallpaper
slideshow can reach the budget legitimately, so the protection against a runaway loop becomes a permanent loss of
the feature. The same path is also taken when `try_borrow_mut` fails (line 683), which cannot happen today but would
turn a transient reentrancy into permanent removal.
**Fix:** Back off instead of stopping: keep the hook, skip restores while over budget, and re-check when the oldest
entry leaves the window (the next event after `GUARD_WINDOW` will succeed). If removal is kept, report it through
the renderer (log at error level and exit with the surface-lost code so the service replaces the renderer), and
return `false` (skip, do not remove) from the `try_borrow_mut` failure arm.

## Info

### IN-01: Playing is now only reported after the first frame, with no path for a file that never starts

**File:** `apps/sucrose/src/render.rs:545-570`, `crates/sucrose-engine-mpv/src/player.rs:548-552`
**Issue:** `Started` is set only by the first `PLAYBACK_RESTART`, and `END_FILE` is only logged (`log_end_file`).
If mpv cannot open or decode the file the renderer stays alive, no `Playing` is sent, the tray status stays "starting"
and no master is elected, where before the plan the status said "playing" at once. D-030 and FEATURES do not mention
this state. Not an error in the design, but the failure is invisible to the user.
**Fix:** Treat an `END_FILE` with an error reason before `Started` as a start failure (report it with
`report_failure` and exit with the start-failure code), or time out the first frame and report it.

### IN-02: Stale comments after the 02-19/02-20/UAT changes

**File:** `crates/sucrose-engine-mpv/src/player.rs:268`, `apps/sucrose/src/render.rs:219`, `apps/sucrose/src/supervisor.rs:190-191`, `apps/sucrose/src/service.rs:995`
**Issue:** `set_speed` still documents "drift correction uses 0.98 and 1.02", but the controller now sends any value
within +-5 %. The renderer logs "the desktop surface was destroyed by the shell" and the supervisor/service docs say
explorer.exe "took Progman and with it every renderer's surface", while the updated constant docs and FEATURES state
that a killed explorer sends no WM_DESTROY and recovery is via TaskbarCreated only.
**Fix:** Reword to "a speed within 5 % of the master's (see `sync.rs`)" and "Windows destroyed the desktop surface".

### IN-03: The GPU preference overwrites a choice the user made in Windows Settings, on every start

**File:** `apps/sucrose/src/service.rs:1400-1420`, `crates/sucrose-platform/src/os_windows/gpu.rs:88-109`, `docs/FEATURES.md` (GPU preference row)
**Issue:** With no `GraphicPreference` key the default is HighPerformance, so a user who set "Power saving" for
`sucrose.exe` in Settings > Display > Graphics gets it replaced at each service start (`current_ordinal != 2`). This
matches the C# `EnsureHighPerformance`, but FEATURES documents only "skipping an unchanged value" and that the value
persists, not the override.
**Fix:** Say so in the FEATURES row (parity behavior), or only write when the value is absent or the setting was
explicitly present in Engine.json.

### IN-04: The renderer creates a Direct3D device just to log the adapter

**File:** `apps/sucrose/src/render.rs:117`, `crates/sucrose-platform/src/os_windows/gpu.rs:262-310`
**Issue:** `log_video_adapter` runs `D3D11CreateDevice` and queries the adapter in every renderer on every start,
before mpv creates its own device, only to emit one info line. The cost (driver load, a device on the dGPU) is
paid by a process whose low resource use is a stated goal (CLAUDE.md non-negotiable 6), and nothing consumes the
value except the human reading the log and the measurement run.
**Fix:** Run it only when the `debug` level is enabled (`tracing::enabled!(tracing::Level::DEBUG)`) or behind an
env switch used by `cargo xtask measure`.

### IN-05: Smaller inconsistencies in the new code

**File:** `xtask/src/measure.rs:1012-1021` (`checked_row`), `crates/sucrose-platform/src/os_windows/gpu.rs:116-121`
**Issue:** (a) `checked_row` appends ` GPU: ...` to `notes` with a plain space, so a note without a final period
produces the row now in MEASUREMENTS.md ("...GPU = max engine per process GPU: NVIDIA ..."). (b) `win32_error`
converts the code to an HRESULT while `create_window` in `surface.rs` and `event_loop.rs` store the raw Win32 code
(`code.0 as i32`) in the same `PlatformError::Os` field, so callers cannot interpret `code` uniformly.
**Fix:** (a) Insert a separator: `format!("{notes}; GPU: ...")` or trim and add a period. (b) Pick one representation
(HRESULT through `HRESULT::from_win32`) and use it in all three places.

---

_Reviewed: 2026-11-23_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
