---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 45
subsystem: display-modes
tags: [screen-layout, stretch-mode, ipc, live-settings, desk-04, desk-05, portal]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-41..04-44: placement targets, Fleet::move_to (SetRect), live_message/apply_live (04-14), Engine::set_stretch on mpv, the built-in GIF renderer, CEF and WebView2, PORTAL_PROTOCOL_VERSION 10"
provides:
  - "SettingChange::ScreenType and StretchType (appended), PortalSettings.screen_type and .stretch, PORTAL_PROTOCOL_VERSION 11"
  - "ToRenderer::SetStretch (appended, wire index 14): the renderer calls Engine::set_stretch, no restart"
  - "Fleet::change_screen: every job covers the new area and every surface (the Span surface over the union) moves with SetRect; a slot closed by a fullscreen rule only gets its stored rectangle updated"
  - "Stretch Mode applied live to every renderer; the same value twice sends nothing"
  - "The Screen Layout and Stretch Mode rows on the Wallpaper page, in the legacy order"
affects: [04-46, 04-56, DESK-04, DESK-05]
requirements-completed: []
actuals:
  tokens: 13100
  tasks: 2
  commits: 2
plan_head_before: 6599976ddd26d01fa74595facc2d37f0f300b90d
plan_head_after: b7afea7bc83cfc313763e6a960cc14c177fd7af2
tech-stack:
  added: []
  patterns: ["a live renderer setting is a ToRenderer message plus a put_live write into jobs and assignments, so a later restart keeps it", "a layout change re-plans through apply_or_defer and gets only SetRect actions, because targets and themes do not change"]
key-files:
  created: []
  modified:
    - crates/sucrose-ipc/src/portal.rs
    - crates/sucrose-ipc/src/protocol.rs
    - apps/sucrose/src/settings_host.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/tests/portal.rs
    - apps/sucrose-portal/src/settings/wallpaper.rs
    - apps/sucrose-portal/src/settings/other.rs
    - apps/sucrose-portal/src/i18n.rs
    - docs/FEATURES.md
key-decisions:
  - "Screen Layout is applied through the same apply_snapshot path as a topology change, with no deferral while a fullscreen rule holds the wallpaper: only SetRect actions result, and a closed slot has no connection, so its stored assignment rectangle is what its restart uses"
  - "A stretch every job already has is not sent again (checked in Fleet::apply_live, not in the settings host), so the Portal can send the same value twice without touching the renderers"
coverage:
  - id: D1
    description: "A Stretch Mode change refits the running wallpaper on every renderer without a restart, is stored as Engine.StretchType and survives in the assignments"
    requirement: DESK-05
    verification:
      - kind: unit
        ref: "apps/sucrose service::tests a_stretch_change_reaches_every_renderer_and_the_assignments_and_the_same_value_twice_sends_nothing, a_stretch_change_while_the_wallpaper_is_paused_is_applied_and_kept_for_the_resume, live_message_maps_loop_shuffle_and_stretch_and_nothing_else"
        status: pass
      - kind: integration
        ref: "apps/sucrose tests/portal.rs a_stretch_and_a_layout_change_reach_the_running_renderer_without_a_restart (GIF on the built-in renderer: 'stretch changed' in every renderer log, one engine start per slot)"
        status: pass
    human_judgment: false
  - id: D2
    description: "A Screen Layout change moves every surface onto the working area or the display bounds (the Span surface onto the union) without a restart; a renderer closed by a fullscreen rule only gets its stored rectangle updated"
    requirement: DESK-04
    verification:
      - kind: unit
        ref: "apps/sucrose service::tests a_screen_layout_change_only_moves_the_surfaces_without_a_restart, a_screen_layout_change_moves_a_span_surface_onto_the_union_of_the_chosen_areas, a_screen_layout_change_while_a_rule_has_closed_a_renderer_updates_only_its_stored_rectangle"
        status: pass
      - kind: integration
        ref: "tests/portal.rs a_stretch_and_a_layout_change_reach_the_running_renderer_without_a_restart (service log: 'the screen layout was applied', 're-placing its surface', no closing)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Both values are stored under their legacy names (Engine.ScreenType, Engine.StretchType) and shown in the settings view; PORTAL_PROTOCOL_VERSION is 11 and the new variants keep their wire index"
    requirement: DESK-04
    verification:
      - kind: unit
        ref: "settings_host a_screen_layout_and_a_stretch_change_are_stored_under_their_legacy_names_and_shown; sucrose-ipc the_portal_protocol_is_at_version_eleven_..., set_stretch_keeps_its_wire_index_and_survives_the_wire, the settings-view and wire-index tests"
        status: pass
    human_judgment: false
  - id: D4
    description: "The Wallpaper page lists Interaction Method, Screen Layout, Stretch Mode, Loop Mode, Shuffle Mode, Hardware Acceleration with the legacy titles, descriptions and enum labels"
    requirement: DESK-04
    verification:
      - kind: unit
        ref: "sucrose-portal settings::wallpaper wallpaper_group_screen_layout_lists_..., wallpaper_group_stretch_mode_lists_..."
        status: pass
      - kind: other
        ref: "capture target/rows-lib/portal-slice4-rows.png (not committed)"
        status: pass
    human_judgment: true
    rationale: "The look of the two new cards on the real page is an eye check; the capture was described below, not signed off by the maintainer"
duration: 15min
completed: 2026-11-28
status: complete
---

# Phase 4 Plan 45: Screen Layout and Stretch Mode Summary

**Screen Layout and Stretch Mode on the Wallpaper page, applied live: StretchType becomes ToRenderer::SetStretch for every renderer and ScreenType moves every surface with SetRect (Span over the union), with no restart.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-11-28T16:52:00Z
- **Completed:** 2026-11-28T17:08:00Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- IPC: `SettingChange::ScreenType(ScreenArea)` and `StretchType(Stretch)` appended, `PortalSettings.screen_type` and `.stretch`, `PORTAL_PROTOCOL_VERSION` 11, `ToRenderer::SetStretch` (wire index 14, `PROTOCOL_VERSION` unchanged). settings_host stores them under `Engine.ScreenType` and `Engine.StretchType`.
- Service: `live_message` maps StretchType to SetStretch and ScreenType to nothing; `put_live` writes the stretch into jobs and assignments (a restarted renderer keeps it); `apply_live` sends nothing when every job already has that stretch. `Fleet::change_screen` sets the area on every job and re-places through `apply_or_defer`, which gives SetRect actions only (Span included); the Portal's setting handler calls it.
- Renderer: `SetStretch` calls `engine.set_stretch` (and is skipped before the assignment). Engines that do not stretch (YouTube, URL and Web pages) accept the call as before; a Web page still gets StretchMode through the compatible API.
- Portal: Screen Layout (Working Area, Display Bound) and Stretch Mode (None, Fill, Uniform, Uniform to Fill) rows after Interaction Method and before Loop Mode, with the legacy keys; each choice sends one SettingChange and the select shows only the broadcast value. The English strings already existed in legacy.en.yml, so sucrose.en.yml needed nothing; the new keys are in `REQUIRED_KEYS`.
- Capture of the Wallpaper page (`target/rows-lib/portal-slice4-rows.png`, fixture library, `--no-focus`): the Wallpaper group shows Interaction Method with its module and desktop-focus option, then a Screen Layout card with a select reading "Display Bound" (the default), a Stretch Mode card reading "Uniform to Fill" (the default), then the Loop Mode and Shuffle Mode switches, all in the same card style as the existing rows. No clipping at the 916x608 window.
- FEATURES: the working area row and the stretch row record that both apply live (CRLF kept, no final newline kept). The Stay awake row already says Phase 5 (DESK-11); nothing to correct.

## Task Commits

1. **Task 1: a stretch or screen layout change refits the running wallpaper without a restart** - `f8058a39` (feat)
2. **Task 2: the Screen Layout and Stretch Mode rows on the Wallpaper page** - `b7afea7b` (feat)

**Plan metadata:** the docs commit that follows this SUMMARY (SUMMARY, STATE, ROADMAP).

## Files Created/Modified

- `crates/sucrose-ipc/src/portal.rs`, `protocol.rs` - the two SettingChanges, the view fields, version 11, SetStretch and their wire tests
- `apps/sucrose/src/settings_host.rs` - stores and shows both
- `apps/sucrose/src/service.rs` - live message, put_live, apply_live guard, `Fleet::change_screen`, unit tests
- `apps/sucrose/src/render.rs` - the SetStretch arm
- `apps/sucrose/tests/portal.rs` - the ignored live test
- `apps/sucrose-portal/src/settings/wallpaper.rs`, `other.rs`, `i18n.rs` - the rows, test literals, required keys
- `docs/FEATURES.md` - the two rows

## Decisions Made

- A Screen Layout change goes through the normal snapshot application and is not deferred while a fullscreen rule pauses or closes the wallpaper: a layout change only yields SetRect, a closed slot has no connection, and `move_to` stores the new rectangle in the slot's assignment, which its restart uses. This is the behavior the plan's edge case asks for, without a second mechanism.
- The "same value twice" guard lives in `Fleet::apply_live` for SetStretch only; Loop and Shuffle keep sending as before.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] PortalSettings test literals fixed in Task 1, not Task 2**
- **Found during:** Task 1 (clippy over the workspace)
- **Issue:** the new view fields broke the test literals in `other.rs` and `wallpaper.rs` of the Portal, so the Task 1 commit would not have compiled its tests
- **Fix:** both literals got `screen_type` and `stretch` in the Task 1 commit
- **Files modified:** apps/sucrose-portal/src/settings/other.rs, apps/sucrose-portal/src/settings/wallpaper.rs
- **Commit:** f8058a39

**2. [Rule 2 - Missing critical] The new keys were added to `REQUIRED_KEYS` in i18n.rs**
- **Found during:** Task 2
- **Issue:** the list of keys every language file must carry did not name the new rows' keys
- **Fix:** six keys added (ScreenLayout, StretchMode and their descriptions, Enum.ScreenType.*)
- **Files modified:** apps/sucrose-portal/src/i18n.rs
- **Commit:** b7afea7b

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 missing critical). **Impact:** none on scope. apps/sucrose-portal/locales/sucrose.en.yml was in the plan's file list and is unchanged because every needed key was already in legacy.en.yml.

## Issues Encountered

- The first version of two unit tests read a recording log twice with `sent_to`, which returns the earlier sentinel Close on a second read; the tests were restructured to read once (the helper stays as it was, because an older test relies on its second read being empty).
- `cargo fmt` through the Windows wrapper rewrote the CRLF files to LF; CRLF was restored before each commit.

## Line endings

Restored to CRLF after `cargo fmt` before the Task 1 commit: apps/sucrose/src/service.rs, settings_host.rs, render.rs, crates/sucrose-ipc/src/protocol.rs. LF files left alone: crates/sucrose-ipc/src/portal.rs, apps/sucrose/tests/portal.rs, the Portal files. docs/FEATURES.md stays CRLF with no final newline.

## Verification

- `cargo test -p sucrose-ipc`, `cargo test -p sucrose settings_host`, `live_message`, the whole `sucrose` bin tests (472 passed), `cargo test -p sucrose-portal` (325 passed): ok.
- `cargo test -p sucrose --test portal -- --ignored a_stretch_and_a_layout_change_reach_the_running_renderer_without_a_restart`: passed; `tasklist.exe` shows no Rust Sucrose process afterwards.
- `cargo xtask ci` (fmt, clippy -D warnings, tests): exit 0. No flaky test needed a rerun.

## Known Stubs

None.

## Threat Flags

None. Both values are enums, decoded by postcard and stored by their legacy names (T-04-100); no crate was added (T-04-SC).

## Next Phase Readiness

Ready for 04-46. DESK-04 and DESK-05 are not marked: the final Round C checkpoint 04-56 marks DESK-04; DESK-05 completes in Round E (libVLC and Media Foundation). Not checked here: the stretch of a Span renderer and a two-monitor layout change on real hardware (unit-tested with fake monitors only; the live test ran on the monitors of this machine).

## Self-Check: PASSED

- apps/sucrose-portal/src/settings/wallpaper.rs, crates/sucrose-ipc/src/protocol.rs, apps/sucrose/src/service.rs: present; commits f8058a39 and b7afea7b are ancestors of HEAD.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-28*
