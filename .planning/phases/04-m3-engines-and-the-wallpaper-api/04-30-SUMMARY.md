---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 30
subsystem: engines
tags: [gif, direct2d, built-in-renderer, decode-thread, windows]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-22 SurfaceHooks, 04-29 now playing; the Engine and SurfacePump traits, choose_engine and the service's PLAYABLE gate"
provides:
  - "crate sucrose-engine-gif: the built-in GIF renderer (GifEngine handle, GifPump on the surface thread, lazy Animation decoder, Direct2D presenter)"
  - "BuiltInGif arm of detect and start_engine; Gif in PLAYABLE (Gif cards usable in the Portal)"
  - "engine-aware Use rule: load in place only between two mpv wallpapers"
  - "cargo xtask synth-library --media-fixtures (theme gif-anim)"
affects: [04-31, 04-32, 04-37]
actuals:
  tokens: 22500
  tasks: 2
  commits: 2
plan_head_before: 479d218b4394de23aed29099fb203a85e0580623
plan_head_after: 798428f4ff411643b042276c9a834cf45be8baf0
tech-stack:
  added: []
  patterns:
    - "Engine = thread-safe handle + SurfacePump (as CEF and WebView2), with a Presenter trait and a TimerPort trait so the pump is tested with doubles"
    - "decode thread hands frames over a bounded(2) channel; a pass end message says whether the decoder wraps by itself; dropping the receiver stops the thread"
key-files:
  created:
    - crates/sucrose-engine-gif/Cargo.toml
    - crates/sucrose-engine-gif/src/lib.rs
    - crates/sucrose-engine-gif/src/decode.rs
    - crates/sucrose-engine-gif/src/present.rs
    - crates/sucrose-engine-gif/src/engine.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - apps/sucrose/Cargo.toml
    - apps/sucrose/src/render.rs
    - apps/sucrose/src/engines.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/tests/portal.rs
    - xtask/src/synth_library.rs
    - docs/FEATURES.md
    - docs/ARCHITECTURE.md
key-decisions:
  - "The decode thread opens the Animation itself (image's Frames is not Send); start waits up to 8 s for its first message, so a damaged file is a start failure with a reason"
  - "PassEnd carries 'continues': the decoder wraps gaplessly when loop was on, otherwise it stops and a later set_loop(true) restarts it at frame 0"
  - "decide_use compares engines, not wallpaper kinds: UseInput.running_engine replaces running_kind"
requirements-completed: []
duration: 25 min
completed: 2026-11-27
status: complete
---

# Phase 4 Plan 30: The built-in GIF renderer Summary

**GIF themes set to XavierLive or VexanaLive now loop on a built-in renderer: a lazy decode thread (bounded channel of 2) feeding a Direct2D presenter on a child of the desktop surface, with loop, Light pause and resize, and Gif cards usable in the Portal.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-11-27T01:30:09Z
- **Completed:** 2026-11-27T01:55:00Z
- **Tasks:** 2
- **Files modified:** 15 (5 new)

## Accomplishments

- Tracer run (`cargo xtask synth-library --out target/gif-lib --count 2 --media-fixtures --select gif-anim --engine Gif=XavierLive --force`, `sucrose.exe --no-portal`, 12 s): the renderer logged `engine started engine="BuiltInGif"`, `gif presenter=Direct2D`, `gif first frame presented frame_ms=50` (the service then promoted the renderer to Master) and 9 `gif loop completed frames=24` lines in about 12 s (1.2 s per pass). No Direct2D warning was logged.
- `sucrose-engine-gif`: `decode` (pure; format read from the first bytes; GIF today, APNG and WebP answer "not supported yet"; delay 0 plays at 100 ms; decoder limits 256 MiB / 16384 px side; scale down with a Triangle filter, never up; premultiplied BGRA), `present` (Presenter trait and the Direct2D presenter: WS_CHILD window at 0,0 of the surface, hit-transparent, 96 dpi render target, one reused bitmap, recreate on D2DERR_RECREATE_TARGET), `engine` (GifEngine, GifPump, `start`, timer id 2).
- Loop, pause and resize: loop off stops on the last frame and loop on afterwards restarts at frame 0; the Light pause (Heavy alike) kills the frame timer and stores the rest of the frame's delay, the decode thread blocks on its full channel; a new surface size re-opens the decoder skipping to the frame on screen (the frame is redrawn at the new size, then the animation goes on), a move re-uses the decoder.
- Service: Gif is playable, `decide_use` loads in place only between two mpv wallpapers (restart for built-in GIF on either side, or any web engine), `detect` reports BuiltInGif installed where built.
- xtask `--media-fixtures` writes the deterministic `gif-anim` theme and `--select gif-anim` works; the marker records the flag.
- Task 2's behaviors (loop, pause, resize) live in the same pump code as Task 1, so both were implemented and tested together in the first commit; the second commit carries the FEATURES and ARCHITECTURE rows.

## Task Commits

1. **Task 1: A GIF theme loops on the built-in renderer** (with the pump's loop, pause and resize of Task 2) - `ff39ccdd` (feat)
2. **Task 2: docs rows (GIF, pause, engine names, ARCHITECTURE)** - `798428f4` (docs)

**Plan metadata:** the docs(04-30) completion commit that follows this SUMMARY.

## Files Created/Modified

- `crates/sucrose-engine-gif/src/decode.rs` - lazy animation decoding, fit, premultiply, 9 tests
- `crates/sucrose-engine-gif/src/present.rs` - Presenter trait and the Direct2D presenter
- `crates/sucrose-engine-gif/src/engine.rs` - handle, pump, decode thread, `start`, 8 pump tests with a recording presenter and timers
- `apps/sucrose/src/{render,engines,service}.rs` - BuiltInGif start and detect arms, PLAYABLE, engine-aware `decide_use` and its tests
- `xtask/src/synth_library.rs` - `--media-fixtures`
- `docs/FEATURES.md`, `docs/ARCHITECTURE.md` - rows for the renderer

## Decisions Made

See key-decisions in the front matter. Open limits recorded for later plans: a decode error after the first frame is logged and the last frame stays (the engine hooks have no way to report a mid-run failure to the service; start failures do reach the tray); the source size cap beyond the decoder's 256 MiB / 16384 px limits comes with 04-32; the animation fills the surface until 04-31.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The service tests that used the Gif fixture as the refused type**
- **Found during:** Task 1 (cargo test -p sucrose)
- **Issue:** `maps_the_real_gate_for_names_that_would_leave_the_library` (service.rs) and the integration test `use_of_a_gif_theme_is_refused_and_the_selection_is_unchanged` (apps/sucrose/tests/portal.rs, not in the plan's file list) expected Gif to be TypeNotPlayable; with Gif playable the second one would also have started a real GIF renderer.
- **Fix:** both use the `url-source` fixture (Url, not playable until 04-35); the integration test is renamed `use_of_a_url_theme_is_refused_and_the_selection_is_unchanged`; a new unit test asserts that the `gif` fixture now passes the gate.
- **Files modified:** apps/sucrose/src/service.rs, apps/sucrose/tests/portal.rs
- **Committed in:** ff39ccdd

**2. [Rule 3 - Blocking] decode.rs tests need the animation in memory**
- **Issue:** `Animation::open` takes a path, but the tests build GIFs in memory.
- **Fix:** a `Source` enum (file path; an in-memory variant under `cfg(test)`) behind `Animation::open_source`; `open(path, max)` keeps the plan's signature.
- **Committed in:** ff39ccdd

---

**Total deviations:** 2 auto-fixed (1 bug-in-tests, 1 blocking)
**Impact on plan:** no scope change. `next_frame` returns `Result<Option<DecodedFrame>, DecodeError>` (a damaged file mid-way is an error value), and the TDD order (tests before code) was followed within each file but not as separate RED commits, as the plan type is `execute`.

## Issues Encountered

- First run showed 2.5 s between "surface attached" and the engine start: that is the debug-only GPU adapter lookup of `log_video_adapter` (any debug directive in `SUCROSE_LOG` enables it), not the presenter, which takes about 0.2 s.
- `capture.ps1` cannot find the renderer's window (it is the desktop surface, not a main window), so no screenshot was taken. Direct2D inside the layered Progman child (assumption A15) is therefore proven only by the logs (target created, frames presented, no draw warning); the eye check is part of the 04-37 checkpoint. The GDI fallback of 04-31 stays the safety net.

## Known Stubs

None. (Stretch fills the surface by design until 04-31; `set_volume`, `set_audio` and `clock()` do nothing by design.)

## Threat Flags

None beyond the plan's threat model: T-04-66 (frames bounded, no cache, decoder limits) and T-04-67 (decode errors are values; truncated-file tests) are mitigated. A new window class `SucroseGifPresenter` (child window, `WM_NCHITTEST` transparent) is the only new OS surface.

## Line endings

Restored or kept: `Cargo.toml`, `apps/sucrose/Cargo.toml`, `render.rs`, `service.rs`, `docs/FEATURES.md` (no final newline kept) and `docs/ARCHITECTURE.md` stayed `w/crlf` (edits made with a CRLF-aware replace; `cargo fmt` did not change them); `engines.rs`, `portal.rs`, `synth_library.rs` and `Cargo.lock` stayed `w/lf`; the new crate files are LF. Flakes: none seen; `cargo xtask ci` exited 0.

## Next Phase Readiness

04-31 adds the four stretch modes and the GDI presenter behind the same `Presenter` trait (`present` takes the frame by value; `resize` is the size hook). 04-32 adds APNG and WebP to `decode::Animation` (the sniffing already names them) and the source size cap. PLAY-01 and PLAY-05 are partial; TYPE-02 and ENG-07 complete at 04-37.

## Self-Check: PASSED

Created files exist, both task commits are ancestors of HEAD, `cargo xtask ci` exited 0.
