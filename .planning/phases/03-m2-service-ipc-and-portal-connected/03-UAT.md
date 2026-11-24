---
status: testing
phase: 03-m2-service-ipc-and-portal-connected
source: [03-VERIFICATION.md]
started: 2026-11-24T20:01:31.075Z
updated: 2026-11-24T21:10:41.768Z
---

## Current Test

number: 1
name: Exit / Close Applications closes every Sucrose process (03-16)
expected: |
  Wallpaper stops, Portal closes, tray icon gone within about 3 s, no sucrose.exe or sucrose-portal.exe left
awaiting: user response

## Tests

### 1. Exit / Close Applications closes every Sucrose process (03-16)
expected: Wallpaper stops, Portal closes, tray icon gone within about 3 s, no sucrose.exe or sucrose-portal.exe left
result: [pending]

### 2. Start with the OS across a real sign-out and sign-in, then set None (03-23)
expected: Sucrose starts after sign-in and the wallpaper plays; the C# entry is unchanged; after None the Sucrose.Service Run value is gone
result: [pending]

### 3. Card click and keyboard Use, Web card tooltip (03-12); search typing, Escape, Ctrl+F, sort (03-14)
expected: Clicked Video theme plays, Selected pill moves; Web card shows the not-supported tooltip; search filters after about 0.5 s; Escape restores; sort reorders at once
result: [pending]

### 4. Second launch focuses the open Portal (03-15)
expected: Existing Portal comes to the front, no second window or tray icon
result: [pending]

### 5. Light/dark toggle and OS following; Acrylic with JPEG and GIF background; Help, About, Premium links; tray hint dialog (03-17, 03-18)
expected: As in the plans' expected text
result: [pending]

### 6. Settings rows show migrated values and survive restart; 250 clamps to 100 (03-20)
expected: Dark, Acrylic, Visible, AppExit checked, library path, pagination 30; changes persist
result: [pending]

### 7. Corrupt themes to the Recycle Bin and restore (03-21); library move with picker, confirmation, progress and non-empty refusal (03-22)
expected: Toast and Recycle Bin contents; library reloads from the new folder, old folder gone, non-empty refused
result: [pending]

### 8. Hover previews on the release Portal, one more look after the final code (03-28 was confirmed by the maintainer on 2026-11-24)
expected: A slowly hovered card animates and hides its title strip; a fast sweep starts nothing
result: [pending]

### 9. Exit / Close Applications closes every Sucrose process (03-16); also quit once while a library move is in its closing stage if it can be provoked (WR-10 wiring is covered by predicate tests only)
expected: Wallpaper stops, Portal closes, tray icon gone within about 3 s, no sucrose.exe or sucrose-portal.exe left
result: [pending]

### 10. Release Portal paging after the WR-12 decode budget (a4e650bc): page through the synthetic 500-theme library once and read the working set
expected: Final working set stays inside the D-25 rule recorded in docs/MEASUREMENTS.md rows 16-17; no visible stall while thumbnails decode
result: [pending]

## Summary

total: 10
passed: 0
issues: 0
pending: 10
skipped: 0
blocked: 0

## Gaps
