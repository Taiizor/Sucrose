# Phase 02 — UI Review

**Audited:** 2026-11-23
**Baseline:** Abstract 6-pillar standards (no UI-SPEC.md for this phase). Scope cross-checked against 02-CONTEXT.md (D-11, D-13, D-14, D-15, D-20, TRAY-01) and the 02-09 plan and summary.
**Screenshots:** not captured (native Windows tray UI; this run forbids launching sucrose.exe or capturing the desktop, CLAUDE.md)
**Interaction captures:** off (workflow.ui_interaction_capture is false)

**Scope note:** The phase's only user-facing UI is the notification-area tray icon, its tooltip and the single Quit menu item. The video wallpaper itself is rendered by libmpv behind the desktop icons and was not visually audited. Color, Typography and Spacing have no app-authored surface here, so they are scored on what exists.

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Copywriting | 3/4 | Tooltip sentences name the cause and the object; menu label is a bare English "Quit"; no i18n (Phase 5). |
| 2. Visuals | 2/4 | One legacy icon with no state indication; icon size uses system DPI, not the taskbar monitor's DPI. |
| 3. Color | 3/4 | No app-authored color; contrast on light and dark taskbars was not checked. |
| 4. Typography | 3/4 | OS-rendered tooltip; a reason longer than 127 UTF-16 units is cut silently. |
| 5. Spacing | 3/4 | No layout surface; not applicable. |
| 6. Experience Design | 2/4 | With AppVisible=false there is no tray and no in-app stop path; a tray that fails to build is only logged. |

**Overall: 16/24** (unchanged from the previous review; one previous top fix is now resolved, two remain open)

---

## Re-check of the Previous Top Fixes

1. **No in-app stop path when AppVisible is false — OPEN.** `start_tray` (service.rs ~327) still returns `None` with no icon, menu or stop path. This is the documented D-15 / D-008 decision, but the user-facing consequence remains severe.
2. **Tray creation failure invisible — OPEN.** `start_tray` (service.rs ~338) still only calls `tracing::error!("cannot show the tray icon, Sucrose can only be stopped from Task Manager")`. The user sees nothing.
3. **Tooltip stale after transitions — LARGELY RESOLVED.** Every status change in the event loop now pushes `set_tooltip` (service.rs ~482, 578, 651, 667). `TaskbarCreated` is registered and mapped to `LoopEvent::ShellRestarted` (os_windows/event_loop.rs ~49, 103, 231), and the service drives the restart and tooltip from it (service.rs ~658). The WR-11, WR-15 and WR-17 changes are in code with matching tests (`a_failed_start_keeps_its_reason_while_the_renderer_is_started_again`, the `stopped_reason` tests). The previous "unverified" note on TaskbarCreated is now answered in code, though it is not exercised on hardware.

---

## Top 3 Priority Fixes

1. **No stop path when AppVisible is false** — the user can only stop a live wallpaper through Task Manager — Add a minimal non-tray stop path (for example a second-launch signal that asks the running instance to quit, D-23), or record AppVisible=false as an explicit Phase 3 blocker in ROADMAP.md.
2. **Tray build failure is invisible** — `start_tray` logs and the service keeps running with no visible signal — Surface the failure through a notification or a findable log-file path, and add the Task Manager fallback to the install hint.
3. **Tray icon scaled with system DPI, not the taskbar monitor's DPI** — `load_icon` (os_windows/tray.rs) uses `GetDpiForSystem` and `GetSystemMetricsForDpi`, so on mixed-DPI setups the icon can be wrong-sized — Use the DPI of the monitor that hosts the taskbar, and verify on a mixed-DPI setup (not verified here; no capture allowed).

---

## Detailed Findings

### Pillar 1: Copywriting (3/4)

- Tooltip strings come from one place, `Status::text` (service.rs ~233-235): `Sucrose — no wallpaper: {reason}`, `Sucrose — no wallpaper on {monitor}: {reason}`, `Sucrose — playing {title}`. They name the cause and the object.
- Failure hints are short; the full search list stays in the log (`MAX_REPORT_CHARS`).
- Menu label is the literal `"Quit"` (os_windows/tray.rs, `MenuItem::new("Quit", true, None)`). It matches TRAY-01 and D-14 and is the only item.
- All strings are English-only. This matches the phase decision (i18n is Phase 5), so it is a known gap for a shipped app, not a defect here.

### Pillar 2: Visuals (2/4)

- The icon is the legacy `apps/sucrose/assets/TrayIcon.ico` (D-14). Consistent with the C# app.
- Icon size comes from `GetSystemMetricsForDpi(SM_CXSMICON, GetDpiForSystem())` (tray.rs `load_icon`), i.e. system DPI, not the monitor hosting the taskbar. Scored as a visual defect risk; not verified on hardware.
- No visual state: `Status` changes only the tooltip. The legacy "grayscale icon while paused" behavior is not implemented; docs/FEATURES.md records only Quit as shipped, so this is a parity note, not a regression.
- The icon has a tooltip, so it is not unlabeled. Pass.

### Pillar 3: Color (3/4)

- The app authors no color. Icon colors come from the legacy ICO; the tooltip and menu use system theme colors.
- No hardcoded colors in the tray path.
- Held at 3, not 4, because icon contrast on light and dark taskbars was not checked (no capture).

### Pillar 4: Typography (3/4)

- No font choices in app code; the tooltip uses the OS notification font.
- `truncate_tooltip` caps at 127 UTF-16 units at a character boundary, with a unit test (`a_long_reason_is_cut_to_127_utf16_units_in_the_tooltip`).
- Held at 3 because a long reason is cut silently; the user cannot tell the text was truncated. Note that the `Failed` status now puts the reason before the hint, so the cut falls on the hint first (WR-17).

### Pillar 5: Spacing (3/4)

- No layout surface. Not applicable to a notification-area icon and its menu.

### Pillar 6: Experience Design (2/4)

- **Quit path (works):** `LoopEvent::Quit` (service.rs ~587) closes renderers on a worker thread with a shared grace period, then kills survivors. The message loop stays responsive.
- **Waiting state (works):** When no wallpaper can start, the service stays in the tray with a tooltip reason (D-13, D-20). Covered by the `Status` tests.
- **Status freshness (works, WR-11/15/17):** Tooltip is refreshed on every transition; late reports from an ended or replaced renderer are ignored (`report_is_stale`); a failed start keeps its reason through restarts; the give-up tooltip names the last failure reason first.
- **AppVisible=false (gap, WARNING):** no tray and no in-app stop (see Top Fix 1).
- **Tray build failure (gap, WARNING):** log-only (see Top Fix 2).
- **explorer.exe restart (implemented, not hardware-verified):** `TaskbarCreated` is registered and forwarded to `ShellRestarted`, which replaces renderers and updates the tooltip. The tray icon itself is recreated by `tray-icon` on a shell restart only if that library handles it, which this audit did not check.
- **Destructive action:** Quit is immediate with no confirmation. Acceptable for a tray app with one item.

---

## Files Audited

- `.planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-CONTEXT.md`
- `.planning/phases/02-m1-video-wallpaper-on-the-desktop-windows/02-09-PLAN.md` and `02-09-SUMMARY.md`
- Summaries 02-01 through 02-22 (frontmatter and first lines; gap-closure plans 02-16 to 02-22 changed no tray files)
- `apps/sucrose/src/service.rs` (Status, tray wiring, event loop, tests; lines ~200-680 and ~1140-1160, ~1940-2120)
- `crates/sucrose-platform/src/os_windows/tray.rs`
- `crates/sucrose-platform/src/os_windows/event_loop.rs` (TaskbarCreated registration and mapping)
- `apps/sucrose/src/render.rs`, `apps/sucrose/src/supervisor.rs` (TaskbarCreated comments)
- `apps/sucrose/assets/TrayIcon.ico` (referenced, not inspected)

**Not audited:** rendering of the tray icon, the taskbar, the wallpaper video, any real click on Quit, and any real explorer.exe restart. No screenshot, no app launch, no synthesized input.
