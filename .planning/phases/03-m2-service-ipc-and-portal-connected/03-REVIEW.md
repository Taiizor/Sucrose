---
phase: 03-m2-service-ipc-and-portal-connected
reviewed: 2026-11-25T15:10:00Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - apps/sucrose-portal/src/app.rs
  - apps/sucrose-portal/src/library/card.rs
  - apps/sucrose-portal/src/library/grid.rs
  - apps/sucrose-portal/src/library/layout.rs
  - apps/sucrose-portal/src/library/mod.rs
  - apps/sucrose-portal/src/library/thumbs.rs
  - apps/sucrose-portal/src/library/atlas.rs
  - apps/sucrose-portal/src/library/preview.rs
  - apps/sucrose-portal/src/theme.rs
  - apps/sucrose-portal/src/main.rs
  - apps/sucrose-portal/src/background.rs
  - apps/sucrose/src/portal_link.rs
  - apps/sucrose/src/service.rs
  - apps/sucrose/src/settings_host.rs
  - crates/sucrose-ipc/src/lib.rs
  - crates/sucrose-config/src/model.rs
  - crates/sucrose-config/src/store.rs
  - crates/sucrose-platform/src/os_windows.rs
  - crates/sucrose-engine-mpv/src/player.rs
  - xtask/src/measure.rs
  - docs/FEATURES.md
  - docs/GPUI.md
findings:
  critical: 0
  warning: 3
  info: 16
  total: 19
  necessity: 0
status: issues_found
---

# Phase 3: Code Review Report

**Reviewed:** 2026-11-25T15:10:00Z
**Depth:** standard
**Files Reviewed:** 9 in scope (plus the callers and the GPUI / gpui-component sources they depend on, listed above)
**Status:** issues_found

## Summary

Incremental re-review of gap-closure plans 03-41 (Root fill under a window backdrop: `theme.rs`, `app.rs`, `docs/GPUI.md`) and 03-42 (in-view thumbnail requests, keep-wanted cache, grid scrollbar, static Idle placeholder: `library/layout.rs`, `thumbs.rs`, `mod.rs`, `card.rs`, `grid.rs`, docs). Read from `git diff 2ce9d51f..HEAD` plus the code the diffs call: the vendored gpui-pre-windows (`set_background_appearance`, `dwm_set_window_composition_attribute`, `set_window_composition_attribute`), gpui's `Interactivity::clamp_scroll_position`, and gpui-component's `ScrollableElement`, `Scrollable` and `system_scrollbar_mode`.

What is sound: `rows_in_view` and `cards_to_request` were re-derived by hand (first row whose bottom is below the viewport top, first row that starts at or below its bottom, ceil/floor boundaries, saturating casts, NaN and zero viewport) and match their tests; `ThumbCache::insert_keeping`, `trim` and `evict_over_caps` keep the byte counter consistent in every branch; `peek`/`touch` split correctly keeps drawing from reordering the LRU; `atlas::release_images` is deferred, so releasing from inside `render` is safe; `apply_root_fill` avoids the `Root::update` panic by looking the entity up first.

The two maintainer leads were traced to code:
- (A) Mica and Tabbed: a confirmed contributor is in gpui-pre-windows (WR-14): the Mica and Tabbed branches never clear an earlier Acrylic accent policy, and no branch ever resets the DWM system backdrop type. The Portal applies the new value without working around either. The remaining "opaque after None" half is not explained by any code I can read and needs a check on an active window (see WR-14).
- (B) Scrollbar not visible: `grid.rs` adds the scrollbar as a child of the scrolling element itself, while gpui-component's own `Scrollable` wrapper deliberately mounts it as a sibling outside the scroll area; and the OS-synced default on Windows is hover-only (WR-15).

Plus one new defect in the request logic: the scroll offset and viewport height are read from the previous frame, and GPUI clamps the offset after render, so a content shrink or resize can leave visible cards on the static Idle fill with no frame ever asking again (WR-16).

The necessity reference (`necessity-review.md`) was not read for this pass, so no `necessity:` count beyond 0 is given and no necessity finding is made.

**Carried findings.** All 14 carried findings were re-opened in their source files. None of those files changed between 2ce9d51f and HEAD except `library/thumbs.rs` (IN-21, IN-22: the code is still as described, line numbers refreshed) and `library/mod.rs` (IN-16: still present, line numbers refreshed). IN-02, IN-04, IN-10, IN-11, IN-13, IN-14, IN-15, IN-16, IN-17, IN-18, IN-19, IN-20, IN-21 and IN-22 all still hold and are re-reported below with their ledger id and title. None was dropped.

New findings are numbered WR-14 to WR-16 and IN-23 to IN-24 (CR-01..02, WR-01..13, IN-01..22 are taken in the ledger).

## Warnings

### WR-14: Switching to Mica or Tabbed keeps an earlier Acrylic, and nothing resets the DWM backdrop type

**File:** `apps/sucrose-portal/src/app.rs:263-264`, `apps/sucrose-portal/src/theme.rs:87-96`; upstream `gpui-pre-windows-0.3.12/src/window.rs:925-941`, `:1594-1614`
**Issue:** `set_background_appearance` in gpui-pre-windows drives two unrelated mechanisms. Acrylic (`Blurred`) and `Opaque` go through `SetWindowCompositionAttribute` (accent state 4 or 0). Mica and Tabbed only call `DwmSetWindowAttribute(DWMWA_SYSTEMBACKDROP_TYPE, 2 | 4)` and return. Consequences that follow from reading that code:
- Acrylic -> Mica or Acrylic -> Tabbed: the accent policy set for Acrylic is never cleared, so the window keeps the Acrylic blur. This is the maintainer's "stays transparent after Acrylic".
- Mica or Tabbed -> Acrylic or None: the DWM system backdrop type stays at 2 or 4 (never set back to `DWMSBT_NONE`), so a later backdrop is stacked on a stale one.
- `dwm_set_window_composition_attribute` discards the HRESULT, and returns silently below build 22621, so a failed Mica call looks like a success in the Portal's `backdrop applied` log line (which reports only the requested value).
`apply_backdrop` calls `set_background_appearance` with the target value only, so none of this is worked around. The other half of the lead ("opaque after None") is not explained by this code: it may be the documented DWM fallback fill of an inactive window (docs/GPUI.md says Mica and Tabbed show it there), or the frame not being extended into the client area (`DwmExtendFrameIntoClientArea` is called nowhere in gpui-pre-windows or the repo); I could not prove either from source, so the fix below is limited to what is proven.
**Fix:** In `apply_backdrop`, apply `WindowBackgroundAppearance::Opaque` first whenever the appearance changes, then the target, so the accent policy is always cleared before Mica or Tabbed:
```rust
let target = theme::appearance(self.backdrop(), self.window_backdrop);
if target != window.background_appearance() {
    window.set_background_appearance(WindowBackgroundAppearance::Opaque);
    window.set_background_appearance(target);
}
```
For the DWM type, add a small `sucrose-platform` function (unsafe stays isolated there, D-011) that sets `DWMWA_SYSTEMBACKDROP_TYPE` to `DWMSBT_NONE`/main/tabbed from the window's HWND and returns the HRESULT, and log its result. Then repeat the maintainer's check with the window active before deciding whether `DwmExtendFrameIntoClientArea` is needed. Correct the docs/GPUI.md bullet once the cause is known.

### WR-15: The grid scrollbar is a child of the scrolled element and defaults to hover-only on Windows

**File:** `apps/sucrose-portal/src/library/grid.rs:146-158`, `docs/FEATURES.md` (Pagination row), `docs/GPUI.md` (Library grid scrolling bullet)
**Issue:** `scroll_area` builds `div().id("library-scroll").overflow_y_scroll().track_scroll(scroll).vertical_scrollbar(scroll).child(..)`. `ScrollableElement::vertical_scrollbar` is `self.child(ScrollbarLayer)`, i.e. an `absolute().inset_0()` overlay appended inside the element that scrolls. GPUI applies the scroll offset to every child of a scroll container (`with_element_offset`), absolute ones included, so the overlay and its thumb move up with the content instead of staying on the right edge. gpui-component's own `Scrollable::render` (`scroll/scrollable.rs` `root_style_from` and the wrapper that follows) avoids this on purpose: the scroll area and the overlay are siblings inside a non-scrolling `relative` wrapper, and the unit tests (`scrollbar_overlay_ignores_content_padding`) cover only that shape. Independently, `system_scrollbar_mode` (theme/mod.rs) gives `Hover` on Windows whenever the OS auto-hides scrollbars (the default), so even a correctly mounted bar draws nothing until the pointer is over its track. Both match the maintainer's "scrollbar not visible". I did not run the app, so which of the two is the visible cause is unconfirmed. FEATURES.md and GPUI.md state that the grid has a scrollbar that follows the OS setting; that is currently unverified.
**Fix:** Mount the scrollbar as a sibling of the scroll area:
```rust
div().relative().size_full()
    .child(div().id("library-scroll").size_full().overflow_y_scroll().track_scroll(scroll)
        .child(v_flex().items_center().p(px(CONTENT_PADDING)).child(block)))
    .child(div().absolute().inset_0()
        .child(Scrollbar::new(scroll).axis(ScrollbarAxis::Vertical).viewport_from_layout()))
```
(`Scrollbar`/`ScrollbarAxis` are public in `gpui_kit::component::scroll`.) Then check on Windows with "Always show scrollbars" both on and off, and record in FEATURES.md that auto-hide shows the bar on hover only.

### WR-16: In-view request uses last frame's viewport and an unclamped scroll offset, and nothing renders again afterwards

**File:** `apps/sucrose-portal/src/library/mod.rs:561-578` (`request_visible`), `apps/sucrose-portal/src/library/layout.rs:101-125`; upstream `gpui-pre-0.3.12/src/elements/div.rs:2488-2547` (`clamp_scroll_position`)
**Issue:** `request_visible` runs inside `render`, before the grid is laid out and painted. It reads `self.scroll.bounds().size.height` and `self.scroll.offset().y`, both from the previous frame. GPUI clamps the offset to the new content height only later, in the scroll container's prepaint. Two cases then request the wrong cards and never correct it, because a changed `requested` is only noticed when the view renders again and the clamp or the new bounds do not notify it:
1. Content shrinks while scrolled down (window widened so `columns` grows, `LibraryMargin` or page size changed). The stale `scroll_top` is past the content: `rows_in_view` returns `rows..rows` (empty) or rows that are no longer the visible ones, `cards_to_request` returns nothing, `wanted` is cleared and every card renders as `Thumb::Idle`. The clamp then moves the grid up into view; the cards that are now visible stay on the static grey fill until the next wheel step. `Idle` asks for no animation frame, so nothing prompts that render (the reason it was introduced).
2. The window gets taller: the viewport used is the old, smaller one, so the newly exposed bottom row is not wanted and stays `Idle`.
The unit tests cover `rows_in_view` and the cache only, never this render-time input, and the first-paint case (zero bounds) is the only stale-input case handled.
**Fix:** Derive the scroll limit from the same numbers the request uses instead of the lagging handle, and notify once when the painted geometry differs from what was used:
```rust
let stride = layout::CARD_HEIGHT + settings.margin;
let content = 2.0 * grid::CONTENT_PADDING + rows as f32 * stride - settings.margin;
let max_top = (content - viewport).max(0.0);
let scroll_top = (-f32::from(self.scroll.offset().y)).clamp(0.0, max_top);
```
and compare the scroll bounds height seen at the next frame with the one used (`window.on_next_frame(|_, cx| cx.notify())` through `cx.entity()` when they differ), so a resize settles on the final geometry. Add a test that a stale offset beyond the content still requests the last rows.

## Info

### IN-02: Endpoint hardening is Windows-only while Linux gets an abstract socket

**Origin:** carried from the 2026-11-24 review (re-confirmed in the 03-42 gap round)
**File:** `crates/sucrose-ipc/src/lib.rs:141-148`, `:160-168`
**Issue:** Still present. `GenericNamespaced::is_supported()` is true on Linux, so the endpoint is an abstract Unix socket that any local user can connect to, and `owner_sid` is discarded (`let _ = owner_sid`). Latent until the Linux service exists, as the ledger says.
**Fix:** As deferred: before Linux support is enabled, use a filesystem socket in a mode 0700 per-user folder, or check peer credentials on accept.

### IN-04: Stale `#[allow(dead_code)]` with an outdated reason

**Origin:** carried from the 2026-11-24 review (re-confirmed in the 03-42 gap round)
**File:** `apps/sucrose-portal/src/main.rs:22`
**Issue:** Still present: `#[allow(dead_code, reason = "the Portal pages of the following plans call these helpers")]` on `mod i18n`. The reason refers to plans that have since landed, so it hides genuinely unused helpers.
**Fix:** Remove the allow and delete or use whatever the compiler then reports.

### IN-10: An invalid `Version` is read as current and then overwritten

**Origin:** carried from the 2026-11-24 review (re-confirmed in the 03-42 gap round)
**File:** `crates/sucrose-config/src/model.rs:743-753`
**Issue:** Still present: an invalid `Version` logs "using the current version" and the next update writes the file. Since WR-09 the damage is limited, because such a file still parses (it is not `Corrupt`), so `persist` keeps it as the `.bak` before replacing it; but that is one level only and the next save replaces the backup with the rewritten file.
**Fix:** As deferred: treat an unparsable `Version` as "keep a copy aside before the first write" (the same `keep_corrupt_aside` path), or refuse to write over it.

### IN-11: Background image path check accepts NUL and UNC paths

**Origin:** carried from the 2026-11-24 review (re-confirmed in the 03-42 gap round)
**File:** `apps/sucrose/src/settings_host.rs:121-142`
**Issue:** Still present: `check_image` checks emptiness, length, absoluteness and extension only. A path with an embedded NUL passes and is stored; a UNC path (`\\\\server\\share\\a.png`) makes the Portal open a network location on the user's behalf. Same-user IPC only, as the ledger says.
**Fix:** Reject `\0` in the text and, unless UNC backgrounds are wanted, reject paths that start with two backslashes.

### IN-13: The doc comment of `monotonic_ns` is attached to `disconnect_pipe`

**Origin:** carried from the 2026-11-24 review (re-confirmed in the 03-42 gap round)
**File:** `crates/sucrose-platform/src/os_windows.rs:105-118`
**Issue:** Still present: the two doc lines of `monotonic_ns` ("QueryPerformanceCounter scaled to nanoseconds ... RESEARCH A4") sit directly above the doc of `disconnect_pipe` (line 105 against 107), so rustdoc merges them into the documentation of `disconnect_pipe` and `monotonic_ns` (line 118) has none.
**Fix:** Move the two lines down to `pub(crate) fn monotonic_ns`.

### IN-14: A Portal dropped by `send` during Close Applications is not counted as gone

**Origin:** carried from the 2026-11-24 review (re-confirmed in the 03-42 gap round)
**File:** `apps/sucrose/src/portal_link.rs:218-229`, `apps/sucrose/src/service.rs:1517-1533`, `:168-176`
**Issue:** Still present. `PortalRegistry::send` clears `registered` when the queue is full. During Close Applications `closing` waits for exactly that Portal, but the later `PortalGone` is classed as "not the registered one" (`was_registered` is false), so `closing.portal_gone()` is not called. The service then waits for `CLOSE_TIMER` (2 s) and ends the process, instead of leaving at once. `portal_gone_effect` also returns `check_exit: true` in both cases.
**Fix:** Remember the pid or connection of the Portal the closing sequence waits for (`closing_portal`) and call `closing.portal_gone()` when a connection of that Portal ends; inline `check_exit = true`.

### IN-15: GPUI.md contradicts itself about whether the preview cost is measured

**Origin:** carried from the 2026-11-24 review (re-confirmed in the 03-42 gap round)
**File:** `docs/GPUI.md:46-47`
**Issue:** Still present: the hover-preview bullet ends "The preview cost is still not in the UI-07 numbers until the D-25 runs are repeated", and the next bullet and `docs/MEASUREMENTS.md` say the runs were repeated with animating previews and pass. The project rule is that docs stay true to each other in the same change.
**Fix:** Replace the sentence with a pointer to the repeated runs ("the preview cost is inside the 2026-11-24 repeat rows of MEASUREMENTS.md").

### IN-16: `request_page` reads heap and DXGI memory on the GPUI thread

**Origin:** carried from the 2026-11-24 review (line numbers refreshed after plan 03-42)
**File:** `apps/sucrose-portal/src/library/mod.rs:536-543`, `crates/sucrose-platform/src/os_windows/memory.rs:33-97`
**Issue:** Still present. With debug logging on, every page change (and every bucket change from `render`) calls `memory_detail()` on the UI thread: `HeapSummary` takes the process heap lock and walks every block, and `CreateDXGIFactory1` plus adapter enumeration is a COM round trip. The UI rule (D-12, `.claude/rules/ui.md`) is that the GPUI thread does no blocking work. It is gated by `tracing::enabled!(DEBUG)` and is a measurement aid, so the cost is only in diagnostic runs, but it also perturbs the measurements it supports and the heap lock stalls decode threads. Since 03-42 `request_page` no longer requests anything (it logs, clears the want set and pumps), so this reading is now its only substantial work.
**Fix:** Take the reading on `cx.background_executor()` and log it from there, or sample it from the measurement tool only.

### IN-17: `measure --side portal` reports success when the Portal crashed

**Origin:** carried from the 2026-11-24 review (re-confirmed in the 03-42 gap round)
**File:** `xtask/src/measure.rs:2143-2162`
**Issue:** Still present. `ended_by` is `portal-exit` for any early end, so a crash and the maintainer closing the Portal after the idle window look the same, and the command exits 0 in both cases. The page-log summary accepts such a series; rounds and 20 s idle checks catch most crashed runs but not a crash during the final idle window, which then produces a "final" value from a shortened tail.
**Fix:** Record the Portal's exit code or ask the maintainer to close it only after the planned time, and keep the non-zero exit when the process ended with a failure code.

### IN-18: The scratch endpoint name hashes the unnormalized data-folder text

**Origin:** carried from the 2026-11-24 review (re-confirmed in the 03-42 gap round)
**File:** `crates/sucrose-ipc/src/lib.rs:115-126`
**Issue:** Still present. `endpoint_name_for` hashes `SUCROSE_DATA_DIR` as written. `E:\scratch`, `e:\scratch\` and a junction to it give three endpoint names, so two services can run on one data folder, which breaks the rule that the service is the only writer of `Sucrose.json`. It also lossy-converts non-UTF-8 paths, so two different folders can collide. Only scratch runs set the variable.
**Fix:** Hash the result of the same resolution that `synth_library::resolve` does (absolute, canonical where it exists, verbatim prefix stripped, lowercase on Windows), or document that the value must be spelled the same way for every process.

### IN-19: A synchronous `loadfile` failure marks every earlier in-flight request as finished

**Origin:** carried from the 2026-11-24 review (re-confirmed in the 03-42 gap round)
**File:** `crates/sucrose-engine-mpv/src/player.rs:429-436`, `:215-217`
**Issue:** `finish_loading` uses `fetch_max`, so `load_source` failing at `self.load(text)` for generation N also finishes any request N-1 whose first frame has not been reported. mpv keeps loading file N-1 (the failed command did not replace it), but `is_loading()` is now false, so `has_started()` is true and the position and first-frame flag are read as belonging to the new file, which is the state the generation counter was added to prevent. It needs two back-to-back loads with the second failing synchronously, so it is rare; the second unit test pins the opposite case only (a newer request after the failed one).
**Fix:** On a failed command restore the counter instead of finishing it: `self.shared.load_generation.compare_exchange(generation, generation - 1, ..)` (falling back to `finish_loading` only when another request already followed), so an older in-flight request keeps `is_loading()` true. Add a unit test for request 1 pending, request 2 failing.

### IN-20: A corrupt file is kept in one slot only, and a failed move still lets the write replace it

**Origin:** carried from the 2026-11-24 review (re-confirmed in the 03-42 gap round)
**File:** `crates/sucrose-config/src/store.rs:194-202`
**Issue:** `keep_corrupt_aside` renames onto `Sucrose.json.corrupt`, so a second corruption silently destroys the copy of the first (documented as "replacing an older one", but the first is usually the file the user most wants to fix by hand). When the rename fails for a reason that does not also block the later replace of `Sucrose.json` (a read-only or directory-shaped `.corrupt`, which `is_busy` retries three times as PermissionDenied), the function only logs and `write_atomic` then overwrites the corrupt content, which is the loss WR-09 closed. The new tests cover neither case.
**Fix:** Pick the next free name (`Sucrose.json.corrupt`, `.corrupt.1`, ... capped at a few) or keep the older copy, and on a failed rename fall back to `fs::copy`; when both fail, return an error from `persist` (as for an unreadable file) instead of writing over the corrupt file.

### IN-21: The GIF paths decode outside the shared decode budget the docs say covers the whole Portal

**Origin:** carried from the 2026-11-24 review (re-confirmed in the 03-42 gap round)
**File:** `apps/sucrose-portal/src/background.rs:174-176`, `:194-235`; `apps/sucrose-portal/src/library/preview.rs:45-80`; `apps/sucrose-portal/src/library/thumbs.rs:39-42`
**Issue:** `DECODE_BUDGET` is documented as bounding "the image decodes of the whole Portal (thumbnails and the background image)", but an animated background GIF returns from `decode_bytes` into `decode_gif` before `decode_budgeted` is reached, and the hover preview decodes its GIF with its own `GifDecoder` and no permit. A GIF frame is a full logical-screen RGBA buffer (up to 32 Mi pixels, 128 MB by the declared size), plus the decoder's canvas and the resized copy, so a hostile or large GIF (a theme preview or the chosen background) decodes next to the budgeted thumbnail decodes without being counted. Each decode stays bounded by its own `max_alloc` (and the preview by one live decode), so this is a gap in the documented aggregate bound, not an unbounded allocation.
**Fix:** Take a permit priced from the GIF's logical screen (`width * height * 4 * 2`) around `decode_gif` and `decode_preview` (export a small `acquire` helper from `thumbs.rs`), or narrow the comment on `DECODE_BUDGET` to say that GIF decodes are outside it.

### IN-22: `OTHER_BYTES_PER_PIXEL` prices 16-bit and float images at a fraction of what they hold

**Origin:** carried from the 2026-11-24 review (re-confirmed in the 03-42 gap round)
**File:** `apps/sucrose-portal/src/library/thumbs.rs:37-38`, `:380-389`
**Issue:** Every non-JPEG decode is charged `width * height * 4`, "RGBA, 4 bytes per pixel". The enabled formats include 16-bit PNG (8 bytes per pixel as RGBA16) and TIFF (up to 16 bytes per pixel as RGBA32F), and the `finish` closures then call `into_rgba8`, which copies a non-8-bit image while the source is still alive (12 to 20 bytes per pixel at the peak). A 32 Mi pixel RGBA16 PNG is charged 128 MB but holds about 256 MB plus a 128 MB copy, so the budget admits more concurrent decodes than `DECODE_BUDGET` intends. The price also comes from the header the `image` crate read, while the scaled JPEG path allocates by `jpeg-decoder`'s own header (checked again with `check_dimensions` but not re-priced); the removed code priced from the decoder that allocates. Each decode is still capped by `MAX_DECODE_ALLOC`, so the effect is a weaker aggregate bound, not a new unbounded path.
**Fix:** Price non-JPEG decodes from the decoder's own `ImageDecoder::total_bytes()` (via `ImageReader::into_decoder`) or from `color_type().bytes_per_pixel()`, and add the `into_rgba8` copy; or use a conservative 12 bytes per pixel for formats other than 8-bit.

### IN-23: The deferred Root-fill path in `apply_backdrop` has no test

**File:** `apps/sucrose-portal/src/app.rs:276-282`, `apps/sucrose-portal/src/theme.rs:155-205`
**Issue:** The tests pin `apply_root_fill` on a window with a Root and on a window without one, but the path the fix exists for is not exercised: a Welcome applied from `PortalApp::new` (no Root yet, `mounted == false`) that is completed by `cx.defer_in` once `open_window` has mounted the Root. If `defer_in` ran before the Root exists in some GPUI version, the only symptom is a `warn!` log line and an opaque window under a stored Mica, Acrylic or Tabbed backdrop, which is the regression G-03-5 was opened for.
**Fix:** Add a `gpui_kit::test` that opens a window through the same `open_window` wrapper with an `early` Welcome carrying `Backdrop::Acrylic` and asserts `root.style().background` is transparent after the first draw.

### IN-24: The thumbnail wait and first-frame fallback of `request_visible` depend on an untested GPUI fact

**File:** `apps/sucrose-portal/src/library/mod.rs:561-578`, `docs/GPUI.md` (Library grid scrolling bullet)
**Issue:** The design rests on two GPUI behaviours that the docs state as fact but no test in the repo pins: that an `overflow_y_scroll` div notifies the view that rendered it on each wheel step, and that a scrollbar drag redraws the window and re-runs `LibraryView::render`. The `thumbs_scrolling_a_100_card_page_keeps_every_wanted_card` test feeds scroll positions straight into `wanted_at`, so a GPUI change in either behaviour would stop requests (cards stay `Idle`) while every test stays green. With WR-15 the scrollbar may also be mounted differently, which changes which view a drag notifies.
**Fix:** Add one `VisualTestContext` test that renders `LibraryView` with 100 cards, scrolls the grid with a simulated wheel event and asserts the cards of the new rows are no longer `Idle` after `run_until_parked`.

---

_Reviewed: 2026-11-25T15:10:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
