---
phase: 03-m2-service-ipc-and-portal-connected
plan: 35
subsystem: platform
tags: [rust, windows, autostart, registry, gap-closure, wr-05, tdd]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: enable_autostart, disable_autostart, set_startup and the StartupOs seam (03-23)
provides:
  - "remove_entry in os_windows/autostart.rs: deletes Sucrose's Run value first, Task Manager's StartupApproved flag is best effort (warning and Ok), the rule enable_autostart already applied"
  - "settled_by_state in startup.rs: after a failed enable or disable call set_startup trusts what the OS reports, so General.RunStartup and Sucrose's own entry agree"
affects: [03-verification, tray-03]
actuals:
  tokens: 4500
  tasks: 2
  commits: 4
plan_head_before: 59b4bb8ba2680ad10ee80bfee89a6809403c3fee
plan_head_after: f477252e45ab53b0032e03ec450ede0ca76270cc
tech-stack:
  added: []
  patterns:
    - "A multi-step OS removal is a pure function over step closures, unit-tested without touching the registry"
    - "A request that hit an OS error settles by re-reading the OS state, not by the error"
key-files:
  created: []
  modified:
    - crates/sucrose-platform/src/os_windows/autostart.rs
    - apps/sucrose/src/startup.rs
key-decisions:
  - "The Task Manager flag is cosmetic once the Run value is gone, so its removal failure is logged, never returned (same rule as on enable)"
  - "set_startup decides a failed OS call by os.state(exe).entry_present compared with the wanted state, in both the Normal and the None arm"
requirements-completed: [TRAY-03]
coverage:
  - id: D1
    description: "Turning startup off succeeds once Sucrose's own Run value is deleted, even if the Task Manager flag cannot be removed (WR-05)"
    requirement: TRAY-03
    verification:
      - kind: unit
        ref: "crates/sucrose-platform/src/os_windows/autostart.rs#a_flag_that_cannot_be_removed_after_the_run_value_went_is_not_an_error"
        status: pass
      - kind: unit
        ref: "crates/sucrose-platform/src/os_windows/autostart.rs#a_run_value_that_cannot_be_removed_is_an_error_and_the_flag_is_left"
        status: pass
    human_judgment: false
  - id: D2
    description: "The startup setting and Sucrose's own entry never disagree after a Normal or None request that hit an OS error"
    requirement: TRAY-03
    verification:
      - kind: unit
        ref: "apps/sucrose/src/startup.rs#a_removal_that_fails_after_the_entry_is_gone_stores_zero"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/startup.rs#a_write_that_fails_with_the_entry_present_afterwards_stores_one"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/startup.rs#a_removal_that_fails_with_the_entry_still_there_leaves_the_setting_alone"
        status: pass
    human_judgment: false
  - id: D3
    description: "SC4: with start with the OS on, Sucrose starts after sign-in and resumes the wallpaper, and turning it off removes the Run value on the real registry"
    requirement: TRAY-03
    verification: []
    human_judgment: true
    rationale: "A real sign-out and sign-in cannot be automated; the pending 03-VERIFICATION.md check (plan 03-23) runs Normal, sign-out, sign-in, then None on the real registry"
duration: 4min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 35: Startup removal and setting agree Summary

**Windows startup removal deletes the Run value first and treats the Task Manager flag as best effort, and set_startup settles the RunStartup setting by the OS-reported entry state after a failed call (WR-05).**

## Performance

- **Duration:** 4 min
- **Started:** 2026-11-24T15:25:00Z
- **Completed:** 2026-11-24T15:29:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- `remove_entry` extracted in `os_windows/autostart.rs`: Run value first (its error is returned, the flag is left), then the StartupApproved flag as best effort with a `tracing::warn!`. `disable_autostart` uses it; a missing key counts as nothing to remove.
- `settled_by_state` in `startup.rs`: when `os.enable` or `os.disable` returns an error, `set_startup` re-reads `os.state(exe)`. An entry that is gone after None stores 0 and reports Disabled; an entry that is present after Normal stores 1 and reports Enabled; otherwise Failed as before. The take-back logic for a setting that cannot be saved is unchanged.
- The C# entries (legacy Run value, scheduled task) are not touched; no real-registry test was run (those stay `#[ignore]`).

## Task Commits

1. **Task 1: two-step removal rule** - `a93679eb` (test, RED), `062fdfd9` (fix, GREEN)
2. **Task 2: setting follows the OS state** - `12c5ca72` (test, RED), `f477252e` (fix, GREEN)

**Plan metadata:** recorded in the docs(03-35) commit that follows this summary.

RED evidence: Task 1, `a_flag_that_cannot_be_removed_after_the_run_value_went_is_not_an_error` failed on its assertion (`Err(Os { context: "test", code: 5 })`); Task 2, `a_removal_that_fails_after_the_entry_is_gone_stores_zero` (left Failed, right Disabled) and `a_write_that_fails_with_the_entry_present_afterwards_stores_one` (left Failed, right Enabled) failed on the planned assertions.

## Files Created/Modified

- `crates/sucrose-platform/src/os_windows/autostart.rs` - `remove_entry`, new `disable_autostart`, three unit tests (CRLF kept)
- `apps/sucrose/src/startup.rs` - `settled_by_state`, settling in both arms of `set_startup`, `FakeOs` can change state in a failing call, three new tests (LF kept)

## Decisions Made

- A flag that cannot be removed after the Run value is gone is logged, not returned, mirroring `enable_autostart`.
- The OS state, not the error, decides the outcome of a failed enable or disable.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Existing test fixture modelled the entry as absent**
- **Found during:** Task 2 (GREEN)
- **Issue:** `a_failed_removal_leaves_the_setting_alone` used a `FakeOs` whose entry was absent while `disable` failed. With the new rule that is correctly a success (the entry is gone), so the test failed.
- **Fix:** The fixture now starts with the entry present, which is what "removal failed, entry still there" means; the assertions are unchanged.
- **Files modified:** apps/sucrose/src/startup.rs
- **Verification:** all 13 startup tests pass
- **Committed in:** f477252e

---

**Total deviations:** 1 auto-fixed (1 test fixture bug)
**Impact on plan:** None on behavior; the test now states its scenario correctly.

## Issues Encountered

None. `cargo test -p sucrose-platform`, `cargo test -p sucrose --bin sucrose startup`, clippy with `-D warnings` and `cargo xtask ci` pass.

## Known Stubs

None.

## Threat Flags

None.

## Human-pending

SC4 is not claimed here. "SC4: with start with the OS on, Sucrose starts after sign-in and resumes the wallpaper (TRAY-03)" stays human-pending in 03-VERIFICATION.md (`behavior_unverified_items`); the sign-out and sign-in check from plan 03-23 also turns startup off and confirms the Run value is gone, which is the path this plan changed.

## Next Phase Readiness

WR-05 is fixed. Remaining for TRAY-03 is the human sign-in check in /gsd-verify-work.

## Self-Check: PASSED

- FOUND: crates/sucrose-platform/src/os_windows/autostart.rs, apps/sucrose/src/startup.rs
- FOUND commits: a93679eb, 062fdfd9, 12c5ca72, f477252e
- acceptance: `fn remove_entry` count 1, `delete_value(&approved, RUN_VALUE_NAME)?` count 0, `fn settled_by_state` count 1, SC4 item count in 03-VERIFICATION.md 1
