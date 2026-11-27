---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 33
subsystem: engines
tags: [generated-pages, gif, video, cef, webview2, property-sets, volume, d-06, wapi-08]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-24 WebView2 on the shared origin and page calls, 04-09 page calls and property_call text, 04-32 media fixtures (gif-anim, video-clip)"
provides:
  - "sucrose_engine_web::pages: GeneratedPage::{Video, Gif}, generated_url, ensure_written, ensure_file(_with), prepare(_in), builtin_set, volume_script, loop_script, stretch_script, engine_volume_level, effective_volume"
  - "VideoContent.html and GifContent.html embedded in the binary, written into <cache_dir>/Content and loaded as https://sucrose.wallpaper.engine/<Name>Content.html?src=<relative path>"
  - "properties-cef.json and properties-webview.json: the legacy built-in sets with the D-06 volume slider right after muted"
  - "js::property_calls_of (slider values held inside their bounds), js::KIND_ENGINE, page::PageKind, page::engine_calls, page::volume_call, PageState::audio"
  - "CEF and WebView2 play Gif and Video themes; set_audio implemented on both (a follower sends volume 0); availability_for lets both web engines play Gif and Video"
affects: [04-34, 04-35, 04-37]
actuals:
  tokens: 20465
  tasks: 3
  commits: 3
plan_head_before: b564a43953053b60960b2b1f7806d1ab4c052381
plan_head_after: 4fa51d419e3178e4128e328ea7fc8a3fda942a7b
tech-stack:
  added: []
  patterns:
    - "A generated page reads its media path from location.search and requests it from the virtual origin (theme folder first); the path is never written into the page text"
    - "Generated pages are embedded, compared before writing, replaced through a temporary file and a rename; a refused rename is success when the file already holds the same bytes"
    - "PageKind::Generated(page, set): the built-in set replaces the theme's own SucroseProperties.json, no compatible calls and no system data; the current volume, loop and stretch run once after the set"
key-files:
  created:
    - crates/sucrose-engine-web/src/pages.rs
    - crates/sucrose-engine-web/src/pages/VideoContent.html
    - crates/sucrose-engine-web/src/pages/GifContent.html
    - crates/sucrose-engine-web/src/pages/properties-cef.json
    - crates/sucrose-engine-web/src/pages/properties-webview.json
    - crates/sucrose-engine-web/tests/pages.rs
  modified:
    - crates/sucrose-engine-web/src/lib.rs
    - crates/sucrose-engine-web/src/js.rs
    - crates/sucrose-engine-web/src/page.rs
    - crates/sucrose-engine-web/tests/page.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - crates/sucrose-engine-webview2/src/engine.rs
    - apps/sucrose/src/engines.rs
    - docs/FEATURES.md
key-decisions:
  - "The volume control uses the legacy key style: text Property.Localization.Volume and help Property.Localization.Volume.Help, with an English PropertyLocalization table (the legacy sets have no localization table at all)"
  - "property_calls_of clamps slider values to their bounds; property_calls (a theme's own file) does not, so Web themes keep the legacy behavior"
  - "PageKind::Generated carries the GeneratedPage as well as the set, because the loop and stretch scripts are per page"
  - "set_audio on a generated page re-sends the volume (0 for a follower); a theme's own page ignores it until Round D"
requirements-completed: []
duration: 21 min
completed: 2026-11-27
status: complete
coverage:
  - id: D1
    description: "A Video theme set to CEF or WebView2 plays through VideoContent.html on the virtual origin: the page loads with status 200 and the media is requested from the origin (206 Range for the video)"
    requirement: WAPI-08
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/pages.rs#a_video_address_carries_the_path_relative_to_the_theme_folder_as_one_encoded_query_value"
        status: pass
      - kind: integration
        ref: "sucrose.exe on target/web-lib, Video=CefSharpLive and Video=WebViewLive, video-clip: page loaded status=200, /clip.mp4 served 206 bytes 0-15/16"
        status: pass
    human_judgment: true
    rationale: "The video-clip fixture holds 16 placeholder bytes, so decoding and seeking of a real video was not observed; the maintainer checkpoint 04-37 plays real media"
  - id: D2
    description: "A Gif theme plays through GifContent.html on WebView2 (the legacy canvas player and decoder, source from location.search)"
    requirement: WAPI-08
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/pages.rs#the_gif_page_has_every_legacy_function_and_its_decoder_with_the_license_notice"
        status: pass
      - kind: integration
        ref: "sucrose.exe on target/web-lib, Gif=WebViewLive, gif-anim: page loaded status=200, /animation.gif served 200"
        status: pass
    human_judgment: true
    rationale: "The canvas drawing was not looked at; only that the page loaded and requested the GIF from the origin"
  - id: D3
    description: "Built-in property sets of the generated pages (legacy keys and order, D-06 volume after muted, bounds, defaults, ordering) and their calls after the page loaded"
    requirement: PROP-05
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/pages.rs (both_built_in_sets_parse_*, the_volume_is_a_slider_*, a_slider_value_equal_to_its_bounds_*, a_control_without_a_value_*, a_generated_page_gets_the_set_then_the_engine_values_*)"
        status: pass
      - kind: integration
        ref: "sucrose.exe, Video=CefSharpLive: page call kind=property for the 16 keys of the set in order, then engine volume, loop, stretch"
        status: pass
    human_judgment: false
  - id: D4
    description: "Effective page volume = engine volume x property volume / 100, muted or a follower gives 0"
    requirement: PROP-05
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/pages.rs#the_page_volume_is_the_engine_volume_times_the_property_volume"
        status: pass
    human_judgment: true
    rationale: "The JavaScript half of the product (SucroseApplyVolume) runs inside the page and is not covered by a Rust test; nobody listened to it"
---

# Phase 4 Plan 33: GIF and video themes as generated pages Summary

**Gif and Video themes play on CEF and WebView2 through the embedded legacy GifContent.html and VideoContent.html (media path from the query, served by the virtual origin, built-in property sets with the D-06 volume, set_audio and loop/stretch commands on the pages).**

## Performance

- **Duration:** 21 min
- **Started:** 2026-11-27T02:59:20Z
- **Completed:** 2026-11-27T03:21:00Z
- **Tasks:** 3
- **Files modified:** 14 (6 created, 8 modified)

## Accomplishments

- Task 1 (tracer): a Video theme with `Engine.Video = CefSharpLive` on the fixture library loads `VideoContent.html?src=clip.mp4` with status 200, `/clip.mp4` is served 206 (`bytes 0-15/16`) and `Cache/Content/VideoContent.html` exists. Tracer verified end-to-end, expansion continued.
- Task 2: `GifContent.html` (the legacy page; its GIF decoder is gifuct-js and jsBinarySchemaParser, now with their MIT notice) and WebView2 on both pages: `GifContent.html?src=animation.gif` and `VideoContent.html?src=clip.mp4` load with status 200 on WebView2 and the media is requested from the origin.
- Task 3: the legacy CEF and WebView sets decoded from `Internal.cs` (keys, order, texts of the legacy sets unchanged) with a `volume` slider right after `muted`; `property_calls_of`; `PageKind`; `set_audio`, volume, loop and stretch commands on generated pages in both engines. A CEF run logged the 16 property calls of the set in order, then the engine volume, loop and stretch calls.
- `cargo xtask ci` (fmt, clippy -D warnings, tests) passes.

## GIF parser bundle and its license

The bundle in the legacy `Source.cs` (webpack build of the gifuct-js demo player) carries no license notice. I identified it by its code (parseGIF, decompressFrames, the LZW and de-interlace functions, the binary schema parser with `conditional` and `loop`) as **gifuct-js** and **jsBinarySchemaParser** by Matt Way and read both upstream LICENSE files on GitHub: **MIT, Copyright (c) 2015 Matt Way**. The plan says a bundle with another license or none stops the task; I read "none" as "the license cannot be established" and, having established it, kept the bundle and put the full MIT notice into the page. **The maintainer should confirm this reading**; if not accepted, GifContent.html is the one file to drop (Gif themes would then play through mpv or the built-in renderer only). The player code around the decoder is the gifuct-js demo (same author, MIT).

## Task Commits

1. **Task 1: A Video theme on CEF plays through VideoContent.html** - `e5d2f64a` (feat)
2. **Task 2: GifContent.html, and both generated pages on WebView2** - `af69c499` (feat)
3. **Task 3: Built-in property sets with the D-06 volume, and the engine commands on the generated pages** - `4fa51d41` (feat)

**Plan metadata:** the closing docs commit (docs: complete plan)

## Files Created/Modified

- `crates/sucrose-engine-web/src/pages.rs` - generated pages, addresses, safe writing, built-in sets and the engine scripts
- `crates/sucrose-engine-web/src/pages/{VideoContent,GifContent}.html` - the legacy pages with the Round B functions
- `crates/sucrose-engine-web/src/pages/properties-{cef,webview}.json` - the built-in sets with the volume
- `crates/sucrose-engine-web/src/{js,page}.rs` - `property_calls_of`, `PageKind`, engine calls, audio flag
- `crates/sucrose-engine-{cef,webview2}/src/engine.rs` - start through `pages::prepare`, `set_audio`, generated-page commands
- `apps/sucrose/src/engines.rs` - `availability_for` lets both web engines play Gif and Video
- `docs/FEATURES.md` - generated pages row (in progress) and the new per-wallpaper volume row

## Decisions Made

See `key-decisions`. Reversibility: the `volume` key name is rated costly in the plan (Round C writes it into working copies); it was used as the plan says.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The offered-engines test of apps/sucrose expected the old state**
- **Found during:** Task 1 (`cargo test -p sucrose`)
- **Issue:** `the_view_offers_each_type_the_engines_whose_slice_landed_for_it_installed_or_not` still said CEF offers no Video
- **Fix:** it now expects CEF in the Video offer (the planned behavior change)
- **Files modified:** apps/sucrose/src/engines.rs
- **Committed in:** e5d2f64a

### Plan deviations (not bugs)

**2. [Verify command] `grep -c 'key=volume'` of Task 3 prints 0 for a correct run.** The log writes the key with Debug formatting (`key="volume"`). With the pattern `key="volume"` the same run counts 6 lines (the set's property call and the engine volume calls); the property call with `kind="property" key="volume"` is among them. Likewise the CEF scheme requests are logged under the target `sucrose_engine_web`, so the live runs used `SUCROSE_LOG=info,sucrose_engine_cef=debug,sucrose_engine_web=debug` (WebView2 logs its own copy).

**3. [Staging] Video was lifted for CEF only in Task 1, WebView2 and Gif in Task 2.** The plan lifts both web engines for Video in Task 1, but WebView2 did not navigate to the generated page until Task 2; lifting it earlier would have left a commit where WebView2 claimed Video and loaded the raw file. Final state is as planned.

**4. [Design] `PageKind::Generated(page, set)`** instead of `Generated(set)`: the loop and stretch scripts are per page.

**5. [Design] A PropertyLocalization table was added to the sets** (an English table with only the volume texts) because the plan asks for a text in every culture of the set and the legacy sets have none; every other text keeps its legacy `Property.Localization.*` key.

**6. [Process] Tests and code were written together; a separate failing run (RED) was not recorded for the tdd tasks.** The plan is `type: execute`, so no TDD gate applies, but the red step was not observed.

**Total deviations:** 1 auto-fixed (Rule 1), 5 plan deviations. **Impact:** none on the delivered behavior.

## Issues Encountered

- A volume command queued before the page loaded runs against a page that does not define `SucroseVolume` yet (a console error at most); the page calls that run after the load send the current values again, so the final state is right.
- Not observed: real video decoding and seeking (the `video-clip` fixture is 16 placeholder bytes), the GIF canvas drawing, audible volume. A Gif theme on CEF and the Gif page on a second monitor were not run (same code path as the runs that were). These belong to the maintainer checkpoint 04-37.
- Line endings: `crates/sucrose-engine-cef/src/engine.rs` was converted to LF by `cargo fmt` during Task 3 and restored to CRLF before its commit; `js.rs`, `lib.rs` and `docs/FEATURES.md` stayed CRLF (FEATURES keeps no final newline); `engines.rs`, `page.rs`, the WebView2 `engine.rs` and `tests/page.rs` were LF and stay LF; new files are LF. `git` warns that LF files will become CRLF on checkout (autocrlf), which does not matter for the embedded pages because they are compared with their own embedded text.
- Flaky tests: none failed in any run.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- 04-34 and 04-35 (URL and YouTube) can add `GeneratedPage` variants and `PageKind` values the same way; `availability_for` still holds URL and YouTube back.
- Requirement status (Round B rule): WAPI-08, ENG-02, ENG-03 complete at 04-37; PROP-05 stays partial. `requirements-completed` is empty by design.

## Self-Check: PASSED

- Files found: pages.rs, VideoContent.html, GifContent.html, properties-cef.json, properties-webview.json, tests/pages.rs
- Commits found: e5d2f64a, af69c499, 4fa51d41

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-27*
