---
phase: 02-m1-video-wallpaper-on-the-desktop-windows
fixed_at: 2026-11-23T03:19:46.454Z
review_path: .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 2: Code Review Fix Report

**Fixed at:** 2026-11-23T03:19:46.454Z
**Source review:** .planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4 (fix_scope critical_warning: no Critical, four Warning; the five Info findings are out of scope)
- Fixed: 4
- Skipped: 0

**Verification:** run in the isolated review-fix worktree (`.claude/worktrees/rf-02-*`, removed afterwards) with the main checkout's `target/` directory shared through `--target-dir`, using the Windows toolchain. After the last fix: `cargo clippy --workspace --all-targets -- -D warnings` clean, `cargo test --workspace` all passing (ignored tests need `cargo xtask fetch-engines`), `cargo fmt --all -- --check` clean before every commit. Re-run by the orchestrator in the main checkout after the fast-forward: the shared `target/` first held test binaries built in the worktree with its (deleted) `CARGO_MANIFEST_DIR` embedded, so `sucrose-config`'s fixture test failed until the workspace members were rebuilt (`cargo clean -p` for each member); after that `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` all passed (0 failed). Nothing was run with a GUI, so the runtime behavior of WR-01, WR-03 and WR-04 is not exercised.

## Fixed Issues

### WR-01: A Playing report that is processed after its renderer's Exit leaves a stale "playing" slot

**Files modified:** `apps/sucrose/src/service.rs`
**Commit:** fb1265e2
**Applied fix:** Took the reviewer's cheaper variant: the `Playing` arm only inserts the slot into `fleet.playing` (and elects) when `renderers.connections` still holds a connection for it. `Exited` removes that connection, so a late report from a dead renderer is ignored. The status update still runs. Status: fixed: requires human verification (ordering race, no test; a very narrow window remains if a replacement renderer registers its connection before the dead one's late `Playing` is handled, which the restart delay makes unlikely; the pid-tagged variant from the review would close it fully).

### WR-02: The follower controller records a speed as sent before `set_speed` succeeded

**Files modified:** `apps/sucrose/src/render.rs`, `crates/sucrose-engine-mpv/src/sync.rs`
**Commit:** 398b1601
**Applied fix:** Added `Follower::speed_failed(actual)` (restores the last-sent speed and takes back the change count) and `Follower::at_speed(speed)`. `follow_clock` calls `speed_failed(player.speed())` when `set_speed` fails, and `FollowState::reset` now starts the controller at the player's real speed. Two unit tests added in `sync.rs`. Status: fixed: requires human verification (controller logic).

### WR-03: The video's frame period is cached once, including the fallback default

**Files modified:** `apps/sucrose/src/render.rs`
**Commit:** e71834ab
**Applied fix:** `video_frame_period` now returns `(period, is_final)`, final only when it came from `container-fps`. A new `current_frame_period` caches the value only when final, otherwise reads it again on each reading and logs the `frame period` line when it changes by more than 1 %. Status: fixed: requires human verification (logic; mpv properties not exercised).

### WR-04: The order guard removes itself permanently after ten restores in a minute

**Files modified:** `crates/sucrose-platform/src/os_windows/surface.rs`, `docs/FEATURES.md`
**Commit:** 982bff57
**Applied fix:** Over budget the guard now holds back and keeps its hook; the next event after the oldest restore leaves `GUARD_WINDOW` restores again. A `throttled` flag logs the warning once per episode. The `try_borrow_mut` failure and guard-not-found arms now skip the restore instead of removing the hook (`Budget` enum). Doc comments and the FEATURES row for shell restart recovery state the new behavior. Status: fixed: requires human verification (Win32 hook behavior, not run).

---

_Fixed: 2026-11-23T03:19:46.454Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
