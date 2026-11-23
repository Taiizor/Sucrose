---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
reviewed: 2026-11-23T15:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - apps/sucrose/src/service.rs
  - docs/FEATURES.md
  - apps/sucrose/src/fullscreen.rs
  - crates/sucrose-platform/src/os_windows/surface.rs
  - crates/sucrose-platform/src/os_windows/event_loop.rs
  - apps/sucrose/src/supervisor.rs
findings:
  critical: 0
  warning: 1
  info: 8
  total: 9
  necessity: 1
status: issues_found
---

# Phase 2: Code Review Report (incremental: fix commit WR-18)

**Reviewed:** 2026-11-23
**Depth:** standard
**Files Reviewed:** 6 (the two files of the diff `5f78c99a..HEAD`: `apps/sucrose/src/service.rs` and `docs/FEATURES.md`; plus `fullscreen.rs`, `surface.rs`, `event_loop.rs` read only to re-verify carried findings, and `supervisor.rs` read for the exit and restart contract)

## Summary

Reviewed the diff of commit a436d88d (WR-18): the new pure functions `handle_report` and `status_after_exit`, the rewritten `Event::Renderer` and `Event::Exited` arms of `run()`, the replaced tests and the FEATURES.md Video row. Nothing was built or run (Windows toolchain only, no GUI runs); the reasoning below is from reading the code and the supervisor/IPC paths (`Supervisor::wait_for`, `pids()`, `decide_exit`, `read_connection`, `report_failure` call sites in `render.rs`).

**WR-18 verdict: resolved for the cases it targets.**
- Exit code 2 (`Decision::Leave`) with the Exited handled first: the link is removed, no process runs for the slot, so `replaced` is false, `late` is true, the message is `Failed`, so it is not dropped, and `kept` is false unless the status is already `RendererStopped`. `Status::after` sets `Failed`. Correct.
- Exit code 4 (`Decision::Restart`) with the Exited handled first: the restart is only scheduled (`restart_after` starts the process after the delay), so `pids()` holds nothing for the slot and the late `Failed` reaches the status. `restarting_after` then keeps it through further start failures, and `stopped_reason` names it in `RendererStopped`. Correct; the budget needs three strikes (`MAX_CRASHES`), so the final `GiveUp` always sees a `Failed` kept from an earlier attempt, and the `kept` rule (a late `Failed` does not replace `RendererStopped`) cannot lose the reason in the normal sequence.
- A late `Playing` of an ended renderer is still dropped (`late && Playing`), as WR-15 requires; `first_frame` can only be true for a report whose renderer holds the link, so no stale `playing` entry or election is introduced.
- A `Failed` of a replaced renderer is dropped whether the replacement is connected (`link_pid` differs) or only started (`exit_is_stale` over `supervisor.pids()`).

**Handler side effects:** every side effect of the old inline arms is still applied. In the report arm: the `playing` insert and election (`first_frame`, guarded by `!quitting`), the status assignment, the failure/info log and the tray tooltip. In the exit arm: `fleet.stopped` insert for `Leave` and `GiveUp`, remove for `Restart`, `supervisor.restart_after`, the `GiveUp` warn log, the status assignment and the tooltip. `Decision` is `Copy`, so using `decision` after `status_after_exit` is sound. `monitor_of` is still evaluated after `slots.release`, as before.

One regression-shaped gap remains (WR-19), and FEATURES.md overstates the drop rule (IN-25).

Carried findings: IN-07, IN-19, IN-20, IN-21, IN-22 and IN-23 still hold on the current code. IN-24 and IN-10 are resolved (see the end).

## Warnings

### WR-19: A late `Failed` of a renderer whose slot was released (monitor unplugged) now reaches the global status (introduced by the WR-18 fix)

**Origin:** new; side effect of the WR-18 fix (reopens WR-15 for this case)
**File:** `apps/sucrose/src/service.rs:452-472` (call of `handle_report`), `:1049-1090` (`handle_report`), `:536-544` (the release path of `Exited`)
**Issue:** `handle_report` keeps a `Failed` when "nobody holds or runs the slot". That test is only about the connection and the supervisor, not about whether the slot still belongs to a monitor. When a monitor is unplugged the service sends `Close`; the renderer ends, the `Exited` arm takes the `fleet.slots.release(..)` branch and `continue`s (no status change). A `Failed` queued behind that exit (for example the renderer's start failure racing with the unplug, the same two-thread ordering WR-18 describes) now finds no link and no running process, is kept, and sets the single global status to `Failed` ("no wallpaper: ...") while the remaining monitors play; before the fix it was dropped. The same holds for a renderer closed by the fullscreen rule (`rule_closed.note_exit` branch) and for a slot whose monitor was reassigned. The window is narrow, but the result is a wrong tooltip that stays until some renderer reports `Playing` again, and nothing in the tests covers a released slot.
**Fix:** Give `handle_report` the knowledge of whether the slot is still assigned to a monitor and drop the `Failed` when it is not, for example by passing `slot_assigned: bool` (`fleet.slots.monitor_of(slot).is_some()`, if `monitor_of` returns `None` after `release`; otherwise add the check to `Slots`):
```rust
let replaced = link_pid.is_some_and(|current| current != pid)
    || exit_is_stale(running, slot, pid)
    || !slot_assigned;
```
and add a test with `slot_assigned = false` and `link_pid = None`.

## Info

### IN-07: Generality without instantiation: affected-monitor scope and `PauseKind` plumbing

**Origin:** carried from the previous review (ledger disposition: skipped)
**Category:** necessity
**File:** `apps/sucrose/src/fullscreen.rs:94-105`, `:129-140`; `apps/sucrose/src/service.rs:196-198`, `:1206`, `:1381-1390`
**Issue:** Still holds. `ActionScope::AffectedMonitor` is constructed only in tests (`fullscreen.rs:393-478`); production uses `const ACTION_SCOPE = AllMonitors` (`service.rs:198`). `PauseKind` is read from settings and passed to `enable_fullscreen` only to log that Heavy is handled like Light (`service.rs:1386`). It changes no behavior in M1. The comment at `service.rs:197` says persisting it is Phase 3 while the UI requirement (PERF-01) is scheduled later; whether a later phase consumes it is a roadmap judgment, hence Info. The fixer skipped it citing D-05, a maintainer scope decision.
**Fix:** Keep only if the settings phase consumes it; otherwise remove the unused scope branch and the `pause` parameter until needed, and make the comments agree on the phase.

### IN-19: The order guard no longer limits a failing restore (reopens WR-14)

**Origin:** carried from the previous review (ledger disposition: open)
**File:** `crates/sucrose-platform/src/os_windows/surface.rs:748-811`
**Issue:** Still holds. The budget is recorded only in the `Ok` arm of `SetWindowPos` (`:796-803`). When the call keeps failing (Progman gone, access denied) the budget never fills, so the T-02-36 limit ("a shell that keeps reordering cannot keep the renderer busy") no longer applies: each shell event runs `find_child`, `order_text`, the failing call and a `warn!` (`:807`). There is no feedback loop (a failed call produces no new shell event), so it is bounded by the shell's own event rate.
**Fix:** Count a failed attempt too (`record` in the `Err` arm), or count attempts and drop the success-only rule; the re-check timer already covers the "burst used the budget up" case.

### IN-20: A Quit queued while the surface attaches is not seen until attach ends (reopens WR-08)

**Origin:** carried from the previous review (ledger disposition: open)
**File:** `crates/sucrose-platform/src/os_windows/event_loop.rs:98`, `:167-185`, `:351`; `crates/sucrose-platform/src/os_windows/surface.rs:495-507`
**Issue:** Still holds. `LoopProxy::quit` posts `WM_QUIT_REQUEST` (`:351`), which `loop_proc` turns into a queued `LoopEvent::Quit` (`:98`), not into `WM_QUIT`. `wait_pumping` -> `dispatch_waiting` only stops for a real `WM_QUIT` (`:194`), so a Close from the service (monitor unplugged, Quit from the tray) during the up to 10 s attach retry is dispatched but not acted on; the service waits its 2 s grace and kills the renderer. The warn line "asked to quit while the surface was attaching" (`surface.rs:500`) is unreachable on the normal quit path.
**Fix:** Have `wait_pumping` (or `attach`) also return early when the pending events hold a `LoopEvent::Quit`, without consuming it.

### IN-21: A stuck or broken renderer link is only logged, and warns once per clock reading

**Origin:** carried from the previous review (ledger disposition: open)
**File:** `apps/sucrose/src/service.rs:161-168` (`Link::offer`), `:967-981` (`send_role`), `:994-1014` (`relay_clock`), `:1239` (`send_to`)
**Issue:** Still holds (code unchanged by the diff). `Offer::Failed` (queue full of non-skippable messages, or the writer thread ended after a write error) is logged at warn level and nothing else happens: the link stays registered, a stuck renderer keeps its monitor frozen without a restart or kill, and with a dead writer `relay_clock` warns once per master reading per follower until `Exited` arrives. A full queue of clock readings also fails a `SetRect`/`Close` although queued readings could have been dropped to make room.
**Fix:** On `Failed` for a non-clock message treat the renderer as lost (close its connection or ask the supervisor to end that process so the restart budget applies), and log the dead-writer case once per link.

### IN-22: A refused Hello is still the normal outcome of the shell-restart gap and costs a crash strike (follow-up of WR-16)

**Origin:** carried from the previous review (ledger disposition: open)
**File:** `apps/sucrose/src/service.rs:1572-1590` (`shell_restarted`, the `!running.contains` test at `:1582`), `:1190` (`AlreadyConnected`); `apps/sucrose/src/render.rs:352-377`
**Issue:** Still holds (code unchanged by the diff). The replacement started in the gap is refused, exits with code 1 (`Crashed(1)`), is logged as a crash, spends one of the three strikes (`MAX_CRASHES`) of a monitor whose budget `shell_restarted` had just reset, and the wallpaper is delayed by the 500 ms backoff plus a second start. A robustness cost only.
**Fix:** In `shell_restarted` treat a slot that is not running but still has a registered link as "ending": add it to the set `ShellRestart::begin` remembers (`respawn`) instead of spawning, so its pending `Exited` restarts it with the shortest pause and without a strike. Keep the exit-code-1 rule as the safety net.

### IN-23: `stopped_reason` can name a failure that belongs to another monitor or an earlier attempt, and FEATURES.md promises it unconditionally (follow-up of WR-17)

**Origin:** carried from the previous review (ledger disposition: open)
**File:** `apps/sucrose/src/service.rs:279-287` (`stopped_reason`), `:567-580` (`GiveUp` through `status_after_exit` at `:1095-1108`); `docs/FEATURES.md` (Video row)
**Issue:** Still holds, and the WR-18 fix makes it slightly more reachable: a late `Failed` now also reaches the one global status. `stopped_reason` appends the budget sentence to whatever `Status::Failed` is current, whichever slot reported it. With two monitors, if slot 0's renderer failed to attach and slot 1's renderer then crashes three times, slot 1's tooltip reads "no wallpaper on <slot 1 monitor>: the desktop surface could not be attached (the renderer stopped after 3 crashes in 60 s)", blaming the wrong monitor; with the status `Playing` the reason is not named at all. Within one slot, a final `Crashed` exit after earlier start failures attaches the kept failure text to a crash sentence. The FEATURES.md sentence ("stays through the restarts and is named again when the budget is spent") is true only for a single monitor whose failures all were start failures. `status_after_exit` is now a pure function, so the fix is easy to test.
**Fix:** Remember the failure per slot (for example `last_failure: HashMap<u8, String>` in `Fleet`, set where `Failed` is accepted and cleared on `Playing`) and pass it to `status_after_exit` instead of reading it from the global status, using it only when the ended renderer's class is `StartFailed`; or word the FEATURES.md sentence as "on a single monitor".

### IN-25: The FEATURES.md sentence about dropped reports is wrong in two ways (follow-up of WR-18)

**Origin:** new
**File:** `docs/FEATURES.md:13` (Video row, the parenthesis "a report is dropped only when another renderer holds or runs the monitor's slot")
**Issue:** The new text says a report is dropped *only* when another renderer holds or runs the slot. The code (`handle_report`, `service.rs:1049-1090`) also drops (a) a late `Playing` of a renderer that ended with nobody replacing it (WR-15, by design) and (b) a late `Failed` while the status is `RendererStopped` (a spent budget keeps its verdict). It also depends on the report being a `Failed`, which the sentence does not say. FEATURES.md is the parity record, so the behavior statement should be exact.
**Fix:** Reword the parenthesis, for example: "(a `Failed` that arrives after its renderer's exit still counts; it is dropped when another renderer holds or runs the monitor's slot, or when the status already says the restart budget is spent; a late `Playing` of an ended renderer is always dropped)". If WR-19 is fixed, add "or when the monitor was removed".

### IN-26: The new tests duplicate the budget-spending loop and rely on a magic bound

**Origin:** new
**File:** `apps/sucrose/src/service.rs:1895-1925` (`a_failed_start_whose_exit_is_handled_first_keeps_its_reason_through_the_restart`), `:1927-1955` (`a_late_failed_does_not_replace_the_verdict_of_the_spent_budget`)
**Issue:** Both tests contain the same `for _ in 0..10 { decide_exit; status_after_exit; break on GiveUp }` loop with the literal 10 (an arbitrary bound above `MAX_CRASHES`; if the cap changes the first test's `expect("the budget runs out")` is the only signal and the second would assert on a status that never reached `RendererStopped` with an unclear message). The helper `exit_then_failed` shows the intended shape but is not reused. Test reliability, not behavior.
**Fix:** Extract a helper `spend_budget(status, budget) -> Status` that loops until `GiveUp` with an `unreachable!`-style message, and use it in both tests.

## Carried findings resolved

- **IN-24** (the WR-15 test re-implements the handler instead of testing it): resolved by the WR-18 fix. The closure-copy test is gone; the decision of the `Event::Renderer` arm is the pure `handle_report` (`service.rs:1049-1090`) and the status part of the `Exited` arm is `status_after_exit` (`:1095-1108`). The new tests call the real functions with the ordering cases (exit first with codes 2 and 4, replacement connected, replacement running, other slot, spent budget). What stays outside unit tests is the glue (the `playing` insert, election and tooltip), which is trivial.
- **IN-10** (recorded GPU regression, first suspect `deband=yes`/`profile=low-latency`): resolved by evidence; `docs/MEASUREMENTS.md` and the FEATURES.md Video row record the same-GPU measurement (RTX 3050, GPU 16.4 % vs 16.1 %, R = 1.02).

---

_Reviewed: 2026-11-23_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
