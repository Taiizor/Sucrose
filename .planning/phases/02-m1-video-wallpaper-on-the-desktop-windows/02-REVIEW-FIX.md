---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
fixed_at: 2026-11-23T14:01:00Z
review_path: .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 2: Code Review Fix Report

**Fixed at:** 2026-11-23
**Source review:** .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (WR-15, WR-16, WR-17; no Critical finding)
- Fixed: 3
- Skipped: 0

Out of scope (fix_scope `critical_warning`) and untouched: IN-07 (necessity finding), IN-19, IN-20, IN-21.

All three fixes are state-machine logic changes. Each has unit tests on a pure seam, but none was exercised against a real renderer, so each is marked "requires human verification".

## Fixed Issues

### WR-15: A late `Playing` or `Failed` of an already ended renderer still rewrites the status (reopens IN-16 and WR-01)

**Status:** fixed: requires human verification
**Files modified:** `apps/sucrose/src/service.rs`
**Commit:** 3a6224b8
**Applied fix:** `report_is_stale(link_pid, pid)` is now `link_pid != Some(pid)`, so every report from a renderer whose connection the slot no longer holds (ended or replaced) is dropped before it reaches `playing`, the election or `Status::after`. The now redundant `link_pid == Some(pid)` test on the Playing branch was removed. The old test was rewritten (no connection is stale) and a test was added that a late Playing or Failed yields no status change.

### WR-16: A refused Hello makes the renderer exit with code 0, and the monitor stays without a wallpaper (reopens WR-05)

**Status:** fixed: requires human verification
**Files modified:** `apps/sucrose/src/render.rs`, `apps/sucrose/src/service.rs`
**Commit:** b3d55ef4
**Applied fix:**
- `render.rs`: the reply handling moved into the pure `judge_hello_reply`. A connection closed before an assignment (`IpcError::Closed`) is now an error, so the renderer exits with code 1 (counted as a crash, restarted within the budget once the old link is gone). `Close` is still the clean stop. Tests added for both.
- `service.rs`: the `Exited` handler now remembers whether the ended renderer's link was the registered one. On the stale-exit path (a replacement already runs) it removes the slot's `playing` entry through `forget_replaced_playing` and re-elects the master, so the replacement is not counted as showing a frame before it has one. Test added.
- Not done: the alternative of making `shell_restarted` skip slots that still have a registered link. The two changes above already make the refused replacement restart.

### WR-17: A failed start loses its reason from the tooltip right after the exit (reopens IN-01, WR-11)

**Status:** fixed: requires human verification
**Files modified:** `apps/sucrose/src/service.rs`, `docs/FEATURES.md`
**Commit:** 3f2417b9
**Applied fix:** I fixed the status mapping rather than the documented behavior.
- New `Status::restarting_after(class, title)`: an `ExitClass::StartFailed` exit leaves a `Failed` status as it is (no change to `Starting`), so mpv's reason stays in the tooltip while the restart runs. The shell-restart path keeps using `restarting`.
- New `Status::stopped_reason(exhausted)`: when the budget is spent and the status still holds a failure, `RendererStopped` reads "<reason> (the renderer stopped after 3 crashes in 60 s)", with the failure first because the tooltip is cut at the end.
- `docs/FEATURES.md` (Video row): "mpv's reason in the tray tooltip" now adds that it stays through the restarts and is named again when the budget is spent. The behavior now matches the sentence's original claim.
- Not done: `Decision::Leave` for an explained start failure. `START_FAILED` (exit 4) also covers a surface that could not be attached, which a restart can fix (for example during a shell restart), so a deterministic missing file still costs three restarts, each loading libmpv. That is a supervisor policy change for the maintainer to decide.

## Verification

Ran in an isolated worktree with its own `target/` (no shared target dir, see the stale-target note in memory). Per commit: `cargo fmt --all`, `cargo test -p sucrose` (121 tests, 120 passed, 1 ignored by design) and `cargo clippy --workspace --all-targets -- -D warnings` (clean). After the fast-forward of `develop-rust`, `cargo test --workspace` ran in the **main checkout**: all test results ok, no failures. No GUI build or sucrose.exe was started, and no real settings or registry were touched. The worktree, temporary branch and recovery sentinel were removed.

---

_Fixed: 2026-11-23_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
