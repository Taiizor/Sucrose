---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
reviewed: 2026-11-23T19:30:00Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - apps/sucrose/src/service.rs
  - crates/sucrose-platform/src/os_windows/surface.rs
  - docs/DECISIONS.md
  - docs/FEATURES.md
  - docs/MEASUREMENTS.md
findings:
  critical: 0
  warning: 0
  info: 7
  total: 7
  necessity: 0
status: issues_found
---

# Phase 2: Code Review Report (consolidated re-review since 6fe31044)

**Reviewed:** 2026-11-23
**Depth:** standard
**Files Reviewed:** 5
**Status:** issues_found

## Summary

Consolidated re-review: the diff `6fe31044..HEAD` (gap-closure plans 02-27, 02-28, 02-29) plus a re-verification, against the current code, of every finding that was open in `02-REVIEW-DISPOSITION.md` (WR-19, WR-20, IN-19..IN-23, IN-25..IN-28), IN-24 (text from review 5f78c99a) and IN-10 (text from review ae3b8c76). Nothing was built or run (Windows toolchain only); the reading is of the diff and of the current files, and of `render.rs`, `event_loop.rs`, `topology.rs` and the 02-26 summary where a claim had to be checked.

**Verdict on the three plans:** no new bug and no new finding. Plan 02-27 fixes WR-19 correctly: `Fleet::slot_assigned` (`service.rs:1386`) is `slots.monitor_of(slot).is_some() && !rule_closed.is_closed(slot)`; `monitor_of` reads only the live `monitors` map, so a retired (draining) or free slot is "not assigned" (`topology.rs:138-142`), and the fullscreen-closed case is covered by `rule_closed.is_closed`. The check sits in the pure `handle_report` (`:1096-1098`) and drops a `Playing` too, so `fleet.playing` and the master election cannot be touched by a removed monitor's renderer. The WR-18 path is kept for assigned slots (test at `:1960`). The `SlotState` struct replaces the growing parameter list, and its tests cover removed, connected-but-unassigned, assigned-and-ended and the `slot_assigned` truth table (`:1922-1992`). Plan 02-28 records the accepted G-02-7 limitation; the numbers in D-031 and in the FEATURES.md row (4 display changes, 8 of 8 Progman moves, episodes 1.73-2.11 s, an approximately 90 s UAT episode that did not recur) match the table in `02-26-SUMMARY.md`. Plan 02-29's MEASUREMENTS.md row and paragraph are internally consistent (299 vs 301 MB, 0.43 vs 0.40 % CPU, C# column empty with the LUID reason stated).

Six carried findings and IN-24/IN-10 are resolved (see the last section); the seven that remain are all Info, none was touched by the three plans, and none has a new consequence.

## Info

### IN-19: The order guard no longer limits a failing restore (reopens WR-14)

**Origin:** carried from review 22cb1022 (ledger: open; re-verified, unchanged)
**File:** `crates/sucrose-platform/src/os_windows/surface.rs:993-1014` (budget consulted), `:1033-1046` (`record` only in the `Ok` arm), `:1047` (`Err` arm only warns)
**Issue:** Only a restore that worked is recorded in the budget. If `SetWindowPos` keeps failing (Progman gone, access denied) the budget never fills, so the T-02-36 limit no longer applies: each shell event runs `find_child`, `order_text` (twice) and a `warn!`. No feedback loop exists (a failed call raises no new shell event), so the cost is bounded by the shell's own event rate.
**Fix:** Record a failed attempt too (call `record` in the `Err` arm), or count attempts instead of successes; the re-check timer already covers "the burst used the budget up".

### IN-20: A Quit queued while the surface attaches is not seen until attach ends (reopens WR-08)

**Origin:** carried from review 22cb1022 (ledger: open; re-verified, unchanged)
**File:** `crates/sucrose-platform/src/os_windows/event_loop.rs:98`, `:170-185` (`wait_pumping`), `:189-198` (`dispatch_waiting`), `:350-352` (`LoopProxy::quit`); `crates/sucrose-platform/src/os_windows/surface.rs:654-660`
**Issue:** `LoopProxy::quit` posts `WM_QUIT_REQUEST`, which `loop_proc` turns into a queued `LoopEvent::Quit` (`:98`), not into `WM_QUIT`. `dispatch_waiting` only stops for a real `WM_QUIT` (`:194`), so a Close from the service (monitor unplugged, Quit from the tray) during the up-to-10 s attach retry is dispatched into the pending queue and not acted on until the attach ends; the service waits its 2 s grace and kills the renderer. The warn "the message loop was asked to quit while the surface was attaching" (`surface.rs:655`) is unreachable on the normal quit path.
**Fix:** Let `wait_pumping` also return early when the pending queue holds a `LoopEvent::Quit` (without removing it), so `attach` returns its error and the run loop sees the Quit.

### IN-21: A stuck or broken renderer link is only logged, and warns once per clock reading

**Origin:** carried from review 22cb1022 (ledger: open; re-verified, unchanged)
**File:** `apps/sucrose/src/service.rs:162-169` (`Link::offer`), `:974-987` (`send_role`), `:1001-1020` (`relay_clock`), `:1260-1271` (`send_to`), `:1522-1528` (`elect`), `:777-779` (`close`)
**Issue:** `Offer::Failed` (queue full of non-skippable messages, or the writer thread ended after a write error) is logged at warn level and nothing else happens: the link stays registered, a stuck renderer keeps its monitor frozen without a restart or kill, and with a dead writer `relay_clock` warns once per master reading per follower until `Exited` arrives. `SetRect` goes through `send_to` for every renderer after each display change (02-25), so a stuck renderer silently misses its re-place. A full queue of clock readings also fails a `SetRect`/`Close` although queued readings could be dropped to make room.
**Fix:** On `Failed` for a non-clock message treat the renderer as lost (drop its connection or ask the supervisor to end that process so the restart budget applies) and log the dead-writer case once per link.

### IN-22: A refused Hello is still the normal outcome of the shell-restart gap and costs a crash strike (follow-up of WR-16)

**Origin:** carried from review 22cb1022 (ledger: open; re-verified, unchanged)
**File:** `apps/sucrose/src/service.rs:1609-1643` (`shell_restarted`, the `running.contains` test at `:1619`), `:1210` (`Refusal::AlreadyConnected`); `apps/sucrose/src/render.rs:388-395` (`judge_hello_reply`: a closed connection is an error, exit code 1)
**Issue:** A replacement started in the gap, while the old renderer's connection is still registered, is refused; it exits with code 1 (`Crashed(1)`), is logged as a crash, spends one of the three strikes (`MAX_CRASHES`) of a monitor whose budget `shell_restarted` had just reset, and the wallpaper is delayed by the 500 ms backoff plus a second start. Robustness cost only.
**Fix:** In `shell_restarted` treat a slot that is not running but still has a registered connection as "ending": remember it in the set `ShellRestart::begin` keeps (the respawn set) instead of spawning, so its pending `Exited` restarts it without a strike. Keep the exit-code-1 rule as the safety net.

### IN-23: `stopped_reason` can name a failure that belongs to another monitor or an earlier attempt, and FEATURES.md promises it unconditionally (follow-up of WR-17)

**Origin:** carried from review 22cb1022 (ledger: open; re-verified, unchanged)
**File:** `apps/sucrose/src/service.rs:283-288` (`stopped_reason`), `:585-587` (the `GiveUp` path), `:1114-1129` (`status_after_exit`); `docs/FEATURES.md:13` (Video row: "stays through the restarts and is named again when the budget is spent")
**Issue:** The status is one global value. `stopped_reason` appends the budget sentence to whatever `Status::Failed` is current, whichever slot reported it. With two monitors, if slot 0 failed to attach and slot 1's renderer then crashes three times, slot 1's tooltip reads "no wallpaper on <slot 1 monitor>: the desktop surface could not be attached (the renderer stopped after 3 crashes in 60 s)", blaming the wrong monitor; with the status `Playing` the reason is not named at all. Within one slot a final `Crashed` exit after earlier start failures attaches the kept failure text to a crash sentence. The FEATURES.md sentence is true only for one monitor whose failures were all start failures. (The WR-19 fix narrows who can set a `Failed`, but does not make the status per slot.)
**Fix:** Remember the failure per slot (`last_failure: HashMap<u8, String>` in `Fleet`, set where a `Failed` is accepted, cleared on `Playing`) and pass it to `status_after_exit` only when the ended renderer's class is `StartFailed`; or word the FEATURES.md sentence "on a single monitor".

### IN-26: The tests duplicate the budget-spending loop and rely on a magic bound

**Origin:** carried from review 22cb1022 (ledger: open; re-verified, unchanged)
**File:** `apps/sucrose/src/service.rs:2015-2025` and `:2036-2045` (`a_failed_start_whose_exit_is_handled_first_keeps_its_reason_through_the_restart`, `a_late_failed_does_not_replace_the_verdict_of_the_spent_budget`)
**Issue:** Both tests contain the same `for _ in 0..10 { decide_exit; status_after_exit; break on GiveUp }` loop with the literal 10, an arbitrary bound above `MAX_CRASHES`. If the cap changes, the first test's `expect("the budget runs out")` is the only signal and the second asserts on a status that never reached `RendererStopped`, with an unclear failure. The helper `exit_then_failed` (`:1846`) shows the intended shape but is not reused.
**Fix:** Extract `fn spend_budget(status: Status, budget: &mut RestartBudget) -> Status` that loops until `GiveUp` and panics with a clear message otherwise; use it in both tests.

### IN-27: A surface outside its desktop window is warned twice, and on every move

**Origin:** carried from review 22cb1022 (ledger: open; re-verified, unchanged)
**File:** `apps/sucrose/src/render.rs:222-227`; `crates/sucrose-platform/src/os_windows/surface.rs:687`, `:204-211` (`log_episode`)
**Issue:** `log_episode` warns once when an outside episode starts and logs once when it ends, with the same text ("the surface lies outside the desktop window"). The `Wake` handler in render.rs still warns with that text for every `PartlyHidden`/`Hidden` placement returned by `set_rect`. A move that starts an episode logs the warning twice (once from `set_rect`, once from the renderer), and since every renderer is sent a `SetRect` after every display change (02-25), a surface that stays outside warns on each of them, which defeats the "one warning per episode" contract of `outside_episode` that D-031 and FEATURES.md now state ("a warning at its start, an info line at its end").
**Fix:** In render.rs keep the "surface moved" line (it has the verdict) and drop the `PartlyHidden`/`Hidden` warn, since the episode logging in surface.rs covers it; keep the `Misplaced` warn, which the episode record deliberately ignores.

## Resolved since the last review

- **WR-19** (a late `Failed` of a renderer whose slot was released reaches the global status): resolved by 02-27. `service.rs:462` passes `fleet.slot_assigned(slot)` in a `SlotState`; `handle_report` (`:1096-1098`) treats `!holder.assigned` as replaced and returns `None`; `slot_assigned` is `:1386-1388`. Tests: `:1922` (removed monitor, `link_pid = None`, `Failed` gives `None`), `:1939` (connected but unassigned, `Failed` and `Playing`), `:1960` (assigned slot still counts, WR-18 kept), `:1976` (truth table incl. retire and release).
- **WR-20** (FEATURES.md does not record the G-02-7 decision): resolved by 02-28. `docs/FEATURES.md:30` now states the observed behavior (episode ended within 1.73-2.11 s in the spike, watch re-places, about 90 s UAT episode did not recur, cause not established) and "Accepted limitation (G-02-7, D-031)"; `docs/DECISIONS.md:163-168` adds D-031 (Accepted, with rejected alternative and consequences). The figures match `02-26-SUMMARY.md` (timeline table, rows 1-4).
- **IN-25** (FEATURES.md sentence about dropped reports): resolved by 02-27. `docs/FEATURES.md:13` now reads "a `Failed` that arrives after its renderer's exit still counts; a renderer's report is ignored when another renderer holds or runs the monitor's slot, when its monitor was removed or the fullscreen rule closed it, and a late `Failed` does not replace the status once the restart budget is spent; a late `Playing` of an ended renderer is always ignored", which matches `handle_report` (`service.rs:1096-1110`).
- **IN-28** (comments that drifted): resolved. `surface.rs:827-828` now says "it was checked on a real desktop (UAT G-02-7, D-031)"; `service.rs:1531-1533` (`apply_snapshot` doc) says every surface is re-placed after each applied snapshot, and `:1560` points `apply` at `replace_all` (which exists, `topology.rs:62`).
- **IN-24** (the WR-15 test re-implements the handler instead of testing it): resolved. The decision is the pure `handle_report` (`service.rs:1088-1110`) and `status_after_exit` (`:1114-1129`); tests call them directly (`exit_then_failed` `:1846`, `a_report_counts_only_while_its_renderer_holds_the_slot`, the WR-19 tests above); no closure copy of the handler remains.
- **IN-10** (the recorded GPU regression's first suspect is `deband=yes` / `profile=low-latency`): resolved by evidence. `docs/MEASUREMENTS.md` (first 2026-11-23 row and "reading the numbers") records the same-GPU measurement (RTX 3050 on both sides, GPU 16.4 % vs 16.1 %, R = 1.02, `--row` refuses mixed adapters), the earlier gap compared two different adapters, and the options are kept for parity (`docs/FEATURES.md:13` repeats the numbers).

---

_Reviewed: 2026-11-23_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
