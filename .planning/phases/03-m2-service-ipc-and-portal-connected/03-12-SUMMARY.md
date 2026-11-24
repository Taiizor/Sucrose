---
phase: 03-m2-service-ipc-and-portal-connected
plan: 12
subsystem: portal
tags: [rust, gpui, ipc, use, toast, keyboard, accessibility]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: PortalRequest::Use, UseResult and Runtime events, ServiceLink, the library page and its cards (plans 06, 08, 09, 10)
provides:
  - "A card click sends Use through the service; the spinner shows until the UseResult, the Selected pill follows the service's runtime broadcast, the last click wins"
  - "use_state / CardUse / DisabledReason: the UI-SPEC priority of disabled reasons, each as tooltip and accessible description, default cursor, still focusable"
  - "Enter, Space (GPUI click on key up) and arrow-key focus movement between cards in row-major order"
  - "toast.rs: show_info and show_error over gpui-component notifications, bottom right with a 48 px bottom margin, at most 3 stacked, errors 8 s"
  - "Failed or refused Use and a Rejected setting show their toast"
  - "FEATURES.md library card row notes for D-17 and D-18"
affects: [03-13, 03-14, 03-20]
actuals:
  tokens: 9000
  tasks: 2
  commits: 2
plan_head_before: 535a025d08309f3e0cfec08980f7e3214980f612
plan_head_after: 52a3b7e21e610dfa730111991f17cb89737da93d
tech-stack:
  added: []
  patterns: ["pure decision functions (use_state, layout::neighbor, ToastStack) with unit tests, GPUI wiring around them", "toast numbers kept in an App global and passed to Notification::id1 so the 4th toast can close the oldest by number"]
key-files:
  created:
    - apps/sucrose-portal/src/toast.rs
  modified:
    - apps/sucrose-portal/src/library/card.rs
    - apps/sucrose-portal/src/library/mod.rs
    - apps/sucrose-portal/src/library/layout.rs
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/main.rs
    - docs/FEATURES.md
key-decisions:
  - "Every non-Applied UseOutcome (Failed and every Refused) shows the Use-failed toast; the service's refusals only happen when the Portal's own disabled state was stale, and the user clicked, so silence would be worse"
  - "Keyboard Use rides on GPUI's click-on-Enter/Space for focused elements instead of a key handler; only the arrow keys have an on_key_down"
  - "Errors use autohide(false) plus our own 8 s timer, because the notification layer's built-in timeout is a fixed 5 s"
requirements-completed: [ARCH-05, LIB-08]
coverage:
  - id: D1
    description: "A click on a Video theme's card applies it through the service; the Selected pill moves only on the service's broadcast"
    requirement: ARCH-05
    verification:
      - kind: other
        ref: "Task 1 run on the real library and target/scratch-data (portal registered, library loaded); the maintainer clicked a Video card and the Selected pill moved"
        status: pass
    human_judgment: true
    rationale: "The click and the desktop result need a person (synthesized input is not allowed); the maintainer confirmed the pill moved"
  - id: D2
    description: "use_state returns the disabled reasons in the UI-SPEC priority and the tooltips read as the UI-SPEC texts"
    requirement: LIB-08
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/card.rs tests (use_state and tooltip tests)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Arrow keys move focus in row-major order; Enter and Space are Use on a focused card"
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/library/layout.rs tests (neighbor and Arrow)"
        status: pass
    human_judgment: true
    rationale: "The pure neighbor rule is unit tested; the focus ring and the key handling on a real window were not driven (synthesized input is not allowed)"
  - id: D4
    description: "Toasts: at most 3, the 4th closes the oldest, errors 8 s and others 5 s; failed Use and rejected settings show their toast"
    requirement: ARCH-05
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/toast.rs tests (stack rule and durations)"
        status: pass
    human_judgment: true
    rationale: "The placement (bottom right, 48 px up), wrapping and the close button on screen were not observed: no automated run produces a failing Use or a rejected setting"
duration: ~27 min for this continuation (the first executor's Task 1 and the checkpoint wait excluded)
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 12: Use wired into the library cards Summary

**A card click now applies the theme through the service with a spinner and a broadcast-driven Selected pill, every reason Use is unavailable shows as tooltip and accessible description in the UI-SPEC priority, cards take Enter, Space and arrow keys, and failures surface as bottom-right toasts.**

## Performance

- **Duration:** ~27 min for Task 2 and close-out (Task 1 was done by the first executor, then the maintainer ran the tracer check)
- **Completed:** 2026-11-24
- **Tasks:** 2 (Task 1 tracer, Task 2 auto with tdd)
- **Files modified:** 7 (one new)

## Accomplishments

- Tracer (Task 1): the click goes card -> `PortalRequest::Use` -> service -> `UseResult` and `Runtime` -> pill; `pending_use` keeps one request in flight, last click wins, a lost connection clears it.
- Disabled reasons: not connected "Use (Unavailable)", newer theme "Update Sucrose to use this theme", non-Video (GIF, URL, Web, YouTube and Application, every kind but Video) "This wallpaper type is not supported yet.", selected and running "Use (Selected)", rule "Use (Paused)" or "Use (Closed)". A selected theme that is not running can be used again.
- Cards are focus tab stops with `Role::Button`, the theme title as label and the reason as accessible description; the focus ring is 2 px in the theme's primary color, the hover ring and the hand cursor only appear on enabled cards. There is no context menu (D-18).
- `toast.rs` configures the notification layer once (bottom right, 48 px bottom margin = status bar plus 16 px, max 3) and shows info and error toasts; `app.rs` shows the Use-failed toast and the setting-not-saved toast (consumed by plan 20's controls).
- FEATURES.md: library card row (D-17 temporary non-Video disabling until Phase 4, D-18 no "..." menu until LIB-05) and the incompatible-version row.

## Maintainer notes from the tracer check (Task 1)

1. "I'm not sure the spinner appears; if it does, it disappears very fast." The click applied the theme and the Selected pill moved. Expected with a fast local service (the request is answered within a frame or two); no artificial delay was added.
2. "There was no Web theme in the list, but there were GIF themes. They are not clickable and the cursor stays an arrow." This matches the tracer: only Video themes are usable in this phase. Task 2 gives GIF and every other non-Video kind the "not supported yet" tooltip (unit tested for all five kinds), not only Web.

## Task Commits

1. **Task 1: A card click applies the theme through the service (tracer)** - `13aa22da` (feat)
2. **Task 2: Disabled reasons, keyboard Use, toasts, FEATURES notes** - `52a3b7e2` (feat)

**Plan metadata:** docs commit for this SUMMARY, STATE.md, ROADMAP.md and REQUIREMENTS.md (follows this file).

## Files Created/Modified

- `apps/sucrose-portal/src/toast.rs` - toast helper and the pure stack rule
- `apps/sucrose-portal/src/library/card.rs` - use_state, CardUse, DisabledReason, focus, keyboard, tooltips
- `apps/sucrose-portal/src/library/layout.rs` - Arrow and the pure neighbor rule
- `apps/sucrose-portal/src/library/mod.rs` - per-card focus handles, arrow handler, use_state in the click path
- `apps/sucrose-portal/src/app.rs` - Use-failed and setting-not-saved toasts
- `apps/sucrose-portal/src/main.rs` - `mod toast;` and `toast::configure`
- `docs/FEATURES.md` - library card and incompatible-version rows

## Decisions Made

See key-decisions. The UnsupportedType tooltip is the locale key's text, which ends with a period; the plan's behavior line wrote it without one.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] clippy type_complexity on the Use handler types**
- **Found during:** Task 2 (clippy -D warnings)
- **Issue:** `Rc<dyn Fn(&mut Window, &mut App)>` in `CardData` and `cards()` (from Task 1) failed `clippy::type_complexity`; Task 1's verify does not run clippy.
- **Fix:** `card::UseHandler` and `card::ArrowHandler` type aliases.
- **Files modified:** apps/sucrose-portal/src/library/card.rs, apps/sucrose-portal/src/library/mod.rs
- **Committed in:** 52a3b7e2

**2. [Process] TDD without a separate RED commit**
- RED was run for real (14 tests failing on `todo!()` stubs: use_state, neighbor, toast stack), then GREEN made all 59 Portal tests pass. The tests are inline in the files they test (a Rust project cannot commit a failing inline test without the implementation file, the known gap in tdd.md), so Task 2 is one commit and there is no `test(...)` commit.

---

**Total deviations:** 1 auto-fixed (1 blocking), 1 process note
**Impact on plan:** none on scope.

## Issues Encountered

- A first build failed on a moved `data.title` (label taken before the strip moves it); fixed in the same task.
- The error toast's close button is gpui-component's built-in one, which shows on hover; the UI-SPEC's "with a close button" is met but it is not permanently visible.

## Known Stubs

None.

## Threat Flags

None. T-03-30 (repeated Use clicks) is mitigated: one pending Use, last wins, the identical folder is not sent twice.

## Verification

- `cargo test -p sucrose-portal`: 59 passed. `cargo clippy -p sucrose-portal --all-targets -- -D warnings`: clean. `cargo xtask ci`: passed.
- Acceptance: `fn use_state` 1, `context_menu` 0, `fn show_error` 1, `D-17` in FEATURES.md at least 1.
- Smoke run of the Task 2 build on a `synth-library` data folder (target/scratch-synth): the Portal opened without a panic (the toast configuration runs at startup), "library loaded themes=12" and "portal registered" logged, capture shows the card grid. Both apps were stopped afterwards.
- Not observed: a toast on screen, the focus ring, and keyboard handling (they need a failing Use or real key input).

## Next Phase Readiness

- Plan 20's controls can call `toast::show_error`/`show_info`; the Rejected toast is wired already.
- Maintainer end-of-phase check still open: Use on another Video theme in a release build, and the tooltips on GIF/Web cards.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
