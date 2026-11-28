---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 43
subsystem: service
tags: [duplicate, span, display-modes, time-base, gif, desk-02, desk-04]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-41/04-42: placement.rs, one job per theme, per-group election, Portal protocol 9"
provides:
  - "Duplicate (one renderer per monitor, one sound) and Span (one renderer over the union rectangle, target span:all)"
  - "SettingChange::DisplayScreenType applied at once (Portal protocol 10)"
  - "Shared time base: Assignment.time_base, ToRenderer::TimeBase, Engine::set_time_base (protocol 13); the built-in GIF renderer follows it"
  - "D-043"
affects: [04-44, 04-46, 04-56, DESK-02, DESK-04]
requirements-completed: []
actuals:
  tokens: 27900
  tasks: 3
  commits: 3
plan_head_before: 9c7dc9c29dba423533f9f83549d76dacb4b80023
plan_head_after: 2ff55391225158b71b5be8f722cf6d04725825e9
tech-stack:
  added: []
  patterns: ["the service keeps one epoch per theme group and the engines count their content position from it", "a display mode change is a diff of target lists (placement_actions): close, re-place, replace the theme of a survivor, spawn"]
key-files:
  created: []
  modified:
    - apps/sucrose/src/placement.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/settings_host.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/src/fullscreen.rs
    - crates/sucrose-ipc/src/protocol.rs
    - crates/sucrose-ipc/src/portal.rs
    - crates/sucrose-engine/src/engine.rs
    - crates/sucrose-engine-gif/src/engine.rs
    - crates/sucrose-engine-gif/src/decode.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose-portal/src/settings/other.rs
    - apps/sucrose-portal/src/settings/wallpaper.rs
    - docs/DECISIONS.md
key-decisions:
  - "D-043: the epoch is kept by the service and handed to renderers; mpv keeps the D-030 clock"
  - "Span is one pseudo-monitor named span:all whose id is the slot key, so the slot map, the diff, the web profile owner and the renderer need no special case"
  - "The fullscreen rule treats a fullscreen window on any monitor as covering the span surface"
  - "The Portal sees a Span as every connected monitor playing its theme (spread_span)"
  - "A whole-group pause moves the epoch by the shorter pause; a pause of only some renderers keeps it"
coverage:
  - id: D1
    description: "Duplicate runs one renderer per monitor with the selected theme, one master clock and one sound"
    requirement: DESK-02
    verification:
      - kind: unit
        ref: "placement duplicate_gives_each_monitor_its_own_renderer_with_the_selected_theme_and_one_monitor_gives_one; service the_master_of_a_duplicate_group_does_not_depend_on_the_order_the_os_lists_the_monitors_in"
        status: pass
      - kind: integration
        ref: "cargo test -p sucrose --test portal -- --ignored a_display_mode_change_moves_the_wallpaper_to_the_union_and_back (DISPLAY1 + DISPLAY9, Duplicate step)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Span runs one renderer over the union rectangle of the display bounds or the work areas; touching monitors give exactly their union, a gap gives the bounding rectangle"
    requirement: DESK-04
    verification:
      - kind: unit
        ref: "placement monitors_that_touch_give_a_span_rectangle_with_no_gap_and_no_overlap, monitors_with_a_gap_or_a_negative_origin_give_the_bounding_rectangle, one_monitor_spans_to_its_own_rectangle_none_to_nothing_and_the_order_changes_nothing"
        status: pass
      - kind: integration
        ref: "the same ignored test: the Service log line names span:all and rect=[0, 0, 3840, 1080]"
        status: pass
    human_judgment: true
    rationale: "The rectangle is proven by the log line and unit tests; that the span looks right across both monitors on the desktop is 04-56's glance check"
  - id: D3
    description: "A Display Mode change is saved and applied at once; renderers whose target or theme changes restart or load in place, the others keep playing; while a rule holds the wallpaper it waits for the rule"
    requirement: DESK-02
    verification:
      - kind: unit
        ref: "service duplicate_loads_the_selected_theme_into_the_renderer_that_played_another_and_leaves_the_other_alone, duplicate_restarts_a_renderer_whose_engine_cannot_load_the_theme_in_place, a_display_mode_change_while_the_rule_holds_the_wallpaper_waits_for_the_rule; settings_host a_display_mode_change_is_stored_under_its_legacy_name_and_shown"
        status: pass
      - kind: integration
        ref: "the ignored two-monitor test: Per display, Span, Duplicate and Per display again; the applied log line per step"
        status: pass
    human_judgment: false
  - id: D4
    description: "Duplicated GIFs keep in step: the loop wraps of DISPLAY1 and DISPLAY9 are within one frame (50 ms) of each other"
    requirement: DESK-02
    verification:
      - kind: unit
        ref: "engine-gif frame_at_*, at_a_wrap_the_animation_moves_to_the_frame_the_time_base_gives_when_it_is_more_than_one_frame_away, a_frame_in_step_or_one_frame_away_is_left_alone_in_both_directions; service a_theme_on_several_monitors_shares_one_epoch_..., a_group_that_grows_hands_its_epoch_..., a_pause_of_the_whole_group_moves_the_epoch_forward_..."
        status: pass
      - kind: integration
        ref: "task 1 desktop run, 12 s, Renderer logs of both monitors: 10 wraps each, the largest distance of a wrap of DISPLAY1 (after the first) to the nearest wrap of DISPLAY9 was 31 ms"
        status: pass
    human_judgment: false
duration: 24min
completed: 2026-11-28
status: complete
---

# Phase 4 Plan 43: Duplicate and Span in the service Summary

**Duplicate and Span now run in the service (one renderer per monitor, or one over the union rectangle), the Display Mode setting applies at once, and renderers of one theme share a service-kept epoch that the built-in GIF renderer follows so that two monitors' loops wrap within 31 ms of each other.**

## Performance

- **Duration:** 24 min
- **Started:** 2026-11-28T15:56:20Z
- **Completed:** 2026-11-28T16:21:00Z
- **Tasks:** 3
- **Files modified:** 14

## Accomplishments

- Protocol 13: `Assignment.time_base` and `ToRenderer::TimeBase` (appended, wire-index tested), `Engine::set_time_base` (default ignores it; the renderer calls it after the engine starts and on every `TimeBase`). The service keeps `Fleet.time_bases` (one epoch per theme group, created at the first spawn, dropped with the last renderer), gives it to a group of more than one monitor, tells the renderers that were there first when the group grows, and moves it forward by a whole-group fullscreen pause.
- Built-in GIF renderer: `frame_at` (pure, a zero delay plays at 100 ms), delays learned in the first pass, a jump to `frame_at(now - epoch)` at a loop wrap when it is more than one frame away (cyclic), a `gif loop wrapped unix_ms=...` debug line at each wrap, a one-frame file ignores the epoch. Desktop run: both monitors, 10 wraps each in 12 s, the largest distance from a wrap of DISPLAY1 to the nearest wrap of DISPLAY9 31 ms (the first wrap of each renderer is not in step by design).
- Span: `span_target` (union of display bounds or work areas, touching monitors exactly their union, a gap the bounding box, DPI and primary of the primary monitor), `SPAN_TARGET = "span:all"`, `plan_targets` for Span, `placement_actions` (closes, re-placements, theme replacements, spawns). A survivor whose theme changes goes through `retarget`, then loads in place (mpv to mpv) or restarts. The `wallpaper placed` log line now names the rectangle.
- `SettingChange::DisplayScreenType` (Portal protocol 10, `PortalSettings.display_screen_type`, stored under its legacy name) applies at once, or when the fullscreen rule lets go (`Fleet::replan`, `replan_pending`). In Span a fullscreen window on any monitor covers the one surface (`covered_surfaces`); the Portal sees every monitor playing the span theme (`spread_span`).
- Two-monitor integration test `a_display_mode_change_moves_the_wallpaper_to_the_union_and_back` passes on DISPLAY1 + DISPLAY9: Per display, then Span (one renderer, `rect=[0, 0, 3840, 1080]`), Duplicate (a renderer on each monitor), Per display again (D9 back to its own theme).

## Task Commits

1. **Task 1: Duplicate keeps GIFs in step through a shared time base** - `dc3f3f98` (feat)
2. **Task 2: Span over the union rectangle and the Display Mode setting** - `7ffd2817` (feat)
3. **Task 3: D-043** - `2ff55391` (docs)

**Plan metadata:** the commit that adds this SUMMARY and the STATE/ROADMAP update (docs).

## Files Created/Modified

See the front matter. The GIF test fixture (`decode.rs` `gif_bytes`/`frame_number`) now gives up to 32 frames a number of their own (red `(i % 4) * 70`, green `20 + (i / 4) * 30`); the old fixture wrapped after four frames.

## Decisions Made

D-043 (new) and an amendment line on D-042. Runs that needed the built-in GIF renderer pass `--engine Gif=XavierLive` to `synth-library` (the default Gif engine is mpv); the Task 1 desktop run used it.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Plan defect] The GIF renderer cannot jump at its first frame**
- **Found during:** Task 1
- **Issue:** the plan says the pump moves to `frame_at(delays, now - epoch)` "at the first frame and at every loop wrap", but the delays of the animation are only known after the first pass (the decoder streams, nothing holds the animation).
- **Fix:** the pump records the delays during the first pass from frame 0 and acts at every wrap after it; a time base that arrives later acts at the next wrap. Recorded in D-043 (the first pass of each renderer is not in step, every later wrap is).
- **Files modified:** crates/sucrose-engine-gif/src/engine.rs
- **Commit:** dc3f3f98

**2. [Rule 3 - Blocking] The GIF test fixture could not tell more than four frames apart**
- **Found during:** Task 1
- **Issue:** frame `i` had red `i * 70 % 256`, so frame 4 looked like frame 0; a time-base jump test needs at least four distinct frames at distance more than one.
- **Fix:** green carries `i / 4`; `frame_number` reads both channels. The existing tests (at most four frames) are unchanged.
- **Files modified:** crates/sucrose-engine-gif/src/decode.rs
- **Commit:** dc3f3f98

**3. [Rule 3 - Signature] `span_target` and `placement_actions` take one more argument than the plan names**
- **Issue:** a `Target` carries its theme, so `span_target(monitors, area, theme)`; the service needs both the diff of changed rectangles (for the log and the election) and the re-placement of every survivor, so `placement_actions(old, new, every_survivor)` returns `PlacementAction` (Close, SetRect, Replace { target, from }, Spawn).
- **Files modified:** apps/sucrose/src/placement.rs
- **Commit:** 7ffd2817

**4. [Rule 2 - Missing critical] A whole-group pause moves the epoch, a partial one does not**
- **Issue:** the plan says "after a rule pause ends the epoch moves forward by the paused time"; with the Affected Monitor scope only some renderers of a group pause, and moving the epoch would shift the ones that played on.
- **Fix:** the epoch moves by the shorter pause only when every renderer of the group was paused; the paused ones join the grid at their next wrap. Unit tested.
- **Commit:** dc3f3f98

**5. [Process] Tasks were committed once each, not as RED and GREEN commits**
- **Issue:** the tasks carry `tdd="true"`, but the plan is not `type: tdd` and `cargo test` output is not a format `gsd_run check tdd-red-evidence` reads. Tests were written with the code and run before each commit; the GIF tests failed first on the old fixture and on a timing assumption (the wrap is handled at 400 or 450 ms depending on the decode thread), then were fixed.

**6. [Process] Docs edit made append-only**
- **Found during:** Task 3
- **Issue:** a first edit rewrote a sentence of D-042 in place, against the docs rule.
- **Fix:** reverted; D-042 got an appended amendment line pointing at D-043.

### Line endings

`cargo fmt` (the Windows wrapper) rewrote `apps/sucrose/src/service.rs`, `fullscreen.rs` and `settings_host.rs` to LF; CRLF was restored on those three before the commits (`git ls-files --eol` shows `w/crlf` for them). `render.rs`, `protocol.rs` and `sucrose-engine/src/engine.rs` kept CRLF; `placement.rs`, the engine-gif files, `portal.rs`, `tests/portal.rs`, the Portal files and `docs/DECISIONS.md` are LF and stayed LF.

**Total deviations:** 4 auto-fixed (1 plan defect, 1 missing critical, 2 blocking or signature) plus 2 process notes.
**Impact on plan:** none on scope; the GIF renderer syncs from its second pass.

## Issues Encountered

None that blocked. `cargo xtask ci` passed (exit 0). No flaky test needed a rerun.

## Not checked

- The plan's backstop statement (taskbar auto-hidden: Working Area and Display Bound give the same rectangles): not checked, the maintainer's glance in 04-56 only if his taskbar is auto-hidden.
- A Display Mode change while a fullscreen rule holds the wallpaper is unit tested (the change waits, `replan_pending`), but the release on the desktop (fullscreen app, then leave it) was not driven.
- Span over a surface larger than 3840 x 1080 (T-04-96, Chromium texture limits for the web engines) was not tried; only the built-in GIF renderer ran in Span here. The web engines follow the time base in 04-44.

## Threat Flags

None: no new network endpoint, auth path or file access; the span rectangle and the epoch are bounded by the monitors the OS reports and by the frame list.

## Known Stubs

None. `Assignment.time_base` reaches the web engines but they ignore it until 04-44 (the default `Engine::set_time_base`).

## Next Phase Readiness

04-44 can follow the epoch in the web engines through `Engine::set_time_base`; 04-46 builds the Display Preferences radios on `SettingChange::DisplayScreenType`.

## Self-Check: PASSED

- `fn span_target`, `TimeBase`, `fn set_time_base`, `pub fn frame_at`, `DisplayScreenType` in portal.rs, `## D-043` found by grep.
- Commits `dc3f3f98`, `7ffd2817`, `2ff55391` are ancestors of HEAD.
- `tasklist.exe` shows no Rust Sucrose process.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-28*
