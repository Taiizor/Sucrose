---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 44
subsystem: engines
tags: [duplicate, time-base, webview2, cef, web-pages, desk-02, sync]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-43: Assignment.time_base, ToRenderer::TimeBase, Engine::set_time_base, D-043"
provides:
  - "The generated Video, GIF and YouTube pages follow the group's time base on CEF and WebView2 (SucroseSyncTime)"
  - "Web and URL pages are synced best effort by a document-start shim (performance.now and requestAnimationFrame offset, endless CSS animations aligned after load)"
  - "CEF registers the shim with DevTools Page.addScriptToEvaluateOnNewDocument through the in-process browser host; WebView2 with AddScriptToExecuteOnDocumentCreated"
  - "Fixture Web theme sync (window.__syncNow) and the read-only probe approach for sync checks"
affects: [04-46, 04-56, DESK-02]
requirements-completed: []
actuals:
  tokens: 21000
  tasks: 3
  commits: 3
plan_head_before: d52aeba1ac222677aaaa2cc9d1a549514fe2bb62
plan_head_after: ccf3735f06a1a801479eab750a3a237b257afe9c
tech-stack:
  added: []
  patterns: ["a page follows the time base only through a script string (SucroseSyncTime) or a document-start script, never a host object or channel", "the CEF browser is made on about:blank when a document-start script is registered; the blank load end is not reported"]
key-files:
  created:
    - apps/sucrose/tests/fixtures/web-themes/sync/SucroseInfo.json
    - apps/sucrose/tests/fixtures/web-themes/sync/index.html
  modified:
    - crates/sucrose-engine-web/src/js.rs
    - crates/sucrose-engine-web/src/page.rs
    - crates/sucrose-engine-web/src/pages/VideoContent.html
    - crates/sucrose-engine-web/src/pages/GifContent.html
    - crates/sucrose-engine-web/src/pages/YouTubeContent.html
    - crates/sucrose-engine-web/tests/pages.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - crates/sucrose-engine-cef/src/browser.rs
    - crates/sucrose-engine-webview2/src/engine.rs
    - crates/sucrose-engine-webview2/src/host.rs
    - docs/FEATURES.md
key-decisions:
  - "The CEF shim goes through DevTools Page.addScriptToEvaluateOnNewDocument (browser host, in process); the late-run fallback exists but was not needed"
  - "Only endless CSS animations are aligned to the epoch after load, so a finite intro animation never jumps to its end"
  - "The video page keeps the rate the page itself set (playbackRate property) as the base of its drift correction"
coverage:
  - id: D1
    description: "On WebView2 in Duplicate with a real video, the two renderers' video pages play the same moment (correction by playbackRate, seek over 1 s)"
    requirement: DESK-02
    verification:
      - kind: integration
        ref: "probe V-WV2 target/sync-probe/video-wv2.json: pages 2, difference_s 0.016 about 12 s after the start (first run 0.046)"
        status: pass
      - kind: unit
        ref: "sucrose-engine-web tests/pages.rs a_generated_page_that_loads_after_the_time_base_arrived_gets_the_call_last, the_time_base_call_is_one_function_call_...; webview2 engine a_time_base_is_kept_and_queued_for_the_page"
        status: pass
    human_judgment: false
  - id: D2
    description: "The built-in GIF page of CEF picks its frame from the elapsed time at each loop wrap; both monitors show frames within one"
    requirement: DESK-02
    verification:
      - kind: integration
        ref: "probe G-CEF target/sync-probe/gif-cef.json: frame_difference -1 (samples 0,-1,-1,-1,0,0)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Web and URL pages are synced best effort: performance.now() agrees between the monitors while Date is untouched; a single wallpaper registers no shim"
    requirement: DESK-02
    verification:
      - kind: integration
        ref: "probes S-CEF (5.1 ms) and S-WV2 (0.8 ms) with the sync fixture: sync-cef.json, sync-wv2.json"
        status: pass
      - kind: unit
        ref: "tests/pages.rs only_a_web_or_url_page_with_a_time_base_gets_the_document_start_shim, the_sync_shim_offsets_the_page_clocks_by_the_epoch_and_never_touches_date_or_opens_a_channel"
        status: pass
    human_judgment: false
  - id: D4
    description: "The YouTube page seeks when the player is over 0.5 s from the time base"
    requirement: DESK-02
    verification:
      - kind: integration
        ref: "probe Y-CEF youtube-cef.json: both players in state 1, largest difference 0.022 s"
        status: pass
    human_judgment: false
  - id: D5
    description: "Edge (DESK-02, adjacency): with the two monitors touching, each video page fills its own monitor and the shared edge shows no seam"
    requirement: DESK-02
    verification: []
    human_judgment: true
    rationale: "A glance check on the desktop, planned for 04-56; nothing in this plan measures it"
duration: 18min
completed: 2026-11-28
status: complete
---

# Phase 4 Plan 44: The web engines follow the shared time base Summary

**Duplicate now keeps the generated Video, GIF and YouTube pages on CEF and WebView2 in step with the group's time base (playbackRate correction and seeks, frame pick at the loop wrap, `seekTo`), and Web and URL pages get a best-effort document-start shim that offsets `performance.now()` and the `requestAnimationFrame` timestamp, registered through DevTools `Page.addScriptToEvaluateOnNewDocument` on CEF and `AddScriptToExecuteOnDocumentCreated` on WebView2.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-11-28T16:27:00Z
- **Completed:** 2026-11-28T16:45:00Z
- **Tasks:** 3
- **Files modified:** 13 (2 created)

## Accomplishments

- `js::time_base_call` (`SucroseSyncTime(<epoch>);`, only for generated pages) and `js::sync_shim_script` (built from one integer, no global, no channel, `Date` never touched); `PageState` keeps the epoch and a page that loads late gets the call after its other calls; `page::document_start_script` returns the shim only for a Web or Url page that started with a time base.
- Video page: once a second, expected position = (now - epoch) x base rate (modulo the duration when looping); drifts over 1 s seek, smaller ones change `playbackRate` proportionally (at most 25 %), the page's own rate is kept as the base and restored.
- GIF page: the loop wrap of the bundled frame loop asks `SucroseGifWrap`, which moves to the frame the clock gives when it is more than one frame away. YouTube page: a 2 s timer seeks over 0.5 s, only while playing.
- WebView2: `set_time_base` queues the call; the shim is registered (and awaited) before the first Navigate. CEF: `set_time_base` likewise; with a time base the browser is made on `about:blank`, the shim registered through `execute_dev_tools_method` of the browser host (no remote port needed), then the page loaded; the blank load end is skipped. If DevTools refused, the shim would run after the page loaded (logged, not needed in these runs).
- Fixture theme `sync` (a canvas bar from `performance.now()`, `window.__syncNow`). FEATURES records the sync per engine.

## Probe results (DISPLAY1 + DISPLAY9, SameDuplicate, scratch data dirs, DeveloperPort 9422, Windows bun.exe, Runtime.evaluate only)

| Probe | Result file | Number |
|---|---|---|
| V-WV2 video, real 8.5 s mp4 (a Video theme of the library copied read-only over the fixture clip) | target/sync-probe/video-wv2.json | 0.016 s (limit 0.15 s), all six samples under 0.017 s; an earlier run 0.046 s at 3 s after start |
| G-CEF gif-anim (24 frames, 50 ms) | gif-cef.json | frame difference at most 1 (0, -1, -1, -1, 0, 0) |
| S-CEF sync fixture | sync-cef.json | 5.1 ms (limit 30 ms) |
| S-WV2 sync fixture | sync-wv2.json | 0.8 ms |
| Y-CEF youtube fixture | youtube-cef.json | 0.022 s (limit 0.5 s), both players playing (network was reachable) |

Every run logged `engine started` twice (`WebView2` or `Cef`) and `renderers started count=2` with both monitors; `tasklist.exe` showed no Rust Sucrose process afterwards; H:\Sucrose was only read (one video copied into `target/`). The CEF runs logged `time base script registered at document start` for both renderers: the mechanism is the DevTools protocol through the in-process browser host, not the fallback.

## Task Commits

1. **Task 1: video page of two WebView2 renderers in step** - `fa3298aa` (feat)
2. **Task 2: GIF, YouTube, CEF time base, Web/URL shim, sync fixture** - `69aaf6af` (feat)
3. **Task 3: FEATURES** - `ccf3735f` (docs)

**Plan metadata:** the commit that adds this SUMMARY and the STATE/ROADMAP update.

## Deviations from Plan

### Auto-fixed Issues

None of Rules 1-3 beyond the following notes.

**1. [Process] Tasks were committed once each, not as RED and GREEN commits**
- The tasks carry `tdd="true"` but the plan is not `type: tdd`. Tests were written first and run once red (the generated-page test failed on the GIF page before it had `SucroseSyncTime`), then went green before each commit. `gsd_run check tdd-red-evidence` was not used (cargo test output is not a format it reads), as in 04-43.

**2. [Interpretation] `document_start_script` lives in `sucrose-engine-web`**
- The "no shim without a time base" unit test sits next to the function both engines call (`page::document_start_script`), because the engine start functions need CEF or WebView2 to run; each engine's start passes its result to the host.

### Line endings

`cargo fmt` rewrote the CRLF files `crates/sucrose-engine-web/src/js.rs`, `crates/sucrose-engine-cef/src/browser.rs` and `engine.rs` to LF; CRLF was restored on those three before the commits. `docs/FEATURES.md` (CRLF, no final newline) was restored the same way after my edit helper wrote LF. The other edited files (`page.rs`, `pages.rs`, the three HTML pages, `tests/pages.rs`, the WebView2 files) are LF and stayed LF; the two new fixture files are LF.

**Total deviations:** 0 auto-fixed, 2 process notes.

## Issues Encountered

None that blocked. `cargo xtask ci` passed (exit 0); no flaky test needed a rerun.

## Not checked / limits

- The G-CEF number alone does not prove the wrap logic acted: both renderers started within a few ms of each other, so two pages that never synced would also start near one frame apart for the first cycles. The mechanism is unit tested (page text) and its arithmetic checked with a scratch bun run (frame 6 at 5 frames + 10 ms, 0 at a whole cycle); a control run with a deliberately late second page was not made.
- A YouTube playlist was not tried (the fixture is a single video). In a playlist the expected position is (now - epoch) modulo the duration of the video shown, so two monitors that show different videos are not made equal and a video may start away from its beginning; FEATURES says "within the video both show".
- Web and URL pages: only the sync fixture was probed; a real site, a CSS-animation theme and the `load`-time alignment of endless animations were not driven. A renderer that receives its first time base after start (a group that grows) gets no shim, only the generated pages follow it.
- The edge check (touching monitors, no seam) is a desktop glance for 04-56.
- DESK-02 is not marked complete here, as the plan says.

## Threat Flags

None. The shim text is built from one integer; the DevTools call is made in process through the browser host and opens no port; DeveloperPort was set only in scratch settings under `target/`.

## Known Stubs

None.

## Next Phase Readiness

04-46 builds the Display Preferences radios; 04-56 holds the Round C checkpoint (the edge glance, the whole DESK-02 set).

## Self-Check: PASSED

- `fn sync_shim_script`, `fn time_base_call`, `AddScriptToExecuteOnDocumentCreated`, `SucroseSyncTime` in the three pages and `D-043` in FEATURES found by grep; the sync fixture files exist.
- Commits `fa3298aa`, `69aaf6af`, `ccf3735f` are ancestors of HEAD; `git rev-list --count` from the plan start gives 3.
- `cargo xtask ci` exit 0; `tasklist.exe` shows no Rust Sucrose process.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-28*
