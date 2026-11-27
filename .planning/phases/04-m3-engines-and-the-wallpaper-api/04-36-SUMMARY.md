---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 36
subsystem: portal
tags: [portal, wallpaper-settings, engine-select, gif, url, youtube, ui-spec-e1, d-10, d-11]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-13 engine rows and SettingChange::{GifEngine, UrlEngine, YouTubeEngine}, 04-30 built-in GIF renderer, 04-34/04-35 Url and YouTube playable in the service"
provides:
  - "EngineRow::{Gif, Url, YouTube} and the Gif Player, Url Engine and YouTube Engine rows on the Wallpaper page in the legacy order"
  - "The two legacy built-in GIF names (VexanaLive, XavierLive) shown as one Built-in choice, written as XavierLive"
  - "Integration proof that a Gif engine change moves a running GIF from mpv to the built-in renderer and that libVLC falls back without rewriting the setting"
affects: [04-37]
actuals:
  tokens: 9500
  tasks: 2
  commits: 3
plan_head_before: 10db4f4912da8ce52540b2906fa77754741ca961
plan_head_after: 8c09175a259c535c49665b7c0cf05b2d8cfdb0a1
tech-stack:
  added: []
  patterns:
    - "A select list drops engine names that alias another (VexanaLive into XavierLive) so one renderer is one choice"
    - "write_gif_theme test helper builds a Type 0 theme from the sucrose-engine-gif animation fixture"
key-files:
  created: []
  modified:
    - apps/sucrose/tests/portal.rs
    - apps/sucrose/tests/common/mod.rs
    - apps/sucrose-portal/src/settings/wallpaper.rs
    - apps/sucrose-portal/src/library/card.rs
    - apps/sucrose-portal/src/i18n.rs
    - docs/FEATURES.md
key-decisions:
  - "Select order follows each type's legacy allowed list (Gif: Built-in, WebView2, CEF, mpv, libVLC), the same rule the Video row already used, not the order the plan prose names (mpv before WebView2)"
  - "A stored VexanaLive is shown as the Built-in choice (XavierLive) and VexanaLive never appears as a second entry, so the row alert does not read the missing Vexana state"
requirements-completed: []
duration: 21 min
completed: 2026-11-27
status: complete
coverage:
  - id: D1
    description: "A Gif engine change from a Portal client restarts a running GIF from mpv on the built-in renderer with no fallback; libVLC keeps the stored value and reports the fallback to the built-in renderer"
    requirement: ENG-07
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs#a_gif_engine_change_moves_the_running_gif_to_the_built_in_renderer (cargo test -p sucrose --test portal -- --include-ignored, passed in 9 s)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The Wallpaper page shows Gif Player and Video Player in Extensions and Url, Web and YouTube Engine in Engines with the legacy keys, the report's choices and the Not installed suffix"
    requirement: TYPE-02
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/settings/wallpaper.rs tests wallpaper_the_rows_follow_the_legacy_page_order_and_each_one_owns_its_type, wallpaper_the_gif_player_*, wallpaper_the_video_url_and_youtube_rows_list_what_the_report_offers"
        status: pass
    human_judgment: true
    rationale: "The capture shows the rows; the maintainer checkpoint 04-37 judges the page and the toast copy"
  - id: D3
    description: "Gif, Url and YouTube cards are usable when the service reports them; Application cards stay not supported yet"
    requirement: TYPE-04
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/card.rs#gif_url_and_youtube_themes_are_usable_once_the_service_reports_them_and_application_themes_are_not"
        status: pass
    human_judgment: false
---

# Phase 4 Plan 36: Gif, Url and YouTube engine rows Summary

**The Wallpaper page now has the Gif Player, Url Engine and YouTube Engine rows in the legacy order, with a stored VexanaLive shown as the single Built-in choice, and an integration test proves a Gif engine change restarts a running GIF on the built-in renderer.**

## Performance

- **Duration:** 21 min
- **Tasks:** 2 (a tracer and a TDD task)
- **Commits:** 3 task commits
- **Files modified:** 6

## Accomplishments

- Task 1 (tracer): `a_gif_engine_change_moves_the_running_gif_to_the_built_in_renderer` starts the real service on a scratch folder with Engine.Gif = MpvPlayerLive, plays a GIF theme (`write_gif_theme`), sends `SettingChange::GifEngine(XavierLive)` as a Portal does and sees the second renderer start as `BuiltInGif` with no Gif fallback in the last engine report; then `GifEngine(VlcPlayerLive)` stores VlcPlayerLive (D-11) and the report names the fallback (configured VlcPlayerLive, playing XavierLive). It is `#[ignore]` because it plays a wallpaper on the desktop; it passed in 9 s with `--include-ignored` and left no Sucrose process.
- Task 2: `EngineRow` now has Gif, Url and YouTube (type, allowed list, stored value, change, legacy title keys, alert and install ids). `ALL` is `[Gif, Video, Url, Web, YouTube]` and the page draws Gif and Video in Extensions and Url, Web and YouTube in Engines. Each row has the same row alert and install action as Video and Web; the fallback toast needed no change (it is already generic).
- Capture (not committed): `target/web-lib/portal-slice3-rows.png`, the Wallpaper page of a fixture library at a maximized 1936x1048 window. Extensions holds Gif Player (mpv, the legacy default) above Video Player (mpv); Engines holds Url Engine, Web Engine and YouTube Engine, each on CEF with the legacy descriptions ("Select the plugin that will play Gif-themed wallpapers in your library." and the engine variants). No row alert shows because every chosen engine is installed.

## Task Commits

1. **Task 1: Gif engine change integration test** - `8e79a37f` (test)
2. **Task 2 RED: failing tests for the new rows** - `32552a06` (test; includes the EngineRow variants so the tests fail on assertions, not on compilation; the first run before that was a compile failure and is not counted as RED)
3. **Task 2 GREEN: the three rows, FEATURES row** - `8c09175a` (feat)

**Task 1 TDD note:** the tracer test passed on its first real run, because the service has restarted any type on an engine change since 04-13/04-30. It is a characterization of the full chain (unexpected green investigated: the chain exists, this plan only proves it for Gif). A first run hung in a drain loop of the test itself (Memory events never stop, so a read-until-silence never ends); the loop now ends at a 3 s deadline.

## Files Created/Modified

- `apps/sucrose/tests/portal.rs` - the Gif engine change test
- `apps/sucrose/tests/common/mod.rs` - `write_gif_theme`
- `apps/sucrose-portal/src/settings/wallpaper.rs` - the three rows, VexanaLive aliasing, eight tests
- `apps/sucrose-portal/src/library/card.rs` - one test for the Round B types
- `apps/sucrose-portal/src/i18n.rs` - the six legacy keys of the new rows join the required key list
- `docs/FEATURES.md` - row "Engine names and fallback" (the plan names row 159; the row is at line 160)

## Decisions Made

- Order of the Gif select: the legacy allowed list (Built-in, WebView2, CEF, mpv, libVLC), as the Video row already does. The plan prose says "Built-in, mpv, WebView2, CEF" while also saying "in the legacy order"; the two disagree and the legacy order was kept. Switching to the UI-SPEC order (mpv first) would be one reordering of `EngineType::GIF`/`VIDEO` in sucrose-core.
- VexanaLive is never a list entry; a stored VexanaLive selects the Built-in entry.

## Deviations from Plan

**1. [Rule 3 - Blocking] i18n.rs key list**
- **Found during:** Task 2
- **Issue:** the Portal checks a required-key list in `apps/sucrose-portal/src/i18n.rs`; the new rows' keys were not in it. The file is not in the plan's `files_modified`.
- **Fix:** six keys added (Gif, Url and YouTube title and description).
- **Commit:** `8c09175a`

**2. [Rule 1 - Bug] Duplicate Built-in choice for a stored VexanaLive**
- **Found during:** Task 2 (behavior review)
- **Issue:** the allowed Gif list holds both VexanaLive and XavierLive while the service reports only XavierLive; a stored VexanaLive would list "Built-in" twice and read as missing.
- **Fix:** see Decisions Made; covered by `wallpaper_the_gif_player_shows_the_two_legacy_built_in_names_as_one_choice`.

**3. Card change is tests only.** `use_state` already follows the service's `supported` list, so no production change in card.rs was needed; the plan listed the file.

**4. Capture window.** The Portal window is a fixed 900x600 and the new rows sit below the fold; without synthesized input the first capture showed only the top. A one-off inline PowerShell `ShowWindowAsync(SW_MAXIMIZE)` on my own Portal process (a scratch script under the scratchpad, nothing in the repository) preceded the unchanged capture.ps1. No mouse or keyboard input was sent.

**Total deviations:** 4 (1 blocking, 1 bug, 2 notes). **Impact:** none on scope.

## Issues Encountered

- The first test run hung for 9 minutes in the test's own drain loop; the stuck `sucrose.exe` processes were stopped with `taskkill.exe /IM sucrose.exe /F`.
- Line endings: working copies of the edited files, except `docs/FEATURES.md`, are LF and stayed LF; `docs/FEATURES.md` stayed CRLF (the edit was inside one line). No file needed a CRLF restore; `cargo fmt` changed none of them.
- No flaky test failed in `cargo xtask ci` (exit 0).

## Known Stubs

None.

## Threat Flags

None. The service still validates every engine choice against the legacy allowed list (T-04-76); the Portal sends the same SettingChange variants from 04-13.

## Next Phase Readiness

04-37 (the final Round B checkpoint) can check the Wallpaper page rows, the Gif fallback toast copy and the select order. ENG-07, TYPE-02, TYPE-04 and TYPE-05 are not marked here (`requirements-completed: []`).

## Self-Check: PASSED

- Files: apps/sucrose/tests/portal.rs, apps/sucrose/tests/common/mod.rs, apps/sucrose-portal/src/settings/wallpaper.rs, apps/sucrose-portal/src/library/card.rs, apps/sucrose-portal/src/i18n.rs, docs/FEATURES.md exist.
- Commits 8e79a37f, 32552a06 and 8c09175a are on develop-rust; `git rev-list --count 10db4f49..HEAD` was 3 before this SUMMARY.
- Acceptance: `grep -c GifPlayer apps/sucrose-portal/src/settings/wallpaper.rs` >= 1, `grep -c YouTubePlayer ...` >= 1, wallpaper.rs w/lf, FEATURES.md w/crlf; no Rust Sucrose process after the tests.
