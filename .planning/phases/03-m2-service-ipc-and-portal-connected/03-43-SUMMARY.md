---
phase: 03-m2-service-ipc-and-portal-connected
plan: 43
subsystem: ui
tags: [uat, gap-closure, backdrop, scrollbar, thumbnails, hover-preview, gpui]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "Backdrop fill fix (03-41) and thumbnails-follow-the-scroll with a grid scrollbar (03-42), checked here on the release Portal"
provides:
  - "The maintainer's verbatim verdicts on G-03-5 (backdrop), G-03-6 / G-03-8 (100-per-page library) and the hover previews, with Portal log counts"
  - "Two remaining gaps with leads: Mica/Tabbed look the same as Acrylic or stay opaque after None (A), and the grid scrollbar is not visible (B)"
affects: [03-44, gap-closure]

actuals:
  tokens: 4200
  tasks: 2
  commits: 0
plan_head_before: e4fe9bd8e3361446f165184113c5014b40648b4e
plan_head_after: e4fe9bd8e3361446f165184113c5014b40648b4e

tech-stack:
  added: []
  patterns: []

key-files:
  created: []
  modified: []

key-decisions:
  - "03-43 records the maintainer's check only; no code was changed, A and B go to gap closure"
  - "UI-01, UI-02, LIB-04 and LIB-06 are not marked Complete on this evidence (A and B are not a full yes)"

requirements-completed: []

coverage:
  - id: D1
    description: "Acrylic blurs what is behind the window and None is fully opaque on the release Portal (G-03-5, part)"
    requirement: UI-01
    verification: []
    human_judgment: true
    rationale: "Visual judgment by the maintainer; Acrylic and None confirmed, Mica and Tabbed refuted (see Maintainer check)"
  - id: D2
    description: "Mica and Tabbed look different from Acrylic and the backdrop stays visible switching from None (G-03-5, rest)"
    requirement: UI-02
    verification: []
    human_judgment: true
    rationale: "Maintainer answered no: the window does not change from Acrylic, and does not become transparent from None"
  - id: D3
    description: "Thumbnails of every card, past the 64th included, appear while scrolling 90 themes on one page (G-03-6, G-03-8)"
    requirement: LIB-04
    verification: []
    human_judgment: true
    rationale: "Maintainer confirmed thumbnails appear while scrolling down; the scrollbar is not visible, so the item stays open"
  - id: D4
    description: "Hover previews play the GIFs with Library Preview on"
    requirement: LIB-06
    verification: []
    human_judgment: true
    rationale: "Maintainer confirmed (C yes); log has preview playing lines"

duration: ~40min
completed: 2026-11-25
status: complete
---

# Phase 3 Plan 43: Maintainer check of the backdrop, the 100-per-page library and the hover previews Summary

**The maintainer checked 03-41 and 03-42 by hand on the release Portal: hover previews pass (C yes), thumbnails now follow the scroll to the bottom (B partly), Acrylic and None are right but Mica and Tabbed do not change the window (A no), and the grid scrollbar is not visible (B no). Nothing tracked changed.**

## Performance

- **Duration:** about 40 min (across the first executor and this continuation, including the maintainer's hand check)
- **Completed:** 2026-11-25
- **Tasks:** 2 (Task 1 tracer: prepare and prove unattended; Task 2 maintainer check)
- **Files modified:** 0 tracked (scratch only: `target/uat-gaps/Setting/Sucrose.json`, gitignored)

## Accomplishments

- Release build and libmpv (v0.42.0-38) were up to date; the scratch data folder `target/uat-gaps` was migrated read-only from the maintainer's real legacy settings (`settings loaded outcome=Migrated volume=100 looping=true stretch=UniformToFill`).
- The unattended smoke proved the run before the maintainer saw it; the maintainer then drove the release Portal with focus and answered A, B and C.
- Two gaps are now sharply described with log lines and code facts below, ready for gap closure.

## Task 1 evidence (tracer)

Scratch `Sucrose.json` before edits: DeleteCorrupt=false, ThemeMode=Dark, Backdrop=Acrylic, LibraryPagination=30, LibraryPreview=false, LibraryPreviewHide=true, HintTrayIcon=true. Changed only: LibraryPagination 30 to 100, LibraryPreview false to true, HintTrayIcon true to false.

Unattended smoke (`--no-focus`) log lines:

```
INFO sucrose_portal::app: backdrop applied backdrop=Acrylic supported=true appearance=Blurred root_transparent=true
INFO sucrose_portal::library: library loaded themes=90 corrupt=16 ignored=2
DEBUG sucrose_portal::library: page shown page=1 pages=1 thumbs=0 ... preview_playing=false
DEBUG sucrose_portal::library: thumbnails requested first=0 last=11 wanted=12 queued=12
DEBUG sucrose_portal::library: thumbnail cache images=12 bytes=3733920
```

`tasklist.exe` was empty afterwards, Task 1 verify printed VERIFY_OK, the smoke Log folder was removed, `cargo fmt` had no changes and `cargo xtask ci` exited 0.

## Maintainer check

Verbatim reply (Turkish, as sent):

> b. scroll bar yok, ya da görünmüyor. küçük resimler görünüyor artık aşağıya indikçe.
> c. sorun yok oynatıyor gifleri.
> a. arcylic dediğin gibi. none de öyle tam opak. acrylic seçtikten sonra mica ve tabbede geçince hiç bir şey değişmiyor, saydam oluyor yani. noneden mica ve tabbede geçince saydam olmuyor ama. C# halinde tabbed'de sanırım masaüstü resmi silüeti/gölgesi gibi bir şey görünüyordu normalde

Reading:

| Check | Result | Meaning |
|-------|--------|---------|
| A Backdrop | partly no | Acrylic blurs as expected and None is fully opaque. Switching Acrylic to Mica or Tabbed changes nothing (the window stays transparent like Acrylic). Switching None to Mica or Tabbed does not make the window transparent at all. The maintainer recalls that in the C# app Tabbed showed something like a silhouette/shadow of the desktop wallpaper. |
| B Library | partly no | Thumbnails now appear as they scroll down (past the 64th card too), but there is no scrollbar, or it is not visible. |
| C Hover | yes | Hover previews play the GIFs. |

Portal log counts (`target/uat-gaps/Log/Portal.2026-11-25.log`, 258 lines, one Portal run 14:03 to 14:36):

- `grep -c 'thumbnails requested'`: 104 (more than one, so scrolling requested thumbnails)
- Last `thumbnail cache` line: `2026-11-25T14:31:19.588303Z DEBUG sucrose_portal::library: thumbnail cache images=64 bytes=18900000`
- `grep -c 'preview playing'`: 24 (first `frames=16 reduce_motion=false`)
- `grep -o 'reason="[^"]*"' | sort | uniq -c`: no output; the Portal log carries no `reason="..."` field, so there are no hover reasons to count
- No WARN or ERROR lines in the log.
- `backdrop applied` lines (every one; the sequence is the maintainer switching in Settings):

```
14:03:32 Acrylic appearance=Blurred          root_transparent=true   (start)
14:29:20 None    appearance=Opaque           root_transparent=false
14:29:21 Mica    appearance=MicaBackdrop     root_transparent=true
14:29:22 Acrylic appearance=Blurred          root_transparent=true
14:29:24 None    appearance=Opaque           root_transparent=false
14:29:25 Acrylic appearance=Blurred          root_transparent=true
14:29:27 Tabbed  appearance=MicaAltBackdrop  root_transparent=true
14:29:29 Acrylic appearance=Blurred          root_transparent=true
14:32:10 None    appearance=Opaque           root_transparent=false
14:32:26 Mica    appearance=MicaBackdrop     root_transparent=true
14:32:38 Tabbed  appearance=MicaAltBackdrop  root_transparent=true
14:33:06 Mica / 14:33:07 Acrylic / 14:33:14 Tabbed / 14:33:24 Acrylic / 14:33:25 Mica
14:33:27 None / 14:33:28 Mica / 14:33:38 Tabbed / 14:33:43 Acrylic
14:34:13 None / 14:34:14 Acrylic / 14:34:20 Tabbed / 14:34:22 Mica
14:34:58 None / 14:34:59 Mica / 14:35:00 None / 14:35:01 Tabbed
14:36:17 None / 14:36:18 Tabbed  (last)
```

(All lines are `supported=true`; every Mica, Tabbed and Acrylic line has `root_transparent=true`, every None line `false`.)

Both apps were stopped with `taskkill.exe /IM sucrose-portal.exe /F` and `taskkill.exe /IM sucrose.exe /F` (PIDs 30616, 18636, 11368, 29484 terminated); `tasklist.exe` shows no Rust Sucrose process afterwards.

## Next lead for the gaps (nothing fixed in this plan)

**A (Mica/Tabbed)**: the Portal does what its own contract says. Each switch produced the right `appearance` (MicaBackdrop, MicaAltBackdrop, Blurred, Opaque) and a transparent root fill whenever a backdrop is active (`apps/sucrose-portal/src/theme.rs`: `appearance`, `backdrop_active`, `root_fill`, `apply_root_fill`; `apps/sucrose-portal/src/app.rs`: `apply_backdrop`, logged as "backdrop applied"). The symptom is therefore below the Portal: either GPUI's Windows backend does not clear the previously set DWM accent/backdrop when the appearance changes (Acrylic to Mica keeps the Acrylic blur; None to Mica shows no Mica), or the Mica effect is hidden by the page tint (`theme::TINT_ALPHA` 0.78 over the page) and a Mica material has little visible change on a dark page, which would not explain None to Mica staying opaque. First check: how `gpui`'s Windows window code (the fork pinned by gpui-kit) handles `WindowBackgroundAppearance::MicaBackdrop` and `MicaAltBackdrop` on a window created with another appearance (the call path of `set_background_appearance`), and whether the window needs to be created transparent for Mica to show. The maintainer's C# comparison (Tabbed shows a desktop-wallpaper silhouette/shadow) is the visual target for Tabbed.

**B (scrollbar)**: `apps/sucrose-portal/src/library/grid.rs` `scroll_area` builds `div().id("library-scroll").size_full().overflow_y_scroll().track_scroll(scroll).vertical_scrollbar(scroll)` and the doc comment says the scrollbar "follows the OS scrollbar mode gpui-component syncs at init". With the Windows overlay/auto-hide mode the bar only shows while scrolling or when the pointer is over its strip, and it may be thin or drawn beneath the page tint; the maintainer saw none even after scrolling. First check: the gpui-component scrollbar mode at init (always-visible versus auto-hide) and whether the bar's track is clipped by the parent of `library-scroll` or covered by a sibling in `app.rs`. Thumbnails are fine: the cache stayed at its 64-image cap (`images=64 bytes=18900000`) while cards after the 64th were drawn, so `insert_keeping` / `trim` behave as 03-42 designed.

**C**: no gap. The hover reason breakdown is empty because the Portal debug line carries no `reason` field; if the hover diagnostic needs reasons later, add the field at gap closure (out of scope here).

## Task Commits

No task produced a tracked change (`files_modified: []`), so there are no task commits.

**Plan metadata:** the docs(03-43) commit that adds this SUMMARY with STATE.md and ROADMAP.md.

## Files Created/Modified

- None tracked. Scratch only: `target/uat-gaps/Setting/Sucrose.json` and `target/uat-gaps/Log/*` (gitignored).

## Decisions Made

- The check is recorded, not fixed: the plan forbids code changes, so A and B go to gap closure after 03-44's measurement.
- Requirement status is left open: UI-01, UI-02, LIB-04 and LIB-06 are not marked Complete on this evidence because the maintainer's answers for A and B are not a full yes. The shared-ID gate also keeps them open while 03-44 has no SUMMARY.

## Deviations from Plan

None - plan executed exactly as written. The maintainer's two "no" answers are the check's findings, not deviations; they are carried as leads above.

## Issues Encountered

- G-03-5 is only partly closed (Acrylic and None right; Mica and Tabbed do not change the window).
- G-03-6 is only partly closed (thumbnails follow the scroll; the scrollbar is not visible).
- G-03-8 is closed on the maintainer's answer C yes and the `preview playing` log lines.

## Known Stubs

None found; no files were created or modified.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- 03-44 (the D-25 measurement, UAT test 10) is not blocked by these findings.
- Gap closure needs two new items before phase 3 verification can pass: the Mica/Tabbed backdrop and the library scrollbar visibility.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-25*

## Self-Check: PASSED

- SUMMARY exists; no tracked files were created or modified (commits: 0, matches git rev-list); tasklist.exe shows no Rust Sucrose process.
