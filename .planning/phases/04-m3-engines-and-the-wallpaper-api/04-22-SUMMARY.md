---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 22
subsystem: engines
tags: [webview2, webview2-com, web-wallpaper, virtual-origin, reply-planner, com]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "the CEF engine, the virtual origin resolver (origin::resolve), the profile allocator, the availability report and the surface-thread engine seam (04-01..04-21)"
provides:
  - "crate sucrose-engine-webview2: runtime_version(), BUILT, engine::start, WebView2Engine, WebView2Pump, a COM host on the surface thread"
  - "sucrose_engine_web::serve::{plan, Reply, Content}: the one reply planner both web engines stream from"
  - "sucrose_engine::engine::SurfaceHooks (moved from the CEF crate)"
  - "WebView2 detection from the runtime version against the 131.0.2903.70 floor, and the WebView2 arm of start_engine"
affects: [04-23, 04-24, 04-25, 04-27, 04-33, 04-37]

actuals:
  tokens: 20000
  tasks: 2
  commits: 2
plan_head_before: 056770fd4dce3fa45a5ef0a5a32399f63883927d
plan_head_after: ae474433794215f8472573d098530992bef1674b

tech-stack:
  added: ["webview2-com 0.39.1 (with webview2-com-sys and webview2-com-macros, static WebView2Loader)"]
  patterns:
    - "an engine adapter streams what a pure planner decides; it decides nothing itself"
    - "a COM host that is created, used and closed on the surface thread, with its own timed message pump instead of an unbounded wait"

key-files:
  created:
    - crates/sucrose-engine-webview2/Cargo.toml
    - crates/sucrose-engine-webview2/src/lib.rs
    - crates/sucrose-engine-webview2/src/host.rs
    - crates/sucrose-engine-webview2/src/engine.rs
    - crates/sucrose-engine-web/src/serve.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - apps/sucrose/Cargo.toml
    - apps/sucrose/src/render.rs
    - apps/sucrose/src/engines.rs
    - apps/sucrose/tests/portal.rs
    - crates/sucrose-engine/src/engine.rs
    - crates/sucrose-engine/src/lib.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - crates/sucrose-engine-cef/src/scheme.rs
    - crates/sucrose-engine-web/src/lib.rs
    - docs/FEATURES.md
    - docs/ARCHITECTURE.md

key-decisions:
  - "WebView2 answers every request of https://sucrose.wallpaper.engine/ through serve::plan with the roots [theme folder, cache/Content]; a refusal is a 404 with an empty body, a Range request gets the whole file and 200 until 04-23"
  - "The page gets no host object and no message channel (IsWebMessageEnabled and AreHostObjectsAllowed off); developer tools and the context menu only in developer mode, like CEF; every download is cancelled"
  - "Environment creation retries once after 2 s when it fails (a previous renderer's browser process may still hold the folder); a 30 s cap with a pump that keeps dispatching messages replaces webview2-com's unbounded wait_with_pump"
  - "A renderer that could not lease a profile folder runs on <temp>/SucroseWebView2/<pid>, removed at close, so a page never lands in the exe folder"

patterns-established:
  - "Pure helpers of a COM adapter (bounds, scale, header block, stream path) live outside cfg(windows) and are unit tested"

requirements-completed: []

coverage:
  - id: D1
    description: "With Engine.Web = WebViewLive the hello fixture plays in a WebView2 controller that is a child of the desktop surface: engine started engine=WebView2, page loaded url=https://sucrose.wallpaper.engine/index.html status=200, the service reports playing (ENG-02 tracer)"
    requirement: ENG-02
    verification:
      - kind: e2e
        ref: "scratch run target/web-lib: Renderer log 'engine started engine=\"WebView2\"', 'page loaded url=\"https://sucrose.wallpaper.engine/index.html\" status=200'; Service log 'wallpaper status status=Sucrose - playing Sucrose fixture: hello'"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen"
        status: pass
    human_judgment: true
    rationale: "That the wallpaper is actually visible behind the icons is not asserted by a test; the final maintainer checkpoint (04-37) looks at it"
  - id: D2
    description: "CEF answers the virtual origin as before through the moved planner: the traversal fixture gives 17 refusals (11 at info) and no served line names the file behind the junction"
    requirement: TYPE-03
    verification:
      - kind: e2e
        ref: "scratch run, theme traversal with a junction outside: 17 'origin request rejected', 0 served lines naming secret (04-06: 17)"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-web/src/serve.rs tests (6 moved tests)"
        status: pass
    human_judgment: false
  - id: D3
    description: "WebView2 detection from the runtime version: newer than the floor is Installed with the version, the floor or older is RuntimeOutdated, none is RuntimeMissing, no engine is NotInThisBuild; WebView2 and CEF are offered for Web themes only"
    requirement: ENG-06
    verification:
      - kind: unit
        ref: "apps/sucrose/src/engines.rs#webview2_is_installed_with_the_version_of_a_runtime_newer_than_the_floor and 4 more webview2/web-engine tests"
        status: pass
      - kind: e2e
        ref: "Service log 'engines detected ... webview2=Installed { version: Some(\"158.0.4370.45\") }'"
        status: pass
    human_judgment: false
  - id: D4
    description: "The first WebView2 renderer runs on <data_dir>/Web/WebView2 leased through profile::acquire; closing a renderer closes its controller and posts nothing to any window: three theme starts and two closes in a row log no WARN"
    requirement: ENG-02
    verification:
      - kind: e2e
        ref: "Renderer log 'web profile kind=Persistent engine=\"WebView2\" path=...\\Web\\WebView2 slot=0'; target/web-lib/Web/WebView2 exists"
        status: pass
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#switching_web_themes_on_webview2_closes_each_renderer_cleanly"
        status: pass
    human_judgment: true
    rationale: "The Round A CEF close defect was the Shut Down Windows dialog on a real desktop; the test proves the logs and the missing WARN, the maintainer confirms the dialog stays away at 04-37"

duration: 30min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 22: WebView2 plays a Web theme Summary

**WebView2 (the user-installed Runtime behind the static loader) runs as a child of the desktop surface and serves https://sucrose.wallpaper.engine/ through the reply planner it now shares with CEF, chosen per type from a real runtime-version detection.**

## Performance

- **Duration:** 30 min
- **Started:** 2026-11-26T22:12:10Z
- **Completed:** 2026-11-26T22:42:00Z
- **Tasks:** 2 (Task 1 tracer with a commit, Task 2 regression run with no code change of its own)
- **Files modified:** 18 (5 created)

## Accomplishments

- New crate `sucrose-engine-webview2`: `runtime_version()` (loader call, no WebView created), a COM host (environment with the leased profile, controller under the surface, raw-pixel bounds with the monitor's DPI scale, black background, WebResourceRequested filter on the one origin, downloads cancelled, no host object, no message channel), the engine handle and the surface pump. Non-Windows builds compile to a stub (`BUILT` false, start answers Unsupported).
- `sucrose_engine_web::serve` holds `plan`, `Reply` and `Content` (moved from the CEF crate with their tests); CEF's `scheme.rs` keeps `register` and the streaming. `SurfaceHooks` moved to `sucrose_engine`.
- The service detects WebView2 (found version vs 131.0.2903.70), offers it for Web themes only, and the renderer starts it. Tracer run on the scratch library: the hello fixture loads with status 200 on the persistent `Web/WebView2` profile and the service reports Playing.
- CEF regression: the traversal fixture gives exactly the Round A result.

## Task Commits

1. **Task 1: hello fixture plays on WebView2** - `5379858a` (feat)
2. **Task 2: CEF through the moved planner and hooks** - no separate commit: the move itself is Task 1 step 3 (its files list names scheme.rs and serve.rs), and Task 2 is the regression run; the tests it asks for are the moved tests in `serve.rs`. Verified, see below.
3. **Tests for the new behavior** - `ae474433` (test): the engine-change portal test and the two-switch WebView2 close test.

**Plan metadata:** the docs commit that carries this file.

## Verification

- `cargo xtask ci`: exit 0 (fmt, clippy -D warnings, 46 test result lines all ok).
- Task 1 automated chain, run step by step: engine-web, engine-cef and sucrose engines tests pass; `cargo build -p sucrose`; hello fixture with `Engine.Web = WebViewLive`: `engine started engine="WebView2"` (1), `page loaded url="https://sucrose.wallpaper.engine/index.html" status=200` (1), `web profile ... engine="WebView2"` (1), `target/web-lib/Web/WebView2` exists. The service logged `engines detected ... webview2=Installed { version: Some("158.0.4370.45") }` (maintainer's runtime; newer than the floor).
- Task 2 chain: CEF traversal fixture with a junction behind `outside`: **17** `origin request rejected` (11 at info), 0 served lines naming `secret`; 04-06 measured 17 (11 at info). Bound was 14.
- Acceptance greps: `sucrose_engine_webview2::engine::start` in render.rs (1), `serve::plan` in host.rs (1), `pub fn plan` in serve.rs (1), `runtime_version` in engines.rs (1+), `webview2-com = ` in Cargo.toml (1), `serve::` in scheme.rs (1), CRLF still on Cargo.toml, cef engine.rs and render.rs.
- A7 checked: `webview2-com-sys` declares `link(name = "WebView2LoaderStatic", kind = "static")` for msvc and its build.rs only sets the search path.

## Files Created/Modified

See the front matter. Line endings: `git ls-files --eol` was recorded before the first edit; the editing helper of this run dropped CRs, so CRLF was restored (`sed`) before each commit on the files that were w/crlf: Cargo.toml, apps/sucrose/Cargo.toml, apps/sucrose/src/render.rs, crates/sucrose-engine-cef/src/engine.rs, crates/sucrose-engine-cef/src/scheme.rs, crates/sucrose-engine-web/src/lib.rs, crates/sucrose-engine/src/engine.rs (docs/FEATURES.md and docs/ARCHITECTURE.md were patched CRLF-preserving). `crates/sucrose-engine/src/lib.rs` was not recorded before the edit and is LF now (its earlier ending is unknown; git normalizes both). `apps/sucrose/src/engines.rs`, `apps/sucrose/tests/portal.rs` and the new files are LF. `.planning/STATE.md` was already modified before this plan started and is not part of the plan's commits except through the final state update.

## Decisions Made

See `key-decisions`. No new DECISIONS.md entry: D-14, D-33 and D-35 were settled.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] An existing portal test asserted the old fallback**
- **Found during:** Task 1 (full `cargo test -p sucrose`)
- **Issue:** `an_engine_change_restarts_the_running_wallpaper_of_that_type` expected "WebView2 is not in this build, so the new choice falls back to CEF"; with WebView2 playing, no fallback exists any more.
- **Fix:** renamed to `..._on_the_engine_chosen`; it now expects the second renderer to start on WebView2 (skips without WebView2). The fallback itself stays covered by the engines unit tests.
- **Files modified:** apps/sucrose/tests/portal.rs
- **Commit:** ae474433

**2. [Rule 2 - Missing critical functionality] Test of the close truth**
- **Found during:** Task 1 (the plan's must-have "two theme switches in a row log the close and leave no WARN" has no step in its verify command)
- **Fix:** added `switching_web_themes_on_webview2_closes_each_renderer_cleanly` (real service, three Uses, at least two `webview2 closed`, every renderer on WebView2, no WARN).
- **Commit:** ae474433

**3. [Rule 3 - Blocking] Log target of the planner moved, so the Task 2 verify filter hid the debug lines**
- **Found during:** Task 2
- **Issue:** `SUCROSE_LOG="info,sucrose_engine_cef=debug"` printed 11 refusals, not 17: the planner now logs from `sucrose_engine_web::serve`, so its debug lines (not-a-file refusals, served files) need that target too.
- **Fix:** the run used `info,sucrose_engine_cef=debug,sucrose_engine_web=debug`; the answers were never different, only the filter. Nothing weakened.
- **Commit:** none (run command only)

**4. [Rule 2 - hardening within Task 1] WebView2 settings**
- The plan names the prohibitions (no host object, no message channel) but not the settings that enforce them; the host sets `IsWebMessageEnabled` and `AreHostObjectsAllowed` to false, the status bar off, and devtools and context menu on only in developer mode (as CEF).

**5. [Plan wording] `wait_with_pump` replaced**
- `webview2_com::wait_with_pump` blocks in `GetMessageA` with no timeout; the plan asks for a 30 s cap, so the host has its own pump (`MsgWaitForMultipleObjects` + `PeekMessageW`) that also re-posts a `WM_QUIT` it meets.

**Total deviations:** 5 (1 bug, 2 missing critical, 1 blocking, 1 wording). **Impact:** none on scope; the plan's files list grew by apps/sucrose/tests/portal.rs.

## Issues Encountered

- `file().text()` of the REPL helper strips CRs, which turned CRLF files into LF at first; found by `git ls-files --eol` before any commit and repaired (see Files).
- The Round A close defect cannot be seen from automation; `taskkill` of the service kills renderers through the job object, so a close is only exercised through a service that sends Close (the new integration test does).

## Known Stubs

- `crates/sucrose-engine-webview2/src/engine.rs` (`WebView2Pump::drain`): pause, volume, loop, shuffle and stretch commands are kept in state and logged at debug only; intentional, applied by 04-24.
- `crates/sucrose-engine-webview2/src/host.rs` (`answer`): a `Range` request gets the whole file and 200; intentional, 04-23 adds partial replies and the WebView2 traversal proof.
- `SurfaceHooks::on_data_wanted` and `takes_input`/`forward_input`/`open_devtools` are not wired for WebView2; 04-24 and later.

## Authentication Gates

None.

## Threat Flags

None beyond the plan's register: T-04-50 (every request through serve::plan, the adapter decides nothing; its traversal run on WebView2 is 04-23) and T-04-51 (no host object, no web message channel, downloads cancelled) are implemented.

## Next Phase Readiness

04-23 can add Range replies and the WebView2 traversal fixture on top of `serve::plan` and `host::answer`; 04-24 applies the queued commands and the page calls. Open for the maintainer at 04-37: look at the wallpaper on screen and the absence of the Shut Down dialog on switches.

## Self-Check: PASSED

Created files exist on disk; commits 5379858a and ae474433 are ancestors of HEAD.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*
