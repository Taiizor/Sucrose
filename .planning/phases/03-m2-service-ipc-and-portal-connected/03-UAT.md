---
status: complete
phase: 03-m2-service-ipc-and-portal-connected
source: [03-VERIFICATION.md]
started: 2026-11-24T20:01:31.075Z
updated: 2026-11-25T18:45:47.755Z
---

## Current Test

[testing complete]

## Tests

### 1. Exit / Close Applications closes every Sucrose process (03-16)
expected: Wallpaper stops, Portal closes, tray icon gone within about 3 s, no sucrose.exe or sucrose-portal.exe left
result: pass

### 2. Start with the OS across a real sign-out and sign-in, then set None (03-23)
expected: Sucrose starts after sign-in and the wallpaper plays; the C# entry is unchanged; after None the Sucrose.Service Run value is gone
result: pass
note: "Verified from the registry, settings and logs only; the maintainer could not sign out. Normal wrote Run value Sucrose.Service = quoted target\\release\\sucrose.exe, no arguments, RunStartup 1 (22:15:05Z); a no-argument start of that exe played the wallpaper (22:14:40Z); None removed the value, RunStartup 0 (22:17:35Z); the C# task \\Sucrose\\Autorun for Sucrose XML was identical before and after. A real sign-in was not exercised."

### 3. Card click and keyboard Use, Web card tooltip (03-12); search typing, Escape, Ctrl+F, sort (03-14)
expected: Clicked Video theme plays, Selected pill moves; Web card shows the not-supported tooltip; search filters after about 0.5 s; Escape restores; sort reorders at once
result: pass

### 4. Second launch focuses the open Portal (03-15)
expected: Existing Portal comes to the front, no second window or tray icon
result: pass

### 5. Light/dark toggle and OS following; Acrylic with JPEG and GIF background; Help, About, Premium links; tray hint dialog (03-17, 03-18)
expected: As in the plans' expected text
result: issue
reported: "1 pass\n2 backdrop etkisiz. hiç bir şey değişmedi\n3 pass\n4 pass\n5 pass\n6 pass"
severity: major

### 6. Settings rows show migrated values and survive restart; 250 clamps to 100 (03-20)
expected: Dark, Acrylic, Visible, AppExit checked, library path, pagination 30; changes persist
result: issue
reported: "1 pass\n2 pass\n3 250 yapınca kütüphaneye döndüm kaydırma çubuğu yani scroll bar yok. ayrıca tekerlek ile aşağı inince temaların thumbnaili görünmüyor, üzerine fare tutunca da preview oynamıyor"
severity: major

### 7. Corrupt themes to the Recycle Bin and restore (03-21); library move with picker, confirmation, progress and non-empty refusal (03-22)
expected: Toast and Recycle Bin contents; library reloads from the new folder, old folder gone, non-empty refused
result: pass
note: "Logs: Corrupt Themes on 22:36:47Z, 16 corrupt themes to the Recycle Bin 22:37:07Z and restored; library moved away 22:38:47Z and back to H:\\Sucrose 22:39:19Z (entries=108 renamed=108 left_behind=0). The non-empty refusal is not logged; the maintainer answered pass."

### 8. Hover previews on the release Portal, one more look after the final code (03-28 was confirmed by the maintainer on 2026-11-24)
expected: A slowly hovered card animates and hides its title strip; a fast sweep starts nothing
result: issue
reported: "iki durum da çalışmıyor. bu kütüphane sayfasını 250 yapınca başladı sorun galiba. alttaki duvar kağıtlarının thumbnaili bile görünmüyor"
severity: major

### 9. Exit / Close Applications closes every Sucrose process (03-16); also quit once while a library move is in its closing stage if it can be provoked (WR-10 wiring is covered by predicate tests only)
expected: Wallpaper stops, Portal closes, tray icon gone within about 3 s, no sucrose.exe or sucrose-portal.exe left
result: pass
note: "Run together with test 1 (same scenario, release build of 718081ed); the maintainer answered pass to the combined checkpoint"

### 10. Release Portal paging after the WR-12 decode budget (a4e650bc): page through the synthetic 500-theme library once and read the working set
expected: Final working set stays inside the D-25 rule recorded in docs/MEASUREMENTS.md rows 16-17; no visible stall while thumbnails decode
result: pass
note: "Unblocked: G-03-8 resolved by 03-43. Repeated as one run with test 11 on the release build of 9067953d (target/synth-500, 17 pages of 30, previews on, every page scrolled to its last row), T0 21:35:44 local. Rule: pass, see test 11 and docs/MEASUREMENTS.md (row after row 17). Maintainer, verbatim: \"done\n\nfarksızdı her tur\noynadı\nperformanslıydı\""

### 11. Re-run UAT test 10 (D-25 scenario 2) on the final build, or record an explicit maintainer decision accepting rows 16-17 as the evidence for UI-07
expected: Final working set and private bytes inside the D-25 rule (first page + 48 MiB; round 3 end at most round 1 end x 1.05); a new row recorded in docs/MEASUREMENTS.md
result: pass
note: "Measured (maintainer chose 1). target/measure/portal-synth-4.json, 762 samples / 381 s, rounds from the page log: 21:36:39.8-21:38:24.5, 21:38:27.3-21:39:51.0, 21:39:56.4-21:41:20.3. Working set: first page 118.4, round ends 175/169/177, peak 197.4, final 163.4 (rule: 163 <= 168, 177 <= 184). Private: first page 110.9, round ends 167/162/170, peak 190.2, final 155.4 (155 <= 161, 170 <= 175). Attribution: unattributed, gpu atlas +25.0 of +58.9 MB private. 343 'preview playing', 400 'thumbnails requested'. Portal exit code 0; the log ends with the six close-time 'invalid window handle' ERROR lines already recorded in 03-37/03-38. Row and paragraph added to docs/MEASUREMENTS.md; no cap or code changed."

### 12. Hover previews with Library Preview switched on on the final build (optional)
expected: A slowly hovered card animates; thumbnails keep showing with the scrollbar overlay mounted
result: pass
note: "Covered by the test 10/11 run (release build of 9067953d, LibraryPreview true, 03-46 scrollbar overlay): maintainer reported previews played and thumbnails without a stall; 343 'preview playing' lines. Maintainer answered pass."

### 13. Start with the OS across a real sign-out and sign-in (optional hardening of UAT test 2)
expected: Sucrose starts after sign-in and the wallpaper plays
result: pass
note: "Maintainer exercised a real sign-out and sign-in with start with the OS on Normal and answered pass."

## Summary

total: 13
passed: 10
issues: 3
pending: 0
skipped: 0
blocked: 0

## Gaps

- gap_id: G-03-5
  truth: "With the Dark theme and the Acrylic backdrop setting, the Portal window shows the blurred Acrylic backdrop, and changing the backdrop setting changes what the window shows"
  status: resolved
  resolved_by: 03-47-PLAN.md
  resolved_at: 2026-11-25
  reason: "User reported: 2 backdrop etkisiz. hiç bir şey değişmedi (step 2 of test 5, Acrylic; steps 1, 3, 4, 5 and 6 passed)"
  severity: major
  test: 5
  root_cause: "The backdrop reaches DWM (GPUI sets the acrylic accent for Blurred and DWMWA_SYSTEMBACKDROP_TYPE for Mica/MicaAlt on a DirectComposition premultiplied-alpha swapchain), but an opaque full-window fill hides it: gpui_kit::open_window wraps the Portal view in gpui_base::Root, whose gpui-component RootPlugin::style paints .bg(tokens.background), opaque #202020 (Dark) / #F3F3F3 (Light). Plan 03-17 lowered the alpha only on the Portal's own page div (theme::page_alpha, 78 %), so Acrylic, Mica, Tabbed and None all produce the same pixels. A probe with the same gpui-kit 0.10.0 open path showed identical captures for every backdrop with the Root fill and a blurred desktop once only the Root background was made transparent."
  artifacts:
    - path: "apps/sucrose-portal/src/app.rs"
      issue: "render (435-462) lowers alpha only on the page div; apply_backdrop (258-262) never touches the Root surface fill"
    - path: "apps/sucrose-portal/src/theme.rs"
      issue: "page_alpha / backdrop_active apply the transparency to the wrong layer"
    - path: "apps/sucrose-portal/themes/sucrose.json"
      issue: "background token is opaque; it is the source of the Root fill (not a defect by itself)"
  missing:
    - "While a backdrop is active, make the Root surface background transparent (gpui_kit::base::Root::update -> style().background), at open, on Welcome and on Changed(Backdrop); restore the opaque fill for Backdrop None so GPUI's clear does not show under a 55 % background image"
    - "Do not make theme tokens transparent through Theme::global_mut (docs/GPUI.md rule; components such as the switch thumb use tokens.background)"
    - "An e2e check that compares a real screen capture (not PrintWindow, which captures no content under Acrylic) for None against Acrylic instead of trusting the 'backdrop applied' log line"
    - "Maintainer eye check of Mica and Tabbed on the active window (the probe only saw them unfocused)"
  debug_session: ".planning/debug/portal-backdrop-no-visible-effect.md"

- gap_id: G-03-6
  truth: "With Library Pagination at its maximum (250 typed, clamped to 100, stored as LibraryPagination 100), the library page shows a scrollbar, every card scrolled into view with the mouse wheel shows its thumbnail, and hovering such a card plays its preview"
  status: resolved
  resolved_by: 03-47-PLAN.md
  resolved_at: 2026-11-25
  reason: "User reported: 3 250 yapınca kütüphaneye döndüm kaydırma çubuğu yani scroll bar yok. ayrıca tekerlek ile aşağı inince temaların thumbnaili görünmüyor, üzerine fare tutunca da preview oynamıyor (steps 1 and 2 of test 6 passed; the clamp itself worked, Sucrose.json holds LibraryPagination 100)"
  severity: major
  test: 6
  root_cause: "Three separate causes. (1) No scrollbar: the library grid scrolls with overflow_y_scroll() and draws no scrollbar (dialogs use overflow_y_scrollbar()). (2) Missing thumbnails: request_page() queues only the first CACHE_IMAGES (64) cards of the page (.take(CACHE_IMAGES), since 03-10, e5d7d870); with 100 per page all 90 themes sit on one page and cards 65-90 are never requested and stay skeletons. Same cause as G-03-8. (3) No hover preview: the maintainer's real Sucrose.json has LibraryPreview false (migrated from the C# Portal.json, never changed in the logs), so hover_card returns early with 'preview switched off'; a setting, not a code defect."
  artifacts:
    - path: "apps/sucrose-portal/src/library/grid.rs"
      issue: "line 149 overflow_y_scroll() without a visible scrollbar"
    - path: "apps/sucrose-portal/src/library/mod.rs"
      issue: "request_page() .take(CACHE_IMAGES) (line 546) never requests cards past the 64th; cards() reorders the cache on every render and evicted keys that are still on screen are not requested again"
    - path: "apps/sucrose-portal/src/library/thumbs.rs"
      issue: "CACHE_IMAGES 64 and CACHE_BYTES 48 MiB (about 41 images at W720): removing .take alone would evict card 1 when card 65 arrives and never reload it"
  missing:
    - "A visible scrollbar on the library grid"
    - "Request thumbnails for every card the page can show (preferably the rows in view while scrolling) and re-request evicted keys that are on screen again, within the D-25 memory rule"
    - "Stop the endless skeleton animation frames for cards that will never load (side effect noticed by the diagnosis, not measured)"
    - "Re-run the hover part with Library Preview (Settings > Personal > Theme Preview) switched on"
  debug_session: ".planning/debug/hover-preview-and-thumbnails-stall.md"

- gap_id: G-03-8
  truth: "On the release Portal a slowly hovered card animates and hides its title strip, and a fast sweep starts nothing"
  status: resolved
  resolved_by: 03-43-PLAN.md
  resolved_at: 2026-11-25
  reason: "User reported: iki durum da çalışmıyor. bu kütüphane sayfasını 250 yapınca başladı sorun galiba. alttaki duvar kağıtlarının thumbnaili bile görünmüyor (maintainer screenshot with LibraryPagination 100: only the first four cards show a thumbnail, from the fifth card on the thumbnail area is empty even on the first screen; status bar shows 854 MB)"
  severity: major
  test: 8
  root_cause: "Two independent causes, and decoding never stalls. (1) The empty thumbnails are the 64-card request cap of G-03-6: the screenshot is scrolled, the visible cards are 61-69 in Creation Descending order, so the four with thumbnails are 61-64 and the empty ones start at card 65, the first never requested. At 30 per page every card loaded; WR-11/12/13 and the decode budget are not involved (each thumbnail holds 12.4 MB of the 256 MB budget, no decode error in the logs). (2) The hover previews do not play because the maintainer's real settings have LibraryPreview false (hover_card logs reason='preview switched off'); the UAT prompt wrongly said the preview was on."
  artifacts:
    - path: "apps/sucrose-portal/src/library/mod.rs"
      issue: "request_page() .take(CACHE_IMAGES) leaves cards past the 64th without a thumbnail"
    - path: "apps/sucrose-portal/src/library/thumbs.rs"
      issue: "cache caps (64 images / 48 MiB) smaller than a 100-card page"
  missing:
    - "Fix the request cap as listed under G-03-6 (one fix closes both)"
    - "Re-run test 8 with Library Preview switched on before judging the hover previews"
  debug_session: ".planning/debug/hover-preview-and-thumbnails-stall.md"
