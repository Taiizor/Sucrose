---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 31
subsystem: engines
tags: [gif, stretch, direct2d, gdi, built-in-renderer, windows]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-30 built-in GIF renderer (GifEngine, GifPump, Animation decoder, Direct2D presenter, Engine::set_stretch queued and ignored)"
provides:
  - "fit::fit(stretch, source, surface) -> (dest, src): the pure stretch rectangles shared by every presenter; fit::scaling and fit::pixels"
  - "decode::Scaling (Within, Cover, Crop) and Animation::open_scaled: frames prepared for the stretch mode"
  - "Presenter trait with PresentError, set_stretch and kept_frame; GdiPresenter (StretchDIBits); Fallback (Direct2D then GDI) and present::open"
  - "GifPump follows the stretch setting at start and through set_stretch, with the gif fit debug line"
affects: [04-32, 04-37]
actuals:
  tokens: 20800
  tasks: 2
  commits: 2
plan_head_before: 2331a61c70603787c680e3b58a3a64efe21fe8d4
plan_head_after: 3279d2b7fc6cad03a323816b34bdc9c0b242bc21
tech-stack:
  added: []
  patterns:
    - "Stretch handled in two places: the decoder prepares only what the mode can show (bounded memory, T-04-66), the presenter draws the fit rectangles"
    - "Presenter switch logic is pure (Fallback over Factory closures) and tested with presenter doubles; the Windows presenters are thin"
key-files:
  created:
    - crates/sucrose-engine-gif/src/fit.rs
  modified:
    - crates/sucrose-engine-gif/src/lib.rs
    - crates/sucrose-engine-gif/src/decode.rs
    - crates/sucrose-engine-gif/src/engine.rs
    - crates/sucrose-engine-gif/src/present.rs
    - docs/ARCHITECTURE.md
    - docs/FEATURES.md
key-decisions:
  - "The decoder is stretch-aware: Fill and Uniform fit the frame to the surface, UniformToFill scales down only as far as it still covers the surface, None crops to the centre that fits; a mode change that needs another preparation re-opens the decoder at the shown frame (as a resize does)"
  - "The third D2DERR_RECREATE_TARGET in a row switches to GDI (two recreations first), a success in between resets the count; the switch builds the GDI window before the Direct2D one is dropped and hands over the frame on screen"
  - "Equal aspect ratio is decided by integer cross-multiplication, so Uniform, UniformToFill and Fill give exactly the same rectangles"
requirements-completed: []
duration: 11 min
completed: 2026-11-27
status: complete
coverage:
  - id: D1
    description: "The Engine section's StretchType reaches the built-in renderer (PlaybackSettings.stretch at start, set_stretch while it plays) and decides the rectangles drawn; the gif fit line is logged at debug when the mode or a size changes"
    requirement: DESK-05
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-gif/src/engine.rs#the_stretch_at_start_decides_how_the_first_frame_is_prepared, #a_new_stretch_reaches_the_presenter_and_reopens_the_decoder_only_when_frames_are_prepared_differently"
        status: pass
      - kind: e2e
        ref: "synth-library gif-anim, StretchType Uniform, sucrose.exe --no-portal 12 s: grep 'gif fit' Renderer log | grep -c 'stretch=Uniform ' = 1"
        status: pass
    human_judgment: true
    rationale: "The tests prove the rectangles and the log; that the picture on the desktop looks right (bars, crop) with a non-16:9 animation is an eye check at 04-37"
  - id: D2
    description: "The four stretch modes map as engines.md section 3 says (None centres at 1:1 and crops when larger, Fill stretches, Uniform letterboxes, UniformToFill crops through the source rectangle); equal aspect gives the same full rectangle"
    requirement: TYPE-02
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-gif/src/fit.rs#none_centres_a_small_frame_at_one_to_one, #none_crops_a_large_frame_to_its_centre, #uniform_letterboxes_and_uniform_to_fill_crops, #the_same_aspect_ratio_gives_the_same_rectangles_for_every_stretching_mode"
        status: pass
    human_judgment: false
  - id: D3
    description: "GDI StretchDIBits presenter draws the same rectangles where Direct2D cannot create its target, and a target lost three times in a row switches to GDI"
    requirement: DESK-05
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-gif/src/present.rs#gdi_is_chosen_when_direct2d_cannot_create_its_target, #the_third_loss_in_a_row_moves_drawing_to_gdi_with_the_frame_and_the_settings, #a_success_in_between_starts_the_count_again, #the_bars_are_what_the_destination_leaves_of_the_surface"
        status: pass
    human_judgment: true
    rationale: "The GDI path cannot be forced on the maintainer's machine without a test-only switch (assumption A15), so the Windows GdiPresenter itself is compiled and clippy-checked but never seen drawing; the choice logic is unit tested"
---

# Phase 4 Plan 31: Stretch modes and the GDI fallback of the built-in GIF renderer Summary

**The built-in GIF renderer now follows the Engine section's stretch mode (None, Fill, Uniform, UniformToFill) through a pure `fit::fit` that both a Direct2D presenter and a new GDI `StretchDIBits` presenter draw, with a Fallback wrapper that moves to GDI when Direct2D cannot create its target or loses it three times in a row.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-11-27T02:13:00Z
- **Completed:** 2026-11-27T02:24:00Z
- **Tasks:** 2
- **Files modified:** 7 (1 new)

## Accomplishments

- Tracer run (`cargo xtask synth-library --out target/gif-lib --count 2 --media-fixtures --select gif-anim --engine Gif=XavierLive --force`, `StretchType` edited to `Uniform` in the scratch settings, `sucrose.exe --no-portal`, 12 s): the renderer logged `gif presenter=Direct2D stretch=Uniform`, `gif fit stretch=Uniform dest=[0.0, 0.0, 1920.0, 1080.0] src=[0.0, 0.0, 480.0, 270.0]` (the fixture is 16:9, so the full rectangle) and 7 `gif loop completed frames=24` lines; the grep count of `stretch=Uniform ` was 1. `taskkill.exe /IM sucrose.exe /F` stopped it.
- `fit.rs`: `fit`, `scaling` (the decoder preparation per mode) and `pixels` (GDI integer rectangles), 11 unit tests including the plan's cases (None 320x180 centred, None 3840x2160 cropped to the centre, Uniform and UniformToFill of 1000x1000 on 1920x1080, equal aspect ratio gives identical rectangles).
- `decode.rs`: `Scaling` and `scaled_size`; `Animation::open_scaled` (`open_source` stays the Within form). A cropped frame keeps only the centre that fits the surface, so even a 16384 px animation under None holds at most a surface of pixels per frame.
- `engine.rs`: `set_stretch` queues `Command::Stretch`; the pump applies it, tells the presenter and re-opens the decoder at the frame on screen only when the preparation differs (Fill to Uniform does not). `GifPump::new` takes the stretch from `PlaybackSettings.stretch`. The `gif fit` debug line is logged when the mode or a size changes, not on every frame.
- `present.rs`: `PresentError`, `Presenter::set_stretch` and `kept_frame`, `GdiPresenter` (child window, `GetDC`, black bars via `PatBlt`, `StretchDIBits` of the premultiplied BGRA frame, HALFTONE), `Fallback` (open: Direct2D else GDI with the reason logged; present: third loss in a row switches, a success resets) and `present::open`. 8 presenter tests with doubles.
- `cargo xtask ci` passes (fmt, clippy -D warnings, full workspace tests); `cargo test -p sucrose-engine-gif` runs 40 tests, all passing.

## Task Commits

1. **Task 1: A Uniform setting reaches the built-in renderer's rectangles** - `b98b724a` (feat). This commit also holds the GDI presenter and the Fallback wrapper, see Deviations.
2. **Task 2: The GDI fallback presenter and the docs of the built-in renderer** - `3279d2b7` (docs: ARCHITECTURE and FEATURES only; the code is in the first commit)

**Plan metadata:** the commit that adds this SUMMARY, STATE.md and ROADMAP.md.

## Files Created/Modified

- `crates/sucrose-engine-gif/src/fit.rs` - the stretch rectangles, the decoder preparation per mode, GDI pixel rectangles
- `crates/sucrose-engine-gif/src/decode.rs` - `Scaling`, `scaled_size`, `Animation::open_scaled`
- `crates/sucrose-engine-gif/src/engine.rs` - stretch in the pump, `Command::Stretch`, `gif fit` line, presenter opened through `present::open`
- `crates/sucrose-engine-gif/src/present.rs` - trait changes, `GdiPresenter`, `Fallback`, shared child-window module
- `crates/sucrose-engine-gif/src/lib.rs` - exports `fit`
- `docs/ARCHITECTURE.md`, `docs/FEATURES.md` - the engine line and GIF row 14

Line endings: the four `crates/sucrose-engine-gif/src/*.rs` files and the new `fit.rs` are LF in the working copy (as the plan's recorded `git ls-files --eol` showed before the first edit), so nothing needed restoring; `docs/ARCHITECTURE.md` and `docs/FEATURES.md` were `w/crlf` and still are (edited inside single lines; FEATURES.md still has no newline after its last line). Git warns that LF in the crate files "will be replaced by CRLF" (autocrlf); that is the existing state of the crate, not a change.

## Decisions Made

- The decoder prepares frames per stretch mode instead of always fitting them to the surface. Without it None could never show the original at 1:1 (a large frame was already scaled down) and UniformToFill would upscale a downscaled frame. Memory stays bounded: Crop keeps at most a surface of pixels, Cover never exceeds the original.
- The third consecutive loss switches to GDI (the behaviour line of the plan); T-04-68's "at most 3 times" holds (two recreations happen first). The switch builds the GDI window before dropping the Direct2D one and carries the kept frame, so the picture is never missing.
- GDI redraws on every frame, resize and stretch change but does not repaint itself on an invalidate (no WM_PAINT handler); a finished one-pass GIF on the GDI path would lose its picture if the system invalidated the window. This is the fallback path only and is noted here, not fixed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The GDI presenter and Fallback landed in the Task 1 commit**
- **Found during:** Task 1
- **Issue:** Task 1 needs the Presenter trait to change (`PresentError`, `set_stretch`); the plan's Task 2 changes the same file (`present.rs`) and `engine.rs`. The Windows presenters were restructured together (shared child-window module), so a clean split of the code between two commits was not possible without partial-hunk staging.
- **Fix:** Task 1's commit holds all code; Task 2's commit holds the ARCHITECTURE and FEATURES lines. Both tasks' verify commands ran after the code was complete.
- **Files modified:** crates/sucrose-engine-gif/src/present.rs, engine.rs
- **Commit:** b98b724a, 3279d2b7

**2. [Rule 1 - Bug] The decoder had to know the stretch mode (decode.rs added to the plan's file list)**
- **Found during:** Task 1
- **Issue:** The 04-30 decoder scales a frame down to fit the surface. Under None the plan's behaviour (centre at 1:1, cropped when larger) and under UniformToFill the crop quality could not hold with a frame already fitted inside the surface.
- **Fix:** `Scaling` (Within, Cover, Crop) in `decode.rs`, chosen by `fit::scaling`; the pump re-opens the decoder when a mode change needs another preparation.
- **Files modified:** crates/sucrose-engine-gif/src/decode.rs
- **Commit:** b98b724a

**3. [Rule 3 - Blocking] A clippy type_complexity error on the pump's last-logged tuple**
- **Fix:** a `FitKey` type alias. **Commit:** b98b724a

---

**Total deviations:** 3 auto-fixed (1 bug, 2 blocking)
**Impact on plan:** All needed for the plan's stated behaviour; no scope creep. "Tests first" was followed per unit (the fit tests and the presenter doubles are written together with the code, not committed as separate failing commits; the plan is `type: execute`, so no RED/GREEN gate applies).

## Issues Encountered

None. No flaky test needed a rerun in `cargo xtask ci`.

## Known Stubs

None.

## Threat Flags

None. The decoder keeps the T-04-66 bounds (the Crop mode is smaller than a surface), and T-04-68 is mitigated by the three-in-a-row switch and the per-frame (never looping) recreation.

## Next Phase Readiness

- 04-32 (APNG, WebP, delay edge cases) can reuse `Scaling` and the presenters unchanged.
- Eye check of a non-16:9 animation under each stretch mode, and a real GDI draw, are left for the 04-37 checkpoint (the scratch fixture is 16:9, so the full-rectangle line is what the run showed).
- DESK-05 stays partial (D-09 union rectangle of Span is Round C); `requirements-completed` is empty.

## Self-Check: PASSED

- FOUND: crates/sucrose-engine-gif/src/fit.rs
- FOUND commits b98b724a and 3279d2b7 (ancestors of HEAD)
- `cargo xtask ci` exit 0; `grep -c 'pub fn fit'` fit.rs = 1, `grep -c 'fit::fit'` engine.rs >= 1; docs/FEATURES.md has no final newline and both docs are w/crlf
