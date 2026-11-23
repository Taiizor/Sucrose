---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
reviewed: 2026-11-23T14:30:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - apps/sucrose/src/render.rs
  - apps/sucrose/src/service.rs
  - docs/FEATURES.md
  - apps/sucrose/src/fullscreen.rs
  - crates/sucrose-platform/src/os_windows/surface.rs
  - crates/sucrose-platform/src/os_windows/event_loop.rs
findings:
  critical: 0
  warning: 1
  info: 7
  total: 8
  necessity: 1
status: issues_found
---

# Phase 2: Code Review Report (incremental: fix commits WR-15, WR-16, WR-17)

**Reviewed:** 2026-11-23
**Depth:** standard
**Files Reviewed:** 6 (the three files of the diff `d2a4fdfc..HEAD`, plus three files read only to re-verify carried findings)
**Status:** issues_found

## Summary

Reviewed `git diff d2a4fdfc..HEAD` for `apps/sucrose/src/render.rs`, `apps/sucrose/src/service.rs` and `docs/FEATURES.md` (commits 3a6224b8, b3d55ef4, 3f2417b9), reading each touched function whole with its callers (`read_connection`, `register_writer`, `accept_hello`, `shell_restarted`, `start_shutdown`, `Supervisor::wait_for`, `decide_exit`, `report_failure`, `publish_clock`). Nothing was built or run (Windows toolchain only, no GUI runs). The reference `necessity-review.md` was read; only IN-07 (carried) is a necessity finding.

Verdict per fix:
- **WR-15 (reports of a renderer without the slot's connection are dropped):** fully resolves the stale `Playing` case (the original WR-01/IN-16 scenario) and the stale `Failed` that overwrites a restart's `Starting`. But it is too broad for `Failed`: a renderer reports `Failed` and exits right after, so a legitimate `Failed` can be queued behind its own `Exited` and is now dropped (WR-18, which reopens WR-15 and weakens WR-17).
- **WR-16 (a closed connection before the assignment is exit code 1; the stale-exit path forgets the old `playing` entry):** sufficient without the `shell_restarted` change. I traced the gap: `Supervisor::wait_for` removes the slot from `running` before `Exited` is queued, `shell_restarted` starts the replacement, the replacement's Hello is refused (`AlreadyConnected`), it exits with 1, `Exited(old)` removes the old link, `Exited(new)` is a normal crash exit and `restart_after` starts a third renderer 500 ms later that connects. The monitor is no longer left dead, and the old `playing` entry no longer counts for the replacement (`forget_replaced_playing` runs only when the old link was the registered one, and a replacement cannot have reported `Playing` before that link was removed, so it cannot remove the replacement's own entry). No new race found in that path. What remains is a cost, not a defect (IN-22). Exit code 1 for a service that quits is harmless: shutdown sets `closing`, which answers a Hello with `Close`, and no restart happens while `quitting`.
- **WR-17 (failure kept in the status through restarts and named in `RendererStopped`):** works for the sequence it targets. It inherits WR-18 (the reason is lost when the `Failed` report is dropped) and has a misattribution when the status holds another monitor's failure (IN-23). Not applying `Decision::Leave` for an explained start failure is a policy choice and is accepted.
- **docs/FEATURES.md:** the Video row now matches the code, with the two caveats in IN-23.

Carried findings: IN-07, IN-19, IN-20 and IN-21 still hold on the current code (`fullscreen.rs`, `surface.rs` and `event_loop.rs` are unchanged by the diff; `service.rs` line numbers moved). IN-10 stays resolved (see the end).

## Warnings

### WR-18: A legitimate `Failed` queued behind its own renderer's `Exited` is dropped (reopens WR-15; the reason of WR-17 can be lost)

**Origin:** new; the WR-15 fix is incomplete for `Failed`
**File:** `apps/sucrose/src/service.rs:452-465` (`report_is_stale` at `:1050-1052`), `:486-497` (link removed in `Exited`); `apps/sucrose/src/render.rs:114-135` (`report_failure` then `return`), `:634-644`
**Issue:** `Failed` is the one report a renderer sends right before it ends: after the attach failure and the engine failure `run` returns at once after `report_failure`, and the clock thread asks the loop to quit after it. The report reaches the main loop through the reader thread of the connection (`read_connection` -> `notifier.send`), the exit through the waiter thread (`wait_for` -> sink). These are two senders into one channel, so `Exited` can be queued first (the same race the WR-15 comment describes for `Playing`). Handling `Exited` removes the renderer's link, so the late `Failed` finds `link_pid == None` and is now dropped with a debug line. Consequences:
- an engine failure (exit code 2, `Decision::Leave`, status unchanged): the tooltip stays "starting" with no renderer and no restart, for good (before WR-15 the late `Failed` still set the status);
- a start failure (exit code 4): `Status::Failed` is never set, so `restarting_after` has nothing to keep and `stopped_reason` has nothing to name; mpv's reason is only in the log, which is the loss WR-17 set out to remove.
The window is small (the renderer's process teardown normally takes longer than the reader thread's wake-up), which is why this is not a blocker, but the failure mode is a permanent wrong tooltip and the paths are exactly the ones that exit immediately after the send.
**Fix:** Drop a late report only when it can do harm. A late `Playing` (stale wallpaper claim) stays dropped. A `Failed` should be dropped only when a different renderer holds or runs the slot, not when nobody does:
```rust
let replaced = link_pid.is_some_and(|current| current != pid)
    || exit_is_stale(&supervisor.pids(), slot, pid); // another process runs for the slot
let late_playing = link_pid != Some(pid) && matches!(message, ToService::Playing);
if replaced || late_playing {
    tracing::debug!(slot, pid, "ignoring a report of a renderer that is gone or was replaced");
    continue;
}
```
A late `Failed` of the last renderer then still reaches `Status::after`; if `RendererStopped` was already set it would replace it, so keep `RendererStopped` as it is for that case. Add a test on the real decision, not on a closure (see IN-24).

## Info

### IN-07: Generality without instantiation: affected-monitor scope and `PauseKind` plumbing

**Origin:** carried from the previous review (ledger disposition: skipped)
**Category:** necessity
**File:** `apps/sucrose/src/fullscreen.rs:94-105`, `:129-140`, `:186-188`; `apps/sucrose/src/service.rs:196-198`, `:1325-1336`
**Issue:** Still holds (`fullscreen.rs` is unchanged by the diff). `ActionScope::AffectedMonitor` (`Rule::acted`, the second arm of `plan_action`/`acts_on`, an `#[allow(dead_code)]` at `fullscreen.rs:102` whose reason says "Phase 3 persists it") is reachable only through `const ACTION_SCOPE = AllMonitors`; `PauseKind` is read from settings and passed to `enable_fullscreen` only to log "Heavy is handled like Light". It is exercised only by tests and changes no behavior in M1. The comment at `service.rs:197` names Phase 3 for persisting the switch while `.planning/REQUIREMENTS.md` schedules the UI (PERF-01) later; whether a later phase consumes it is a roadmap judgment, hence Info. The fixer skipped it citing D-05, a maintainer scope decision.
**Fix:** Keep only if the settings phase consumes it; otherwise remove the unused scope branch and the `pause` parameter until needed, and make the comments agree on the phase.

### IN-19: The order guard no longer limits a failing restore (reopens WR-14)

**Origin:** carried from the previous review (ledger disposition: open)
**File:** `crates/sucrose-platform/src/os_windows/surface.rs:748-811`
**Issue:** Still holds. The budget is recorded only in the `Ok` arm of `SetWindowPos` (`:796-803`). When the call keeps failing (Progman gone, access denied) the budget never fills, so the T-02-36 limit ("a shell that keeps reordering cannot keep the renderer busy") no longer applies: each shell event runs `find_child`, `order_text`, the failing call and a `warn!`. There is no feedback loop (a failed call produces no new shell event), so it is bounded by the shell's own event rate.
**Fix:** Count a failed attempt too (`record` in the `Err` arm), or count attempts and drop the success-only rule; the re-check timer already covers the "burst used the budget up" case.

### IN-20: A Quit queued while the surface attaches is not seen until attach ends (reopens WR-08)

**Origin:** carried from the previous review (ledger disposition: open)
**File:** `crates/sucrose-platform/src/os_windows/event_loop.rs:170-185`, `:98` (`WM_QUIT_REQUEST`); `crates/sucrose-platform/src/os_windows/surface.rs:500-507`
**Issue:** Still holds. `LoopProxy::quit` posts `WM_QUIT_REQUEST`, which `loop_proc` turns into a queued `LoopEvent::Quit`, not into `WM_QUIT`. `wait_pumping` -> `dispatch_waiting` only stops for a real `WM_QUIT`, so a Close from the service (monitor unplugged, Quit from the tray) during the up to 10 s attach retry is dispatched but not acted on; the service waits its 2 s grace and kills the renderer. The log line "asked to quit while the surface was attaching" is unreachable on the normal quit path.
**Fix:** Have `wait_pumping` (or `attach`) also return early when the pending events hold a `LoopEvent::Quit`, without consuming it.

### IN-21: A stuck or broken renderer link is only logged, and warns once per clock reading

**Origin:** carried from the previous review (ledger disposition: open)
**File:** `apps/sucrose/src/service.rs:161-168` (`Link::offer`), `:959-974` (`send_role`), `:988-1007` (`relay_clock`), `:1183-1194` (`send_to`)
**Issue:** Still holds. `Offer::Failed` (queue full of non-skippable messages, or the writer thread ended after a write error) is logged at warn level and nothing else happens: the link stays registered, a stuck renderer keeps its monitor frozen without a restart or kill (before WR-13 it froze the service, so this is an improvement), and with a dead writer `relay_clock` warns once per master reading per follower until `Exited` arrives. A full queue of clock readings also fails a `SetRect`/`Close` although queued readings could have been dropped to make room.
**Fix:** On `Failed` for a non-clock message treat the renderer as lost (close its connection or ask the supervisor to end that process so the restart budget applies), and log the dead-writer case once per link.

### IN-22: A refused Hello is still the normal outcome of the shell-restart gap and costs a crash strike (follow-up of WR-16)

**Origin:** new; residue of WR-16, which the fixer deliberately left
**File:** `apps/sucrose/src/service.rs:1516-1533` (`shell_restarted`, the `!running.contains` test at `:1526`), `:1126-1135` (`AlreadyConnected`); `apps/sucrose/src/render.rs:352-377`
**Issue:** After the WR-16 fix the monitor recovers, but only through the crash path: the replacement started in the gap is refused, exits with code 1 (`Crashed(1)`, "killed" class), is logged as a crash, spends one of the three strikes (`MAX_CRASHES`) of a monitor whose budget `shell_restarted` had just reset, and the wallpaper is delayed by the 500 ms backoff plus a second libmpv-less start. The user sees nothing wrong, so this is a robustness cost only. Skipping a slot that still has a registered link would avoid it, but it is not free: the old exit must then restart the slot, which `decide_exit` does only for a `SurfaceLost` inside the shell window or for a slot in `respawn`.
**Fix:** In `shell_restarted` treat a slot that is not running but still has a registered link as "ending": add it to the set that `ShellRestart::begin` remembers (`respawn`) instead of spawning, so its pending `Exited` restarts it with the shortest pause and without a strike. Keep the exit-code-1 rule as the safety net.

### IN-23: `stopped_reason` can name a failure that belongs to another monitor or an earlier attempt, and FEATURES.md promises it unconditionally (follow-up of WR-17)

**Origin:** new; residue of WR-17
**File:** `apps/sucrose/src/service.rs:279-287` (`stopped_reason`), `:561-573`; `docs/FEATURES.md` (Video row)
**Issue:** The status is one global value, but a `GiveUp` is per slot. `stopped_reason` appends the budget sentence to whatever `Status::Failed` is current. With two monitors, if slot 0's renderer failed to attach and slot 1's renderer then crashes three times, slot 1's tooltip reads "no wallpaper on <slot 1 monitor>: the desktop surface could not be attached (the renderer stopped after 3 crashes in 60 s)", blaming the wrong monitor; with the status `Playing` (slot 0 recovered first) the reason is not named at all. The same happens within one slot when the last exit that exhausts the budget is a crash (`Crashed`) after earlier start failures: the kept failure text is attached to a crash sentence. The FEATURES.md sentence ("stays through the restarts and is named again when the budget is spent") is true only for a single monitor whose failures all were start failures.
**Fix:** Remember the failure per slot (for example `last_failure: HashMap<u8, String>` in `Fleet`, set where `Failed` is accepted and cleared on `Playing`) and use it in `stopped_reason` only when the ended renderer's class is `StartFailed`; or word the FEATURES.md sentence as "on a single monitor".

### IN-24: The WR-15 test re-implements the handler instead of testing it

**Origin:** new
**File:** `apps/sucrose/src/service.rs:1731-1744` (`a_late_report_of_an_ended_renderer_cannot_rewrite_the_restart_status`), `:1746-1763`
**Issue:** The test builds a closure `(!report_is_stale(link_pid, 7)).then(|| starting().after(..))` that copies the loop's logic, so it passes whatever the real `Event::Renderer` arm does (it cannot catch WR-18, nor a regression that moves the check). `forget_replaced_playing` is a one-line `&&` and its test checks the same boolean. The real decision sits in an inline closure of `run` that no test reaches.
**Fix:** Move the arm's decision (stale or not, playing insert, status change) into a pure function that takes `link_pid`, the pids running for the slot, the message and the status, and test that function with the ordering cases (Exited first, replacement connected, replacement running).

## Carried findings resolved

- **IN-10** (recorded GPU regression, first suspect `deband=yes`/`profile=low-latency`): resolved by evidence, not carried. `docs/MEASUREMENTS.md` and the FEATURES.md Video row record the same-GPU measurement (RTX 3050, GPU 16.4 % vs 16.1 %, R = 1.02); the earlier gap compared two different adapters. The row can be closed.

---

_Reviewed: 2026-11-23_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
