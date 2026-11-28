---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 47
subsystem: portal-ui
tags: [library-card, monitor-badges, selected-pill, gpui, desk-02, desk-03]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-42/04-46: RuntimeState.displays and the target-based RuntimeState.selected, DisplayScreenType in PortalSettings, the Monitor icon, the Display Preferences dialog"
provides:
  - "library/card.rs: Badge, monitor_badges, MAX_MONITOR_BADGES, CardData.badges and the badge pills"
  - "The Portal.LibraryCard.MonitorBadge* strings"
  - "The FEATURES row of Display Preferences completed with the card badges"
affects: [04-54, 04-56, DESK-02, DESK-03]
requirements-completed: []
actuals:
  tokens: 8200
  tasks: 2
  commits: 3
plan_head_before: 29f24cbef70011a52871990277c82c978cf5170f
plan_head_after: c7d48dce1aef0c8cec41a535f83c12943b755f98
tech-stack:
  added: []
  patterns: ["a pure function (monitor_badges) from RuntimeState.displays and the stored placement mode to the badges, tested without a window", "the Selected pill and the badges share one absolutely placed row, so the 8 px gap holds with or without the pill"]
key-files:
  created: []
  modified:
    - apps/sucrose-portal/src/library/card.rs
    - apps/sucrose-portal/src/library/mod.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/locales/sucrose.en.yml
    - docs/FEATURES.md
key-decisions:
  - "A badge counts only for a connected monitor whose renderer runs the theme (DisplayView.connected and running); an assignment that does not run, or a monitor that is not connected, gives none"
  - "The placement mode comes from the stored settings the Portal already receives (LibrarySettings.display_screen_type), not from a new field, so no protocol version changes"
  - "The Selected pill needed no change: RuntimeState.selected already follows the target monitor (04-42), which a card test pins"
status: complete
coverage:
  - id: D1
    description: "A theme card shows one badge per connected monitor that runs the theme (Monitor icon and number, at most two), then +N with a tooltip listing the rest; one All badge in Span and Duplicate; none when the theme does not run"
    requirement: DESK-03
    verification:
      - kind: unit
        ref: "apps/sucrose-portal library::card::tests badges_* (6 behavior tests) and badges_say_where_in_the_words_of_the_ui_spec"
        status: pass
      - kind: integration
        ref: "library::card::tests::badges_render_without_the_selected_pill_and_with_it (GPUI window)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Badges are decoration: no click of their own, a click reaches the card"
    requirement: DESK-03
    verification:
      - kind: integration
        ref: "library::card::tests::a_click_on_a_badge_is_a_click_on_the_card_and_nothing_else"
        status: pass
    human_judgment: false
  - id: D3
    description: "Selected pill then badges with an 8 px gap on a real two-monitor run (DISPLAY1 and DISPLAY9), in Duplicate (All) and in Per display (1 and 9)"
    requirement: DESK-02
    verification: []
    human_judgment: true
    rationale: "A visual check of the pill row on the captured window; the maintainer's glance in 04-56 decides, the captures are below"
  - id: D4
    description: "docs/FEATURES.md Display Preferences row records the card badges and the legacy sub-radios not shown"
    requirement: DESK-03
    verification:
      - kind: other
        ref: "grep -c 'Remove Wallpaper' docs/FEATURES.md is 1; git ls-files --eol docs/FEATURES.md is w/crlf"
        status: pass
    human_judgment: false
duration: 13min
completed: 2026-11-28
---

# Phase 4 Plan 47: Monitor badges on the cards Summary

**Cards say where their theme runs: a neutral Monitor-and-number pill per connected monitor that plays it (at most two, then +N with a tooltip), one All pill in Span and Duplicate, placed right of the Selected pill and driven only by the service's RuntimeState.displays and the stored placement mode.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-11-28T17:48:00Z
- **Completed:** 2026-11-28T18:01:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- `monitor_badges(folder, displays, mode)` returns `Badge::Monitor(n)` per connected, running monitor of that theme (two at most, then `More` with the rest), `Badge::All` in Span and Duplicate, and nothing for a theme that does not run (T-04-103: at most two pills and one +N pill per card).
- The pill row (Selected pill, 8 px gap, badges) sits at the card's top-left; badges are black at 55 % with white text (image-overlay exception), 20 px high, with an accessible label ("Playing on monitor 1"); only +N has a tooltip ("Also playing on monitors 3, 4, 9"). They have no click handler, so a click goes to the card.
- The grid fills `CardData.badges` from the newest RuntimeState and the placement mode of the settings broadcast; nothing moves on a click.
- FEATURES: the Display Preferences part of the "Per display / span across / duplicate" row now records the card badges (new, D-02) and that the legacy Default and Advanced sub-radios are not shown.

## Captures (real run, DISPLAY1 and DISPLAY9 both connected, 2 renderers started)

Scratch, not committed: `target/badge-lib/portal-badges.png` (Duplicate, gif-anim) and `target/badge-lib2/portal-badges.png` (Per display, gif-anim assigned to both monitors).

- Duplicate: the gif-anim card shows the blue Selected pill, then one dark pill with the Monitor icon and "All", 8 px apart.
- Per display with both monitors on gif-anim: Selected, then two pills "1" and "9", each with the Monitor icon.

## Task Commits

1. **Task 1 (tracer, TDD): RED** - `6384091a` (test): five behavior tests failed on their assertions against a stub that returned no badge (left `[]`, right `[Monitor(1), Monitor(9)]` and the like); `cargo test` output, not the JS classifier (no Rust report adapter exists); semantic assessment: the target tests ran and failed on the planned assertions, no build or fixture fault.
2. **Task 1: GREEN** - `3c0949c8` (feat): monitor_badges, the pills, CardData.badges, the grid wiring, the strings; 46 card tests and 6 i18n tests pass, clippy clean.
3. **Task 2: FEATURES row** - `c7d48dce` (docs)

**Plan metadata:** the docs(04-47) commit that holds this SUMMARY, STATE and ROADMAP.

## Files Created/Modified

- `apps/sucrose-portal/src/library/card.rs` - Badge, monitor_badges, render_badge, the pill row, 11 new tests
- `apps/sucrose-portal/src/library/mod.rs` - LibrarySettings.display_screen_type, the badges of each card
- `apps/sucrose-portal/src/i18n.rs` - the new keys in REQUIRED_KEYS
- `apps/sucrose-portal/locales/sucrose.en.yml` - MonitorBadge, MonitorBadgeAll, MonitorBadgeAllLabel, MonitorBadgeMore, MonitorBadgeMoreOne
- `docs/FEATURES.md` - the Display Preferences row

Line endings: every Rust and YAML file was and stayed LF (the Windows `cargo fmt` left them LF); `docs/FEATURES.md` stayed CRLF with no final newline (edited with a byte-level replace). No file needed a CRLF restore.

## Decisions Made

See key-decisions. The mode is read from LibrarySettings, so a mode change redraws the cards on the same settings broadcast that already redraws them.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] i18n.rs touched**
- **Found during:** Task 1
- **Issue:** the `every_new_key_is_required` test fails for a key of sucrose.en.yml that is not in REQUIRED_KEYS (it lives in i18n.rs, which the plan's file list omits)
- **Fix:** the new keys were added to REQUIRED_KEYS
- **Files modified:** apps/sucrose-portal/src/i18n.rs
- **Committed in:** 3c0949c8

**2. [Rule 2 - Missing critical] Two keys beyond the three the plan names**
- **Found during:** Task 1
- **Issue:** the visible "All" needs a string of its own (the accessible description says "Playing on all monitors"), and the +N tooltip needs a singular form (`tr_count_with` reads `<key>One`)
- **Fix:** `Portal.LibraryCard.MonitorBadgeAllLabel` and `Portal.LibraryCard.MonitorBadgeMoreOne`
- **Committed in:** 3c0949c8

**3. [Rule 1 - Bug in my own run] Scratch data dir not honored on the first two Portal starts**
- **Found during:** the capture step of Task 1
- **Issue:** the first two Portal launches (an unquoted backslash path in my own command) created a junk relative data folder in the repository root, and the Portal and its service then read the maintainer's legacy data read-only (the screenshot showed the real library) and played the real selected wallpaper for about 20 s each time
- **Fix:** both runs were stopped with taskkill and the junk folder (Cache/mpv-shaders, Log, Setting, all created by me) was removed from the repository root; the real `%APPDATA%\Sucrose` was checked afterwards: no file there was modified since the runs (Setting files still dated 2026-11-25 and 2026-11-28 16:13, Log untouched). The two proof captures were taken from clean runs started through `cmd.exe /c "set SUCROSE_DATA_DIR=...&& start /B ..."`, with the scratch folder confirmed by its own Log and the `renderers started count=2` line
- **Verification:** `find %APPDATA%\Sucrose -newermt` after the runs printed nothing; `tasklist.exe` showed no sucrose process after each stop

---

**Total deviations:** 3 (1 blocking, 1 missing critical, 1 own-run incident, no data changed)
**Impact on plan:** none on scope.

## Issues Encountered

- The click test clicks a point on the badge row of the test window; it proves the card gets the click (Use), not that no other handler runs; the badge elements carry no handler by construction.
- No flaky test came up; `cargo xtask ci` passed on the first run.

## Known Stubs

None.

## Threat Flags

None. T-04-103 (at most two badges and one +N pill) is enforced by `MAX_MONITOR_BADGES` and pinned by `badges_the_third_and_later_monitors_collapse_into_more`.

## Next Phase Readiness

Requirements stay as the Round C map says: DESK-02 and DESK-03 are marked by the later Round C plans (04-56), `requirements-completed` is empty here.

## Self-Check: PASSED

- apps/sucrose-portal/src/library/card.rs has `fn monitor_badges` (grep count 1); the locale file has the MonitorBadge keys (5)
- commits 6384091a, 3c0949c8 and c7d48dce are ancestors of HEAD
- `cargo xtask ci` exit 0; both captures exist and were viewed

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-28*
