---
status: testing
phase: 02-m1-video-wallpaper-on-the-desktop-windows
source: [02-VERIFICATION.md]
started: 2026-11-22T17:34:00Z
updated: 2026-11-22T17:34:00Z
---

## Current Test

number: 1
name: Every monitor, same frame, single audio source, audio promotion on unplug
expected: |
  On two displays (or a virtual display), with the C# app stopped, run sucrose.exe and watch both monitors for 5 minutes, then unplug or disable the primary display. Expected: one surface per monitor, the same frame on each (no growing offset), audio from one monitor only; after unplugging the primary the next renderer takes over audio at the configured volume.
awaiting: user response

## Tests

### 1. Every monitor, same frame, single audio source, audio promotion on unplug
expected: On two displays (or a virtual display), with the C# app stopped, run sucrose.exe and watch both monitors for 5 minutes, then unplug or disable the primary display. Expected: one surface per monitor, the same frame on each (no growing offset), audio from one monitor only; after unplugging the primary the next renderer takes over audio at the configured volume.
result: [pending]

### 2. explorer.exe restart, monitor plug/unplug, display scale change while running
expected: taskkill /IM explorer.exe /F then start explorer.exe; plug/unplug or enable/disable a (virtual) display; change the display scale while running. Expected: the wallpaper comes back on every monitor within seconds and the tray icon returns; plugging adds a renderer, unplugging closes only that monitor's renderer; a scale change moves the surfaces without restarting playback.
result: [pending]

### 3. Real fullscreen app with FullScreenPerformance Pause, then Close
expected: Set Backgroundog.json FullScreenPerformance=Pause (then Close), start a borderless-fullscreen game or browser F11, leave it, then restore Backgroundog.json. Expected: pause after 5 consecutive 1 s checks and resume after 3 without; Close stops the renderers and starts them again; desktop focus never counts.
result: [pending]

### 4. Display scale 125/150 % sharpness; tray tooltip, menu and clicking Quit
expected: Set the display scale to 125 % and 150 % and compare sharpness with 100 %; hover the tray icon, open its menu, click Quit. Expected: the video fills the monitor in physical pixels and is as sharp as at 100 %; the tooltip says what plays or why nothing does; the menu has exactly one item, Quit; Quit closes every renderer, exits 0, leaves no sucrose.exe and the normal wallpaper returns (confirm no black surface lingers after a clean Quit or a service kill).
result: [pending]

### 5. Service pipe refuses a second Windows account (T-02-11)
expected: Run the service as one user and connect a client from another Windows account. Expected: the client gets access denied (DACL D:P(A;;GA;;;<SID>)).
result: [pending]

### 6. GPU load decision: Rust renderer 93.7 % GPU 3D vs 16.5 % for the C# MpvPlayer
expected: Decide whether to fix in Phase 2 (try deband=no and the default profile instead of profile=low-latency, see REVIEW IN-10) or accept and defer to the Phase 4 performance work / Phase 8 sign-off. Also check the debug-build A/V desync warnings on a release build.
result: [pending]

## Summary

total: 6
passed: 0
issues: 0
pending: 6
skipped: 0
blocked: 0

## Gaps
