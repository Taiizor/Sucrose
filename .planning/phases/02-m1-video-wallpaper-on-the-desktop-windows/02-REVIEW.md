---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
reviewed: 2026-11-23T18:00:00Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - apps/sucrose/src/render.rs
  - apps/sucrose/src/service.rs
  - apps/sucrose/src/topology.rs
  - crates/sucrose-engine-mpv/src/config.rs
  - crates/sucrose-platform/src/geometry.rs
  - crates/sucrose-platform/src/lib.rs
  - crates/sucrose-platform/src/os_windows/event_loop.rs
  - crates/sucrose-platform/src/os_windows/surface.rs
  - crates/sucrose-platform/src/unsupported.rs
  - docs/FEATURES.md
  - docs/MEASUREMENTS.md
findings:
  critical: 0
  warning: 2
  info: 9
  total: 11
  necessity: 0
status: issues_found
---

# Phase 2: Code Review Report (consolidated re-review since 22cb1022)

**Reviewed:** 2026-11-23
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found

## Summary

Consolidated re-review: the diff `22cb1022..HEAD` (G-02-7 plans 02-23 placement check, 02-24 location watch, 02-25 re-place-all; 02-26 changed no source) plus a re-verification of the 10 open ledger findings. Nothing was built or run (Windows toolchain only); the reading is of the diff and of the current files.

**02-23 / 02-24 / 02-25 verdict: the code is sound; no critical finding and no new bug in the placement logic.** What was traced:
- `placement_verdict` (geometry.rs): `Misplaced` wins, then `covers` gives `Shown`, then `intersection` gives `PartlyHidden` or `Hidden`. Right and bottom are exclusive in both `covers` and `intersection`, so touching edges are `Hidden`; the layout C replay is a unit test. A zero-size target is `Hidden` in `intersection` but `Shown` in `covers` when inside the parent; harmless (never produced by a real monitor).
- `place_checked` / `read_placement` (surface.rs): the move is unchanged from `place`; the read-back uses the same parent origin, a failed read gives `Err` and the attach path treats it as "verdict unknown" and never refuses the attach. `set_rect` propagates the new target to the watch through `note_placement` with `try_borrow_mut`, never holding the registry while a window moves (same shape as `GUARDS`).
- Location watch: the hook is filtered to explorer's process and the thread owning the root, `OBJID_WINDOW` only, `window == parent || window == root`, and the parent's client rectangle must differ from the recorded one. Re-placing our own surface produces no event for the hook (own process skipped), so there is no feedback loop. `Drop` removes the watch before destroying the window; a failed `place_checked` leaves the record stale so the next event retries. A fake hook handle in the tests is only ever used through the thread-local registry.
- `replace_all` / `apply_snapshot` (topology.rs, service.rs): closes, SetRects, spawns in id order, shared walk with `diff`; the initial start still uses `diff(&[], ...)` and spawns only. `move_to` updates the stored assignment before sending, so a renderer that connects later gets the new rectangle; a slot without a connection returns `told = false` as before. The election now runs only when `diff` is non-empty, which is right because an unchanged-rect snapshot cannot change the candidates; a primary switch always changes at least one rectangle (the primary sits at the origin), so it still elects. `self.monitors` is now refreshed also for an unchanged snapshot, which is more correct than the old early return.

Two findings are new: a doc row that did not follow the maintainer's G-02-7 decision (WR-20) and a duplicated warning (IN-27). The carried findings WR-19, IN-19 to IN-23, IN-25 and IN-26 still hold on the current code (ids kept); IN-24 and IN-10 are resolved (end of the report). IN-07 (ledger: skipped) is in `fullscreen.rs`/`service.rs` scope that this run did not re-open and is not repeated here.

## Warnings

### WR-19: A late `Failed` of a renderer whose slot was released (monitor unplugged) reaches the global status (introduced by the WR-18 fix)

**Origin:** carried (ledger: open)
**File:** `apps/sucrose/src/service.rs:453-473` (call of `handle_report`), `:1069-1090` (`handle_report`), `:537-545` (the release path of `Exited`)
**Issue:** Still holds; the code is unchanged by 02-23..02-25. `handle_report` decides "replaced" only from the connection (`link_pid`) and the supervisor's running set (`exit_is_stale`). It does not know whether the slot still belongs to a monitor. When a monitor is unplugged the service sends `Close`, the renderer ends, and the `Exited` arm takes the `slots.release` branch and `continue`s without touching the status (`:537-545`). A `Failed` queued behind that exit (a start failure racing with the unplug, the two-thread ordering of WR-18) finds no link and no running process, is kept (`:1085`, `late` and not `RendererStopped`), and `Status::after` sets the single global status to `Failed` ("no wallpaper: ...") while the other monitors play. Before the WR-18 fix it was dropped. The same holds for a renderer closed by the fullscreen rule (`rule_closed.note_exit` branch, `:546`). The tooltip stays wrong until some renderer reports `Playing`. No test covers a released slot.
**Fix:** Pass whether the slot is still assigned and drop the report when it is not:
```rust
// at the call site
let assigned = fleet.slots.active_slots().contains(&slot);
// in handle_report
let replaced = link_pid.is_some_and(|current| current != pid)
    || exit_is_stale(running, slot, pid)
    || !slot_assigned;
```
Add a test with `slot_assigned = false`, `link_pid = None`, `Failed` (expect `None`).

### WR-20: FEATURES.md does not record the maintainer's G-02-7 decision and still states a cause that the 02-26 spike did not confirm

**Origin:** new (02-24, 02-26)
**File:** `docs/FEATURES.md:30` (row "Monitor changes, shell restart recovery (CrashExplorer)", last sentence)
**Issue:** The row ends with "In UAT (G-02-7) explorer most likely left Progman at an earlier layout's rectangle after a display change, so a monitor lay outside it and showed the static wallpaper." Plan 02-26 ran the spike with the instrumented build: 4 display changes, 8 of 8 Progman moves caught by the watch, every outside episode ended within 2.11 s, classified (a) "explorer keeps up", and the maintainer answered "wait": G-02-7 is an accepted limitation, not a closed gap (02-26-SUMMARY, `key-decisions`). 02-26 made no commit, so none of this reached the parity record. A reader of FEATURES.md still sees a hypothesis ("most likely") as the cause and no statement of the behavior the user can see: after a display change a monitor can show the static wallpaper for the time explorer needs to move Progman (measured up to about 2 s, not bounded by the app), with the watch re-placing the surface as soon as it moves. CLAUDE.md non-negotiable 1 says behavior changes and gaps are recorded in FEATURES.md in the same change; `.claude/rules/docs.md` says keep docs true.
**Fix:** Replace the last sentence by the observed behavior and the decision, for example: "After a display change explorer moves Progman by itself; until it has, a monitor can show the static wallpaper (UAT G-02-7; in the 02-26 spike on the maintainer's machine every such episode ended within 2.11 s). The location watch re-places the surface as soon as Progman moves, and the renderer logs the episode (warning at its start, info at its end). Accepted limitation (maintainer, 02-26), not bounded by the app." Add the decision to `docs/DECISIONS.md` if the project treats accepted gaps as decisions.

## Info

### IN-19: The order guard no longer limits a failing restore (reopens WR-14)

**Origin:** carried (ledger: open)
**File:** `crates/sucrose-platform/src/os_windows/surface.rs:1033-1049` (`restore_order`, the budget `record` only in the `Ok` arm at `:1034-1041`), `:994-1002`
**Issue:** Still holds. Only a restore that worked is recorded. If `SetWindowPos` keeps failing (Progman gone, access denied) the budget never fills, so the T-02-36 limit no longer applies: each shell event runs `find_child`, `order_text` (twice, `before` and the warn) and a `warn!` (`:1048`). No feedback loop exists (a failed call raises no new shell event), so the cost is bounded by the shell's own event rate.
**Fix:** Record a failed attempt too (call `record` in the `Err` arm), or count attempts instead of successes; the re-check timer already covers "the burst used the budget up".

### IN-20: A Quit queued while the surface attaches is not seen until attach ends (reopens WR-08)

**Origin:** carried (ledger: open)
**File:** `crates/sucrose-platform/src/os_windows/event_loop.rs:98`, `:170-185` (`wait_pumping`), `:189-197` (`dispatch_waiting`), `:349-352` (`LoopProxy::quit`); `crates/sucrose-platform/src/os_windows/surface.rs:654-660`
**Issue:** Still holds. `LoopProxy::quit` posts `WM_QUIT_REQUEST`, which `loop_proc` turns into a queued `LoopEvent::Quit` (`:98`), not into `WM_QUIT`. `dispatch_waiting` only stops for a real `WM_QUIT` (`:194`), so a Close from the service (monitor unplugged, Quit from the tray) during the up-to-10 s attach retry is dispatched into `PENDING` and not acted on until the attach ends; the service waits its 2 s grace and kills the renderer. The warn "asked to quit while the surface was attaching" (`surface.rs:655`) is unreachable on the normal quit path.
**Fix:** Let `wait_pumping` also return early when `PENDING` holds a `LoopEvent::Quit` (without removing it), so `attach` returns its error and the run loop sees the Quit.

### IN-21: A stuck or broken renderer link is only logged, and warns once per clock reading

**Origin:** carried (ledger: open)
**File:** `apps/sucrose/src/service.rs:162-169` (`Link::offer`), `:968-981` (`send_role`), `:995-1014` (`relay_clock`), `:1240-1250` (`send_to`)
**Issue:** Still holds. `Offer::Failed` (queue full of non-skippable messages, or the writer thread ended after a write error) is logged at warn level and nothing else happens: the link stays registered, a stuck renderer keeps its monitor frozen without a restart or kill, and with a dead writer `relay_clock` warns once per master reading per follower until `Exited` arrives. With 02-25 `SetRect` goes through `send_to` for every renderer after each display change, so a stuck renderer now also silently misses its re-place. A full queue of clock readings also fails a `SetRect`/`Close` although queued readings could be dropped to make room.
**Fix:** On `Failed` for a non-clock message treat the renderer as lost (drop its connection or ask the supervisor to end that process so the restart budget applies) and log the dead-writer case once per link.

### IN-22: A refused Hello is still the normal outcome of the shell-restart gap and costs a crash strike (follow-up of WR-16)

**Origin:** carried (ledger: open)
**File:** `apps/sucrose/src/service.rs:1582-1616` (`shell_restarted`, the `running.contains` test at `:1592`), `:1191` (`Refusal::AlreadyConnected`); `apps/sucrose/src/render.rs:385-395` (`judge_hello_reply`: a closed connection is an error, exit code 1)
**Issue:** Still holds (code unchanged by the diff). A replacement started in the gap, while the old renderer's connection is still registered, is refused; it exits with code 1 (`Crashed(1)`), is logged as a crash, spends one of the three strikes (`MAX_CRASHES`) of a monitor whose budget `shell_restarted` had just reset, and the wallpaper is delayed by the 500 ms backoff plus a second start. Robustness cost only.
**Fix:** In `shell_restarted` treat a slot that is not running but still has a registered connection as "ending": remember it in the set `ShellRestart::begin` keeps (the respawn set) instead of spawning, so its pending `Exited` restarts it without a strike. Keep the exit-code-1 rule as the safety net.

### IN-23: `stopped_reason` can name a failure that belongs to another monitor or an earlier attempt, and FEATURES.md promises it unconditionally (follow-up of WR-17)

**Origin:** carried (ledger: open)
**File:** `apps/sucrose/src/service.rs:283-288` (`stopped_reason`), `:572-581` (the `GiveUp` path), `:1094-1109` (`status_after_exit`); `docs/FEATURES.md:13` (Video row: "stays through the restarts and is named again when the budget is spent")
**Issue:** Still holds. The status is one global value. `stopped_reason` appends the budget sentence to whatever `Status::Failed` is current, whichever slot reported it. With two monitors, if slot 0 failed to attach and slot 1's renderer then crashes three times, slot 1's tooltip reads "no wallpaper on <slot 1 monitor>: the desktop surface could not be attached (the renderer stopped after 3 crashes in 60 s)", blaming the wrong monitor; with the status `Playing` the reason is not named at all. Within one slot a final `Crashed` exit after earlier start failures attaches the kept failure text to a crash sentence. The FEATURES.md sentence is true only for one monitor whose failures were all start failures.
**Fix:** Remember the failure per slot (`last_failure: HashMap<u8, String>` in `Fleet`, set where a `Failed` is accepted, cleared on `Playing`) and pass it to `status_after_exit` only when the ended renderer's class is `StartFailed`; or word the FEATURES.md sentence "on a single monitor".

### IN-25: The FEATURES.md sentence about dropped reports is wrong in two ways (follow-up of WR-18)

**Origin:** carried (ledger: open)
**File:** `docs/FEATURES.md:13` (Video row, "a report is dropped only when another renderer holds or runs the monitor's slot")
**Issue:** Still holds. The code (`handle_report`, `service.rs:1069-1090`) also drops (a) a late `Playing` of a renderer that ended with nobody replacing it (WR-15, by design) and (b) nothing-changes for a late `Failed` while the status is `RendererStopped` (the report is accepted but its status effect is discarded, `:1085`). The sentence also does not say it is about a `Failed`. After WR-19 is fixed it must also name a removed monitor. FEATURES.md is the parity record, so the statement should be exact.
**Fix:** Reword, for example: "(a `Failed` that arrives after its renderer's exit still counts; it is dropped when another renderer holds or runs the monitor's slot, and it does not replace the status once the restart budget is spent; a late `Playing` of an ended renderer is always dropped)".

### IN-26: The new tests duplicate the budget-spending loop and rely on a magic bound

**Origin:** carried (ledger: open)
**File:** `apps/sucrose/src/service.rs:1899-1909` and `:1920-1929` (`a_failed_start_whose_exit_is_handled_first_keeps_its_reason_through_the_restart`, `a_late_failed_does_not_replace_the_verdict_of_the_spent_budget`)
**Issue:** Still holds. Both tests contain the same `for _ in 0..10 { decide_exit; status_after_exit; break on GiveUp }` loop with the literal 10, an arbitrary bound above `MAX_CRASHES`. If the cap changes, the first test's `expect("the budget runs out")` is the only signal and the second asserts on a status that never reached `RendererStopped`, with an unclear failure. The helper `exit_then_failed` (`:1810`) shows the intended shape but is not reused.
**Fix:** Extract `fn spend_budget(status: Status, budget: &mut RestartBudget) -> Status` that loops until `GiveUp` and panics with a clear message otherwise; use it in both tests.

### IN-27: A surface outside its desktop window is warned twice, and on every move

**Origin:** new (02-23, 02-24)
**File:** `apps/sucrose/src/render.rs:222-227`; `crates/sucrose-platform/src/os_windows/surface.rs:687`, `:1099-1106` (`log_episode`)
**Issue:** 02-24 made `log_episode` warn once when an outside episode starts and log once when it ends, with the same message and fields ("the surface lies outside the desktop window"). The `Wake` handler in render.rs, added by 02-23 and not adjusted afterwards, still warns with the same text for every `PartlyHidden`/`Hidden` placement returned by `set_rect`. A move that starts an episode therefore logs the warning twice (once from `set_rect`, once from the renderer), and with 02-25 every renderer is sent a `SetRect` after every display change, so a surface that stays outside warns on each of them, which defeats the "one warning per episode" contract of `outside_episode`.
**Fix:** In render.rs keep the "surface moved" line (it has the verdict) and drop the `PartlyHidden`/`Hidden` warn, since the episode logging in surface.rs covers it; keep the `Misplaced` warn, which the episode record deliberately ignores.

### IN-28: Comments that drifted with the 02-23 to 02-26 changes

**Origin:** new (02-24, 02-25, 02-26)
**File:** `crates/sucrose-platform/src/os_windows/surface.rs:827-829` ("the maintainer run of 02-26 checks on the desktop that a parent move produces the 'desktop window moved' line"); `apps/sucrose/src/service.rs:1505-1506` (`apply_snapshot` doc says nothing of re-placing; it was "applies the monitors"), `:1533` (`apply` doc "see [`diff`]" while the actions now come from `replace_all`)
**Issue:** The surface.rs comment refers to a future verification that has been done (02-26 ran, 8 of 8 moves caught) and names a plan id that means nothing to a later reader of the code. The two service.rs doc comments still describe the old per-monitor-diff behavior, which 02-25 changed.
**Fix:** surface.rs: "Hook delivery cannot be tested in-process; it was checked on a real desktop (UAT G-02-7)." service.rs: say in the `apply_snapshot` doc that every surface is re-placed after each applied snapshot and point `apply`'s doc at `replace_all`.

## Resolved since the last review

- **IN-24** (the WR-15 test re-implements the handler instead of testing it): resolved. The closure-copy test is gone; the decision is the pure `handle_report` (`service.rs:1069-1090`) and the exit side is `status_after_exit` (`:1094-1109`). The tests call the real functions: `exit_then_failed` (`:1810-1821`), `a_report_counts_only_while_its_renderer_holds_the_slot` (`:1824`), and the spent-budget tests (`:1890`, `:1917`).
- **IN-10** (the GPU regression's first suspect is `deband=yes` / `profile=low-latency`): resolved by evidence. The premise was a GPU regression recorded on two different GPUs. `docs/MEASUREMENTS.md` (rows of 2026-11-23, "reading the numbers") now holds the same-GPU measurement (RTX 3050 for both renderers, same LUID, `cargo xtask measure --row` refuses mixed adapters): GPU 16.4 % vs 16.1 %, R = 1.02, and the doc records the decision to keep the two options for parity. `config.rs:118` and `:131` still generate the options, as intended; FEATURES.md (Video row) carries the same numbers.

---

_Reviewed: 2026-11-23_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
