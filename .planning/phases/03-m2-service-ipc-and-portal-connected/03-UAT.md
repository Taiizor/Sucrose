---
status: partial
phase: 03-m2-service-ipc-and-portal-connected
source: [03-VERIFICATION.md]
started: 2026-11-24T20:01:31.075Z
updated: 2026-11-24T22:43:55.217Z
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
result: blocked
blocked_by: other
reason: "Maintainer chose to block it (answer: 1) until G-03-8 is fixed: thumbnail and hover-preview decoding stalls on the final build (test 8), so a D-25 working-set reading now would be falsely low. Repeat after the gap-closure plans."

## Summary

total: 10
passed: 6
issues: 3
pending: 0
skipped: 0
blocked: 1

## Gaps

- gap_id: G-03-5
  truth: "With the Dark theme and the Acrylic backdrop setting, the Portal window shows the blurred Acrylic backdrop, and changing the backdrop setting changes what the window shows"
  status: failed
  reason: "User reported: 2 backdrop etkisiz. hiç bir şey değişmedi (step 2 of test 5, Acrylic; steps 1, 3, 4, 5 and 6 passed)"
  severity: major
  test: 5
  artifacts: []
  missing: []

- gap_id: G-03-6
  truth: "With Library Pagination at its maximum (250 typed, clamped to 100, stored as LibraryPagination 100), the library page shows a scrollbar, every card scrolled into view with the mouse wheel shows its thumbnail, and hovering such a card plays its preview"
  status: failed
  reason: "User reported: 3 250 yapınca kütüphaneye döndüm kaydırma çubuğu yani scroll bar yok. ayrıca tekerlek ile aşağı inince temaların thumbnaili görünmüyor, üzerine fare tutunca da preview oynamıyor (steps 1 and 2 of test 6 passed; the clamp itself worked, Sucrose.json holds LibraryPagination 100)"
  severity: major
  test: 6
  artifacts: []
  missing: []

- gap_id: G-03-8
  truth: "On the release Portal a slowly hovered card animates and hides its title strip, and a fast sweep starts nothing"
  status: failed
  reason: "User reported: iki durum da çalışmıyor. bu kütüphane sayfasını 250 yapınca başladı sorun galiba. alttaki duvar kağıtlarının thumbnaili bile görünmüyor (maintainer screenshot with LibraryPagination 100: only the first four cards show a thumbnail, from the fifth card on the thumbnail area is empty even on the first screen; status bar shows 854 MB)"
  severity: major
  test: 8
  artifacts: []
  missing: []
