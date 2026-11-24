---
phase: 03-m2-service-ipc-and-portal-connected
reviewed: 2026-11-24T21:52:06Z
depth: standard
files_reviewed: 2
files_reviewed_list:
  - apps/sucrose-portal/src/background.rs
  - apps/sucrose-portal/src/library/thumbs.rs
findings:
  critical: 0
  warning: 0
  info: 14
  total: 14
status: issues_found
---

# Phase 3: Code Review Report

**Reviewed:** 2026-11-24T21:52:06Z
**Depth:** standard
**Files Reviewed:** 2
**Status:** issues_found

## Summary

Incremental re-review of the WR-13 fix (commit 718081ed), read from `git diff 594ec313..HEAD` for `background.rs` and `thumbs.rs`, plus the surrounding code (`decode_budgeted`, `DecodeBudget::acquire`, `DecodePermit`, `decode_jpeg_scaled`, `decode_generic`, both `decode_bytes`, `decode_gif`, and `library/preview.rs`).

- **WR-13 is fixed.** Both entry points (thumbnail `decode_bytes` and background `decode_bytes`) now go through `decode_budgeted`, which takes one permit of the shared `IMAGE_DECODES` budget before either decoder runs and holds it until the `finish` closure (fit/resize/BGRA conversion) has returned. The scaled JPEG decoder and the generic fallback are therefore under the same permit, the two decoders are private so no caller can bypass it, no nested `acquire` exists (no self-deadlock), and the new test pins that the permit is held during `finish` on the fallback path. It is not re-reported. Two remaining gaps in the budget's coverage are new, minor findings (IN-21, IN-22).
- All twelve carried findings (IN-02, IN-04, IN-10, IN-11, IN-13 to IN-20) were re-opened in their source files and all still hold; they are re-reported below under their ledger ids and titles with line numbers refreshed. None of them was affected by the WR-13 commit.
- No critical issue and no warning was found. The necessity reference (`necessity-review.md`) was not read for this pass, so no `necessity:` count is given.

## Info

### IN-02: Endpoint hardening is Windows-only while Linux gets an abstract socket

**Origin:** carried from the 2026-11-24 review
**File:** `crates/sucrose-ipc/src/lib.rs:141-148`, `:160-168`
**Issue:** Still present. `GenericNamespaced::is_supported()` is true on Linux, so the endpoint is an abstract Unix socket that any local user can connect to, and `owner_sid` is discarded (`let _ = owner_sid`). Latent until the Linux service exists, as the ledger says.
**Fix:** As deferred: before Linux support is enabled, use a filesystem socket in a mode 0700 per-user folder, or check peer credentials on accept.

### IN-04: Stale `#[allow(dead_code)]` with an outdated reason

**Origin:** carried from the 2026-11-24 review
**File:** `apps/sucrose-portal/src/main.rs:22`
**Issue:** Still present: `#[allow(dead_code, reason = "the Portal pages of the following plans call these helpers")]` on `mod i18n`. The reason refers to plans that have since landed, so it hides genuinely unused helpers.
**Fix:** Remove the allow and delete or use whatever the compiler then reports.

### IN-10: An invalid `Version` is read as current and then overwritten

**Origin:** carried from the 2026-11-24 review
**File:** `crates/sucrose-config/src/model.rs:743-753`
**Issue:** Still present: an invalid `Version` logs "using the current version" and the next update writes the file. Since WR-09 the damage is limited, because such a file still parses (it is not `Corrupt`), so `persist` keeps it as the `.bak` before replacing it; but that is one level only and the next save replaces the backup with the rewritten file.
**Fix:** As deferred: treat an unparsable `Version` as "keep a copy aside before the first write" (the same `keep_corrupt_aside` path), or refuse to write over it.

### IN-11: Background image path check accepts NUL and UNC paths

**Origin:** carried from the 2026-11-24 review
**File:** `apps/sucrose/src/settings_host.rs:121-142`
**Issue:** Still present: `check_image` checks emptiness, length, absoluteness and extension only. A path with an embedded NUL passes and is stored; a UNC path (`\\server\share\a.png`) makes the Portal open a network location on the user's behalf. Same-user IPC only, as the ledger says.
**Fix:** Reject `\0` in the text and, unless UNC backgrounds are wanted, reject paths that start with two backslashes.

### IN-13: The doc comment of `monotonic_ns` is attached to `disconnect_pipe`

**Origin:** carried from the 2026-11-24 review
**File:** `crates/sucrose-platform/src/os_windows.rs:105-118`
**Issue:** Still present: the two doc lines of `monotonic_ns` ("QueryPerformanceCounter scaled to nanoseconds ... RESEARCH A4") sit directly above the doc of `disconnect_pipe` (line 105 against 109), so rustdoc merges them into the documentation of `disconnect_pipe` and `monotonic_ns` (line 118) has none.
**Fix:** Move the two lines down to `pub(crate) fn monotonic_ns`.

### IN-14: A Portal dropped by `send` during Close Applications is not counted as gone

**Origin:** carried from the 2026-11-24 review
**File:** `apps/sucrose/src/portal_link.rs:218-229`, `apps/sucrose/src/service.rs:1517-1533`, `:168-176`
**Issue:** Still present. `PortalRegistry::send` clears `registered` when the queue is full. During Close Applications `closing` waits for exactly that Portal, but the later `PortalGone` is classed as "not the registered one" (`was_registered` is false), so `closing.portal_gone()` is not called. The service then waits for `CLOSE_TIMER` (2 s) and ends the process, instead of leaving at once. `portal_gone_effect` also returns `check_exit: true` in both cases.
**Fix:** Remember the pid or connection of the Portal the closing sequence waits for (`closing_portal`) and call `closing.portal_gone()` when a connection of that Portal ends; inline `check_exit = true`.

### IN-15: GPUI.md contradicts itself about whether the preview cost is measured

**Origin:** carried from the 2026-11-24 review
**File:** `docs/GPUI.md:46-47`
**Issue:** Still present: the hover-preview bullet ends "The preview cost is still not in the UI-07 numbers until the D-25 runs are repeated", and the next bullet and `docs/MEASUREMENTS.md` say the runs were repeated with animating previews and pass. The project rule is that docs stay true to each other in the same change.
**Fix:** Replace the sentence with a pointer to the repeated runs ("the preview cost is inside the 2026-11-24 repeat rows of MEASUREMENTS.md").

### IN-16: `request_page` reads heap and DXGI memory on the GPUI thread

**Origin:** carried from the 2026-11-24 review
**File:** `apps/sucrose-portal/src/library/mod.rs:520-531`, `crates/sucrose-platform/src/os_windows/memory.rs:33-97`
**Issue:** Still present. With debug logging on, every page change calls `memory_detail()` on the UI thread: `HeapSummary` takes the process heap lock and walks every block, and `CreateDXGIFactory1` plus adapter enumeration is a COM round trip. The UI rule (D-12, `.claude/rules/ui.md`) is that the GPUI thread does no blocking work. It is gated by `tracing::enabled!(DEBUG)` and is a measurement aid, so the cost is only in diagnostic runs, but it also perturbs the measurements it supports and the heap lock stalls decode threads.
**Fix:** Take the reading on `cx.background_executor()` and log it from there, or sample it from the measurement tool only.

### IN-17: `measure --side portal` reports success when the Portal crashed

**Origin:** carried from the 2026-11-24 review
**File:** `xtask/src/measure.rs:2143-2162`
**Issue:** Still present. `ended_by` is `portal-exit` for any early end, so a crash and the maintainer closing the Portal after the idle window look the same, and the command exits 0 in both cases. The page-log summary accepts such a series; rounds and 20 s idle checks catch most crashed runs but not a crash during the final idle window, which then produces a "final" value from a shortened tail.
**Fix:** Record the Portal's exit code or ask the maintainer to close it only after the planned time, and keep the non-zero exit when the process ended with a failure code.

### IN-18: The scratch endpoint name hashes the unnormalized data-folder text

**Origin:** carried from the 2026-11-24 review
**File:** `crates/sucrose-ipc/src/lib.rs:115-126`
**Issue:** Still present. `endpoint_name_for` hashes `SUCROSE_DATA_DIR` as written. `E:\scratch`, `e:\scratch\` and a junction to it give three endpoint names, so two services can run on one data folder, which breaks the rule that the service is the only writer of `Sucrose.json`. It also lossy-converts non-UTF-8 paths, so two different folders can collide. Only scratch runs set the variable.
**Fix:** Hash the result of the same resolution that `synth_library::resolve` does (absolute, canonical where it exists, verbatim prefix stripped, lowercase on Windows), or document that the value must be spelled the same way for every process.

### IN-19: A synchronous `loadfile` failure marks every earlier in-flight request as finished

**Origin:** carried from the 2026-11-24 review
**File:** `crates/sucrose-engine-mpv/src/player.rs:429-436`
**Issue:** `finish_loading` uses `fetch_max`, so `load_source` failing at `self.load(text)` for generation N also finishes any request N-1 whose first frame has not been reported. mpv keeps loading file N-1 (the failed command did not replace it), but `is_loading()` is now false, so `has_started()` is true and the position and first-frame flag are read as belonging to the new file, which is the state the generation counter was added to prevent. It needs two back-to-back loads with the second failing synchronously, so it is rare; the second unit test pins the opposite case only (a newer request after the failed one).
**Fix:** On a failed command restore the counter instead of finishing it: `self.shared.load_generation.compare_exchange(generation, generation - 1, ..)` (falling back to `finish_loading` only when another request already followed), so an older in-flight request keeps `is_loading()` true. Add a unit test for request 1 pending, request 2 failing.

### IN-20: A corrupt file is kept in one slot only, and a failed move still lets the write replace it

**Origin:** carried from the 2026-11-24 review
**File:** `crates/sucrose-config/src/store.rs:194-202`
**Issue:** `keep_corrupt_aside` renames onto `Sucrose.json.corrupt`, so a second corruption silently destroys the copy of the first (documented as "replacing an older one", but the first is usually the file the user most wants to fix by hand). When the rename fails for a reason that does not also block the later replace of `Sucrose.json` (a read-only or directory-shaped `.corrupt`, which `is_busy` retries three times as PermissionDenied), the function only logs and `write_atomic` then overwrites the corrupt content, which is the loss WR-09 closed. The new tests cover neither case.
**Fix:** Pick the next free name (`Sucrose.json.corrupt`, `.corrupt.1`, ... capped at a few) or keep the older copy, and on a failed rename fall back to `fs::copy`; when both fail, return an error from `persist` (as for an unreadable file) instead of writing over the corrupt file.

### IN-21: The GIF paths decode outside the shared decode budget the docs say covers the whole Portal

**File:** `apps/sucrose-portal/src/background.rs:174-176`, `:194-235`; `apps/sucrose-portal/src/library/preview.rs:45-80`; `apps/sucrose-portal/src/library/thumbs.rs:39-42`
**Issue:** `DECODE_BUDGET` is documented as bounding "the image decodes of the whole Portal (thumbnails and the background image)", but an animated background GIF returns from `decode_bytes` into `decode_gif` before `decode_budgeted` is reached, and the hover preview decodes its GIF with its own `GifDecoder` and no permit. A GIF frame is a full logical-screen RGBA buffer (up to 32 Mi pixels, 128 MB by the declared size), plus the decoder's canvas and the resized copy, so a hostile or large GIF (a theme preview or the chosen background) decodes next to the budgeted thumbnail decodes without being counted. Each decode stays bounded by its own `max_alloc` (and the preview by one live decode), so this is a gap in the documented aggregate bound, not an unbounded allocation.
**Fix:** Take a permit priced from the GIF's logical screen (`width * height * 4 * 2`) around `decode_gif` and `decode_preview` (export a small `acquire` helper from `thumbs.rs`), or narrow the comment on `DECODE_BUDGET` to say that GIF decodes are outside it.

### IN-22: `OTHER_BYTES_PER_PIXEL` prices 16-bit and float images at a fraction of what they hold

**File:** `apps/sucrose-portal/src/library/thumbs.rs:37-38`, `:338-343`
**Issue:** Every non-JPEG decode is charged `width * height * 4`, "RGBA, 4 bytes per pixel". The enabled formats include 16-bit PNG (8 bytes per pixel as RGBA16) and TIFF (up to 16 bytes per pixel as RGBA32F), and the `finish` closures then call `into_rgba8`, which copies a non-8-bit image while the source is still alive (12 to 20 bytes per pixel at the peak). A 32 Mi pixel RGBA16 PNG is charged 128 MB but holds about 256 MB plus a 128 MB copy, so the budget admits more concurrent decodes than `DECODE_BUDGET` intends. The price also comes from the header the `image` crate read, while the scaled JPEG path allocates by `jpeg-decoder`'s own header (checked again with `check_dimensions` but not re-priced); the removed code priced from the decoder that allocates. Each decode is still capped by `MAX_DECODE_ALLOC`, so the effect is a weaker aggregate bound, not a new unbounded path.
**Fix:** Price non-JPEG decodes from the decoder's own `ImageDecoder::total_bytes()` (via `ImageReader::into_decoder`) or from `color_type().bytes_per_pixel()`, and add the `into_rgba8` copy; or use a conservative 12 bytes per pixel for formats other than 8-bit.

---

_Reviewed: 2026-11-24T21:52:06Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
