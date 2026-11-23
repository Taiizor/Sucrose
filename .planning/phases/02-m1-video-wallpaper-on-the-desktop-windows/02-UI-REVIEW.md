# Phase 02 — UI Review

**Audited:** 2026-11-23
**Baseline:** Abstract 6-pillar standards (no UI-SPEC.md for this phase). Scope cross-checked against 02-CONTEXT.md (D-13, D-14, D-15, D-20, TRAY-01) and 02-09-PLAN.md / 02-09-SUMMARY.md.
**Screenshots:** not captured (native Windows tray UI, no web dev server; this run forbids launching the app or capturing the desktop)
**Interaction captures:** off (workflow.ui_interaction_capture is false)

**Scope note:** Phase 02's only user-facing UI is the Windows notification-area tray icon, its tooltip, and its single Quit menu item. The Phase 02 wallpaper itself (video behind the desktop icons) is rendered by libmpv and was not visually audited. Its pillar scores below are for the tray surface only. Color, Typography and Spacing have no in-app surface here: the tray icon is the legacy ICO, and the menu and tooltip are drawn by Windows. Those three pillars are scored on what exists, with the reasoning stated.

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Copywriting | 3/4 | Tooltip sentences are clear and name the cause ("no wallpaper: ...", "playing <Title>"); menu label is a bare English "Quit"; no i18n (deferred to Phase 5 by design). |
| 2. Visuals | 2/4 | Single legacy icon with no state indication; the planned grayscale-while-paused state is not implemented; icon size uses system DPI, not the monitor the tray sits on. |
| 3. Color | 3/4 | No app-authored color; icon is the legacy ICO (color depth chosen by bit count). Nothing to grade beyond asset reuse. |
| 4. Typography | 3/4 | No in-app text rendering; tooltip is OS-rendered and cut to 127 UTF-16 units, which the code handles at a character boundary. |
| 5. Spacing | 3/4 | No layout surface; no spacing decisions exist in this phase. |
| 6. Experience Design | 2/4 | With AppVisible=false there is no tray and no stop path except Task Manager (documented, D-15). A tray that fails to build is only logged; the user gets no visible signal. |

**Overall: 16/24**

---

## Top 3 Priority Fixes

1. **No in-app stop path when AppVisible is false** — the user can only stop the wallpaper through Task Manager, which is a poor recovery path for a live-wallpaper app — Add a minimal, non-tray stop path (for example a second-launch signal that asks the running service to quit, D-23 single instance) or make AppVisible=false explicitly a Phase 3 blocker in ROADMAP.md rather than a silent gap.
2. **Tray creation failure is invisible to the user** — `start_tray` logs `cannot show the tray icon` and the service keeps running with no way to see or stop it — Surface the failure through a notification or a log-file pointer the user can find, and document the Task Manager fallback in the install hint.
3. **Tooltip is the only status surface and is not updated for every state** — the tooltip is the sole channel for "no wallpaper" reasons, so any state that is not pushed through `set_tooltip` stays stale — Audit each `Status` transition (renderer restart, backoff exhausted, explorer.exe restart) against a test that asserts the tooltip after the transition, and confirm the tray survives a `TaskbarCreated` re-creation (not verified in code; see Experience Design).

---

## Detailed Findings

### Pillar 1: Copywriting (3/4)

- Tooltip strings are built in one place (`apps/sucrose/src/service.rs`, `Status::text`, lines ~157-159): `Sucrose — no wallpaper: {reason}`, `Sucrose — no wallpaper on {monitor}: {reason}`, `Sucrose — playing {title}`. These name the cause and the object, which is good.
- Failure hints are short: a renderer failure appends the loader's hint, and the full search list stays in the log (`MAX_REPORT_CHARS`, lines ~133-135). This keeps the tooltip actionable.
- Menu label is the literal `"Quit"` in `crates/sucrose-platform/src/os_windows/tray.rs` (`MenuItem::new("Quit", true, None)`). It matches TRAY-01 and D-14, and it is the only item.
- All strings are English-only. This matches the phase decision (i18n is Phase 5), so it is not a defect here, but it is a known gap for a shipped app.
- Gap: no copy for the "waiting in the tray" state beyond the reason itself; the user is not told that Quit is the only action available.

### Pillar 2: Visuals (2/4)

- The icon is the legacy `apps/sucrose/assets/TrayIcon.ico` (reused per D-14). That is consistent with the C# app, which is a valid choice.
- Icon size is picked from `GetSystemMetricsForDpi(SM_CXSMICON, GetDpiForSystem())` in `os_windows/tray.rs` (`load_icon`). This uses the system DPI, not the DPI of the monitor the taskbar sits on, so on mixed-DPI setups the icon can be scaled wrong. Not verified on hardware (no capture allowed).
- No visual state: `Status` changes only the tooltip. The feature row in `docs/FEATURES.md` lists "grayscale icon while paused" for the legacy app; the M1 code does not implement it, and the M1 row correctly says only Quit is shipped, so this is a parity note, not a regression.
- Icon-only UI: the tray icon has a tooltip, so it is not an unlabeled icon. Pass.

### Pillar 3: Color (3/4)

- No color is authored by the app. The icon's colors come from the legacy ICO, and the tooltip and menu are drawn by Windows with system theme colors.
- `grep` for hardcoded colors in the tray path finds none. Nothing to fault.
- Score held at 3 (not 4) only because the icon's contrast on light and dark taskbars was not checked; no capture was taken in this run.

### Pillar 4: Typography (3/4)

- No font choices exist in app code; the tooltip uses the OS notification font.
- Tooltip length is enforced: `truncate_tooltip` caps at 127 UTF-16 units at a character boundary, with a unit test (`a_long_reason_is_cut_to_127_utf16_units_in_the_tooltip`). That is a real, tested guarantee.
- Score held at 3 because the tooltip is cut silently when long; the user does not see that the reason was truncated.

### Pillar 5: Spacing (3/4)

- No layout exists. Spacing is not applicable to a notification-area icon and menu. Scored neutrally at 3 with no finding.

### Pillar 6: Experience Design (2/4)

- **Quit path (works):** `LoopEvent::Quit` in `apps/sucrose/src/service.rs` (~line 430) closes renderers on a worker thread with a shared 2 s grace, then kills survivors. The message loop stays responsive. Tested by `shutdown_closes_all_then_waits_then_kills_only_the_survivors`.
- **Waiting state:** When no wallpaper can start, the service stays in the tray with a tooltip reason (D-13, D-20). This is intended. Tested by the `Status` tests.
- **AppVisible=false (gap, WARNING):** `start_tray` returns `None` and no icon exists (D-15, D-008). The app then cannot be stopped from inside the app. This is a documented decision, but the user-facing consequence is severe for a live wallpaper.
- **Tray build failure (gap, WARNING):** `start_tray` logs `cannot show the tray icon, Sucrose can only be stopped from Task Manager`. The user sees nothing; the message exists only in the log.
- **explorer.exe restart (not verified in code):** D-11 requires event-driven recovery on `TaskbarCreated`. The tray code in `tray.rs` does not re-create the icon itself; reliance on `tray-icon` for that is an assumption that was not checked. Treat as unverified.
- **Destructive action:** Quit is immediate with no confirmation. For a tray-only app with one item, this is acceptable.
- **Error and empty states:** covered by the `Status` enum (`NoWallpaper`, `Failed`, `RendererStopped`, `Playing`, `Starting`), each with a tooltip test. Good coverage.

---

## Files Audited

- `.planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-CONTEXT.md` (full)
- `.planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-09-PLAN.md` (full)
- `.planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-09-SUMMARY.md` (full)
- Frontmatter and objective (first ~40 lines) of plans 02-01..02-08 and 02-10..02-22 (no UI work; gap-closure plans 02-16..02-22 changed no tray files)
- `crates/sucrose-platform/src/os_windows/tray.rs`
- `crates/sucrose-platform/src/icon.rs`
- `crates/sucrose-platform/src/os_windows/event_loop.rs` (tray-related lines)
- `crates/sucrose-platform/src/lib.rs`, `unsupported.rs`, `os_linux.rs`, `os_macos.rs` (tray stubs)
- `apps/sucrose/src/service.rs` (tray wiring, Status, Quit path, tests)
- `apps/sucrose/assets/TrayIcon.ico` (referenced, not inspected)
- `docs/FEATURES.md` (Tray and app lifecycle row)

**Not audited:** visual rendering of the tray icon, the taskbar, the wallpaper video, and any real click on Quit. The app was not launched and no desktop capture was taken, per this run's constraints.
