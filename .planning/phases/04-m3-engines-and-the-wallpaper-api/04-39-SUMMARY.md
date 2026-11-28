---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 39
subsystem: engines
tags: [webview2, cef, youtube-playlist, web-wallpapers, maintainer-checkpoint, round-b, re-check]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-37 verdicts and defects D1-D9, the 2026-11-27 debug sessions and quick batch 261127-uor (D-040, D-041), 04-38 measurements"
provides:
  - "The Round B re-check on one release build that carries all 14 fix commits: Task 1 machine proofs (S1-S7, P1), the maintainer's Part 1 (CEF) and Part 2 (WebView2) verdicts, C1 and C2 with DISPLAY9 attached, and the log checks L1-L7"
  - "Six requirements marked Complete (TYPE-03, TYPE-04, WAPI-05, WAPI-06, WAPI-08 and ENG-03, the last two via the 2026-11-28 04:37 addendum for TYPE-03 and ENG-03); TYPE-05 and ENG-02 stay open for the P1a-WV2 FAIL"
affects: [verify-work, gsd-debug, round-c, round-d]
plan_head_before: b5eec90ff5dc3107c7e746df4ed897d08cef21d1
plan_head_after: b5eec90ff5dc3107c7e746df4ed897d08cef21d1
actuals:
  tokens: 8969
  tasks: 3
  commits: 0
tech-stack:
  added: []
  patterns:
    - "A measured FAIL of one probe run (P1a-WV2 run 1) keeps every requirement whose cell it is open, even though both reruns passed"
    - "A step the maintainer called ok is still judged against the Service log: Farm Weather Cycles had no Use event on CEF in A1, so that cell was recorded as not checked until the addendum re-check (2026-11-28 04:37) closed it"
key-files:
  created: []
  modified: []
key-decisions:
  - "Complete rule case 1 applies to TYPE-04, WAPI-05, WAPI-06 and WAPI-08 (every cell of their rows passed), and after the addendum to TYPE-03 and ENG-03 (Farm Weather Cycles now Used on CEF, every other cell passed); case 2 applies to TYPE-05 and ENG-02 (P1a-WV2 run 1 measured a FAIL)"
  - "WAPI-08 is listed although P1a-WV2 run 1 failed: the cell of WAPI-08 that names P1a-WV2 measures the scale key reaching the YouTube page, and that read was scale(1) in all three runs; the FAIL measured the playlist start index"
requirements-completed: [TYPE-03, TYPE-04, WAPI-05, WAPI-06, WAPI-08, ENG-03]
duration: "about 26 min of maintainer run (T0 03:56:30 to the stop at 04:22:16 local); Task 1 and the ready gate were earlier"
completed: 2026-11-28
status: complete
coverage:
  - id: D1
    description: "Release build carries every fix: 14 fix commits are ancestors of HEAD, no build input differs, four fingerprints found in the binaries"
    verification:
      - kind: other
        ref: "git merge-base --is-ancestor x14; git status --porcelain on build inputs; fingerprint counts (Task 1)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Unattended machine proofs S1-S7 on CEF and WebView2 and the automated tests of the requirement-parts table"
    verification:
      - kind: integration
        ref: "target/round-b-recheck-probe/*.json (S3-S7, P1b, P1c); cargo test pages 36 passed, sucrose-sysinfo lib 48 passed 1 ignored, payload 8, visualizer 4"
        status: pass
    human_judgment: false
  - id: D3
    description: "P1 playlist probes: CEF clean in 4 runs; WebView2 run 1 started at index 17 with shuffleMode false (FAIL), reruns 1 and 2 passed"
    requirement: TYPE-05
    verification:
      - kind: integration
        ref: "target/round-b-recheck-probe/p1a-wv2.json"
        status: fail
    human_judgment: true
    rationale: "A measured FAIL in one of three runs; a race is the analysed cause, not a proven one; the verifier or /gsd-debug must judge"
  - id: D4
    description: "Maintainer re-check of A1, A3, A5-CEF, B2-WV2, B3, B4, B5, C1 and C2 on CEF and WebView2 with two monitors"
    verification:
      - kind: manual_procedural
        ref: "Part 1 and Part 2 replies below; log checks L1-L7"
        status: pass
    human_judgment: true
    rationale: "The replies are the maintainer's eye verdicts; the one step the logs contradicted (A1, Farm Weather Cycles not Used on CEF) was re-checked in the addendum, and C2's slower-than-1-s appearance needs a human decision"
---

# Phase 4 Plan 39: Round B re-check Summary

**The re-check of the failed 04-37 steps on one release build with all 14 fixes passed url-input, the audio bars, now playing and the generated GIF, video and YouTube pages on both web engines (TYPE-04, WAPI-05, WAPI-06, WAPI-08 marked), but the WebView2 playlist probe started at index 17 with Shuffle Mode off in 1 of 3 runs (TYPE-05, ENG-02 stay open); the log showed Farm Weather Cycles was never Used on CEF in A1, and the addendum re-check that night passed it (TYPE-03, ENG-03 marked).**

## Performance

- **Duration:** about 26 min of maintainer run (Task 3); Task 1 and the ready gate ran in the earlier session of the first executor
- **Run start (T0):** 2026-11-28 03:56:30 +03 (the orchestrator started the Portal; the first Portal log line is 00:56:27.9Z = 03:56:27.9 local); the Rust processes were stopped at 04:22:16 local
- **Completed:** 2026-11-28
- **Tasks:** 3 (Task 1 and 2 by the first executor, Task 3 driven by the orchestrator and closed by this executor)
- **Files modified:** 0 source files (files_modified: []); only the SUMMARY, STATE.md, ROADMAP.md and REQUIREMENTS.md change

## Accomplishments

- Task 1 proved unattended that HEAD `b5eec90f` carries the 14 fix commits, built and fingerprinted the release binaries, migrated a new scratch data folder read-only, ran the machine proofs S1..S7 and the playlist probe P1 on both engines, and found a playlist (`PLATNX_t55hdpWZS7sShzhjQIw5q6WEG5w`, 29 videos) that plays embedded.
- The maintainer re-checked Part 1 (CEF) and Part 2 (WebView2) with DISPLAY1 and DISPLAY9 attached; every step reply was "ok" except C2, where the windows appeared within a few seconds instead of about 1 s.
- D-040 held on the desktop: only DISPLAY1's renderer logged `kind=Persistent` (67 lines, CEF and WebView2), DISPLAY9's logged `kind=Ephemeral` (67 lines), and the storage fixture counted 1, 2, 3 on DISPLAY1 and 1, 1, 1 on DISPLAY9.
- The D-041 browser-exit fix held: 83 of 84 WebView2 browser exits took 174 to 461 ms; one took 1052 ms (finding F4).
- Four requirements were marked at the end of the sitting (TYPE-04, WAPI-05, WAPI-06, WAPI-08); TYPE-03 and ENG-03 were marked after the addendum re-check (below); TYPE-05 and ENG-02 stay open with the failure recorded verbatim below.

## Task 1 proofs (first executor, results in `target/round-b-recheck-probe/`)

- **Room and build:** HEAD `b5eec90ff5dc3107c7e746df4ed897d08cef21d1`; all 14 fix commits are ancestors (a0e22d9a, cf8f7d6a, 44758204, 747bbf92, 6c6d9a3c, adfb0922, 6fb2c103, d860631f, bd84d9af, 8668fb8b, c1ac7f05, e747c7be, cd43de7e, 9e5c67e6); `git status --porcelain` over the build inputs printed nothing; `cargo xtask fetch-engines` and `cargo build --release -p sucrose -p sucrose-portal` ran after that check. Fingerprints: embedded Windows 10 manifest GUID 1, `msSmartScreenProtection` 1 and `SucroseHidePlayerSiblings` 2 in `sucrose.exe`, `WebViewSmartScreen` 1 in `sucrose-portal.exe`. E: had 123 GB free at the end of this run.
- **S1 (CEF smoke):** CritterCrossing loaded with status 200; the GPU process exited 0 in both CefRenderer logs; the Service logged `renderers started count=2 monitors=["\\.\DISPLAY1", "\\.\DISPLAY9"] primary=Some("\\.\DISPLAY1") web_profile_owner=Some("\\.\DISPLAY1")`; DISPLAY1 kind=Persistent, DISPLAY9 kind=Ephemeral.
- **S2 (WebView2 smoke):** the engine started and the page loaded on both monitors.
- **Reachability (curl.exe):** the url-input site, expired.badssl.com and the YouTube playlist page each answered 200 (url-input code 200, so B3 was a real check).
- **Library and playlist:** 38 theme folders including `Ray Music Visualizer-1` and `youtube-playlist`; `seek/clip.mp4` is vp09; the first list `PLATNX_t55hdpWZS7sShzhjQIw5q6WEG5w` plays embedded with n=29 and the first title "Dark King Crown Red Aesthetic Live Wallpaper" (the second list was not needed).
- **P1a CEF (playlist, Shuffle Mode off, Loop Mode on):** passed in 4 of 4 runs: indices seen 0, 1, 28, 0 (start at 0, advance to 1 at the video's end, the last video loops back to 0), `shuffleMode` false reached the page, `scaleKey` scale(1). **P1b CEF (single video):** passed: the end restarts from the start (series 43198, 43199.1, 0).
- **P1a-WV2 (playlist on WebView2): run 1 FAILED, reruns 1 and 2 passed.** Run 1 (probe started 00:47:50Z), verbatim fields: `startIndex: 17`, `shuffleMode: false`, `advancedTo: 18`, `expectedAdvance: 18`, `loopedToZero: true`, `indicesSeen: [17, 18, 28, 0]`, `scaleKey: scale(1)`. With Shuffle Mode off the list must start at index 0; it started at 17. Reruns 1 (00:48:13Z) and 2 (00:48:28Z): `startIndex: 0`, `advancedTo: 1`, `indicesSeen: [0, 1, 28, 0]`. The first executor analysed the cause as a race: the YouTube page defaults `shuffleMode` to true, so the first `-1` state change can call `playVideoAt(random)` before the engine's `setShuffle(false)` reaches the page; CEF was clean in 4 runs. This is an analysis, not a proof.
- **P1b-WV2:** passed (series 43198.4, 43199.4, 0.3, restarted from the start). **P1c-WV2 (Shuffle Mode on):** passed: `shuffleMode: true`, start index 26, advance 27.
- **S3 (D3, certificates, WebView2):** with Allow invalid certificates on, expired.badssl.com loaded in 4 of 5 runs (0.55 s, 1.06 s, 1.57 s and 3.59 s after the probe start) and run 1 did not load within about 13 s (analysed as badssl.com being slow: curl.exe answered 200 and the later runs loaded); the Renderer logs hold 0 `certificate error allowed` lines (re-counted in this run: 0 in all six copied logs). With the setting off the frame was refused (`chrome-error://chromewebdata/`, `unreachableUrl: https://expired.badssl.com/`).
- **S4 (video page, WebView2):** passed (3840x2160 clip, not paused, time advancing 0.63 to 1.64 s, fills the 1920x1080 viewport, `scale(1)`). **S7 (video page, CEF):** passed (time 1.35 to 2.36 s, same checks).
- **S5 and S5-CEF (audio page):** passed on both engines: three arrays of 128 values, each in [0,1] (silence is enough), the media keys present (State, Title, Artist, Thumbnail and the rest; no values recorded, by T-04-83).
- **S6 and S6-CEF (GIF page):** passed (canvas 480x270 fills the 1920x1080 viewport, `scale(1)`, the two checksums differ so the frames move).
- **Automated tests of the parts table:** crates/sucrose-engine-web tests/pages.rs 36 passed; sucrose-sysinfo lib 48 passed, 1 ignored; tests/payload 8 passed; tests/visualizer 4 passed.
- **Side effect:** the step-3 migration start briefly showed the maintainer's real selected wallpaper on both monitors for about 1 s (the migration reads his settings read-only; nothing was written to his real data). Final check in this run: the scratch `Sucrose.json` holds `DeveloperPort` 0 as in the migrated copy, and `git status` showed a clean tree.

## Ready gate (Task 2)

Reply, verbatim: "start" (translation: start). Precondition at that moment, checked by the orchestrator: no Rust and no C# Sucrose process ran; DISPLAY9 attached (\\.\DISPLAY1 1920x1080 primary, \\.\DISPLAY9 1920x1080 at X=1920).

## Task 3: the maintainer's run

**T0:** the orchestrator started the Portal at 03:56:30 local (+03:00) with focus from the repository root: `SUCROSE_DATA_DIR="$(wslpath -w target/round-b-recheck)" SUCROSE_LOG=info,sucrose_engine_webview2=debug WSLENV=SUCROSE_DATA_DIR:SUCROSE_LOG target/release/sucrose-portal.exe`. 8 s later: 1 sucrose-portal.exe and 11 sucrose.exe processes (the service, two renderers and their engine processes); the Log folder held the six logs.

**Timeline read from the logs** (local = UTC+3; the relay times in the run log, "Part 1 answered about 04:18", are the times the replies reached the orchestrator): Part 1 steps ran from 03:57:37 (first Use) to 04:05:00; the engines were switched to WebView2 at 04:05:59 (Url) and 04:06:01 (Web, the wallpaper restarted on WebView2 on both monitors) and 04:06:02 (YouTube); Part 2 steps ran from 04:06:33 to 04:18:51 (Video Player back to the maintainer's choice, Mpv). Both replies reached the orchestrator after the steps were done.

### Replies (verbatim, with translations)

Part 1 reply (his own line breaks; he sent the step list instead of the words "Part 1 done" and the orchestrator took it as Part 1 done):

```
a1 ok
a5 ok
b3 ok
b4 ok
b5 ok
```

Translation: A1 ok, A5-CEF ok, B3-CEF ok, B4-CEF ok, B5-CEF ok.

Part 2 reply (verbatim, his own line breaks):

```
a3 ok
c1 ok
c2 her şey ok ama 1sn içinde değil bir kaç saniye içinde iki monitörde de görünüyor. yine çok uzun değil..
b3 ok
b4 ok
b5 ok
b2 ok
```

Translation: A3 ok; C1 ok; C2: "everything is ok, but not within 1 s - it appears on both monitors within a few seconds. Still not very long.."; B3-WV2 ok; B4-WV2 ok; B5-WV2 ok; B2-WV2 ok.

His one-word "ok" covers the step as the orchestrator presented it (each sub-check: the white-frame check, the "Test Sucrose API" note, the shared-edge glances, the drag on the empty desktop and the window-in-front sentence, no player UI in the first 10 s, Shuffle Mode off then on, bars and title/artist/cover, the 1/2/3 storage counts, the four Uses about 2 s apart). The optional DISPLAY9 glance of B4-CEF was not reported separately: it is recorded as not separately confirmed and is no required proof.

### What the Service log shows he Used (local times; the check behind each "ok")

| Step | Uses in the Service log |
|---|---|
| A1 (CEF) | Critter Crossing (started by the Portal 03:56:29), Frog Rush 03:57:39, Glitch Maze 03:58:02, Interactive Rocket 03:58:17, Sky Blitz 03:58:37, Voxel Heat 03:58:49, Test Sucrose API 03:59:21: **7 of his 8 Web themes; Farm Weather Cycles has no Use event on CEF** |
| A5-CEF | seek 03:59:50 and again 04:00:18 |
| B3-CEF | url-input 04:00:31 |
| B4-CEF | Floating in Space 04:01:35; Shuffle Mode off 04:02:03; youtube-playlist 04:02:07; Shuffle Mode on 04:02:23; hello 04:02:28; youtube-playlist 04:02:33, 04:02:56 and 04:03:25 |
| B5-CEF | Ray Music Visualizer 04:04:15, Music Tunnel 04:04:24, the audio fixture 04:04:43 (Neomorphic Music Player was not Used on CEF; the plan allows Music Tunnel) |
| A3 (WebView2) | Critter Crossing 04:06:35, Frog Rush 04:06:42, Glitch Maze 04:06:48, Interactive Rocket 04:06:53, Sky Blitz 04:07:01, Voxel Heat 04:07:04, Test Sucrose API 04:07:11, and Farm Weather Cycles 04:12:28 (later in the sitting): all 8 Used |
| C1 | storage 04:08:02, hello 04:08:29, storage 04:08:36, hello 04:09:24, storage 04:09:28 |
| C2 | Critter Crossing 04:10:32, Frog Rush 04:10:39, Glitch Maze 04:10:41, Sky Blitz 04:10:44, Test Sucrose API 04:10:47, Sky Blitz 04:10:51 (six Uses, not four; the 4th was Sky Blitz and the last one stayed) |
| Between C2 and B3-WV2 (no step) | two synthetic Video themes at 04:11:07 and 04:11:13 (see L1), Synthetic theme 3, the audio and calls fixtures, Spirit Tree, Space Train, seek, traversal, GIF and video fixtures on the built-in renderer |
| B3-WV2 | url-input 04:13:29, 04:13:56 and 04:16:11 |
| B4-WV2 | Floating in Space 04:14:31, Spirit Tree 04:14:35, Space Train 04:14:38, youtube-playlist 04:15:42 (Shuffle Mode was still on from Part 1) |
| B5-WV2 | audio fixture 04:17:13, Ray Music Visualizer 04:17:18, Music Tunnel 04:17:22, Neomorphic Music Player 04:17:31 |
| B2-WV2 | Video Player set to WebView2 04:17:57, video-clip 04:18:19 (WebView2 engine started 04:18:17), Video Player back to Mpv 04:18:51 (the wallpaper restarted on Mpv on both monitors) |

## Log findings (copied to `target/round-b-recheck-logs/` before and again after the stop; the numbers are from the second copy)

- **L1 (panicked and ERROR):** `panicked`: 0 in all six logs. ERROR lines: 36 in the two Renderer logs (18 each), none in the Service or Portal logs; all are one failure repeated: `libmpv: Failed to recognize file format`, `the video cannot be played error=unrecognized file format`, `the video could not be played, the renderer exits as a failed start` (12 each across both renderers). They come from the maintainer Using the synthetic Video themes of the scratch library at 04:11:07 and 04:11:13: their `Video.mp4` is a 16-byte placeholder file made by `cargo xtask synth-library`. After 3 crashes in 60 s the service stopped restarting the renderers and waited in the tray as designed (`the renderer stopped after 3 crashes in 60 s`). It is no defect and no step uses these themes. The Portal was ended by taskkill, so no Portal-close ERROR lines exist.
- **L2 (CEF GPU process):** 0 `GPU process exited` lines in both CefRenderer logs (each holds one line, a Chromium identity-consistency WARNING). a0e22d9a holds.
- **L3 (profile kinds, D-040):** Renderer-0 (slot 0, `monitor="\\.\DISPLAY1"`): 25 lines `kind=Persistent engine="Cef"` (path `target\round-b-recheck\Web\Cef`) and 42 lines `kind=Persistent engine="WebView2"` (path `...\Web\WebView2`). Renderer-1 (slot 1, `monitor="\\.\DISPLAY9"`): 25 lines `kind=Ephemeral engine="Cef"` (`Cache\WebEphemeral\Cef\1`) and 42 lines `kind=Ephemeral engine="WebView2"` (`Cache\WebEphemeral\WebView2\1`). `kind=Persistent` on DISPLAY9: 0; `no web profile folder`: 0. The plan's second verify command exits 0.
- **L4 (storage, C1):** Renderer-0 (DISPLAY1) `path=/__storage/1` at 04:08:02, `/__storage/2` at 04:08:35, `/__storage/3` at 04:09:28; Renderer-1 (DISPLAY9) `/__storage/1` at 04:08:02, 04:08:35 and 04:09:28. This matches the maintainer's "c1 ok".
- **L5 (WebView2 browser exits, C2 and every switch):** 83 `webview2 browser process exited` lines with `elapsed_ms` 174 to 461 (all under 1000; during C2 241 to 388), and **one** `WARN sucrose_engine_webview2::host::imp: the WebView2 browser process did not exit in time; its profile folder may still be in use elapsed_ms=1052` in Renderer-0 (DISPLAY1) at 04:12:22 local, closing a YouTube page (Spirit Tree, replaced by Space Train at 04:12:21). The plan's Edge truth for ENG-02 concurrency ("the copied logs hold no `did not exit in time`") therefore measured one hit (finding F4). `no web profile folder`: 0.
- **L6 (order, C2):** the `wallpaper status status=Sucrose — playing ...` lines follow the six Uses in order (Critter Crossing, Frog Rush, Glitch Maze, Sky Blitz, Test Sucrose API, Sky Blitz) and the last is Sky Blitz at 04:10:51, before the next theme (the synthetic Video theme at 04:11:07): no status out of order, none left on a previous theme.
- **L7 (engine started after each change):** the Web Engine change at 04:06:01 logged `engine changed, the wallpaper restarts kind=Web engine=WebViewLive renderers=2` and `engine started engine="WebView2"` followed on Renderer-0 at 04:06:01.79 and Renderer-1 at 04:06:01.85; the Url and YouTube Engine changes restarted nothing (no wallpaper of those kinds was running); the Video Engine change to WebView2 at 04:17:57 restarted nothing (a Web wallpaper ran), the next video Use started `WebView2` on both renderers at 04:18:17, and the change back to Mpv at 04:18:51 restarted the wallpaper with `engine started engine="Mpv"` on both at 04:18:51.6. Totals per renderer: 25 Cef, 42 WebView2, 8 Mpv starts.
- **Service WARN lines:** `cannot send a message to a renderer, its writer stops` 43 and `cannot send a role change to a renderer` 12, at renderer switches, as in 04-20 and 04-37.
- **C2 timing (for the pending debug session webview2-browser-exit-slow):** from the Use to the `page loaded` line, DISPLAY1: 2.35, 1.05, 0.84, 2.05, 0.68, 1.19 s; DISPLAY9: 2.36, 1.04, 1.06, 2.06, 0.93, 1.12 s (median 1.09 s; 3 of 12 under 1 s; the two monitors differ by 0.01 to 0.25 s). One slow Use split (04:10:42.8, Sky Blitz): the old browser exited after 331 ms, the new WebView2 started 0.76 s after the Use, and the page itself took 1.29 s more. So the browser exit (the D-041 fix) is no longer the long part; the creation of the new WebView2 and the page load are.
- **Cleanup:** `taskkill.exe /IM sucrose-portal.exe /F` ended PID 12572 and `taskkill.exe /IM sucrose.exe /F` ended PIDs 37184, 93348 and 29480; `tasklist.exe` then showed 0 Rust Sucrose processes. msedgewebview2.exe and the C# Sucrose processes were not touched. The maintainer may start his C# app again.

## 04-37 defects and where they were re-checked (the plan's table with outcomes)

| Defect | Fix in the build | Re-checked by | Outcome |
|---|---|---|---|
| D1 CEF Web themes stuck on loading (and the GPU process crash) | a0e22d9a | A1, L2 | 7 of 8 themes reached "playing" and he said ok; GPU exits 0 (L2); 7 of 8 themes in the sitting; **Farm Weather Cycles re-checked on CEF in the addendum (04:37): ok, page loaded 200, GPU exits 0** |
| D2 white border at the screen edges | 6c6d9a3c | A1, A3 | "a1 ok" and "a3 ok" (white-frame check while loading and after, and the shared-edge glances, each part of the step as presented); not separately reported |
| D3 WebView2 certs frame | cf8f7d6a | S3 (Task 1) | on: loaded in 4 of 5 runs within 3.6 s, 0 `certificate error allowed` lines; one run did not load within about 13 s (badssl.com slow); off: refused. **Mostly re-checked; flagged for /gsd-verify-work (F5); it blocks no mark** |
| D4 CEF seek video never visible | a0e22d9a | A5-CEF | "a5 ok"; seek Used at 03:59:50 and 04:00:18 |
| D5 WebView2 video-clip black | 44758204 | B2-WV2, S4 | "b2 ok"; S4 passed |
| D6 url-input black on CEF, no mouse reaction on WebView2 | a0e22d9a, 747bbf92 | B3-CEF, B3-WV2 | "b3 ok" twice (drag, and the window-in-front sentence of each) |
| D7 YouTube player UI visible | adfb0922 | B4-CEF, B4-WV2 | "b4 ok" twice |
| D8 audio bars silent | none needed (wrong test themes) | B5-CEF, B5-WV2 | "b5 ok" twice with Ray Music Visualizer and Music Tunnel / Neomorphic Music Player |
| D9 persistent profile not on the primary monitor | bd84d9af, 8668fb8b | C1, L3, L4 | "c1 ok"; L3 and L4 confirm (Persistent only on DISPLAY1; storage 1, 2, 3 and 1, 1, 1) |
| D10 slot-1 WebView2 profile folder locked after a quick switch, the about 6 s browser exit | c1ac7f05, e747c7be | C2, L5 | no `no web profile folder` line; exits 174 to 461 ms in 83 of 84; one 1052 ms exit (F4); C2 "ok but a few seconds, not 1 s" (F3) |
| D11 one key press reacts twice | 6fb2c103 | none | not re-run: maps to no requirement open in this plan (input forwarding is WAPI-07, partial until Round E); confirmed on 2026-11-27 (keyboard-input-doubled) |
| D12 CEF takes the foreground at a start | d860631f | none | not re-run: maps to no requirement; confirmed on 2026-11-27 21:24 (cef-start-steals-focus) |
| D13 occasional duplicate mousedown | none | none | not re-run: no fix exists, stays an open finding in STATE.md, not judged here |

## Requirement parts per engine, with each cell's outcome

"pass" = the proof passed; "FAIL" = a proof measured a FAIL; "not checked" = no evidence. A requirement is listed only when every cell of its rows passed.

| Requirement: part | CEF | WebView2 |
|---|---|---|
| TYPE-03: a Web theme (local HTML) plays | A1: pass (7 themes Used in the sitting and ok; Farm Weather Cycles Used on CEF in the addendum, "ok") | A3: pass (8 of 8 Used, "a3 ok") |
| TYPE-04: a URL wallpaper plays | B3-CEF: pass | B3-WV2: pass |
| TYPE-04: it receives forwarded input | B3-CEF: pass (drag, window-in-front sentence) | B3-WV2: pass |
| TYPE-04: its mute and volume set | PROP-05 by the requirement's own text (D-06), not claimed | PROP-05, not claimed |
| TYPE-05: IFrame API page, no API key | pass: `the_youtube_page_has_the_legacy_functions_the_legacy_player_vars_and_only_two_addresses` (pages.rs, 36 passed); loads in B4-CEF: pass | pass: the same test; loads in B4-WV2: pass |
| TYPE-05: playlist | B4-CEF: pass; P1a: pass (4 of 4 runs) | B4-WV2: pass; **P1a-WV2: FAIL (run 1 of 3)** |
| TYPE-05: shuffle | B4-CEF: pass; P1a: pass (shuffleMode false reached the page) | **P1a-WV2: FAIL (run 1: shuffleMode false but start index 17)**; P1c-WV2: pass (shuffleMode true, start 26) |
| TYPE-05: loop | P1a: pass (list loop), P1b: pass | P1a-WV2: pass in all 3 runs (loopedToZero true), P1b-WV2: pass |
| TYPE-05: hidden player UI | B4-CEF: pass | B4-WV2: pass |
| ENG-02: hosts Web wallpapers | - | A3: pass |
| ENG-02: hosts URL wallpapers | - | B3-WV2: pass |
| ENG-02: hosts YouTube wallpapers | - | B4-WV2: pass; **P1a-WV2: FAIL (run 1 of 3)** |
| ENG-02: the C#-compatible JavaScript API | - | 04-37 A5 calls, keys, tables, traversal and seek: stand; C1: pass (L4); S5: pass |
| ENG-03: hosts Web wallpapers | A1: pass (as above, 8 of 8 with the addendum) | - |
| ENG-03: hosts URL wallpapers | B3-CEF: pass | - |
| ENG-03: hosts YouTube wallpapers | B4-CEF: pass; P1a: pass | - |
| ENG-03: the same JavaScript API | 04-37 A5 calls, keys, tables, traversal, storage: stand; A5-CEF: pass; S5-CEF: pass | - |
| WAPI-05: the audio spectrum reaches the page | S5-CEF: pass; B5-CEF: pass | S5: pass; B5-WV2: pass |
| WAPI-05: it follows the music | B5-CEF: pass | B5-WV2: pass |
| WAPI-05: 128 bands | S5-CEF: pass (3 arrays of 128 values in [0,1]); `the_spectrum_arrays_hold_128_numbers_each_in_the_legacy_assignment`: pass | S5: pass; the same test |
| WAPI-05: logarithmic | the visualizer tests (sucrose-sysinfo lib 48 passed, tests/visualizer 4 passed): pass | the same tests: pass |
| WAPI-05: left, right and mono | S5-CEF: pass; `a_left_only_signal_leaves_the_right_bands_at_zero_and_the_mono_peaks_at_half`: pass | S5: pass; the same test |
| WAPI-06: now-playing data (title, artist) | B5-CEF: pass; S5-CEF: keys present | B5-WV2: pass; S5: keys present |
| WAPI-06: with thumbnail | B5-CEF: pass (the cover); S5-CEF: ThumbnailString key present; `media_keys_replace_their_defaults_in_place_and_never_touch_the_spectrum_or_the_thumbnail`: pass | B5-WV2: pass; S5: pass; the same test |
| WAPI-08: GIF as a generated page | S6-CEF: pass (04-37 B2 also stands) | S6: pass |
| WAPI-08: Video as a generated page | S7: pass | B2-WV2: pass; S4: pass |
| WAPI-08: YouTube as a generated page | B4-CEF: pass | B4-WV2: pass |
| WAPI-08: with their property keys | the pages.rs tests (36 passed): pass; the `scale` key reached the GIF page (S6-CEF), the video page (S7) and the YouTube page (P1a): `scale(1)`: pass | the same tests: pass; `scale(1)` reached the GIF page (S6), the video page (S4) and the YouTube page (P1a-WV2, all 3 runs): pass |

## Edge truths and their evidence

- **TYPE-03 adjacency (shared edge, DISPLAY1 and DISPLAY9 at x=1920):** the glance at the end of A1 (CEF) and at the end of C2 (WebView2) was part of each step as presented; his replies "a1 ok" and "c2 her şey ok" cover it; neither glance was reported separately.
- **TYPE-03 empty (the 7 Web themes without SucroseCompatible.json and Test Sucrose API):** WebView2: all 8 left their loading screen ("a3 ok"; Farm Weather Cycles loaded at 04:12:28 with status 200 on both monitors). CEF: 7 of 8 in the sitting, the 8th (Farm Weather Cycles) in the addendum: all 8 left their loading screen. Interactive Rocket's own property calls throwing is theme content and was not counted.
- **TYPE-03 ordering:** L6 holds (statuses in order, the last Use stayed); his C2 verdict says everything was ok.
- **ENG-02 concurrency:** L3 holds (only DISPLAY1 Persistent); L5 measured one `did not exit in time` line (1052 ms), so this Edge truth is **not fully met** (finding F4); no `no web profile folder` line.
- **ENG-03 concurrency:** L2: 0 GPU process exits; L3 holds (Cef folder Persistent on DISPLAY1 only, Ephemeral on DISPLAY9); met.
- **TYPE-04 adjacency:** the window-in-front sentence of each B3 was part of the step; "b3 ok" twice covers it; which monitor gets the input is WAPI-07 (partial until Round E) and not checked here.
- **TYPE-04 empty:** the url-input site answered 200 in Task 1, so B3 was a real check on both engines.
- **TYPE-04 ordering (changing Url Engine while url-input plays):** **not checked in this run** (the engines were switched before the steps); the verifier abstains to human_needed.
- **TYPE-05 adjacency (list loop, single-video loop):** P1a and P1b are quoted above; CEF indices 0, 1, 28, 0, WebView2 run 1 17, 18, 28, 0, reruns 0, 1, 28, 0; the single video restarts from its start on both engines.
- **TYPE-05 empty (a list that does not play embedded is never used):** the first list PLATNX_t55hdpWZS7sShzhjQIw5q6WEG5w plays embedded (n=29), so the second list was not needed.
- **TYPE-05 ordering:** Shuffle Mode off starts at index 0 and advances to 1 on CEF (P1a); on WebView2 only the reruns did (run 1 started at 17, the FAIL); Shuffle Mode on starts at another index (P1c-WV2: 26; B4-CEF "b4 ok").
- **TYPE-05 concurrency (DISPLAY9 plays the playlist too, no player UI):** the optional glance in B4-CEF was **not separately confirmed**; the backstop stays human_needed.

## Outcome by the Complete rule

The shared Round B rule, verbatim: (1) the run finished and every proof of the requirement's last part passed → the SUMMARY lists it in `requirements-completed`; (2) the run finished but a proof of the requirement measured a FAIL or a step of it was not checked (for example no playlist link, an unreachable site, a debug fix that has not landed) → the SUMMARY's front matter says `status: complete` (the run finished), the requirement is left out of `requirements-completed`, and the failure or the unchecked step is recorded verbatim and flagged for /gsd-verify-work; (3) the run was skipped or did not finish → a halted SUMMARY (`status: halted`) with `requirements-completed: []`. The run finished, so `status: complete`. The proofs of each requirement are the cells of its rows in the table above.

**Case 1 - listed in requirements-completed (TYPE-03 and ENG-03 added by the addendum, last two rows):**

| Requirement | Every cell passed | Debug sessions whose fix the passing steps confirm |
|---|---|---|
| TYPE-04 | B3-CEF and B3-WV2 (opens, forwarded drag, nothing with a window in front); site 200 | cef-web-themes-stuck-loading (a0e22d9a), webview2-certs-video-input (747bbf92) |
| WAPI-05 | S5-CEF, S5, B5-CEF, B5-WV2 and the sysinfo tests | audio-bars-silent (the corrected themes, no code fix) |
| WAPI-06 | B5-CEF, B5-WV2, S5-CEF, S5 and the media test | audio-bars-silent (the corrected themes); 5de6cf69 (cover re-encode) |
| WAPI-08 | S6-CEF, S6, S7, S4, B2-WV2, B4-CEF, B4-WV2, the pages.rs tests and the scale-key reads | webview2-certs-video-input (44758204, the video page), youtube-player-ui-visible (adfb0922) |
| TYPE-03 (addendum) | A1 on CEF (7 themes in the sitting, Farm Weather Cycles in the addendum) and A3 on WebView2 (8 of 8); the edge truths (adjacency, empty, ordering) as above | cef-web-themes-stuck-loading (a0e22d9a), the white-border fix (6c6d9a3c) |
| ENG-03 (addendum) | A1 (8 of 8 with the addendum), B3-CEF, B4-CEF, P1a, A5-CEF, S5-CEF; concurrency (L2, L3) met | cef-web-themes-stuck-loading (a0e22d9a) |

WAPI-08 note: P1a-WV2 appears in WAPI-08's property-keys cell for the YouTube page on WebView2. That cell measures the `scale` key reaching the page, and `scaleKey: scale(1)` was read in all three runs including the failed run 1; the FAIL of run 1 measured the playlist start index, which is TYPE-05's and ENG-02's cell. If the verifier reads the probe step as a whole, WAPI-08 is the one mark to reconsider.

**Case 2 - left out of requirements-completed, flagged for /gsd-verify-work (and /gsd-debug where noted); TYPE-03 and ENG-03 were in this table until the addendum closed their one open cell:**

| Requirement | Why it stays open (verbatim) |
|---|---|
| TYPE-05 | **P1a-WV2 run 1 measured a FAIL**: `startIndex: 17` with `shuffleMode: false` (expected 0), `indicesSeen: [17, 18, 28, 0]`; reruns 1 and 2 passed. Flagged for /gsd-debug (F1). The reruns do not undo the measured FAIL. |
| ENG-02 | The same **P1a-WV2 FAIL** is a cell of "ENG-02: hosts YouTube wallpapers"; all other ENG-02 cells passed (A3, B3-WV2, B4-WV2, C1, S5). The one 1052 ms browser exit (L5, F4) is a failed Edge truth of ENG-02 concurrency. |

Never listed, as the plan says: any requirement outside these eight; C2 marks nothing. The case 2 table above keeps TYPE-05 and ENG-02 only.

## Findings for /gsd-debug and /gsd-verify-work

- **F1 (P1a-WV2, TYPE-05 and ENG-02):** the playlist started at index 17 with Shuffle Mode off in 1 of 3 WebView2 runs. Analysed cause (not proven): the YouTube page defaults `shuffleMode` to true and the first state change can pick a random video before `setShuffle(false)` arrives. Shuffle Mode was still on during his B4-WV2 playlist Use, so that step does not exercise the Shuffle-off start. Needs a /gsd-debug session and a rerun of P1a-WV2.
- **F2 (A1 gap, TYPE-03 and ENG-03): closed by the addendum.** Farm Weather Cycles was not Used on CEF in the sitting; the addendum Used it on CEF the same night and the maintainer said ok.
- **F3 (C2, no requirement):** he saw the windows on both monitors after a few seconds, not about 1 s. The log corroborates a part of it (see C2 timing: median 1.09 s, longest 2.36 s, the two monitors within 0.25 s of each other); the long part is the new WebView2 and the page load, not the browser exit. For the pending debug session webview2-browser-exit-slow.
- **F4 (L5, ENG-02):** one `did not exit in time` with `elapsed_ms=1052` on DISPLAY1 at 04:12:22 while closing a YouTube page; 83 other exits took 174 to 461 ms. STATE.md already records BROWSER_EXIT_WAIT 1 s as tight under load (one 1034 ms exit seen earlier); the maintainer's PC was not under a build here.
- **F5 (D3, no requirement):** S3 run 1 did not load expired.badssl.com within about 13 s; the other four runs loaded in 0.55 to 3.6 s and the setting-off run was refused. Flagged for /gsd-verify-work; it blocks no mark.
- **F6 (TYPE-04 ordering):** changing Url Engine while url-input plays was not checked.
- **F7 (TYPE-05 concurrency glance):** the optional DISPLAY9 glance of B4-CEF was not separately confirmed.

## Task Commits

1. **Task 1: Prepare the re-check and prove it unattended** - no commit (no tracked file changed; scratch under `target/` only)
2. **Task 2: Ready gate** - no commit (the maintainer replied "start")
3. **Task 3: The maintainer's re-check** - no commit (verdicts and findings are in this SUMMARY)

**Plan metadata:** the docs commit of this SUMMARY with STATE.md, ROADMAP.md and REQUIREMENTS.md. The front matter `commits: 0` is measured from the ledger base `b5eec90f` (no code commit; docs-only plan).

## Files Created/Modified

- None tracked except the planning files above. Scratch, not committed: `target/round-b-recheck`, `target/round-b-recheck-logs` (six logs), `target/round-b-recheck-probe` (the probe results with their `.engine.txt` files), `target/round-b-lib`.

## Decisions Made

- Complete rule case 1 for TYPE-04, WAPI-05, WAPI-06, WAPI-08 and, after the addendum, TYPE-03 and ENG-03 (their one unchecked theme passed; every other cell of their rows had passed); case 2 for TYPE-05 and ENG-02 (a measured FAIL), applied per cell as the plan defines it. The P1a-WV2 FAIL is not softened because the reruns passed.
- WAPI-08 is listed on the reading that its P1a-WV2 cell measures the scale key (see the note above); flagged so the verifier can reverse it.

## Deviations from Plan

None - the plan was executed as written, except that the Task 3 run was driven by the orchestrator (it started the Portal and relayed the steps and replies) and this executor closed it. The maintainer replied to Part 1 with the step list instead of the words "Part 1 done", and his Part 2 steps in the Service log are not in the plan's order (C2 and B3-WV2 had extra Uses between them); neither changes a verdict.

## Issues Encountered

- The P1a-WV2 FAIL and the missing Farm Weather Cycles Use above (the latter closed by the addendum); nothing was fixed in this plan.
- The 36 ERROR lines come from two Uses of the synthetic placeholder Video themes (L1), not from any step.

## Known Stubs

None (this plan changed no source file).

## Threat Flags

None. The DevTools port used by Task 1's probes is restored (`DeveloperPort` 0 in the final scratch settings, as migrated), and only the two Rust image names were passed to taskkill.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- /gsd-verify-work: TYPE-03 and ENG-03 are closed by the addendum (Farm Weather Cycles on CEF); TYPE-05 and ENG-02 need the P1a-WV2 FAIL fixed (/gsd-debug, F1) and P1a-WV2 rerun, and ENG-02 also F4; the DISPLAY9 glance (F7) and TYPE-04 ordering (F6) stay human_needed.
- Round C (display modes, properties editor) can start; the open Round B requirements above do not block it, but phase verification waits until Round E.

## Addendum (2026-11-28 04:37, A1 Farm Weather Cycles on CEF)

After the sitting above, this SUMMARY recorded TYPE-03 and ENG-03 as case 2 because the Service log had no Use event of Farm Weather Cycles (folder `ynGju7Rw1cDhNQdT2Y6lfcE5B`) on CEF in A1; it was Used only on WebView2 (04:12:28). The orchestrator told the maintainer, he chose an addendum check ("ek kontrol"), and it ran the same night on the same release build (HEAD `b5eec90f` build; no build input changed since, only docs commits).

**Room:** the C# app was closed; DISPLAY9 attached (\\.\DISPLAY1 and \\.\DISPLAY9, 1920x1080 each); no Rust Sucrose process before the start. The scratch settings `target/round-b-recheck/Setting/Sucrose.json` had `Engine.Web` set to CefSharpLive and `Library.Selected` to Critter Crossing-1 for the check (backup `target/round-b-recheck-Sucrose.json.pre-addendum`; restored afterwards, `cmp` same).

**T0:** 04:37:11 local (+03:00): the Portal started with focus, same command and `SUCROSE_LOG` as Task 3. Service log: `engine chosen kind=Web configured=CefSharpLive engine=Cef fell_back=false` and `renderers started count=2 monitors=[DISPLAY1, DISPLAY9] primary=DISPLAY1 web_profile_owner=DISPLAY1` (01:37:08Z), `wallpaper status status=Sucrose — playing Critter Crossing` (01:37:10Z).

**What the maintainer was asked (in Turkish), exactly A1's checks for this one theme:** Use Farm Weather Cycles in the Portal and look for about 10 s: it leaves its loading screen; no white frame at the screen edges while loading or after; at the DISPLAY1/DISPLAY9 edge no white frame and neither monitor shows part of the other's page.

**Reply, verbatim:** `ok` (translation: ok).

**Log lines (copied to `target/round-b-recheck-logs/addendum/`):**

- Service 01:38:35.001581Z: `engine chosen kind=Web configured=CefSharpLive engine=Cef fell_back=false`
- Service 01:38:35.017684Z: `theme used kind=Some(Web) transition=Restart renderers=2`
- Service 01:38:37.106294Z: `wallpaper status status=Sucrose — playing Farm Weather Cycles`
- Both Renderer logs 01:38:37Z: `sucrose_engine_cef::browser::imp: page loaded url="https://sucrose.wallpaper.engine/Farm.html" status=200`
- `GPU process exited`: 0 lines in `CefRenderer-0.log` and `CefRenderer-1.log` (L2 holds for this run too).

**Cleanup:** the apps were stopped at 04:39:13 with `taskkill.exe /IM sucrose-portal.exe /F` and `taskkill.exe /IM sucrose.exe /F`; `tasklist.exe` showed no Rust Sucrose process left.

**Cells changed by the addendum:**

| Cell | Before | After |
|---|---|---|
| TYPE-03: a Web theme plays, CEF, A1 (the 8 themes) | not checked for 1 of 8 | pass (8 of 8: 7 in the sitting, Farm Weather Cycles here) |
| ENG-03: hosts Web wallpapers, A1 | not checked for 1 of 8 | pass |
| A1 verdict | 7 of 8 themes, Farm not Used | 8 of 8 Used and ok on CEF |

**Complete rule re-applied (verbatim rule, case 1) with the plan's "Requirement parts per engine" table as the one authoritative list:** TYPE-03's rows are the one part "a Web theme (local HTML) plays": A1 on CEF now pass, A3 on WebView2 pass (8 of 8, "a3 ok"); no other TYPE-03 cell exists in the table. ENG-03's rows are Web (A1: pass), URL (B3-CEF: pass), YouTube (B4-CEF and P1a: pass) and the same JavaScript API (04-37 A5 stands, A5-CEF and S5-CEF: pass). P1a-WV2 appears in no ENG-03 cell (ENG-03 is CEF only, its YouTube cell names B4-CEF and the CEF P1a, which passed 4 of 4). Every cell of both requirements has passed, so **TYPE-03 and ENG-03 are listed in `requirements-completed`**. The edge truths of both (TYPE-03 adjacency, empty and ordering; ENG-03 concurrency) were met in the sitting and the addendum adds no contrary evidence. TYPE-05 and ENG-02 are untouched: they stay open for the P1a-WV2 FAIL (and ENG-02 for F4).

**Not covered by the addendum:** the DISPLAY1/DISPLAY9 edge look for the other seven CEF themes stays the sitting's "a1 ok" (not separately reported, as recorded above).

## Self-Check

**PASSED.** Checked after writing: the 14 fix commits are ancestors of HEAD (listed above); the six copied logs exist in `target/round-b-recheck-logs/`; the probe results `p1a-wv2.json`, `certs-off.json` and the others exist in `target/round-b-recheck-probe/`; `tasklist.exe` shows 0 Rust Sucrose processes; no code commit exists (`git rev-list --count b5eec90f..HEAD` = 0 before the docs commit), matching `commits: 0`. Addendum (2026-11-28 04:37) checked: the Farm Weather Cycles Service and Renderer log lines are in `target/round-b-recheck-logs/addendum/`; `tasklist.exe` showed no Rust Sucrose process after the stop at 04:39:13; the scratch settings were restored (`cmp` same); this docs-only addendum adds no code commit.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-28*
