---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 35
subsystem: engines
tags: [youtube, iframe-api, generated-page, referer, cef, webview2, type-05, d-06]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-33 generated pages and PageKind, 04-34 the YouTube address gate and youtube_ids, 04-22..04-25 WebView2 host and page calls"
provides:
  - "GeneratedPage::YouTube and YouTubeContent.html: the legacy IFrame API page on the virtual origin, ids from the v and list query"
  - "pages::shuffle_script, page::shuffle_call and the YouTube-specific loop call (setLoop); engine_calls adds the shuffle for YouTube"
  - "sucrose_engine_web::youtube::{referer_for, YOUTUBE_HOSTS, request_filters, https_host}: the Referer fill rule both adapters call"
  - "CEF request handler (OnBeforeResourceLoad, only for YouTube hosts) and WebView2 WebResourceRequested filters for the YouTube hosts"
  - "YouTube in the service's PLAYABLE types and in both web engines' availability"
  - "apps/sucrose/tests/fixtures/web-themes/youtube (Type 4, the Showcase video Floating in Space)"
affects: [04-36, 04-37]
actuals:
  tokens: 12370
  tasks: 2
  commits: 4
plan_head_before: 1e8d771babd2eea93386cabeb65340a7663b56b1
plan_head_after: 45d79dc0153c3df35cff623a5076d1af7e9deb66
tech-stack:
  added: []
  patterns:
    - "A generated page keeps what the engine says before its player exists (volume, pause) and applies it on ready"
    - "A shared pure rule (referer_for) with a thin hook per adapter; CEF returns a request handler only for the hosts the rule covers"
key-files:
  created:
    - crates/sucrose-engine-web/src/pages/YouTubeContent.html
    - crates/sucrose-engine-web/src/youtube.rs
    - crates/sucrose-engine-web/tests/youtube.rs
    - apps/sucrose/tests/fixtures/web-themes/youtube/SucroseInfo.json
  modified:
    - crates/sucrose-engine-web/src/pages.rs
    - crates/sucrose-engine-web/src/page.rs
    - crates/sucrose-engine-web/src/lib.rs
    - crates/sucrose-engine-web/tests/pages.rs
    - crates/sucrose-engine-web/tests/page.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - crates/sucrose-engine-cef/src/browser.rs
    - crates/sucrose-engine-webview2/src/engine.rs
    - crates/sucrose-engine-webview2/src/host.rs
    - apps/sucrose/src/engines.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/tests/portal.rs
    - docs/FEATURES.md
key-decisions:
  - "The page's SucroseStretchMode is a no-op (the player fills the surface; legacy YouTube is not stretched), so the engines' stretch path stays uniform for every generated page"
  - "The engine volume defaults to 0 in the page until the engine's first SucroseVolume call (legacy setVolume(0) on ready), and the product with the property volume is applied once the player is ready"
  - "referer_for treats any existing Referer (even an empty one) as present and never replaces it; the backslash ends the authority like a slash, so a host cannot be smuggled through example.com\\@www.youtube.com"
  - "The CEF resource_request_handler is returned only for requests to a YouTube host, so no other request is touched"
requirements-completed: []
duration: 19 min
completed: 2026-11-27
status: complete
coverage:
  - id: D1
    description: "A YouTube theme plays on CEF: the engine loads the generated YouTubeContent.html from the virtual origin with the video id, the service chooses the engine for kind YouTube and the page is written into the content folder"
    requirement: TYPE-05
    verification:
      - kind: integration
        ref: "sucrose.exe on target/web-lib (synth-library --select youtube): page loaded url=https://sucrose.wallpaper.engine/YouTubeContent.html?v=znNllvBt_Jg status=200; engine chosen kind=YouTube engine=Cef; Cache/Content/YouTubeContent.html written"
        status: pass
    human_judgment: true
    rationale: "The page cannot report playback to the host and no window could be captured; the maintainer checkpoint 04-37 checks the player really plays, a playlist with shuffle and loop, and the embed error codes 150, 152 and 153 (assumption A21)"
  - id: D2
    description: "A YouTube theme plays on WebView2 (YouTube=WebViewLive) with the same page and the same calls"
    requirement: ENG-02
    verification:
      - kind: integration
        ref: "sucrose.exe on target/web-lib with --engine YouTube=WebViewLive: page loaded status=200; engine started engine=WebView2; engine chosen kind=YouTube engine=WebView2"
        status: pass
    human_judgment: true
    rationale: "Playback was not looked at; the maintainer checkpoint 04-37 does"
  - id: D3
    description: "The page and its address: ids from the v and list parameters, the legacy functions, player vars and origin, only the IFrame API script and the virtual origin as addresses, no API key"
    requirement: WAPI-08
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/pages.rs#a_youtube_address_carries_the_ids_of_the_theme_source"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/pages.rs#the_youtube_page_has_the_legacy_functions_the_legacy_player_vars_and_only_two_addresses"
        status: pass
    human_judgment: false
  - id: D4
    description: "The engine drives the page with the legacy calls: SucroseVolume (D-06), setLoop, setShuffle, the built-in property set and the Light pause; a YouTube page takes no input"
    requirement: PROP-05
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/pages.rs#a_youtube_page_gets_the_set_then_volume_loop_shuffle_and_stretch_and_takes_no_input"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/pages.rs#the_youtube_engine_scripts_are_the_legacy_calls"
        status: pass
    human_judgment: true
    rationale: "Shuffle, loop, volume and pause on a real player are checked by the maintainer in 04-37; PROP-05 stays partial in Round B"
  - id: D5
    description: "The Referer hook fills only a missing Referer, only for YouTube hosts, only with the virtual origin, in both adapters"
    requirement: ENG-03
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/youtube.rs#a_host_outside_the_youtube_list_is_never_touched"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/youtube.rs#a_referer_the_request_already_carries_is_never_replaced"
        status: pass
    human_judgment: true
    rationale: "No request lacking a Referer occurred in the automated runs (the page's own requests carry one), so neither hook filled anything; A21 is checked on the Showcase themes in 04-37"
  - id: D6
    description: "YouTube is in PLAYABLE and both web engines are offered for it; the YouTube fixture passes the gate as an address"
    requirement: TYPE-05
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#youtube_is_playable_and_the_gate_gives_its_address_as_the_source"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/engines.rs#the_web_engines_play_the_types_whose_pages_they_have_and_nothing_else"
        status: pass
    human_judgment: false
---

# Phase 4 Plan 35: YouTube wallpapers Summary

**The legacy YouTube IFrame API page (no API key) from the virtual origin, driven by the engine's D-06 volume, loop, shuffle and Light pause on CEF and WebView2, with a Referer-fill safety net in both adapters.**

## Performance

- **Duration:** 19 min
- **Started:** 2026-11-27T03:55:39Z
- **Completed:** 2026-11-27T04:14:30Z
- **Tasks:** 2
- **Files modified:** 17 (4 new)

## Accomplishments

- `YouTubeContent.html` reproduces the legacy page (every function of engines.md 6c, the legacy playerVars with the real `origin`, the legacy end, error and shuffle logic) and reads `v` and `list` from the query, validating them again in the page. It adds `SucrosePause`/`SucrosePlay`, `SucroseVolume` and a no-op `SucroseStretchMode`; a volume or pause that arrives before the player is ready is kept and applied on ready.
- `GeneratedPage::YouTube`: `generated_url` builds `?v=..&list=..` from the theme's address through `youtube_ids` (new `PageError::NoYouTubeId`, nothing written for a refused address). `PageKind` makes it a generated page with the built-in set (caption included) and no input.
- Engine calls: `setLoop` (the page forces false for a single video), `setShuffle` (sent at load and on change), `SucroseVolume`, and the Round A pause script that reaches `SucrosePause`/`SucrosePlay`.
- `referer_for`/`YOUTUBE_HOSTS`/`request_filters`: CEF gets a request handler that exists only for YouTube hosts and sets the referrer in OnBeforeResourceLoad; WebView2 registers filters for the YouTube hosts and sets the header, leaving the response to the network.
- YouTube is in `PLAYABLE` and in both web engines' availability; FEATURES row YouTube is 🟨.

## Task Commits

1. **Task 1: Floating in Space plays on CEF (tracer)** - RED `5f5e2d93` (test), GREEN `f6b4f1e3` (feat)
2. **Task 2: YouTube on WebView2, the engine's calls and the Referer hook** - RED `5e2b27f9` (test), GREEN `45d79dc0` (feat)

**Plan metadata:** the docs commit that follows this SUMMARY.

RED evidence: cargo test is not a report format of `gsd_run check tdd-red-evidence`, so no classifier record was written. Each RED commit holds tests that did not compile because the target symbols did not exist yet (`GeneratedPage::YouTube`, `shuffle_script`, `sucrose_engine_web::youtube`: E0599 and E0432), which asserts nothing and is not a green; the assessment is semantic only.

Tracer gate: the `<verify>` of Task 1 was run end to end (tests, build, synth-library, a CEF run with a page loaded line for the video id and status 200, an engine chosen line for kind=YouTube, the generated page in the content folder) before Task 2 began, then re-run after Task 2 (CEF still loads with the request handler in place).

## Files Created/Modified

- `crates/sucrose-engine-web/src/pages/YouTubeContent.html` - the generated page
- `crates/sucrose-engine-web/src/youtube.rs` - the Referer fill rule and the WebView2 filters
- `crates/sucrose-engine-web/src/pages.rs`, `page.rs`, `lib.rs` - the page kind, scripts and calls
- `crates/sucrose-engine-cef/src/browser.rs`, `engine.rs` - request handler, shuffle command
- `crates/sucrose-engine-webview2/src/host.rs`, `engine.rs` - YouTube filters, Referer fill, shuffle command
- `apps/sucrose/src/engines.rs`, `service.rs` - availability and PLAYABLE
- `apps/sucrose/tests/fixtures/web-themes/youtube/SucroseInfo.json`, `docs/FEATURES.md`, tests

Line endings: `browser.rs`, `engine.rs` (CEF), `engine-web/src/lib.rs`, `service.rs` and `FEATURES.md` were `w/crlf` before and were restored to CRLF before each commit (`cargo fmt` and my edits worked on LF copies); the other files were `w/lf` and new files were left as written. `git ls-files --eol` was checked after the commits.

## Decisions Made

See key-decisions. No new `docs/DECISIONS.md` entry: D-14 and D-021 are applied as written.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] The Use-refusal portal test depended on YouTube being unplayable**
- **Found during:** Task 1 (cargo test -p sucrose)
- **Issue:** `use_of_a_youtube_theme_is_refused_and_the_selection_is_unchanged` asserted `TypeNotPlayable` for a YouTube theme, which now plays.
- **Fix:** The test now uses an Application theme (Type 5), the only type no engine plays yet, and is renamed.
- **Files modified:** apps/sucrose/tests/portal.rs
- **Committed in:** f6b4f1e3

**2. [Rule 1 - Bug] An existing page test listed the generated pages**
- **Found during:** Task 1 (the tests of the web crate)
- **Issue:** `a_different_page_is_replaced_through_a_temporary_file...` listed the files of the content folder without the YouTube page.
- **Fix:** The expected list gained `YouTubeContent.html`.
- **Files modified:** crates/sucrose-engine-web/tests/pages.rs
- **Committed in:** f6b4f1e3

**3. [Rule 2 - Missing critical] Page robustness around the cross-origin frame**
- **Found during:** Task 1 (first CEF run)
- **Issue:** `hideIframeElement` threw a SecurityError in the CEF run, which in the legacy code aborts `onPlayerReady` before the volume and full-screen calls; the volume and mute properties also waited for the player frame.
- **Fix:** `hideIframeElement` runs inside try/catch with a console warning; the volume and muted properties no longer wait for the frame; `requestFullscreen` rejections are swallowed.
- **Files modified:** crates/sucrose-engine-web/src/pages/YouTubeContent.html
- **Committed in:** f6b4f1e3

---

**Total deviations:** 3 auto-fixed (2 bugs in tests, 1 missing robustness)
**Impact on plan:** None on scope.

## Issues Encountered

- **CEF refuses the player-frame access.** In the automated CEF run the renderer log shows `SecurityError: Blocked a frame with origin "https://sucrose.wallpaper.engine" from accessing a cross-origin frame` from `hideIframeElement`, although `--disable-web-security=1`, `--disable-site-isolation-trials=1` and the legacy `disable-features` list are on the command line. The player itself loads (the embed frame is created, page loaded status 200), but the UI hiding and the properties that act inside the frame (scale, filters, mirror, animation, caption) will not apply on CEF. This is the dependency the plan flagged (research Findings 12); it needs a decision (for example `BrowserSettings.web_security` or another way into the frame) that touches D-021 and is not made here. It is recorded in docs/FEATURES.md and must be looked at in 04-37 together with WebView2, whose frame access was not examined (no page console is logged there).
- A harmless `postMessage` target-origin message from the YouTube widget API appears once before the player frame navigates.
- No window could be captured with capture.ps1 (`NO_WINDOW`: the wallpaper surface is no main window), so no screenshot of the player exists.
- The Referer hooks filled nothing in the runs: the page's own requests already carry the origin Referer, as research Findings 12 expected. Neither hook is exercised against a request that lacks one; the shared rule is unit-tested.

## Known Stubs

None.

## Threat Flags

None beyond the plan's threat model: T-04-74 (ids limited at the gate, validated again in the page, never written as markup) and T-04-75 (Referer fills only when missing, only for the YouTube list, only with the origin; tests) are mitigated as planned.

## Next Phase Readiness

04-36 and 04-37 can use the YouTube fixture. The 04-37 checkpoint must check: a single video and a playlist with shuffle and loop, the embed errors 150/152/153 (A21), the CEF frame-access finding above, and that two monitors with the same theme run two independent players until Round C. TYPE-05, WAPI-08, ENG-02, ENG-03 and PROP-05 are not marked complete here (plan rule).

## Self-Check: PASSED

Created files exist, the four task commits are ancestors of HEAD, both acceptance greps and the Task 2 greps hold (`onYouTubeIframeAPIReady` 1, PLAYABLE has YouTube, `referer_for` in browser.rs and host.rs), CRLF files still show w/crlf, and `cargo xtask ci` passed.
