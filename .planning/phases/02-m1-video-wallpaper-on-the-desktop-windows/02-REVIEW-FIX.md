---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
fixed_at: 2026-11-23T14:23:31.559Z
review_path: .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-REVIEW.md
iteration: 1
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 2: Code Review Fix Report

**Fixed at:** 2026-11-23T14:23:31.559Z
**Source review:** .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 1
- Fixed: 1
- Skipped: 0

Info findings (IN-07, IN-19 ... IN-24) are out of scope for `critical_warning` and were not touched. The fix extracts the decision IN-24 asks for as a by-product (see below); IN-24 itself stays open in the review.

## Fixed Issues

### WR-18: A legitimate `Failed` queued behind its own renderer's `Exited` is dropped (reopens WR-15; the reason of WR-17 can be lost)

**Files modified:** `apps/sucrose/src/service.rs`, `docs/FEATURES.md`
**Commit:** a436d88d
**Status:** fixed: requires human verification (logic change in a race handled by two threads; unit-tested, not reproducible on demand)
**Applied fix:**
- The decision of the `Event::Renderer` arm moved out of the inline closure into the pure `handle_report(status, link_pid, running, slot, pid, message, title) -> Option<ReportEffect>`. A report is dropped when its renderer was replaced (another connection holds the slot, or another process runs for it in `supervisor.pids()`) and a late `Playing` of an ended renderer stays dropped (WR-15). A late `Failed` with nobody holding or running the slot now reaches `Status::after`. As the review suggested, a late `Failed` does not replace `Status::RendererStopped` (a spent budget keeps its verdict); a connected renderer's `Failed` still does.
- The status part of the `Exited` arm (`Decision` to new status) moved into `status_after_exit`; side effects (`fleet.stopped`, `restart_after`, logging) stay in the arm.
- Replaced `report_is_stale` (one boolean) and the closure-copy test `a_late_report_of_an_ended_renderer_cannot_rewrite_the_restart_status` with tests of the real functions: `an_engine_failure_whose_exit_is_handled_first_still_names_its_reason` (exit code 2), `a_failed_start_whose_exit_is_handled_first_keeps_its_reason_through_the_restart` (exit code 4, then through the budget until the reason leads the `RendererStopped` text), `a_late_failed_does_not_replace_the_verdict_of_the_spent_budget`, `a_failed_of_a_replaced_renderer_is_dropped_whether_or_not_the_replacement_connected`, `a_late_playing_of_an_ended_renderer_is_dropped`, `a_report_counts_only_while_its_renderer_holds_the_slot`. The exit tests run `classify_exit`, `decide_exit`, `status_after_exit` and `handle_report` in the order the loop runs them.
- `docs/FEATURES.md` Video row: states that the failure reason also arrives when the exit is handled before the report, and when a report is dropped.

**Verification:** run in the isolated worktree (shared `target` not used): `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p sucrose` (124 passed). After the fast-forward, `cargo test --workspace` was run in the main checkout: all passed (engine tests needing `cargo xtask fetch-engines` stay ignored). Nothing was started or run as a GUI.

---

_Fixed: 2026-11-23T14:23:31.559Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
