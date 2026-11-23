---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
reviewed: 2026-11-23T14:00:00Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - Cargo.lock
  - apps/sucrose/src/main.rs
  - apps/sucrose/src/render.rs
  - apps/sucrose/src/service.rs
  - apps/sucrose/src/supervisor.rs
  - crates/sucrose-core/src/theme.rs
  - crates/sucrose-engine-mpv/Cargo.toml
  - crates/sucrose-engine-mpv/src/config.rs
  - crates/sucrose-engine-mpv/src/loader.rs
  - crates/sucrose-engine-mpv/src/player.rs
  - crates/sucrose-engine-mpv/tests/options.rs
  - crates/sucrose-ipc/src/frame.rs
  - crates/sucrose-platform/src/lib.rs
  - crates/sucrose-platform/src/os_windows/event_loop.rs
  - crates/sucrose-platform/src/os_windows/surface.rs
  - crates/sucrose-platform/src/unsupported.rs
  - docs/ARCHITECTURE.md
  - docs/FEATURES.md
  - docs/MEASUREMENTS.md
  - xtask/src/measure.rs
findings:
  critical: 0
  warning: 3
  info: 4
  total: 7
  necessity: 1
status: issues_found
---

# Phase 2: Code Review Report (incremental: fix commits WR-05 .. WR-14, IN-01 .. IN-17)

**Reviewed:** 2026-11-23
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

Incremental re-review of `git diff fcac6699..HEAD` (25 fix commits). Every touched function was read whole, together with its callers in `service.rs`, `supervisor.rs` and `render.rs`. Nothing was built or run (Windows toolchain only, no GUI runs).

Verdict per larger fix:
- **WR-12 / WR-13 (single-write frames, per-connection writer thread and bounded queue):** sound. One writer per connection after registration, the Hello reply goes out before the writer exists, so no interleaving. No send happens while blocked under the `Renderers` lock. Lock order renderers -> supervisor is the only nesting (read_connection, start_shutdown) and no path nests the other way. Residual robustness points are in IN-21.
- **WR-10 (loop wrap from an expected position):** the anchor arithmetic is correct for an ordinary loop (anchor ~0 s at the last wrap, restart one loop later, expected ~duration), for pauses (rate 0 re-anchored on the pause change) and for follower speed changes (margin 1 s + 10 % of the duration). No defect found.
- **WR-09 (LoopEnd::Failed):** correct; the renderer maps it to exit code 5 (crash budget), the service bails.
- **WR-14 (guard re-check timer):** the timer logic (one-shot, killed before the re-check, killed by `remove_guard`, thread timer dispatched by the pump) is correct; one weakened safeguard is IN-19.
- **WR-08 (message pumping during attach):** pumping works; a queued Quit is still not seen (IN-20).
- **IN-11, IN-15, IN-14, IN-06, IN-05, IN-08, IN-09, IN-13, IN-17:** resolved as described, no new defect found (checked `reparent` against the SetParent documentation, `parse_mode`, the `test-support` self dev-dependency, `checked_row`, `printable`).
- **WR-05 (slot/pid binding) and IN-16 (renderer pid tagging):** both are incomplete. The refusal of a Hello turns a legitimate renderer into a permanently dead monitor (WR-16), and the pid tag is not applied to the status change (WR-15).
- **IN-01 + WR-11 (failed start, status):** the interaction loses the failure reason (WR-17).

Carried findings: IN-07 still holds (below). IN-10 no longer holds (see "Carried findings resolved").

## Warnings

### WR-15: A late `Playing` or `Failed` of an already ended renderer still rewrites the status (reopens IN-16 and WR-01)

**Origin:** new; the fix of IN-16 is incomplete
**File:** `apps/sucrose/src/service.rs:430-464` (status change at `:453`), `:1013-1017` (`report_is_stale`)
**Issue:** `report_is_stale(link_pid, pid)` is only true when another renderer's link is registered. When the report's own renderer already ended, its link was removed by `Exited` and `link_pid` is `None`; the unit test documents this as "judged by the Playing rule, not as stale". The Playing rule (`link_pid == Some(pid)`) only guards `fleet.playing.insert` and the election. `status.after(&message, ...)` at line 453 runs for the message anyway. Since WR-11, `Status::after(Playing)` turns `Starting`, `Failed` and `RendererStopped` into `Playing`. If the `Playing` report of a renderer that played and then died is handled after its `Exited` (two threads feed the channel; the case WR-01 was about), the sequence is: `Exited` -> `restarting()` gives `Starting` (or `GiveUp` gives `RendererStopped`) -> late `Playing` -> status `Playing` with no renderer playing. With `GiveUp` the tray then says "playing <title>" for good, because nothing ever corrects it. The mirror case: a late `Failed` of a dead renderer overwrites `Starting` with a stale failure text.
**Fix:** Treat a message from a renderer without a matching link as stale for every status-changing message, not only for the playing set:
```rust
// after computing link_pid
if link_pid != Some(pid) {
    tracing::debug!(slot, pid, "ignoring a report of a renderer that is gone or was replaced");
    continue;
}
```
`Failed` must still be able to arrive from a renderer whose link is registered (it is: `register_writer` runs before the first event is sent), so this only drops reports that come after the `Exited`. Update `a_report_is_stale_only_when_another_process_holds_the_slot` accordingly.

### WR-16: A refused Hello makes the renderer exit with code 0, and the monitor stays without a wallpaper (reopens WR-05)

**Origin:** new; side effect of the WR-05 fix
**File:** `apps/sucrose/src/service.rs:1068-1097` (`accept_hello`), `:1476-1510` (`shell_restarted`), `:465-485` (stale exit); `apps/sucrose/src/render.rs:99-101`, `:352-377`; `apps/sucrose/src/supervisor.rs:352-371`
**Issue:** The service refuses a Hello by dropping the connection (`read_connection` returns). The renderer's `wait_for_assignment` maps `IpcError::Closed` to `Ok(None)` and `run` returns `ExitCode::SUCCESS`, which `classify_exit` calls `Clean` and `decide` answers with `Leave`: the slot is stopped for good. Before WR-05 a legitimate renderer was never refused; now `AlreadyConnected` can hit one:
1. `Supervisor::wait_for` removes the pid and the slot from `running` (line 367) before it queues `Exited` (line 370). The old connection stays in `connections` until the main thread handles `Exited`.
2. `Fleet::shell_restarted` (line 1486) decides from `supervisor.slots()` alone: a slot that is "not running" is started at once (line 1505), without looking at the registered link. A renderer that ends at the moment explorer restarts (a surface-lost exit is the expected case there) can therefore be replaced while its old link is still registered. The new renderer's Hello is refused with `AlreadyConnected`, it exits cleanly, and nothing restarts it (it is not in `shell.respawn`, so `decide_exit` gives `Leave`).
3. In the same window the stale-exit branch (`:478-485`) does `continue` before `fleet.playing.remove(&exited.slot)` and the election, so the old renderer's entry in `playing` survives and the replacement counts as showing its first frame (and may be elected master) before it has one.
The window is small (the Exited event has to be queued but not yet handled), but the result is a monitor with no wallpaper until the next shell restart or monitor change, with a log line that reads like a normal stop.
**Fix:** Make a refusal distinguishable from a normal close and make the stale path do the cleanup it skips:
```rust
// service: before the reply, on a refusal, tell the renderer instead of just dropping the connection
// (a ToRenderer::Refused or reuse Close), and in render.rs treat "closed before the assignment" as a failure:
Err(IpcError::Closed) => bail!("the service closed the connection before it assigned a wallpaper"),
// -> the renderer exits non-zero, the budget restarts it after the old link is gone.
```
and in the `Exited` handler remove the old renderer's `playing` entry (and re-elect when it was playing) before the `exit_is_stale` `continue`, only when the old link was the one removed. Alternatively `shell_restarted` can skip (or delay) a slot that still has a registered link.

### WR-17: A failed start loses its reason from the tooltip right after the exit (reopens IN-01, WR-11)

**Origin:** new; interaction of the IN-01 and WR-11 fixes
**File:** `apps/sucrose/src/service.rs:257-266` (`restarting`), `:524-529`, `:530-542`; `apps/sucrose/src/render.rs:595-610`; `docs/FEATURES.md` (Video row)
**Issue:** A file mpv cannot play makes the renderer report `Failed { reason }` and exit with code 4 (`StartFailed`, restarted within the crash budget). When `Exited` is handled, `Decision::Restart` calls `status.restarting(title)`, and since WR-11 `restarting` maps `Failed` to `Starting`. The reason the renderer reported lives for milliseconds; the tooltip reads "starting" while the restart runs, the restarted renderer fails again, and after three attempts `GiveUp` sets `RendererStopped` with only "the renderer stopped after 3 crashes in 60 s". mpv's reason ends up in the log only, while FEATURES.md says the failed start shows "mpv's reason in the tray tooltip". A deterministic failure (missing or corrupt file) is also restarted three times, each time loading libmpv, for nothing.
**Fix:** Keep the `Failed` status for an `ExitClass::StartFailed` exit (do not map it to `Starting`), or carry the last failure reason into `RendererStopped`. Consider `Decision::Leave` for a start failure that the renderer already explained, as for `EngineMissing`, if restarts cannot help. Correct the FEATURES.md sentence to what is shown.

## Info

### IN-07: Generality without instantiation: affected-monitor scope and `PauseKind` plumbing

**Origin:** carried from the 2026-11-23 review (ledger disposition: skipped)
**Category:** necessity
**File:** `apps/sucrose/src/fullscreen.rs:94-105`, `:129-140`, `:186-188`; `apps/sucrose/src/service.rs:194-195`, `:1285-1294`
**Issue:** Still holds on the current code (`fullscreen.rs` is unchanged since the last review). `ActionScope::AffectedMonitor` (with `Rule::acted`, the second arm of `plan_action`/`acts_on` and an `#[allow(dead_code)]`) is reachable only through `const ACTION_SCOPE = AllMonitors`; `PauseKind` is read from settings and passed to `enable_fullscreen` only to log "Heavy is handled like Light". It runs and is tested but changes no behavior in M1. The repository shows PERF-01 scheduled for Phase 4 (`.planning/REQUIREMENTS.md:267`) while the comments say Phase 3 persists the switch; whether a later phase consumes it is a roadmap judgment, hence Info. The fixer skipped it citing D-05, a maintainer scope decision.
**Fix:** Keep only if the settings phase consumes it; otherwise remove the unused scope branch and the `pause` parameter until needed. Also make the comments agree on the phase (3 vs 4).

### IN-19: The order guard no longer limits a failing restore (reopens WR-14)

**Origin:** new
**File:** `crates/sucrose-platform/src/os_windows/surface.rs:748-811`
**Issue:** The WR-14 fix counts a restore only after `SetWindowPos` succeeded. When the call keeps failing (Progman gone, access denied) the budget never fills, so the T-02-36 limit ("a shell that keeps reordering cannot keep the renderer busy") no longer applies: each shell event runs `find_child`, `order_text`, the failing call and a `warn!`. The loop has no feedback (a failed call produces no new shell event), so it is bounded by the shell's own event rate, not unbounded.
**Fix:** Count a failed attempt too (`record` in the `Err` arm), or count attempts and drop the success-only rule; the re-check timer already covers the "burst used the budget up" case.

### IN-20: A Quit queued while the surface attaches is not seen until attach ends (reopens WR-08)

**Origin:** new
**File:** `crates/sucrose-platform/src/os_windows/event_loop.rs:170-185`; `crates/sucrose-platform/src/os_windows/surface.rs:500-507`; `:86-92` of `event_loop.rs` (`WM_QUIT_REQUEST`)
**Issue:** `LoopProxy::quit` posts `WM_QUIT_REQUEST`, which the loop window procedure turns into a queued `LoopEvent::Quit` in `PENDING`, not into `WM_QUIT`. `wait_pumping` only stops for a real `WM_QUIT`, so a Close from the service (monitor unplugged, Quit from the tray) during the up to 10 s attach retry is dispatched but not acted on; the service waits its 2 s grace and kills the renderer. The log message "asked to quit while the surface was attaching" is therefore unreachable for the normal quit path.
**Fix:** Have `wait_pumping` (or `attach`) also return early when `PENDING` holds a `LoopEvent::Quit`, without consuming it.

### IN-21: A stuck or broken renderer link is only logged, and warns once per clock reading

**Origin:** new (consequence of the WR-13 design)
**File:** `apps/sucrose/src/service.rs:158-167`, `:957-978`, `:1143-1155`
**Issue:** `Offer::Failed` (queue full of non-skippable messages, or the writer thread ended after a write error) is logged at warn level and nothing else happens: the link stays registered, a stuck renderer keeps its monitor frozen without a restart or kill (before WR-13 it froze the service, so this is an improvement, not a regression), and with a dead writer `relay_clock` warns once per master reading per follower until `Exited` arrives. A full queue of clock readings also fails a `SetRect`/`Close` although the queued readings could have been dropped to make room.
**Fix:** On `Failed` for a non-clock message treat the renderer as lost (close its connection or ask the supervisor to end that process so the restart budget applies), and log the dead-writer case once per link.

## Carried findings resolved

- **IN-10** (recorded GPU regression, "first suspect is `deband=yes`/`profile=low-latency`"): no longer holds. `02-UAT.md` G-02-6 gives the root cause (the 93.7 % vs 16.5 % compared the Intel UHD with the RTX 3050); the service now writes the GPU preference, `xtask measure --row` refuses two different adapters, and `docs/MEASUREMENTS.md` shows the same-GPU result 16.4 % vs 16.1 % (R = 1.02), recorded in FEATURES with the options kept for parity. `config.rs` generates the same options as the C# app. Nothing is left to change; the ledger row can be closed as resolved by evidence.

---

_Reviewed: 2026-11-23_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
