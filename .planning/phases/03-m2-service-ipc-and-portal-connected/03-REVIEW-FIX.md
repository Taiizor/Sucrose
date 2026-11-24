---
phase: 03
fixed_at: 2026-11-24T20:30:16.627Z
review_path: .planning/phases/03-m2-service-ipc-and-portal-connected/03-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 03: Code Review Fix Report

**Fixed at:** 2026-11-24T20:30:16.627Z
**Source review:** .planning/phases/03-m2-service-ipc-and-portal-connected/03-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4 (WR-09, WR-10, WR-11, WR-12; Info findings are out of scope)
- Fixed: 4
- Skipped: 0

**Verification:** run in the isolated worktree (`.claude/worktrees/rf-03-*`, own `target/`, removed afterwards), not in the main checkout: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` all pass (28 test binaries ok, no failure). The libmpv integration tests are ignored (they need `cargo xtask fetch-engines`), so WR-11 is covered by unit tests only. No GUI app was started.

## Fixed Issues

### WR-09: A corrupt settings file with no backup is overwritten and its content is lost

**Files modified:** `crates/sucrose-config/src/store.rs`, `crates/sucrose-config/tests/store.rs`, `docs/ARCHITECTURE.md`
**Commit:** 8a60d6f1
**Applied fix:** New `CORRUPT_FILE` (`Sucrose.json.corrupt`, outside the `*.json` glob). `keep_corrupt_aside` renames a corrupt main file there (replacing an older one, retrying a busy rename, logging a failure without stopping the save). It runs in `persist` when the file to replace reads as `Corrupt` (covers migration, defaults and later saves) and in `open` before a corrupt file is overwritten by a restore from the `.bak` (a case the review text did not name but that loses the same data). Three tests: corrupt without `.bak` is kept byte for byte, corrupt with a good `.bak` is kept too, a file that turns corrupt while running is kept by the next save. The settings paragraph of `docs/ARCHITECTURE.md` names the new file. `docs/DECISIONS.md` was not edited because its entries are append-only; the D-01 entry does not mention `.corrupt`, so a maintainer may want to add a superseding line.

### WR-10: A quit that arrives during a library move can never finish (service never exits)

**Files modified:** `apps/sucrose/src/service.rs`
**Commit:** 2a5fde89
**Applied fix:** `Fleet::note_exit_while_quitting` is called in the renderer-exit handler before the `if quitting { continue; }` return. While a library move runs it records the exit in `rule_closed`, so `any_awaiting()` can become false, `MoveEnv::drive` leaves `Closing`, the move ends and `settle_exit` lets the service exit. With no move it does nothing. Tests: exits recorded one slot at a time until the move is no longer awaiting, and no bookkeeping without a move. The full event sequence (Closing plus Quit plus exits through `drive`) is not driven end to end, because `MoveEnv` needs the whole host; the predicate and the new method are tested. Status: fixed: requires human verification (control-flow fix).

### WR-11: The event thread clears `loading` with a plain store and can erase a newer `load_source`

**Files modified:** `crates/sucrose-engine-mpv/src/player.rs`
**Commit:** 7a334708
**Applied fix:** Took the review's second option. The `loading` bool is gone; `Shared` has `started_generation` and `is_loading()` (`started_generation < load_generation`). The event thread calls `finish_loading(switch.requested)` (`fetch_max`), `load_source` only bumps the generation (and calls `finish_loading` for its own generation when the load command fails), so there is no flag to wipe and a newer request stays loading. Two unit tests cover a newer request surviving the finish of an older one and a failed load. The ignored libmpv integration tests were not run. Status: fixed: requires human verification (concurrency logic).

### WR-12: The pixel-area cap bounds one JPEG decode, not memory; three decodes plus the background image can hold about 800 MB

**Files modified:** `apps/sucrose-portal/src/library/thumbs.rs`
**Commit:** a4e650bc
**Applied fix:** `decode_jpeg_scaled` (shared by thumbnails and the background image) now sets `set_max_decoding_buffer_size(MAX_DECODE_ALLOC)`, reads the decoder's own header with `read_info` and re-runs `check_dimensions` on it, then takes a permit from one process-wide `DecodeBudget` (`JPEG_BUDGET` 256 MiB, cost = width x height x 6). A decode that does not fit waits for the running ones; a decode that is alone always runs, so no file within `MAX_PIXELS` is refused for its size alone. Together this bounds the JPEG decodes of the whole Portal to about one worst-case decode instead of four. `MAX_PIXELS` was left at 32 Mi (lowering it would refuse 8K backgrounds and a test pins it); the review listed that only as something to consider. The generic decoder fallback (other formats, or a JPEG the scaled decoder rejects) is still unbudgeted and bounded per image by `MAX_DECODE_ALLOC`. Tests: a JPEG whose own header is over the limit is not decoded, the budget admits what fits and gives it back, an over-budget request runs alone, a request that does not fit waits for the running one.

---

_Fixed: 2026-11-24T20:30:16.627Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
