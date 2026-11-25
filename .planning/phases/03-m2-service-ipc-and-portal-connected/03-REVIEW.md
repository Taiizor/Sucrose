---
phase: 03-m2-service-ipc-and-portal-connected
reviewed: 2026-11-25T18:30:00Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - Cargo.lock
  - Cargo.toml
  - apps/sucrose-portal/src/app.rs
  - apps/sucrose-portal/src/library/grid.rs
  - apps/sucrose-portal/src/library/layout.rs
  - apps/sucrose-portal/src/library/mod.rs
  - apps/sucrose-portal/src/theme.rs
  - crates/sucrose-platform/Cargo.toml
  - crates/sucrose-platform/src/lib.rs
  - crates/sucrose-platform/src/os_windows.rs
  - docs/ARCHITECTURE.md
  - docs/FEATURES.md
  - docs/GPUI.md
findings:
  critical: 0
  warning: 0
  info: 17
  total: 17
  necessity: 0
status: issues_found
---

# Phase 3: Code Review Report

**Reviewed:** 2026-11-25T18:30:00Z
**Depth:** standard
**Files Reviewed:** 13 in scope (plus the files of the carried findings and the vendored gpui-pre 0.3.12 sources the diffs call)
**Status:** issues_found

## Summary

Incremental re-review of gap-closure plans 03-45 (DWM backdrop type and Opaque reset: `theme::appearance_steps`, `Platform::set_window_backdrop`, `apply_backdrop`) and 03-46 (grid scrollbar mounted as a sibling, clamped in-view request: `grid::scroll_area`, `grid::visible_rows`, `layout::clamped_scroll_top`). Read from `git diff c00613e0..HEAD` and the commits since the previous review (938a28f8, 22e7c336, 260765c6, 12563795, 41e18030), plus the GPUI sources `Window::on_next_frame` and the frame loop that runs its callbacks.

The three previous warnings (WR-14, WR-15, WR-16) are fixed; the evidence is under "Resolved since the previous review". No new defect was found in the changed code. What I traced and found sound:
- `appearance_steps`: every changed target goes through Opaque first, a repeated target changes nothing, `None` counts as changed; the DWM call runs after the GPUI calls, so its type (NONE for None and Acrylic, main window for Mica, tabbed window for Tabbed) is the last word. The DWM call is skipped on a build below 22621, and `window_backdrop` is false there, so the path never logs a refusal for an OS that cannot show a backdrop.
- `set_window_backdrop` (Windows): the handle is checked before the build, the `HWND` conversion from `NonZeroIsize` is correct, the pointer/size pair passed to `DwmSetWindowAttribute` matches `DWM_SYSTEMBACKDROP_TYPE`, the HRESULT is kept, and the `// SAFETY:` comment states the invariants. The default trait method returns `Unsupported`, so macOS and Linux keep compiling.
- `visible_rows`: GPUI runs `on_next_frame` callbacks at the start of the next frame, before it draws (`window.rs` frame loop, `pending_next_frame_callbacks`), so `read(&handle)` there is what the previous frame painted. The comparison is of raw reads, so after one extra render the reads are equal and no further frame is asked for; a dropped view is handled (`update(..).ok()`).
- `clamped_scroll_top` and `content_height` match the grid the page builds (padding on both sides, `rows * 160 + (rows - 1) * gap`), and the new tests cover shrink, resize, page end and a short page.

The necessity reference was read; no necessity finding was made (the `u32::try_from(size_of..).unwrap_or(0)` pattern in `os_windows.rs` already exists at the OS-version call and is the file's convention).

**Carried findings.** All 16 remaining carried findings were re-opened in their source files. None of the files they cite changed in the five commits since the previous review except `library/mod.rs` (IN-16: still present, lines refreshed) and `app.rs` / `theme.rs` / `grid.rs` (IN-23, IN-24: still present, lines refreshed). IN-02, IN-04, IN-10, IN-11, IN-13, IN-14, IN-15, IN-16, IN-17, IN-18, IN-19, IN-20, IN-21, IN-22, IN-23 and IN-24 all still hold. The one new finding is IN-25 (CR-01..02, WR-01..16 and IN-01..24 are taken in the ledger and the previous review).

## Resolved since the previous review

- **WR-14: Switching to Mica or Tabbed keeps an earlier Acrylic, and nothing resets the DWM backdrop type** (plans 03-45, commits 938a28f8 and 22e7c336). `apps/sucrose-portal/src/theme.rs:94-105` (`appearance_steps`: Opaque before any changed target) and `apps/sucrose-portal/src/app.rs:266-283` (the steps are applied in order, the applied appearance is remembered, then `Platform::set_window_backdrop` sets or resets the DWM type and its HRESULT is logged as `dwm_result`). The platform side is `crates/sucrose-platform/src/os_windows.rs:222-259` (`system_backdrop_type` maps every `Backdrop` with no wildcard; `set_window_backdrop` returns the OS error) and `crates/sucrose-platform/src/lib.rs:302-313`. Tests: `theme_appearance_steps_*` (theme.rs:263-291), `backdrop_type_maps_every_setting` and `set_window_backdrop_refuses_a_window_without_a_handle` (os_windows.rs). The maintainer's eye check is recorded in docs/GPUI.md and docs/FEATURES.md (03-47).
- **WR-15: The grid scrollbar is a child of the scrolled element and defaults to hover-only on Windows** (plan 03-46, commits 260765c6 and 12563795). `apps/sucrose-portal/src/library/grid.rs:153-179`: the scroll area (`overflow_y_scroll().track_scroll(scroll)`) and an `absolute().inset_0()` overlay holding `Scrollbar::new(scroll)` are siblings in a non-scrolling `relative()` wrapper. Tests at grid.rs:397-479 pin the overlay bounds while the cards scroll, clicks reaching the cards through the overlay, and page end and short pages. The hover-only default is now stated in the doc comment and in docs/FEATURES.md and docs/GPUI.md (confirmed on the desktop in both OS modes, 03-47).
- **WR-16: In-view request uses last frame's viewport and an unclamped scroll offset, and nothing renders again afterwards** (plan 03-46, commit 41e18030). `apps/sucrose-portal/src/library/grid.rs:207-232` clamps the offset with `layout::clamped_scroll_top` (`layout.rs:127-150`) and notifies the view once through `Window::on_next_frame` when the handle's bounds or offset differ from what the render read. Tests: `layout_stale_offset_past_the_content_still_requests_the_last_rows` (layout.rs:463-474), `grid_request_settles_after_a_content_shrink` and `grid_request_settles_after_the_window_grows` (grid.rs:549-586).

## Info

### IN-02: Endpoint hardening is Windows-only while Linux gets an abstract socket

**Origin:** carried from the previous review (03-REVIEW.before.md)
**File:** `crates/sucrose-ipc/src/lib.rs:140-148`, `:160-168`
**Issue:** Still present. `GenericNamespaced::is_supported()` is true on Linux, so the endpoint is an abstract Unix socket that any local user can connect to, and `owner_sid` is discarded (`let _ = owner_sid`). Latent until the Linux service exists, as the ledger says.
**Fix:** As deferred: before Linux support is enabled, use a filesystem socket in a mode 0700 per-user folder, or check peer credentials on accept.

### IN-04: Stale `#[allow(dead_code)]` with an outdated reason

**Origin:** carried from the previous review (03-REVIEW.before.md)
**File:** `apps/sucrose-portal/src/main.rs:22`
**Issue:** Still present: `#[allow(dead_code, reason = "the Portal pages of the following plans call these helpers")]` on `mod i18n`. The reason refers to plans that have since landed, so it hides genuinely unused helpers.
**Fix:** Remove the allow and delete or use whatever the compiler then reports.

### IN-10: An invalid `Version` is read as current and then overwritten

**Origin:** carried from the previous review (03-REVIEW.before.md)
**File:** `crates/sucrose-config/src/model.rs:743-753`
**Issue:** Still present: an invalid `Version` logs "using the current version" and the next update writes the file. Such a file still parses (it is not `Corrupt`), so `persist` keeps it as the `.bak` before replacing it, but that is one level only and the next save replaces the backup with the rewritten file.
**Fix:** As deferred: treat an unparsable `Version` as "keep a copy aside before the first write" (the `keep_corrupt_aside` path), or refuse to write over it.

### IN-11: Background image path check accepts NUL and UNC paths

**Origin:** carried from the previous review (03-REVIEW.before.md)
**File:** `apps/sucrose/src/settings_host.rs:121-142`
**Issue:** Still present: `check_image` checks emptiness, length, absoluteness and extension only. A path with an embedded NUL passes and is stored; a UNC path (`\\\\server\\share\\a.png`) makes the Portal open a network location on the user's behalf. Same-user IPC only, as the ledger says.
**Fix:** Reject `\0` in the text and, unless UNC backgrounds are wanted, reject paths that start with two backslashes.

### IN-13: The doc comment of `monotonic_ns` is attached to `disconnect_pipe`

**Origin:** carried from the previous review (03-REVIEW.before.md)
**File:** `crates/sucrose-platform/src/os_windows.rs:115-119`, `:128`
**Issue:** Still present: the two doc lines of `monotonic_ns` ("QueryPerformanceCounter scaled to nanoseconds ... RESEARCH A4", lines 115-116) sit directly above the doc of `disconnect_pipe` (117-118), so rustdoc merges them into the documentation of `disconnect_pipe`, and `monotonic_ns` (line 128) has none.
**Fix:** Move the two lines down to `pub(crate) fn monotonic_ns`.

### IN-14: A Portal dropped by `send` during Close Applications is not counted as gone

**Origin:** carried from the previous review (03-REVIEW.before.md)
**File:** `apps/sucrose/src/portal_link.rs:218-229`, `apps/sucrose/src/service.rs:1517-1533`, `:168-176`
**Issue:** Still present. `PortalRegistry::send` clears `registered` when the queue is full. During Close Applications `closing` waits for exactly that Portal, but the later `PortalGone` is classed as "not the registered one" (`was_registered` is false), so `closing.portal_gone()` is not called. The service then waits for `CLOSE_TIMER` (2 s) and ends the process, instead of leaving at once. `portal_gone_effect` also returns `check_exit: true` in both cases.
**Fix:** Remember the pid or connection of the Portal the closing sequence waits for and call `closing.portal_gone()` when a connection of that Portal ends; inline `check_exit = true`.

### IN-15: GPUI.md contradicts itself about whether the preview cost is measured

**Origin:** carried from the previous review (03-REVIEW.before.md)
**File:** `docs/GPUI.md:47`
**Issue:** Still present: the hover-preview bullet ends "The preview cost is still not in the UI-07 numbers until the D-25 runs are repeated", while the next bullet and `docs/MEASUREMENTS.md` say the runs were repeated with animating previews and pass. The project rule is that docs stay true to each other in the same change.
**Fix:** Replace the sentence with a pointer to the repeated runs ("the preview cost is inside the 2026-11-24 repeat rows of MEASUREMENTS.md").

### IN-16: `request_page` reads heap and DXGI memory on the GPUI thread

**Origin:** carried from the previous review (03-REVIEW.before.md; lines refreshed)
**File:** `apps/sucrose-portal/src/library/mod.rs:530-541`, `crates/sucrose-platform/src/os_windows/memory.rs:33-97`
**Issue:** Still present. With debug logging on, every page change (and every bucket change from `render`) calls `memory_detail()` on the UI thread: `HeapSummary` takes the process heap lock and walks every block, and `CreateDXGIFactory1` plus adapter enumeration is a COM round trip. The UI rule (D-12, `.claude/rules/ui.md`) is that the GPUI thread does no blocking work. It is gated by `tracing::enabled!(DEBUG)` and is a measurement aid, so the cost is only in diagnostic runs, but it perturbs the measurements it supports and the heap lock stalls decode threads.
**Fix:** Take the reading on `cx.background_executor()` and log it from there, or sample it from the measurement tool only.

### IN-17: `measure --side portal` reports success when the Portal crashed

**Origin:** carried from the previous review (03-REVIEW.before.md)
**File:** `xtask/src/measure.rs:2143-2162`
**Issue:** Still present. `ended_by` is `portal-exit` for any early end, so a crash and the maintainer closing the Portal after the idle window look the same, and the command exits 0 in both cases. The page-log summary accepts such a series, so a crash during the final idle window produces a "final" value from a shortened tail.
**Fix:** Record the Portal's exit code, or ask the maintainer to close it only after the planned time, and keep the non-zero exit when the process ended with a failure code.

### IN-18: The scratch endpoint name hashes the unnormalized data-folder text

**Origin:** carried from the previous review (03-REVIEW.before.md)
**File:** `crates/sucrose-ipc/src/lib.rs:115-126`
**Issue:** Still present. `endpoint_name_for` hashes `SUCROSE_DATA_DIR` as written (`folder.to_string_lossy()`). `E:\scratch`, `e:\scratch\` and a junction to it give three endpoint names, so two services can run on one data folder, which breaks the rule that the service is the only writer of `Sucrose.json`. The lossy conversion also lets two different non-UTF-8 folders collide. Only scratch runs set the variable.
**Fix:** Hash the result of the same resolution that `synth_library::resolve` does (absolute, canonical where it exists, verbatim prefix stripped, lowercase on Windows), or document that the value must be spelled the same way for every process.

### IN-19: A synchronous `loadfile` failure marks every earlier in-flight request as finished

**Origin:** carried from the previous review (03-REVIEW.before.md)
**File:** `crates/sucrose-engine-mpv/src/player.rs:429-436`, `:215-217`
**Issue:** Still present. `finish_loading` uses `fetch_max`, so `load_source` failing at `self.load(text)` for generation N also finishes any request N-1 whose first frame has not been reported. mpv keeps loading file N-1, but `is_loading()` is now false, so `has_started()` is true and the position and first-frame flag are read as belonging to the new file, which is the state the generation counter was added to prevent. It needs two back-to-back loads with the second failing synchronously, so it is rare; the unit tests pin only the opposite case.
**Fix:** On a failed command restore the counter instead of finishing it (`compare_exchange(generation, generation - 1, ..)`, falling back to `finish_loading` only when another request already followed), and add a unit test for request 1 pending, request 2 failing.

### IN-20: A corrupt file is kept in one slot only, and a failed move still lets the write replace it

**Origin:** carried from the previous review (03-REVIEW.before.md)
**File:** `crates/sucrose-config/src/store.rs:194-202`
**Issue:** Still present. `keep_corrupt_aside` renames onto `Sucrose.json.corrupt`, so a second corruption silently destroys the copy of the first. When the rename fails for a reason that does not also block the later replace of `Sucrose.json` (a read-only or directory-shaped `.corrupt`), the function only logs and `write_atomic` then overwrites the corrupt content, which is the loss WR-09 closed. The tests cover neither case.
**Fix:** Pick the next free name (`Sucrose.json.corrupt`, `.corrupt.1`, ... capped), and on a failed rename fall back to `fs::copy`; when both fail, return an error from `persist` instead of writing over the corrupt file.

### IN-21: The GIF paths decode outside the shared decode budget the docs say covers the whole Portal

**Origin:** carried from the previous review (03-REVIEW.before.md)
**File:** `apps/sucrose-portal/src/background.rs:174-176`, `:194-235`; `apps/sucrose-portal/src/library/preview.rs:45-80`; `apps/sucrose-portal/src/library/thumbs.rs:39-42`
**Issue:** Still present. `DECODE_BUDGET` is documented as bounding "the image decodes of the whole Portal (thumbnails and the background image)", but an animated background GIF returns from `decode_bytes` into `decode_gif` before the budgeted decode is reached, and the hover preview decodes its GIF with its own decoder and no permit. A GIF frame is a full logical-screen RGBA buffer (up to 128 MB by the declared size) plus the decoder's canvas and the resized copy, so a large GIF decodes next to the budgeted thumbnail decodes without being counted. Each decode stays bounded by its own `max_alloc`, so this is a gap in the documented aggregate bound, not an unbounded allocation.
**Fix:** Take a permit priced from the GIF's logical screen (`width * height * 4 * 2`) around `decode_gif` and `decode_preview`, or narrow the comment on `DECODE_BUDGET` to say GIF decodes are outside it.

### IN-22: `OTHER_BYTES_PER_PIXEL` prices 16-bit and float images at a fraction of what they hold

**Origin:** carried from the previous review (03-REVIEW.before.md)
**File:** `apps/sucrose-portal/src/library/thumbs.rs:37-38`, `:380-389`
**Issue:** Still present. Every non-JPEG decode is charged `width * height * 4`. The enabled formats include 16-bit PNG (8 bytes per pixel) and TIFF (up to 16 bytes per pixel as RGBA32F), and `into_rgba8` then copies a non-8-bit image while the source is still alive (12 to 20 bytes per pixel at the peak). A 32 Mi pixel RGBA16 PNG is charged 128 MB but holds about 256 MB plus a 128 MB copy, so the budget admits more concurrent decodes than `DECODE_BUDGET` intends. Each decode is still capped by `MAX_DECODE_ALLOC`, so this is a weaker aggregate bound, not a new unbounded path.
**Fix:** Price non-JPEG decodes from the decoder's own `total_bytes()` (via `ImageReader::into_decoder`) or from `color_type().bytes_per_pixel()` plus the `into_rgba8` copy; or use a conservative 12 bytes per pixel for formats other than 8-bit.

### IN-23: The deferred Root-fill path in `apply_backdrop` has no test

**Origin:** carried from the previous review (03-REVIEW.before.md; lines refreshed)
**File:** `apps/sucrose-portal/src/app.rs:295-302`, `apps/sucrose-portal/src/theme.rs:131-146`
**Issue:** Still present. The tests pin `apply_root_fill` on a window with a Root and on a window without one, and (new in 03-45) `dwm_result_text`; the `mod tests` of `app.rs` (line 518) holds nothing else. The path the fix exists for is not exercised: a Welcome applied from `PortalApp::new` (no Root yet, `mounted == false`) completed by `cx.defer_in` once `open_window` has mounted the Root. If `defer_in` ran before the Root exists in some GPUI version, the only symptom is a `warn!` line and an opaque window under a stored Mica, Acrylic or Tabbed backdrop, which is the regression G-03-5 was opened for. The same gap now also covers the order of `apply_backdrop` itself (steps, then DWM call, then Root fill), which only its pure helpers test.
**Fix:** Add a `gpui_kit::test` that opens a window through the same `open_window` wrapper with an `early` Welcome carrying `Backdrop::Acrylic` and asserts `root.style().background` is transparent after the first draw.

### IN-24: The thumbnail wait and first-frame fallback of `request_visible` depend on an untested GPUI fact

**Origin:** carried from the previous review (03-REVIEW.before.md; lines refreshed)
**File:** `apps/sucrose-portal/src/library/mod.rs:560-570`, `apps/sucrose-portal/src/library/grid.rs:368-376`, `docs/GPUI.md` (Library grid scrolling bullet)
**Issue:** Still present, although 03-46 added scroll tests. The design rests on two GPUI behaviours the docs state as fact: that an `overflow_y_scroll` div notifies the view that rendered it on each wheel step, and that a scrollbar drag redraws the window and re-runs `LibraryView::render`. The new tests do not pin either: the `wheel` helper draws the window by hand after the event (`draw(window)`, grid.rs:375), so a missing notify would not show, and `SettleHost` is notified explicitly (`view.update(.., cx.notify())`, grid.rs:556). No test renders `LibraryView`, so a GPUI change in either behaviour would stop requests (cards stay `Idle`) while every test stays green.
**Fix:** Add one `VisualTestContext` test that renders `LibraryView` with 100 cards, delivers a wheel event without drawing by hand, runs `run_until_parked` and the next frame, and asserts the cards of the new rows are no longer `Idle`.

### IN-25: `crates/sucrose-platform/Cargo.toml` lost its final newline and the new dependency is out of order

**File:** `crates/sucrose-platform/Cargo.toml:13`, `:46`
**Issue:** The commit that added `raw-window-handle` (938a28f8) rewrote the last line: at c00613e0 the file ended `workspace = true` plus a newline, now it ends without one (git shows "\ No newline at end of file"; the working copy is CRLF, so this is not a line-ending artifact). The new dependency is also inserted between `serde` and `serde_json` instead of in sorted position, unlike the rest of the list. Both are noise a formatter or the next editor will flag (`taplo`/editor settings add the newline back, producing an unrelated diff).
**Fix:** Restore the final line ending (CRLF in this working copy, as the file had) and move `raw-window-handle = { workspace = true }` above `serde` (the list is alphabetical after the `sucrose-*` entries).

---

_Reviewed: 2026-11-25T18:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
