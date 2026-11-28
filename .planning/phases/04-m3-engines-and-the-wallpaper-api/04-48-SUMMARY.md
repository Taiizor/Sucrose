---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 48
subsystem: properties-editor
tags: [properties, editor, ipc, working-copy, cef, gpui, second-window, spike, prop-01, prop-02]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-47: the card badges and the card render (CardData); 04-43/04-45: IPC versions 13 and 11; Round A/B: sucrose_core::properties, js::property_call, the CEF page-call queue"
provides:
  - "paths: properties_dir and legacy_properties_cache_dir (D-20)"
  - "IPC: PortalRequest::{OpenProperties, ChangeProperty, CloseProperties}, PortalEvent::{Properties, PropertiesBlob, PropertiesFailed}, PropertiesView, PropertiesFailure, MAX_PROPERTY_CONTROL, ToRenderer::ApplyProperty, blob::property_chunks"
  - "Engine::apply_property; CefEngine::apply_property; js::apply_property_call"
  - "service/properties.rs: PropertiesHost (D-20 import, working copy, change validation, debounced atomic write)"
  - "app/editor.rs: PropertiesEditor, open_editor, editor_origin, taskbar_edge, placement, SendGate, the slider control"
  - "The Customize icon on Web cards and the measurement-only switch SUCROSE_MEASURE_EDITOR"
affects: [04-49, 04-50, 04-51, 04-52, 04-53, 04-54, 04-55, 04-56, PROP-01, PROP-02]
requirements-completed: []
actuals:
  tokens: 33000
  tasks: 3
  commits: 3
plan_head_before: d954f1a0ac9869ddad2741cf4fb1196cad9f212e
plan_head_after: 55efef46c599eb973180184aa5f51e76455f3deb
tech-stack:
  added: []
  patterns: ["a host with no sockets or processes (PropertiesHost): the clock is a parameter, so the 300 ms debounce and a fast drag are tested with a fake clock", "only the value of a change is taken from the editor; type, bounds, texts and items stay the working copy's", "a property change is a marker in the CEF command queue and the controls wait in State (the command is logged, a passwordbox must never be)"]
key-files:
  created:
    - apps/sucrose/src/service/properties.rs
    - apps/sucrose-portal/src/app/editor.rs
  modified:
    - crates/sucrose-core/src/paths.rs
    - crates/sucrose-core/src/properties.rs
    - crates/sucrose-ipc/src/portal.rs
    - crates/sucrose-ipc/src/protocol.rs
    - crates/sucrose-ipc/src/blob.rs
    - crates/sucrose-engine/src/engine.rs
    - crates/sucrose-engine-web/src/js.rs
    - crates/sucrose-engine-web/tests/js.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/main.rs
    - apps/sucrose-portal/src/library/card.rs
    - apps/sucrose-portal/src/library/mod.rs
    - apps/sucrose-portal/src/assets.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/locales/sucrose.en.yml
    - docs/FEATURES.md
key-decisions:
  - "The service takes only the value of a ChangeProperty: the stored control keeps its type, bounds, step, texts and items, a number is held inside its bounds, a dropdown index outside its list is refused, a label, button or unknown type is not editable (T-04-104)"
  - "The working copy is held in memory from OpenProperties and written (temporary file, sync, rename) 300 ms after the first change that follows a write, so a drag writes at most every 300 ms; opening writes nothing and an unreadable existing copy is never replaced by the theme's file"
  - "The D-20 import runs on a worker at the first property request into a staging folder that is renamed to Properties at the end (no half import after a crash); requests that arrive meanwhile wait and run in order"
  - "MAX_PROPERTY_CONTROL is 60 KiB, not 64 KiB, because one request is one frame (MAX_FRAME is 64 KiB) with the folder and key texts"
  - "The editor window is a normal, fixed-size window (480 px logical start height, between 320 px and the work area minus 32 px); the content-fitting height follows with the other controls"
patterns-established:
  - "A second GPUI window is opened with gpui_kit::open_window, remembered in a Global slot (window handle plus a weak entity); the main window's events reach it through handle.update"
  - "The app ends when only the editor window is left (app::editor::only_editor_left in the on_window_closed handler)"
status: complete
coverage:
  - id: D1
    description: "A slider change in the editor reaches the running Web wallpaper over IPC: ChangeProperty, the working copy, ApplyProperty to every renderer of the theme, the PropertyListener call on CEF (PROP-02)"
    requirement: PROP-02
    verification:
      - kind: integration
        ref: "apps/sucrose/tests/portal.rs a_property_change_reaches_the_running_web_wallpaper (ignored test, run for this plan: ok in 6.3 s; two renderers logged property applied key=amount)"
        status: pass
      - kind: unit
        ref: "sucrose-engine-web tests/js.rs an_applied_property_*; sucrose-engine-cef engine::tests a_property_change_waits_for_the_surface_thread_and_the_newest_control_of_a_key_wins"
        status: pass
    human_judgment: false
  - id: D2
    description: "The service is the only writer of property files; the working copy is written atomically at most every 300 ms and on close; the theme's own file and the C# folder are never written (Pattern 6, D-20)"
    requirement: PROP-02
    verification:
      - kind: unit
        ref: "service::properties::tests (15 tests: import, once, missing folder, open, refusals, only the value, fast drag, close, legacy text form)"
        status: pass
      - kind: integration
        ref: "the integration test compares the theme file byte for byte before and after and finds Properties/callsBase holding 2.5"
        status: pass
    human_judgment: false
  - id: D3
    description: "Properties travel as PropertiesBlob chunks (a document of BLOB_MAX round-trips) and a missing or unreadable file gives PropertiesFailed"
    requirement: PROP-01
    verification:
      - kind: unit
        ref: "sucrose-ipc blob::tests a_document_of_the_largest_size_round_trips_through_property_chunks; portal::tests (wire indexes, frame limit); editor::tests a_failed_document_and_an_unreadable_one_show_their_message"
        status: pass
    human_judgment: false
  - id: D4
    description: "The editor window opens by the taskbar from the card's Customize icon, one window at most, its slider sends at most every 50 ms and once on release, the click never starts Use"
    requirement: PROP-01
    verification:
      - kind: unit
        ref: "editor::tests (placement, taskbar_edge, editor_origin in all edges and RTL, SendGate, snapped, the second theme replaces the content); card::tests a_click_on_the_customize_icon_customizes_and_never_starts_use"
        status: pass
      - kind: other
        ref: "the spike run: Portal log 'editor opened (measurement switch)' and 'editor placed' (see Spike answer)"
        status: pass
    human_judgment: true
    rationale: "How the window and the hover icon look next to the real taskbar is judged by the maintainer's eye in 04-56 (the capture script only captures a process's first window)"
  - id: D5
    description: "Placement next to a left, right or top taskbar, at 150 percent scale and mirrored for RTL"
    requirement: PROP-01
    verification:
      - kind: unit
        ref: "editor::tests the_editor_goes_to_the_corner_next_to_the_taskbar and the_placement_follows_the_monitor_dpi_and_the_taskbar"
        status: pass
    human_judgment: true
    rationale: "UI Considerations E7 backstop: a bottom taskbar is the only one on the maintainer's machine; the other edges and RTL rest on the pure-function tests"
duration: 29min
completed: 2026-11-28
---

# Phase 4 Plan 48: The slice-5 spike and tracer Summary

**One slider change on a running Web theme now goes from a second Portal window (the properties editor, placed by the taskbar) through the service's property host (D-20 import, atomic working copy, the only writer) to the CEF page's PropertyListener, and the plan's two risks are answered by running code.**

## Performance

- **Duration:** 29 min
- **Started:** 2026-11-28T18:08:00Z
- **Completed:** 2026-11-28T18:37:00Z
- **Tasks:** 3
- **Files modified:** 22 (2 new)

## Spike answer (research open question 5)

A second GPUI window opens from the running Portal and sits at the taskbar corner, with one measured defect that the code now handles.

```
2026-11-28T18:28:13Z INFO sucrose_portal::app::editor: editor placed origin=(1504, 536) size=(400, 480) bounds=((1504, 532), 400, 480) edge=Bottom scale=1.0 monitor=\\.\DISPLAY1 agree=false
```

- The window is at the bottom-right corner of DISPLAY1's work area (1920 x 1032, taskbar at the bottom): 16 px from the right and bottom edges as computed; `taskbar_edge` reads Bottom from the work area against the monitor rectangle.
- **Origin and read-back bounds do not agree within 2 px.** x is exact (1504); y is 4 px higher than computed (532 against 536). Cause, read from gpui-pre-windows: `calculate_window_rect` adds half of the window-minus-client height to the top, but a window with the title bar drawn by the app has its invisible frame at the bottom only, so the client origin lies half a frame above the requested top. The second run (same result, 18:30:22) confirmed it is deterministic.
- Handled in code, not hidden: the first window of a run logs `agree=false` and stores the difference in `FRAME_SHIFT`; the next editor window of the run asks GPUI for the shifted origin and lands on the computed one. The first window of a run stays 4 px (20 px instead of 16 px from the bottom). A platform helper that places the visible frame exactly would remove it; it needs an `unsafe` Win32 call in `sucrose-platform` and is left for the maintainer to ask for.
- Monitor and scale: GPUI places a window in logical pixels of the display named in `display_id`; the editor looks that display up by the physical origin of the Monitor (`cx.displays()`), and converts the physical origin by that monitor's scale (tested at 150 %).
- The editor runs on the main GPUI thread of the Portal process; with `--no-focus` it opens without taking focus, the Portal stays open when the editor closes, and the process ends when only the editor is left.

## Accomplishments

- **Tracer (Task 1):** `ChangeProperty` is validated by `PropertiesHost` (key must exist, same control type, only the value taken and held inside its bounds), the control goes to every running renderer of the theme as `ApplyProperty`, and the CEF engine runs the theme's `PropertyListener` call (`js::apply_property_call`: the control is serialized again, never pasted into the script). The integration test plays `calls` on CEF, opens the properties (1483 bytes through blob chunks), changes `amount` to 2.5, finds `Properties/callsBase` with 2.5 within 3 s and `property applied key=amount` in both renderer logs, and finds the fixture's own file byte for byte unchanged.
- **D-20:** the C# `Cache\Properties` import (36 files in the scratch run, no delta, the C# folder only read) runs once on a worker into a staging folder renamed at the end.
- **Editor (Task 2):** the window (title Sucrose Property, 400 px, header with the title and the Monitor chip, scrolling content, status line), the slider with readout at the step's precision, the muted "Control type '...' is not supported." line for the other types until 04-52, Escape and the close button, and the Customize icon (32 px hit area, 28 px black circle at 55 %, SlidersHorizontal) on Web cards, left of the red pill when both show.
- **Task 3:** the placement rule and the measured quirk are in the module doc of `editor.rs`.

## Task Commits

1. **Task 1: a slider change reaches the running Web theme on CEF** - `0c29ca92` (feat)
2. **Task 2: the editor window by the taskbar, Customize icon** - `550aeaa9` (feat)
3. **Task 3: the GPUI notes of the second window** - `55efef46` (docs)

**Plan metadata:** the commit of this SUMMARY with STATE.md, ROADMAP.md and REQUIREMENTS.md.

## Files Created/Modified

- `apps/sucrose/src/service/properties.rs` - the property host: import, working copy, change rules, the debounced atomic write (15 tests)
- `apps/sucrose/src/service.rs` - the request arms, the import worker with waiting requests, the property timer (id 8), the flush when the Portal leaves or the service stops
- `apps/sucrose-portal/src/app/editor.rs` - the editor window, its placement and its tests
- `crates/sucrose-ipc/src/{portal,protocol,blob}.rs` - the appended messages, `property_chunks`
- `crates/sucrose-engine-cef/src/engine.rs`, `crates/sucrose-engine/src/engine.rs`, `crates/sucrose-engine-web/src/js.rs` - `apply_property` end to end
- `apps/sucrose-portal/src/library/{card,mod}.rs`, `app.rs`, `main.rs`, `assets.rs`, `i18n.rs`, `locales/sucrose.en.yml` - the icon, the event, the wiring, the strings
- `docs/FEATURES.md` - the live apply row (Round C status, CRLF kept, no final newline kept)

## Decisions Made

See `key-decisions`. None of them reopens an Accepted entry of docs/DECISIONS.md; D-20 and D-18 are built as confirmed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The control limit is 60 KiB, not 64 KiB**
- **Found during:** Task 1 (wire test of the largest ChangeProperty)
- **Issue:** the plan caps the control text at 64 KiB, but one request is one frame and `MAX_FRAME` is 64 KiB, so a 64 KiB control plus the folder and key never fits.
- **Fix:** `MAX_PROPERTY_CONTROL` is 60 KiB and `MAX_PROPERTY_NAME` 512 bytes; a test pins that the largest request fits a frame.
- **Files modified:** crates/sucrose-ipc/src/portal.rs
- **Committed in:** 0c29ca92

**2. [Rule 2 - Missing critical] A CloseProperties request**
- **Found during:** Task 1
- **Issue:** the truths say the working copy is written "once more when the editor closes", but no message told the service that the editor closed.
- **Fix:** `PortalRequest::CloseProperties { folder }` (appended, wire index pinned); the editor sends it when its theme is replaced and when its window drops; the service also writes when the Portal leaves or the service stops.
- **Files modified:** crates/sucrose-ipc/src/portal.rs, apps/sucrose/src/service.rs, apps/sucrose-portal/src/app/editor.rs
- **Committed in:** 0c29ca92, 550aeaa9

**3. [Rule 3 - Blocking] Small additions outside the listed files**
- `sucrose_core::properties::properties_text` (the text `write_properties` writes, so the host can write it atomically), `blob::property_chunks`, `js::apply_property_call` with tests in `crates/sucrose-engine-web/tests/js.rs`, and a section in docs/FEATURES.md (CLAUDE.md: docs in the same change).
- **Committed in:** 0c29ca92, 550aeaa9

**4. [Rule 1 - Bug, measured] The window is 4 px higher than computed on the first open**
- See the spike answer: handled with `FRAME_SHIFT` for later windows; the first window of a run keeps the difference. The truth "origin and bounds agree within 2 px" is therefore not met by the first window of a run (y: 4 px); it is met by the following ones by construction (not run: only one editor window was opened per run).

---

**Total deviations:** 4 (1 blocking limit, 1 missing request, 1 small additions, 1 measured placement defect).
**Impact on plan:** none on the architecture; the first-window offset is cosmetic and documented.

## Issues Encountered

- A property change is not yet part of what a renderer starts with: a restarted renderer reads the theme's own SucroseProperties.json, not the working copy, so the page forgets the editor's values after a restart. This is the 04-21 gap that 04-49 onwards closes; the working copy is saved and re-opened by the editor correctly.
- The capture script `capture.ps1` captures only a process's first window, so the editor window and the hover icon were not captured; the hover icon needs a pointer and the Portal rule forbids synthesized input. The maintainer's glance in 04-56 covers both.
- `cargo fmt` through the Windows wrapper rewrote the CRLF files to LF; they were put back (`sed 's/\r\?$/\r/'`) before each commit: restored CRLF on `apps/sucrose/src/render.rs`, `apps/sucrose/src/service.rs`, `crates/sucrose-core/src/paths.rs`, `crates/sucrose-engine-cef/src/engine.rs`, `crates/sucrose-engine/src/engine.rs`, `crates/sucrose-ipc/src/protocol.rs`; the LF files and the new files were left alone. No known-flaky test failed in `cargo xtask ci` (exit 0, 57 test result lines ok).
- The Portal's event queue holds 64 events; a document of BLOB_MAX is 44 events at most, so an open request arriving when the queue is nearly full could end the connection (real documents are one chunk).

## Known Stubs

- `apps/sucrose-portal/src/app/editor.rs`, `render_control`: every control type except label and slider shows the muted line "Control type '{type}' is not supported." This is intentional until 04-52 (the other controls); it is not an unfinished feature of this plan.
- `apps/sucrose-portal/src/app/editor.rs`: the header buttons (Restore Default, Refresh Interface, Delete Copy) are not drawn yet (04-53). `PropertiesView.engine` is carried but not shown (the chip shows monitor numbers only).

## Lessons for docs/GPUI.md (04-54 owns that file)

- A second window is opened with `gpui_kit::open_window` (it mounts the Base `Root`) and its handle plus a `WeakEntity` of its view go into a `Global`; events of the main window reach it with `AnyWindowHandle::update`, which is allowed from inside an entity update of another window.
- GPUI on Windows places a window in logical pixels of the display in `WindowOptions.display_id` (physical origin divided by that monitor's scale); without it the primary display's scale is used, which is wrong on a monitor with another scale. `cx.displays()` has no physical bounds: match a display by `bounds().origin` times the monitor's scale.
- `Window::bounds()` is the client area in logical pixels (physical divided by the window's scale); for a window with the title bar drawn by the app its y is half the invisible frame (4 px at 100 %) above the requested origin.
- `cx.on_window_closed` with "quit when no window is left" keeps the process alive after the main window closes while a second window is open; the Portal's handler also quits when only the editor is left.
- `App::observe_keystrokes` sees every window, so Escape for the editor is handled with `on_key_down` on its root element (focus handle tracked), not with an observer.
- A test module that glob-imports GPUI shadows `#[test]`; name the imports (`#[gpui_kit::test]` works).
- An element hidden with `opacity(0.)` is still hit-tested, so a hover-only icon (`group_hover` plus `focus_visible`) needs its mouse-down consumed to keep the click off the card.

## Threat Flags

None beyond the plan's register (T-04-104..107 are implemented: only the value is taken, bounded, serialized as JSON; no value or control is logged; atomic writes in the Properties folder only; documents above BLOB_MAX refused).

## Next Phase Readiness

- 04-49..04-51 can add the other property sources (mpv and the generated pages' sets, the Sucrose volume set) as other `kind` values of the same host and the same IPC.
- 04-52 renders the remaining controls; 04-53 the header buttons; 04-54 the docs/GPUI.md lessons above; 04-56 measures the editor with `SUCROSE_MEASURE_EDITOR` and the maintainer's eye.
- A renderer that restarts should start from the working copy (the 04-21 gap).

## Self-Check: PASSED

- Files found: apps/sucrose/src/service/properties.rs, apps/sucrose-portal/src/app/editor.rs, crates/sucrose-core/src/paths.rs (`fn properties_dir`: 1).
- Commits found as ancestors of HEAD: 0c29ca92, 550aeaa9, 55efef46.
- Acceptance counts: `ApplyProperty` in protocol.rs 2, `fn ensure_imported` 1, `fn editor_origin` 1, `SlidersHorizontal` in assets.rs 1, `SUCROSE_MEASURE_EDITOR` in app.rs 2; `tasklist.exe` shows no Rust Sucrose process after the tests and the spike.
- `cargo xtask ci` exit 0 (fmt, clippy -D warnings, test) after the last code change; the ignored integration test passed.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-28*
