---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 34
subsystem: engines
tags: [url, theme-gate, address, cef, webview2, input, d-06, type-04]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-33 generated pages and PageKind, 04-25 WebView2 input forwarding, 04-24 WebView2 page calls, 04-02 theme gate"
provides:
  - "sucrose_core::theme::ThemeSource::{File, Address}, SelectedTheme.source (replaces source_path), YouTubeIds and youtube_ids (the YouTube half of the gate for 04-35)"
  - "The address gate for Url and YouTube themes: absolute http or https with a host, at most 2048 bytes, no whitespace or control characters; each refusal has its own reason"
  - "Url in the service's PLAYABLE and in both web engines' availability; CEF and WebView2 navigate a Url assignment straight to its address"
  - "page::PageKind::{Url, for_wallpaper, is_theme, takes_input}: a Url page gets no property, compatible or data calls"
  - "moves::MoveSlot: mouse-move coalescing that keeps the order of every other input event"
  - "apps/sucrose/tests/fixtures/web-themes/url (Type 1, https://example.com/)"
affects: [04-35, 04-37]
actuals:
  tokens: 10570
  tasks: 2
  commits: 3
plan_head_before: 6164edc8ac345c971f872302847977e9d3a1e318
plan_head_after: 8e88317226e439bd31304356a34e8406d8c479f8
tech-stack:
  added: []
  patterns:
    - "A theme's source is typed (file inside the folder or web address) and decided by the theme type; no untyped path-or-URL string travels"
    - "A web engine's page kind is decided in one function (PageKind::for_wallpaper) shared by CEF and WebView2: calls, system data and input all follow it"
    - "Coalesced mouse moves share a slot plus one queue marker; an event of another kind queues the waiting move ahead of itself and the pump skips the orphaned marker"
key-files:
  created:
    - crates/sucrose-engine-web/src/moves.rs
    - apps/sucrose/tests/fixtures/web-themes/url/SucroseInfo.json
  modified:
    - crates/sucrose-core/src/theme.rs
    - crates/sucrose-core/tests/theme.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/engines.rs
    - apps/sucrose/tests/portal.rs
    - crates/sucrose-engine-web/src/page.rs
    - crates/sucrose-engine-web/src/lib.rs
    - crates/sucrose-engine-web/tests/page.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - crates/sucrose-engine-webview2/src/engine.rs
    - .planning/REQUIREMENTS.md
    - docs/FEATURES.md
key-decisions:
  - "Promote the theme source to ThemeSource::{File, Address} instead of a path-or-maybe-URL string (the plan's assumption_delta_decision)"
  - "A repeated v or list parameter is not an id (the legacy reader joins repeated values with a comma); ids are not percent-decoded, so an encoded id is refused"
  - "A Url page keeps the in-page Light pause (media elements paused, visibility hidden); it is not part of the wallpaper API and the only pause there is until Round D adds the process freeze"
  - "takes_input is decided by the page kind: Web and Url true, generated Gif and Video false (legacy: input only for Web, URL and Application); Round A returned true for every page"
requirements-completed: []
duration: 20 min
completed: 2026-11-27
status: complete
coverage:
  - id: D1
    description: "A URL theme plays on CEF: the engine navigates straight to the Source address, the renderer logs page loaded with that address and status 200, and the service logs engine chosen kind=Url"
    requirement: TYPE-04
    verification:
      - kind: integration
        ref: "sucrose.exe on target/web-lib (synth-library --select url): page loaded url=https://example.com/ status=200; engine chosen kind=Url engine=Cef"
        status: pass
    human_judgment: true
    rationale: "The real page was not looked at; the maintainer checkpoint 04-37 checks a URL wallpaper with input"
  - id: D2
    description: "A URL theme plays on WebView2 (Url=WebViewLive) with the same navigation, and the renderer registers raw mouse and keyboard input for it"
    requirement: ENG-03
    verification:
      - kind: integration
        ref: "sucrose.exe on target/web-lib with --engine Url=WebViewLive: page loaded status=200; engine chosen kind=Url engine=WebView2; raw input registered mouse=true keyboard=true"
        status: pass
    human_judgment: true
    rationale: "Forwarded input was registered but not exercised with a real mouse and keyboard; the maintainer checkpoint 04-37 does"
  - id: D3
    description: "The theme gate accepts for Url and YouTube only an absolute http or https address with a host (at most 2048 bytes, no whitespace) and refuses every other scheme, a relative text and an address without a host with its own reason; every other type keeps the file rules"
    requirement: TYPE-04
    verification:
      - kind: unit
        ref: "crates/sucrose-core/tests/theme.rs#a_url_theme_refuses_every_other_scheme_a_relative_text_and_an_address_without_a_host_each_with_its_own_reason"
        status: pass
      - kind: unit
        ref: "crates/sucrose-core/tests/theme.rs#every_type_takes_the_source_its_kind_decides"
        status: pass
    human_judgment: false
  - id: D4
    description: "A YouTube theme needs a valid v or list id; youtube_ids gives the ids for the page of 04-35"
    requirement: TYPE-05
    verification:
      - kind: unit
        ref: "crates/sucrose-core/tests/theme.rs#a_youtube_theme_needs_a_v_or_a_list_id_in_its_address"
        status: pass
      - kind: unit
        ref: "crates/sucrose-core/tests/theme.rs#youtube_ids_reads_the_v_and_list_parameters_as_the_legacy_app_does"
        status: pass
    human_judgment: false
  - id: D5
    description: "A URL page gets no property, compatible or data calls (only the Light pause), and Web and Url pages take forwarded input while generated Gif and Video pages do not"
    requirement: WAPI-07
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/page.rs#a_url_page_gets_no_property_compatible_or_data_calls_even_beside_theme_files"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-web/tests/page.rs#the_page_kind_of_a_wallpaper_decides_its_pages_and_whether_it_takes_input"
        status: pass
    human_judgment: false
  - id: D6
    description: "Forwarded input keeps its order: only consecutive mouse moves are coalesced, a click, key or wheel event stays between the moves around it"
    requirement: TYPE-04
    verification:
      - kind: unit
        ref: "crates/sucrose-engine-web/src/moves.rs#a_click_between_two_moves_keeps_its_place_and_both_moves"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-cef/src/engine.rs#a_click_keeps_its_place_between_the_moves_around_it"
        status: pass
      - kind: unit
        ref: "crates/sucrose-engine-webview2/src/engine.rs#a_click_keeps_its_place_between_the_moves_around_it"
        status: pass
    human_judgment: false
  - id: D7
    description: "Two URL themes with the same address are two themes: Use of the other folder restarts the renderer, only the same folder is AlreadyApplied; Url is in the service's playable types"
    requirement: TYPE-04
    verification:
      - kind: unit
        ref: "apps/sucrose/src/service.rs#two_url_themes_with_the_same_address_are_two_themes_and_only_the_same_folder_is_already_applied"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/service.rs#url_is_playable_and_its_job_carries_the_address_as_the_source"
        status: pass
    human_judgment: false
  - id: D8
    description: "REQUIREMENTS.md TYPE-04 and PROP-05 read as D-06 asks; FEATURES rows for URL and input forwarding record the Round B behavior"
    requirement: PROP-05
    verification: []
    human_judgment: true
    rationale: "Wording of requirements and docs; a reader judges it"
---

# Phase 4 Plan 34: URL wallpapers Summary

**URL themes play on CEF and WebView2 through a typed ThemeSource and an http/https-only address gate, with forwarded input, no API calls, and a mouse-move coalescer that no longer reorders clicks.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-11-27T03:28:20Z
- **Completed:** 2026-11-27T03:47:30Z
- **Tasks:** 2
- **Files modified:** 14 (969 insertions, 127 deletions)

## Accomplishments

- `SelectedTheme.source` is a `ThemeSource` (a verified file or a verified address). Url and YouTube Sources pass an address gate with a reason per class of refusal: empty, over 2048 bytes, whitespace or control characters, not absolute, not http or https, no host; YouTube also needs a valid `v` or `list` id. An invariant test checks every wallpaper type against a file and an address.
- Url is playable: the service's `PLAYABLE`, `availability_for` for both web engines, and both engines' `start` (navigate to the address as given; the page kind `PageKind::Url` runs no property, compatible or data calls).
- Live runs on the fixture library: CEF and WebView2 each logged `page loaded url="https://example.com/" status=200` and `engine chosen kind=Url`; the WebView2 run also logged `raw input registered mouse=true keyboard=true`.
- Input ordering: `MoveSlot` keeps one move per burst and queues the waiting move ahead of any other event, skipping the orphaned marker. `takes_input` follows the page kind (Web and Url yes, generated Gif and Video no).
- REQUIREMENTS.md TYPE-04 and PROP-05 and the FEATURES URL and input rows carry the D-06 wording and the Round B behavior.

## Task Commits

1. **Task 1: A URL theme plays on CEF (tracer)** - RED `720c6d8b` (test: gate and ThemeSource tests, failing on their assertions), GREEN `270b325b` (feat: gate, Url playable, CEF and WebView2 navigation, fixture)
2. **Task 2: URL on WebView2 with input, the TYPE-04 edges, and the D-06 wording** - `8e883172` (feat)

**Plan metadata:** the docs commit that carries this SUMMARY.

## Files Created/Modified

- `crates/sucrose-core/src/theme.rs` - ThemeSource, YouTubeIds, youtube_ids, the address gate
- `crates/sucrose-core/tests/theme.rs` - gate tests, the type table
- `apps/sucrose/src/service.rs`, `engines.rs` - job_for, PLAYABLE, availability, tests
- `apps/sucrose/tests/portal.rs` - the refusal test now uses a YouTube theme
- `crates/sucrose-engine-web/src/page.rs`, `tests/page.rs` - PageKind::Url, for_wallpaper, takes_input
- `crates/sucrose-engine-web/src/moves.rs`, `lib.rs` - MoveSlot
- `crates/sucrose-engine-cef/src/engine.rs`, `crates/sucrose-engine-webview2/src/engine.rs` - Url navigation, input per page kind, ordered input
- `apps/sucrose/tests/fixtures/web-themes/url/SucroseInfo.json` - URL fixture
- `.planning/REQUIREMENTS.md`, `docs/FEATURES.md` - wording

## Decisions Made

See `key-decisions` above. The gate refuses an address the legacy app would have opened (for example one with a space); this is the plan's rule.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Input ordering: a later move was delivered ahead of an earlier click**
- **Found during:** Task 2 (the TYPE-04 ordering truth)
- **Issue:** The Round A coalescer kept one pending move and one marker. A move, a click and another move queued marker, click; the pump then delivered the newest move before the click.
- **Fix:** New `sucrose_engine_web::moves::MoveSlot` (a file the plan did not list) used by both engines: another event flushes the waiting move ahead of itself and the older marker is skipped.
- **Files modified:** crates/sucrose-engine-web/src/moves.rs, lib.rs, both engines
- **Verification:** MoveSlot unit tests, one queue-order test per engine
- **Committed in:** 8e883172

**2. [Rule 3 - Blocking] PageKind::Url, for_wallpaper, takes_input in page.rs**
- **Found during:** Task 1
- **Issue:** The plan allowed a PageKind::Url or a skip at the call site; one shared function keeps both engines identical (calls, system data, input).
- **Files modified:** crates/sucrose-engine-web/src/page.rs, tests/page.rs
- **Committed in:** 270b325b

**3. [Rule 3 - Blocking] Tests that assumed Url is not playable**
- **Found during:** Task 1
- **Issue:** `engines.rs` (web engines do not play Url), the service test `maps_the_real_gate...` and `tests/portal.rs` (`use_of_a_url_theme_is_refused...`, which would now start a real renderer) encoded the old rule.
- **Fix:** Updated the first two to the new rule; the portal test now writes a YouTube theme into its scratch library and checks that refusal (renamed `use_of_a_youtube_theme_is_refused_and_the_selection_is_unchanged`).
- **Committed in:** 270b325b

**4. [Rule 3 - Blocking] The URL fixture needed Thumbnail and Preview keys**
- **Found during:** Task 1 live run
- **Issue:** `synth-library` counted the first version of the fixture as a corrupt folder (the library scan needs the two images; the tool writes them from the pool when the keys exist).
- **Fix:** Added `Thumbnail` and `Preview` like the other fixtures.
- **Committed in:** 270b325b

**5. [Rule 1 - Bug] A race in my own test helper**
- **Found during:** Task 1 GREEN
- **Issue:** The `refusal` helper used one scratch folder name, so parallel tests overwrote each other's theme.
- **Fix:** The folder name carries the thread id.
- **Committed in:** 270b325b

### Process notes

- **TDD:** the Rust test output is not a format `gsd_run check tdd-red-evidence` reads, so the RED evidence is the failing run of `cargo test -p sucrose-core --test theme` after the stubs (ThemeSource, a `youtube_ids` that returned `None`): 7 tests failed on their planned assertions (an address reported as "an address, not a file", the ids not found), 21 passed. Task 2's engine tests were written together with the code.
- **WebView2 navigation landed in the Task 1 commit** because the availability change makes both web engines play Url at once.
- **Line endings:** my first edits through the REPL helper wrote `theme.rs` and `service.rs` as LF; I restored CRLF on both before the first commit. After each `cargo fmt` I restored CRLF on every file that was CRLF before (the snapshot of `git ls-files --eol` taken at the start). New files (`moves.rs`, the fixture's folder aside from its CRLF text) were left as written; `REQUIREMENTS.md` stayed LF; `docs/FEATURES.md` kept CRLF and no final newline.

**Total deviations:** 5 auto-fixed (2 Rule 1, 3 Rule 3).
**Impact on plan:** All needed for the plan's own truths; no scope creep beyond one small new module.

## Issues Encountered

- CEF's GPU process logs crashes and restarts during a run (`GPU process exited unexpectedly`); the `hello` Web fixture shows the same lines on this machine, so it is not caused by this plan.
- No flaky test failed in `cargo xtask ci`.

## Authentication Gates

None.

## Known Stubs

None.

## Threat Flags

None. The new surface is the navigation to a theme-supplied address, which T-04-72 and T-04-73 cover (the gate; no API calls and no host objects on the page).

## Requirement status

`requirements-completed: []` as the plan says. TYPE-04, ENG-02, ENG-03 complete at 04-37 (the maintainer checks a URL wallpaper with input); TYPE-05 completes at 04-37 after the YouTube page of 04-35; WAPI-07 stays partial (Application is Round E).

## Next Phase Readiness

04-35 can build the YouTube page from `youtube_ids` and the gate that already accepts YouTube addresses; YouTube is still not in `PLAYABLE`. Real forwarding of mouse and keyboard to a URL page was registered but not exercised with a real mouse and keyboard.

## Self-Check: PASSED

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-27*
