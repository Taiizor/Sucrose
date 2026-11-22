---
status: complete
phase: 02-m1-video-wallpaper-on-the-desktop-windows
source: [02-VERIFICATION.md]
started: 2026-11-22T17:34:00Z
updated: 2026-11-22T20:25:58.818Z
---

## Current Test

[testing complete]

## Tests

### 1. Every monitor, same frame, single audio source, audio promotion on unplug
expected: On two displays (or a virtual display), with the C# app stopped, run sucrose.exe and watch both monitors for 5 minutes, then unplug or disable the primary display. Expected: one surface per monitor, the same frame on each (no growing offset), audio from one monitor only; after unplugging the primary the next renderer takes over audio at the configured volume.
result: pass

### 2. explorer.exe restart, monitor plug/unplug, display scale change while running
expected: taskkill /IM explorer.exe /F then start explorer.exe; plug/unplug or enable/disable a (virtual) display; change the display scale while running. Expected: the wallpaper comes back on every monitor within seconds and the tray icon returns; plugging adds a renderer, unplugging closes only that monitor's renderer; a scale change moves the surfaces without restarting playback.
result: issue
reported: |
  1. videoda kayma var
  2. tekrar geri gelmedi videolar. sucrose uygulamasından çık gir yaptım ve video donarak oynatılıyor, kasıyor yani. bir süre sonra da ses gelmiyor falan
  3. devam ediyor galiba baştan başlamıyor
  4. düzgün çalışıyor galiba
severity: blocker

### 3. Real fullscreen app with FullScreenPerformance Pause, then Close
expected: Set Backgroundog.json FullScreenPerformance=Pause (then Close), start a borderless-fullscreen game or browser F11, leave it, then restore Backgroundog.json. Expected: pause after 5 consecutive 1 s checks and resume after 3 without; Close stops the renderers and starts them again; desktop focus never counts.
result: pass

### 4. Display scale 125/150 % sharpness; tray tooltip, menu and clicking Quit
expected: Set the display scale to 125 % and 150 % and compare sharpness with 100 %; hover the tray icon, open its menu, click Quit. Expected: the video fills the monitor in physical pixels and is as sharp as at 100 %; the tooltip says what plays or why nothing does; the menu has exactly one item, Quit; Quit closes every renderer, exits 0, leaves no sucrose.exe and the normal wallpaper returns (confirm no black surface lingers after a clean Quit or a service kill).
result: pass

### 5. Service pipe refuses a second Windows account (T-02-11)
expected: Run the service as one user and connect a client from another Windows account. Expected: the client gets access denied (DACL D:P(A;;GA;;;<SID>)).
result: pass
note: "Verified by reading the pipe security descriptor at 19:51Z: O:<user SID> D:P(A;;FA;;;<user SID>) - one protected allow ACE for the interactive user only (no Everyone, Administrators or SYSTEM). The machine has no second Windows account, so the cross-account connection itself was not attempted; the user agreed to this check."

### 6. GPU load decision: Rust renderer 93.7 % GPU 3D vs 16.5 % for the C# MpvPlayer
expected: Decide whether to fix in Phase 2 (try deband=no and the default profile instead of profile=low-latency, see REVIEW IN-10) or accept and defer to the Phase 4 performance work / Phase 8 sign-off. Also check the debug-build A/V desync warnings on a release build.
result: issue
reported: "1 (decision: handle it in Phase 2 - after the GPU preference is ported, re-measure Rust and C# on the same GPU; if Rust still uses notably more, try deband=no and the default profile in the same fix)"
severity: major

## Summary

total: 6
passed: 4
issues: 2
pending: 0
skipped: 0
blocked: 0

## Gaps

- gap_id: G-02-2a
  truth: "After explorer.exe is killed and restarted, the wallpaper comes back on every monitor within seconds and the tray icon returns"
  status: failed
  reason: "User reported: tekrar geri gelmedi videolar (the videos did not come back after the explorer restart)"
  severity: major
  test: 2
  evidence:
    - "Service.log 19:05:34Z: 'the shell restarted, replacing the renderers closing=[0, 1]'; both renderers restarted after 500ms"
    - "Renderer-0/1 19:05:35Z: 'Progman did not answer the WorkerW message within a second', then 'surface attached' anyway, but nothing was visible"
  artifacts: []  # Filled by diagnosis
  missing: []    # Filled by diagnosis

- gap_id: G-02-2b
  truth: "The video wallpaper plays smoothly with audio on the maintainer's hybrid-GPU laptop, as the C# MpvPlayer does"
  status: failed
  reason: "User reported: sucrose uygulamasından çık gir yaptım ve video donarak oynatılıyor, kasıyor yani. bir süre sonra da ses gelmiyor falan / oturumu kapatıp tekrar açtım fakat video yine kasarak oynuyor / C# stabil oynatırken Rust hali donarak kasarak oynatıyor nedense (the video freezes and stutters and the audio drops; still after signing out and in; the C# app plays the same video smoothly)"
  severity: blocker
  test: 2
  evidence:
    - "Not caused by the explorer restart: it persists after signing out and in, and on a single monitor with OBS closed (19:35Z)"
    - "Single monitor, OBS closed: the only renderer uses 98 % of the Intel UHD 3D engine and logs 'Audio/Video desynchronisation detected' within 1 s of 'playback started'; with two monitors the master used ~60 % and the follower ~20 % of the same GPU"
    - "Playback runs far below real time: the master's position reached ~29.5 s of video in ~106 s of wall clock (about 0.28x); 'Audio device underrun detected' repeats; renderer memory ~630-750 MB"
    - "The machine is a laptop with Intel UHD Graphics (drives the 144 Hz panel) and an NVIDIA GeForce RTX 3050 Laptop GPU"
    - "HKCU\\Software\\Microsoft\\DirectX\\UserGpuPreferences has GpuPreference=2 (high performance) for the C# Sucrose.Live.MpvPlayer.exe (installed and dev builds) but no entry for target\\release\\sucrose.exe, so the Rust renderer runs on the power-saving Intel GPU"
    - "docs/FEATURES.md row 'GPU preference (high performance)' is not ported (legacy: Engine.json GraphicPreference, default HighPerformance, applied by SSDHG.Configure / SSSEGP.EnsureHighPerformance to the engine exe); related to UAT test 6 (93.7 % vs 16.5 % GPU 3D)"
    - "Confirmed by the user at 19:38Z: with Windows Settings > Display > Graphics set to High performance for target\\release\\sucrose.exe (adds UserGpuPreferences GpuPreference=2) playback is stable; each renderer then uses ~24 % 3D + ~14 % video decode on the RTX 3050, no A/V desync warnings, ~260-320 MB per renderer"
    - "NOTE for re-verification: that user-made GpuPreference entry for sucrose.exe now exists and masks this gap; remove it before verifying a fix that is meant to set the preference itself"
    - "Open question for diagnosis: before 19:02Z playback was smooth on the same Intel GPU while the audio-playing master renderer was on the virtual display (the follower on the real 144 Hz panel kept up without drift); the slowdown started once the master moved to the real panel (first A/V desync warning 19:02:24Z)"
  artifacts: []  # Filled by diagnosis
  missing: []    # Filled by diagnosis

- gap_id: G-02-2c
  truth: "Plugging a monitor adds a renderer without restarting playback elsewhere, and every monitor shows the same frame (no visible offset)"
  status: failed
  reason: "User reported: videoda kayma var (the video is offset between the monitors), with a screenshot taken about 40 s after re-enabling the virtual display"
  severity: minor
  test: 2
  evidence:
    - "Screenshot at 22:03 local (19:03Z) shows a clearly different pose between the two monitors (several frames apart, under the 0.5 s seek threshold, so no seek was logged)"
    - "Service.log 19:02:26Z: on plug the new renderer (DISPLAY18, primary at that moment) was elected master and the existing renderer was demoted; Renderer-0 then logged 'drift beyond half a second, seeking local=5.783333 position=0.0002494', i.e. the monitor that was already playing jumped back to the start"
    - "19:02:34Z: the layout changed again and the master moved back to slot 0"
  artifacts: []  # Filled by diagnosis
  missing: []    # Filled by diagnosis

- gap_id: G-02-2d
  truth: "Unplugging (disabling) a monitor closes only that monitor's renderer"
  status: failed
  reason: "Found in the logs while recording test 2: the user disabled the non-primary virtual display (Yalnızca 1 üzerinde göster) and saw the real monitor keep playing (\"düzgün çalışıyor galiba\"), but the virtual display's renderer was never closed"
  severity: major
  test: 2
  evidence:
    - "Service.log 19:09:52Z: 'applying a monitor change actions=1' with only 'the monitor changed, moving its surface slot=0 monitor=DISPLAY1 rect=[0, 0, 1920, 1080]'; no 'the monitor is gone' for DISPLAY18"
    - "Renderer-1 (DISPLAY18) kept playing and logging seeks at 19:09:56Z, 19:10:00Z and 19:10:03Z until the user chose Quit at 19:10:05Z"
    - "Contrast: at 18:59:04Z removing the virtual display while it was the PRIMARY was handled correctly (actions=2: monitor gone + surface moved + master re-elected)"
  artifacts: []  # Filled by diagnosis
  missing: []    # Filled by diagnosis

- gap_id: G-02-6
  truth: "The Rust renderer's GPU load is measured against the C# MpvPlayer on the same GPU and is not notably higher"
  status: failed
  reason: "User decided (UAT test 6, option 1): handle the GPU load in Phase 2 rather than deferring it to Phase 4/8"
  severity: major
  test: 6
  evidence:
    - "02-VERIFICATION.md recorded 93.7 % GPU 3D for the Rust renderer vs 16.5 % for the C# MpvPlayer on the same 4K HEVC video; the C# MpvPlayer exe has UserGpuPreferences GpuPreference=2 (RTX 3050) while sucrose.exe had none (Intel UHD), so that comparison was most likely across different GPUs (see G-02-2b)"
    - "With sucrose.exe forced to the RTX 3050 (19:38Z-19:51Z): ~18-27 % 3D + ~14 % video decode per renderer on a 1080p-class wallpaper, 0 A/V desync warnings and 0 drift seeks on the release build"
    - "02-REVIEW.md IN-10 names deband=yes and profile=low-latency (crates/sucrose-engine-mpv/src/config.rs) as the first suspects if a gap remains"
  artifacts: []  # Filled by diagnosis
  missing: []    # Filled by diagnosis
