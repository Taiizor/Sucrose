---
phase: 03-m2-service-ipc-and-portal-connected
plan: 41
subsystem: ui
tags: [gpui, gpui-kit, window-backdrop, acrylic, mica, root-surface]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "Backdrop setting and capability (D-21, D-22), page tint (plan 03-17), apply_backdrop"
provides:
  - "theme::root_fill and theme::apply_root_fill: the base Root surface background per backdrop state"
  - "PortalApp::apply_backdrop clears the Root fill with the window appearance, deferred once while the Root is not mounted"
  - "A measured None-against-Acrylic screen comparison on the release build (D-035)"
  - "docs/GPUI.md Root surface rule"
affects: [03-43, 03-42, 03-44]

actuals:
  tokens: 2200
  tasks: 2
  commits: 3

tech-stack:
  added: []
  patterns:
    - "Root's own style background is refined after the RootPlugin styles, so it wins; looked up through window.root::<gpui_kit::base::Root>() because Root::update panics without a Base Root"
    - "A setting applied inside the view constructor is re-applied once with cx.defer_in after open_window mounts the Root"

key-files:
  created: []
  modified:
    - apps/sucrose-portal/src/theme.rs
    - apps/sucrose-portal/src/app.rs
    - docs/GPUI.md

key-decisions:
  - "The Root fill is a function of backdrop_active only: transparent black while a backdrop shows, None (gpui-component plugin fill, opaque tokens.background) otherwise"
  - "Theme tokens stay opaque; only the Root's own style changes"
  - "One deferral, no loop: a still-missing Root logs one warn and stops"

patterns-established:
  - "GPUI test for a Root-level effect: cx.add_window_view with a Blank view inside gpui_kit::base::Root, then VisualTestContext::update for the call under test (updating the Root from inside its own window.update would double-lease it)"

requirements-completed: [UI-01, UI-02]

coverage:
  - id: D1
    description: "While a backdrop is active the base Root background is transparent; with Backdrop None or an unsupported OS it keeps the opaque plugin fill"
    requirement: UI-02
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/theme.rs#theme_root_fill_is_transparent_only_while_a_backdrop_shows"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/theme.rs#theme_root_fill_follows_the_backdrop_mapping"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/theme.rs#theme_apply_root_fill_clears_and_restores_the_base_root"
        status: pass
    human_judgment: false
  - id: D2
    description: "A missing Base Root never panics or loops; the fill is applied once more after the Root is mounted"
    requirement: UI-02
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/theme.rs#theme_apply_root_fill_reports_a_window_without_a_base_root"
        status: pass
    human_judgment: true
    rationale: "The deferred re-apply inside PortalApp::new is covered by the release run (log shows root_transparent=true and the screen changes), not by a unit test"
  - id: D3
    description: "On the release build a real screen copy of the Portal with Backdrop Acrylic differs from the same window with Backdrop None in the page background (Dark)"
    requirement: UI-01
    verification:
      - kind: other
        ref: "target/backdrop-check/none.txt (flat=true) and target/backdrop-check/acrylic.txt (differs=true), screen copy per D-035"
        status: pass
    human_judgment: true
    rationale: "The difference is small on a dark desktop; the eye check of Acrylic, Mica, Tabbed and None on the active window is plan 03-43"
  - id: D4
    description: "docs/GPUI.md records the Root surface rule"
    verification:
      - kind: other
        ref: "grep -c RootPlugin docs/GPUI.md (1), D-035 (1), 'theme backgrounds need alpha' (0)"
        status: pass
    human_judgment: false

duration: 12min
completed: 2026-11-25
status: complete
plan_head_before: 214e6dca1289765f676c5d3835e28b541c00ae07
plan_head_after: f76da73a017ce9a1805a9b991040dbaaeb17e116
commits: 3
---

# Phase 3 Plan 41: Backdrop visible through the Root fill Summary

**The Portal's Mica/Acrylic/Tabbed backdrop now reaches the screen: the opaque fill gpui-component's RootPlugin paints under the view is cleared through the base Root's own style while a backdrop is active, applied at open (deferred until the Root exists), on Welcome and on Changed(Backdrop), and measured on the release build with a real screen copy.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-11-25T13:22:00Z
- **Completed:** 2026-11-25T13:34:00Z
- **Tasks:** 2
- **Files modified:** 3 (plus the untracked scratch data under target/)

## Accomplishments

- `theme::root_fill(active)` is transparent black while a backdrop shows and None otherwise; `theme::apply_root_fill` sets it on the Base Root found with `window.root::<gpui_kit::base::Root>()` and returns false (no panic) when no Root is mounted. Theme tokens are untouched.
- `PortalApp::apply_backdrop(window, cx)` sets the window appearance, applies the Root fill, and schedules exactly one `cx.defer_in` re-apply when the Root is missing (the Welcome that arrives before `open_window` mounts the Root); a Root still missing then logs one warn. The "backdrop applied" line gained `root_transparent`.
- Release e2e on synthetic data (Dark theme, 902x601 rectangle, 7684 grid points, 8 px grid): the Portal log shows `backdrop=Acrylic supported=true appearance=Blurred root_transparent=true`; measured result below.
- docs/GPUI.md: new Root surface bullet, the wrong "theme backgrounds need alpha" claim replaced, the capture bullet points to D-035.

## Screen measurement (D-035, images deleted, numbers kept)

```
none.txt:     grid=7684 page202020=1274 flat=true rect=902x601
desktop.txt:  desktop_mean=39,49,48 points=1274
acrylic.txt:  fraction_differs=0,640 points=1274 none_mean=32,32,32 desktop_mean=39,49,48 acrylic_mean=34,34,34 differs=true
```

(`0,640` is 0.640 in the Turkish decimal format of the maintainer's PowerShell; the verdict rule is fraction >= 0.5.) On this dark desktop the effect is small (mean 32 to 34 in the page background, 64 % of the page points moved by 3 or more levels); the desktop mean was far enough from #202020 for the reading not to be inconclusive. The maintainer's eye check of all four backdrops is plan 03-43.

## Task Commits

1. **Task 1 (tracer): Acrylic shows through** - RED `b0c9bd9f` (test, stubs so the tests fail on their assertions), GREEN `cad5ce59` (feat)
2. **Task 2: GPUI tests and docs** - `f76da73a` (test + docs)

**Plan metadata:** the docs(03-41) commit that adds this file.

## Files Created/Modified

- `apps/sucrose-portal/src/theme.rs` - root_fill, apply_root_fill and the four tests named in the plan
- `apps/sucrose-portal/src/app.rs` - apply_backdrop takes cx, applies the Root fill, defers once, logs root_transparent
- `docs/GPUI.md` (CRLF kept) - Root surface bullet, capture bullet, corrected backdrop bullet

## Decisions Made

None beyond the plan; D-035 (accepted before this plan) was used as written. The throwaway helper `target/backdrop-check/screen.ps1` stayed untracked under the gitignored `target/`.

## Deviations from Plan

### Auto-fixed Issues

None - plan executed as written, with two notes:

- **Task 2 test order.** The two GPUI tests of Task 2 were written after the implementation, so they passed on first run (no RED). To show they fail for the right reason, `apply_root_fill` was temporarily changed to always clear to None: `theme_apply_root_fill_clears_and_restores_the_base_root` failed with `left: (true, None)`, `right: (true, Some(transparent))`; the change was reverted before the commit.
- **Task 1 RED.** The RED commit carries stub `root_fill`/`apply_root_fill` (None / false) so the tests compile and fail on their assertions instead of on a build error.

**Total deviations:** 0 auto-fixed. **Impact:** none.

## Issues Encountered

- `.planning/STATE.md` already showed as modified when the plan started (not mine); it is committed with the state update.
- The Portal started the service itself ("no service answers, starting one"), so only `sucrose-portal.exe` had to be stopped by name; `tasklist.exe` showed no Rust Sucrose process before and after every run.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None. The RED-commit stubs were replaced in `cad5ce59`.

## Threat Flags

None. The screen copies stayed inside the Portal rectangle, no PNG is left under `target/backdrop-check` (T-03-88), the runs used synthetic data with `"DeleteCorrupt": false` (T-03-89), the page tint keeps its 0.78 alpha (T-03-90), no crate was added (T-03-SC).

## Next Phase Readiness

- G-03-5 fill and e2e done; plan 03-43 does the maintainer's eye check of Acrylic, Mica, Tabbed and None on the active window.
- Mica and Tabbed fall back to DWM's color on an inactive window (recorded in docs/GPUI.md); only Acrylic was measured here.

## Self-Check: PASSED

- FOUND: apps/sucrose-portal/src/theme.rs, apps/sucrose-portal/src/app.rs, docs/GPUI.md
- FOUND commits: b0c9bd9f, cad5ce59, f76da73a
- cargo test -p sucrose-portal theme_ 16 passed; cargo clippy -D warnings clean; cargo xtask ci exit 0
- acceptance greps: fn apply_root_fill 1, apply_root_fill in app.rs 2, root_transparent 3, RootPlugin 1, D-035 1, 'theme backgrounds need alpha' 0; docs/GPUI.md still w/crlf; no PNG under target/backdrop-check

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-25*
