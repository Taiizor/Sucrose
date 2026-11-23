---
status: diagnosed
phase: 02-m1-video-wallpaper-on-the-desktop-windows
source: [02-VERIFICATION.md]
started: 2026-11-22T17:34:00Z
updated: 2026-11-23T15:07:38Z
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

### 7. Re-run UAT test 2 after the gap closure: explorer.exe kill twice; disable the non-primary display (FullScreenPerformance=Resume); re-enable a display that becomes primary
expected: (a) taskkill /IM explorer.exe /F then start explorer.exe, twice in a row, with SUCROSE_LOG=debug: the video is visible again on every monitor within seconds, the Renderer log shows 'surface attached ... order=SHELLDLL_DefView > SucroseDesktopSurface > WorkerW', the tray icon returns; (b) with FullScreenPerformance=Resume disable the NON-primary display: only that monitor's renderer closes within about 3 s; (c) re-enable a virtual display that becomes the primary and watch both monitors for a minute: the monitor that kept playing does not seek back to the start, the master does not move (one 'master clock elected' line), no visible offset between monitors, 'follower drift summary' lines show max drift within one frame. Covers G-02-2a, G-02-2c, G-02-2d.
result: issue
reported: "a) yaptım. video göründü her mönitörde b) düzgün çalıştı c) oldu gibi / 1. evet görünüyor tray icon 2. iki dakika oldu. sanırım gerçek ekranımda video oynatılmıyor şuan. ana ekranım virtual"
severity: major
### 8. Two minutes of 4K60 playback on the hybrid laptop with no manual graphics entry (G-02-2b, 02-18 task D4)
expected: No stutter, no 'Audio/Video desynchronisation' or 'underrun' in the Renderer logs, Task Manager shows the renderers' 3D engine on the RTX 3050, not the Intel UHD.
result: pass
note: "User: no stutter. Logs 14:49:57Z-14:52:16Z: 'GPU preference written for the renderers preference=HighPerformance' (the service wrote it after the manual entry was removed), theme Power Dancing (4K60), 'video GPU adapter name=NVIDIA GeForce RTX 3050 Laptop GPU', hwdec=d3d11va, no A/V desync, underrun or other warning in 2 min 17 s of playback"

## Summary

total: 8
passed: 5
issues: 3
pending: 0
skipped: 0
blocked: 0

## Gaps

- gap_id: G-02-2a
  truth: "After explorer.exe is killed and restarted, the wallpaper comes back on every monitor within seconds and the tray icon returns"
  status: resolved
  resolved_by: 02-16-PLAN.md
  resolved_at: 2026-11-23
  reason: "User reported: tekrar geri gelmedi videolar (the videos did not come back after the explorer restart)"
  severity: major
  test: 2
  evidence:
    - "Service.log 19:05:34Z: 'the shell restarted, replacing the renderers closing=[0, 1]'; both renderers restarted after 500ms"
    - "Renderer-0/1 19:05:35Z: 'Progman did not answer the WorkerW message within a second', then 'surface attached' anyway, but nothing was visible"
  root_cause: "The Raised-layout attach in crates/sucrose-platform/src/os_windows/surface.rs proceeds when Progman does not answer the 0x052C (spawn WorkerW) message within 1 s; the surface is placed under SHELLDLL_DefView with no wallpaper WorkerW yet, verify() only checks root/visibility/rect, and nothing re-checks z-order afterwards. Explorer later processes the queued 0x052C, creates its opaque WorkerW directly under DefView, above both Sucrose surfaces, so the video plays hidden behind the static wallpaper. Trigger: the service restarts renderers 500 ms after TaskbarCreated, while explorer's desktop thread is still busy (TaskbarCreated comes from Shell_TrayWnd's thread, not the desktop's)."
  artifacts:
    - path: "crates/sucrose-platform/src/os_windows/surface.rs"
      issue: "try_attach/send_spawn_message treat a 0x052C timeout as a warning and attach without the WorkerW; verify() does not check DefView > surface > WorkerW order; no z-order re-check after attach"
    - path: "apps/sucrose/src/service.rs"
      issue: "shell_restarted restarts renderers after a fixed 500 ms, before the desktop (Progman) is ready"
    - path: "apps/sucrose/src/render.rs"
      issue: "EXIT_SURFACE_LOST path assumes WM_DESTROY on explorer death, which did not happen in UAT (separate finding)"
  missing:
    - "Treat an unanswered 0x052C or a missing WorkerW after it as a failed attempt and retry within ATTACH_TIMEOUT (legacy retries up to 8 times with a longer timeout)"
    - "After placing the surface, keep the WorkerW below it and make verify() check the DefView > surface > WorkerW order"
    - "Event-driven (no polling, D-11) re-check of the order under Progman for a while after a shell restart, e.g. a WinEvent hook on child create/reorder"
    - "Record in docs/FEATURES.md (shell restart row) that killing explorer does not send WM_DESTROY to the surfaces"
    - "Re-run the explorer restart with SUCROSE_LOG=debug to confirm the window order"
  debug_session: .planning/debug/explorer-restart-wallpaper-not-restored.md

- gap_id: G-02-2b
  truth: "The video wallpaper plays smoothly with audio on the maintainer's hybrid-GPU laptop, as the C# MpvPlayer does"
  status: resolved
  resolved_by: 02-22-PLAN.md
  resolved_at: 2026-11-23
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
  root_cause: "The GPU preference feature (docs/FEATURES.md 'GPU preference (high performance)', DESK-12, scheduled for Phase 5) is not ported, and nothing else selects a GPU: libmpv's d3d11 device (vo=gpu, gpu-context=d3d11, no d3d11-adapter) uses DXGI adapter 0, which is the adapter driving the primary display at process start - the Intel UHD when the internal 144 Hz panel is primary. The Intel UHD cannot render the selected 4K60 HEVC theme (3840x2160, 60 fps, ~22.7 Mbps, deband=yes) in real time: 98 % 3D, A/V desync within 1 s, audio underruns, ~0.28x playback. The C# MpvPlayer writes UserGpuPreferences GpuPreference=2 for its own exe on every start, so it always runs on the RTX 3050. The smooth period before 19:05Z is explained by which display was primary when each renderer started (the virtual display's output is on the RTX 3050), not by which monitor held the master."
  artifacts:
    - path: "crates/sucrose-platform/src (Windows)"
      issue: "no UserGpuPreferences write and no capability for it"
    - path: "crates/sucrose-config/src/legacy.rs"
      issue: "Engine.json GraphicPreference is not read"
    - path: "apps/sucrose/src/service.rs, apps/sucrose/src/supervisor.rs"
      issue: "renderers are spawned from current_exe() (supervisor.rs:292) without the preference being written first"
    - path: "crates/sucrose-engine-mpv/src/config.rs"
      issue: "embedding_options select no adapter; libmpv takes the default DXGI adapter"
    - path: ".planning/REQUIREMENTS.md, docs/FEATURES.md:35"
      issue: "DESK-12 is scheduled for Phase 5 although the Phase 2 video engine depends on it"
  missing:
    - "Pull DESK-12 into the Phase 2 gap closure: sucrose-platform (Windows) writes HKCU\\Software\\Microsoft\\DirectX\\UserGpuPreferences value <current_exe() full path> = 'GpuPreference=<n>;' before the first renderer starts, n from Engine.json GraphicPreference (Default=0, PowerSaving=1, HighPerformance=2; default 2), preserving other key=value pairs and skipping the write when unchanged; exposed through the Platform trait and reported as a capability (unsupported on macOS/Linux)"
    - "Log the D3D adapter the renderer actually uses"
    - "Optional, needs a decision (not parity): pass d3d11-adapter from EnumAdapterByGpuPreference so the choice does not depend on the primary display"
    - "Update docs/FEATURES.md, .planning/REQUIREMENTS.md (DESK-12) and the uninstall item (SVC-04)"
    - "Before verifying: remove the user's manual GpuPreference=2 entry for target\\release\\sucrose.exe, otherwise it masks the fix"
  debug_session: .planning/debug/renderer-runs-on-power-saving-gpu.md

- gap_id: G-02-2c
  truth: "Plugging a monitor adds a renderer without restarting playback elsewhere, and every monitor shows the same frame (no visible offset)"
  status: resolved
  resolved_by: 02-20-PLAN.md
  resolved_at: 2026-11-23
  reason: "User reported: videoda kayma var (the video is offset between the monitors), with a screenshot taken about 40 s after re-enabling the virtual display"
  severity: minor
  test: 2
  evidence:
    - "Screenshot at 22:03 local (19:03Z) shows a clearly different pose between the two monitors (several frames apart, under the 0.5 s seek threshold, so no seek was logged)"
    - "Service.log 19:02:26Z: on plug the new renderer (DISPLAY18, primary at that moment) was elected master and the existing renderer was demoted; Renderer-0 then logged 'drift beyond half a second, seeking local=5.783333 position=0.0002494', i.e. the monitor that was already playing jumped back to the start"
    - "19:02:34Z: the layout changed again and the master moved back to slot 0"
  root_cause: "Master election is stateless: Fleet::elect -> master_slot -> elect_master re-runs after every applied topology change and picks the primary monitor's renderer, counting a renderer that was just spawned and has not connected or started playing. At 19:02:26Z Windows made the re-enabled virtual display primary, so the new renderer became master and the renderer that had played for 8.5 min was demoted; and because publish_clock publishes time-pos with paused=false before the first PLAYBACK_RESTART, the demoted renderer seeked from 5.78 s to 0.0002 s (the visible restart). A second layout change 8.8 s later flipped roles back. Contributing to the visible offset: the follower controller (sync.rs) has a fixed 1/30 s dead band (2 frames at 60 fps), corrects at 2 %/tick once per second and parks at the band edge instead of converging, so after a disturbance - and with the master running below real time on the overloaded Intel GPU (G-02-2b) - sub-0.5 s offsets of up to ~9 frames persist for over 30 s without any seek being logged."
  artifacts:
    - path: "apps/sucrose/src/service.rs"
      issue: "elect_master (795-801), Fleet::master_slot (1132-1140), Fleet::elect (1145-1162), apply_snapshot (1166-1181): stateless primary-wins re-election that also considers unconnected/non-playing renderers"
    - path: "apps/sucrose/src/render.rs"
      issue: "publish_clock (455-489)/read_tick (441-451) publish a running clock before playback starts; follow_clock (416-438) logs only seeks, not drift or speed corrections"
    - path: "crates/sucrose-engine-mpv/src/sync.rs"
      issue: "DEFAULT_FRAME_S = 1/30 fixed, SPEED_STEP = 0.02, controller parks at the band edge; seek target ignores decode time from the keyframe"
  missing:
    - "Sticky master: keep the current master while its monitor exists and it is playing; apply the primary rule only at start and when the master's monitor goes away (or, if audio must follow the primary, let the new renderer join as a follower, align, then take over)"
    - "Never elect or publish the clock of a renderer before its first PLAYBACK_RESTART"
    - "Follower controller: derive the band from the video's real fps, converge to zero instead of parking at the band edge, correct harder or tick more often after a disturbance, and log a periodic drift summary"
    - "Add a test that plugging a monitor keeps the playing master in place; update docs/FEATURES.md 'primary monitor's renderer as master clock' if the policy changes (02-CONTEXT D-02 is a phase decision, not an Accepted entry in docs/DECISIONS.md)"
  debug_session: .planning/debug/monitor-plug-restarts-playback-and-frame-offset.md

- gap_id: G-02-2d
  truth: "Unplugging (disabling) a monitor closes only that monitor's renderer"
  status: resolved
  resolved_by: 02-16-PLAN.md
  resolved_at: 2026-11-23
  reason: "Found in the logs while recording test 2: the user disabled the non-primary virtual display (Yalnızca 1 üzerinde göster) and saw the real monitor keep playing (\"düzgün çalışıyor galiba\"), but the virtual display's renderer was never closed"
  severity: major
  test: 2
  evidence:
    - "Service.log 19:09:52Z: 'applying a monitor change actions=1' with only 'the monitor changed, moving its surface slot=0 monitor=DISPLAY1 rect=[0, 0, 1920, 1080]'; no 'the monitor is gone' for DISPLAY18"
    - "Renderer-1 (DISPLAY18) kept playing and logging seeks at 19:09:56Z, 19:10:00Z and 19:10:03Z until the user chose Quit at 19:10:05Z"
    - "Contrast: at 18:59:04Z removing the virtual display while it was the PRIMARY was handled correctly (actions=2: monitor gone + surface moved + master re-elected)"
  root_cause: "crates/sucrose-platform/src/os_windows/event_loop.rs: EventLoop::run drains the PENDING queue only after GetMessageW returns, but WM_DISPLAYCHANGE, WM_DPICHANGED and WM_SETTINGCHANGE are sent messages: GetMessageW dispatches them to loop_proc internally (which queues DisplayChanged in PENDING) and does not return until a posted message arrives. Once playback runs, nothing posts to the service thread (clock ticks are handled on the reader thread without a wake, Hello/Playing are one-off, and with FullScreenPerformance=Resume no FULLSCREEN_TIMER exists), so the 19:09:15Z removal of the non-primary virtual display never started the debounce; it was drained only ~19:09:50 by an unrelated posted message, after the display had come back at 19:09:41, so no Close was produced. Primary-changing events and the Pause-mode test only looked fine because something else happened to wake the loop (the fullscreen 1 s timer in Pause mode)."
  artifacts:
    - path: "crates/sucrose-platform/src/os_windows/event_loop.rs"
      issue: "run() delivers events to the handler only after GetMessageW returns; loop_proc queues DisplayChanged (and probably ShellRestarted) from sent messages and leaves them in PENDING"
    - path: "apps/sucrose/src/service.rs"
      issue: "contributing: read_connection dispatches clock messages without waking the loop and no timer runs under Resume, so the service has no rhythm of its own; the debounce/apply logic (419-470) itself is correct"
    - path: "02-12 verification"
      issue: "used PostMessage for WM_DISPLAYCHANGE/TaskbarCreated, which cannot reveal the bug"
  missing:
    - "Deliver every queued LoopEvent without waiting for an unrelated post: e.g. loop_proc posts a WM_WAKE to its own window after queuing, or run() drains PENDING after MsgWaitForMultipleObjectsEx/QS_SENDMESSAGE wakes"
    - "Regression test in sucrose-platform: SendNotifyMessageW(WM_DISPLAYCHANGE) from another thread reaches the handler with no post (a probe exists in target/scratch-probe/pending-probe/)"
    - "Verification steps must send (SendMessageTimeout/SendNotifyMessage), not post, the shell and display messages"
    - "Check whether TaskbarCreated is affected by the same PENDING delay (risk for G-02-2a)"
  debug_session: .planning/debug/non-primary-monitor-unplug-not-detected.md

- gap_id: G-02-6
  truth: "The Rust renderer's GPU load is measured against the C# MpvPlayer on the same GPU and is not notably higher"
  status: resolved
  resolved_by: 02-22-PLAN.md
  resolved_at: 2026-11-23
  reason: "User decided (UAT test 6, option 1): handle the GPU load in Phase 2 rather than deferring it to Phase 4/8"
  severity: major
  test: 6
  evidence:
    - "02-VERIFICATION.md recorded 93.7 % GPU 3D for the Rust renderer vs 16.5 % for the C# MpvPlayer on the same 4K HEVC video; the C# MpvPlayer exe has UserGpuPreferences GpuPreference=2 (RTX 3050) while sucrose.exe had none (Intel UHD), so that comparison was most likely across different GPUs (see G-02-2b)"
    - "With sucrose.exe forced to the RTX 3050 (19:38Z-19:51Z): ~18-27 % 3D + ~14 % video decode per renderer on the 4K60 HEVC Power Dancing wallpaper (corrected by diagnosis; not 1080p-class), 0 A/V desync warnings and 0 drift seeks on the release build"
    - "02-REVIEW.md IN-10 names deband=yes and profile=low-latency (crates/sucrose-engine-mpv/src/config.rs) as the first suspects if a gap remains"
  root_cause: "The recorded 93.7 % vs 16.5 % compares two different GPUs: the Rust renderer ran on the Intel UHD (0 MB dedicated / 564 MB shared GPU memory in target/measure/rust.json) because the GPU preference is not ported (G-02-2b), while the C# MpvPlayer ran on the RTX 3050 (733 MB dedicated) thanks to its UserGpuPreferences entry; xtask/src/measure.rs reads each counter's adapter LUID and discards it, so nothing flagged the mismatch. IN-10's suspects (deband=yes, profile=low-latency) cannot explain a Rust-vs-C# difference: the C# engine's MpvPlayer.config uses the same options on effectively the same libmpv (v0.42.0-5 vs v0.42.0-38, identical built-in profiles). The rough same-GPU data shows no per-frame gap (Rust 18-27 % at 4K60 with two renderers vs C# 16.5 % at 4K30 with one). The A/V desync warnings are not a debug-build artifact: they occur on the Intel UHD with the release build (10 of 11 starts) and never on the RTX 3050."
  artifacts:
    - path: "xtask/src/measure.rs"
      issue: "around 422-439 the per-process GPU value is the max over (LUID, engine type); the adapter never reaches the JSON/table and there is no same-adapter check"
    - path: "docs/MEASUREMENTS.md"
      issue: "the M1 row and its paragraph present a cross-GPU comparison as a Rust regression"
    - path: "docs/FEATURES.md:35"
      issue: "GPU preference not ported (same fix as G-02-2b)"
    - path: "crates/sucrose-engine-mpv/src/config.rs"
      issue: "not a cause: options match the C# engine"
  missing:
    - "Close together with the G-02-2b GPU preference port"
    - "cargo xtask measure records each process adapter (LUID plus name or a dedicated-memory flag) and warns or fails when the two sides differ; optionally sample dwm.exe too (the surface is a WS_EX_LAYERED child of Progman)"
    - "Remove the user's manual GpuPreference entry for sucrose.exe, re-measure both apps on the RTX 3050 with the same 4K30 theme as 02-15 (one monitor, 60 s warm-up, 300 s) and replace the MEASUREMENTS row, noting the old one compared two GPUs"
    - "Keep deband=yes and profile=low-latency for parity; revisit options (or profile=fast for iGPU-only machines, Phase 4) only if the controlled run still shows Rust clearly higher"
  debug_session: .planning/debug/renderer-gpu-load-vs-csharp.md

- gap_id: G-02-7
  truth: "After a re-enabled display becomes the primary, every monitor keeps showing the video: the monitor that kept playing does not seek back, the master does not move, no visible offset between monitors"
  status: failed
  reason: "User reported: iki dakika oldu. sanırım gerçek ekranımda video oynatılmıyor şuan. ana ekranım virtual (after making the virtual display the main display, the video no longer plays on the real screen)"
  severity: major
  test: 7
  evidence:
    - "(a) passed: explorer restarts at 14:38:08Z and 14:38:18Z replaced the renderer; each attach logged order=\"SHELLDLL_DefView > SucroseDesktopSurface > WorkerW\"; first frame within 2-3 s on the RTX 3050; the tray icon returned (user confirmed). Only DISPLAY1 was enabled then, so the shell restart was exercised on one monitor"
    - "(b) passed: 14:40:30Z 'the monitor is gone, closing its renderer slot=1 monitor=DISPLAY9'; slot 0 untouched"
    - "Re-enable at 14:40:52Z put DISPLAY9 at [1920,0] (not primary); the new follower seeked once to the master and then held max_drift_frames 1.0-2.0 with seeks=0"
    - "Sticky master held: 14:44:31Z primary moved to DISPLAY9 (service: slot 0 DISPLAY1 -> [-1920,0,0,1080], slot 1 DISPLAY9 -> [0,0,1920,1080]); no new 'master clock elected', no seek on Renderer-0, the follower keeps receiving the master clock"
    - "14:44:40Z the service applied a second change moving slot 0 (DISPLAY1) to [1920,0,3840,1080]; Windows had applied a second layout at 14:44:38.2Z (DISPLAY1 at [1920..3840], primary unchanged) that stayed until 14:46:09Z; the service then applied 14:46:11Z (slot 0 back to [-1920,0,0,1080]), 14:46:54Z, 14:47:02Z and 14:47:22Z, each 2.4-2.7 s after the Intel Graphics Software display event (none missed or debounced away)"
    - "Read-only window snapshot 14:46:38Z: virtual screen x=-1920 w=3840; Progman [-1920,0,1920,1080]; Progman children top-first: SHELLDLL_DefView, SucroseDesktopSurface pid 12364 (slot 1) [0,0,1920,1080], SucroseDesktopSurface pid 14808 (slot 0, DISPLAY1) [-1920,0,0,1080], WorkerW; both mpv child windows visible with the same rects"
    - "Correction (diagnosis): the 14:46:38Z snapshot matches the layout the service last applied (14:46:11Z); the failure window was 14:44:40.9Z-14:46:11.7Z, during which the user reported at 14:45:47.9Z"
    - "The user confirmed the real screen showed the normal static Windows wallpaper (not black, not a frozen frame, not a shifted video) while the snapshot reported the slot 0 surface at [-1920,0,0,1080] above WorkerW"
  root_cause: "After 'Make this my main display' Windows applied two layouts: B at 14:44:28.9Z (DISPLAY9 primary [0..1920], DISPLAY1 [-1920..0]) and C at 14:44:38.2Z (DISPLAY1 moved to [1920..3840], primary unchanged), C stable until 14:46:09Z. The service applied both correctly, but explorer left Progman at layout B's rectangle [-1920..1920] for C (inferred: slot 1 kept its 1920 client offset and sat at [0..1920] in the snapshot). DesktopSurface::set_rect converts the screen target to Progman client coordinates with the origin at that instant, so DISPLAY1 [1920..3840] became client x = 3840, entirely outside Progman: the slot 0 surface was clipped to nothing while its player kept running as master, and DISPLAY1 lay outside Progman altogether, so the bare desktop showed the static wallpaper. Nothing detected or recovered it: set_rect does not verify the result or compare the target against Progman, the order guard does not watch Progman's location, and the service re-places only monitors whose own rectangle changed (the C# app re-places every engine window 2000 + 500 ms after every display change), so it stayed hidden until the next real display change (14:46:11Z)."
  artifacts:
    - path: "crates/sucrose-platform/src/os_windows/surface.rs"
      issue: "set_rect/place map the screen target into Progman client coordinates at one instant and never verify the actual rect against the target or check that Progman covers it (RectPx::covers exists but is unused); the WinEvent order guard hooks EVENT_OBJECT_CREATE..EVENT_OBJECT_REORDER only, no EVENT_OBJECT_LOCATIONCHANGE on Progman"
    - path: "apps/sucrose/src/service.rs"
      issue: "move_to/apply_snapshot send SetRect only for monitors whose rect, work area or DPI changed (no re-place of every surface after a display change, unlike the legacy app)"
    - path: "apps/sucrose/src/topology.rs"
      issue: "diff emits SetRect only for changed monitors; an origin shift or a Progman move leaves unchanged monitors' surfaces misplaced (latent second failure mode)"
    - path: "apps/sucrose/src/render.rs"
      issue: "the SetRect handler logs 'surface moved' without verifying the placement"
    - path: "docs/FEATURES.md"
      issue: "row 30 documents per-monitor diffs, unlike the legacy re-place-all (src/Shared/Engine/Sucrose.Shared.Engine/Event/Handler.cs)"
  missing:
    - "Store each surface's screen target; after every applied display change re-place every surface, not only the changed ones (legacy parity)"
    - "Re-place when Progman moves or resizes: extend the WinEvent hook with EVENT_OBJECT_LOCATIONCHANGE for Progman (event-driven, no polling)"
    - "After place(), verify the actual rect equals the target and that Progman covers it; otherwise log a warning with the Progman rect and the virtual screen and retry on the next Progman change"
    - "Decide what to do when explorer leaves a monitor outside Progman (spike)"
    - "Log the Progman origin and the local rect in 'surface moved'"
    - "Unit tests: a target outside the parent is detected; an origin shift re-places unchanged monitors"
    - "Update docs/FEATURES.md row 30"
    - "Optional confirmation (maintainer acts, no Sucrose needed): with the virtual display at [1920,0] and DISPLAY1 primary, drag the virtual display to the left of DISPLAY1 while a read-only PMv2-aware probe compares Progman's rect with the virtual screen for about 30 s"
  debug_session: .planning/debug/real-screen-static-wallpaper-after-primary-switch.md
