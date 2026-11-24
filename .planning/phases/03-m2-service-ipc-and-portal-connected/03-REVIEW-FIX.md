---
phase: 03-m2-service-ipc-and-portal-connected
fixed_at: 2026-11-24T21:32:32.750Z
review_path: .planning/phases/03-m2-service-ipc-and-portal-connected/03-REVIEW.md
iteration: 1
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 3: Code Review Fix Report

**Fixed at:** 2026-11-24T21:32:32.750Z
**Source review:** .planning/phases/03-m2-service-ipc-and-portal-connected/03-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 1
- Fixed: 1
- Skipped: 0

The Info findings (IN-02, IN-04, IN-10, IN-11, IN-13..IN-20) are out of scope for `critical_warning` and stay open in REVIEW.md. This report replaces the one of the previous round (WR-09..WR-12, already committed).

## Fixed Issues

### WR-13: The JPEG decode budget is bypassed by the generic fallback, so the WR-12 bound does not hold

**Files modified:** `apps/sucrose-portal/src/library/thumbs.rs`, `apps/sucrose-portal/src/background.rs`
**Commit:** 718081ed
**Status:** fixed: requires human verification (concurrency and budget pricing, not only syntax)
**Applied fix:**
- New `decode_budgeted(bytes, size, target, finish)` in `thumbs.rs`. It prices the decode from the header size already read (`width * height * 6` for a JPEG, `* 4` otherwise), takes one permit of the shared budget (renamed `IMAGE_DECODES` / `DECODE_BUDGET`), runs the scaled JPEG decoder with the generic decoder as fallback, and keeps the permit until the `finish` closure has returned, so the full-size fallback image and its resize are counted too.
- `decode_bytes` (thumbs) and `decode_bytes` (background, non-GIF path) both go through it; the thumbnail `fit` and the background `resize_exact` + `to_bgra` run inside `finish`.
- The permit moved out of `decode_jpeg_scaled`, which would otherwise take it a second time (a nested acquire of the same budget could wait on itself). `decode_jpeg_scaled` and `decode_generic` are now private to `thumbs.rs`, so no caller can bypass the budget; the `MAX_PIXELS` / `MAX_SIDE` checks and the decoder's own buffer limit are unchanged.
- Test added: `the_generic_fallback_decodes_under_a_permit_priced_from_the_header` (a PNG takes the fallback path and the budget holds at least its share while `finish` runs).
- Not changed: a GIF in the background path still goes through `decode_gif` (own frame and byte caps) and takes no permit; the review did not name it.
- Side effect to keep in mind: non-JPEG thumbnails are now counted against the 256 MB budget too (4 bytes per pixel), so two near-maximum PNGs (32 Mi pixels, 128 MB each) can run together but a third waits. Ordinary images cost a few MB and never wait.

## Verification

- Gates ran in the isolated worktree for the edit (`cargo clippy -p sucrose-portal --all-targets -D warnings`, `cargo test -p sucrose-portal`: 202 passed) and again in the **main checkout** after the fast-forward: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` all pass (no failing test).
- The worktree used its own `target/` (not shared), so no stale test binaries remain; the worktree, its temp branch and the recovery sentinel were removed after the fast-forward.
- sucrose.exe and sucrose-portal.exe were not started.

---

_Fixed: 2026-11-24T21:32:32.750Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
