---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 37
subsystem: engines
tags: [webview2, cef, builtin-gif, web-wallpapers, maintainer-checkpoint, round-b, defects]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "Plans 04-22..04-36: WebView2 engine, built-in GIF/APNG/WebP renderer, generated pages, URL and YouTube engines, the Wallpaper page engine rows"
provides:
  - "The url-input fixture Web theme (https Source) for the input check"
  - "The recorded Round B maintainer run: verdicts, T0, log findings and defects D1-D9"
  - "Release build and scratch run prepared and proven unattended (Task 1)"
affects: [04-38, gap-closure, verify-work, debug, round-c, round-d]
plan_head_before: 950867c0e11cd04eaaf66556909bee55815375dd
plan_head_after: 86507011d846a0450609f9cfc46ee31088f66783
actuals:
  tokens: 120
  tasks: 3
  commits: 4
tech-stack:
  added: []
  patterns:
    - "Checkpoint verdicts recorded verbatim; the Complete rule lists a requirement only when every mapped step passed, whatever failed elsewhere"
key-files:
  created:
    - apps/sucrose/tests/fixtures/web-themes/url-input/SucroseInfo.json
  modified: []
key-decisions:
  - "Complete rule case 2 applies per requirement: TYPE-02, ENG-07, WAPI-01, WAPI-02, WAPI-04, FMT-07, WAPI-09 and WAPI-10 passed every mapped step and are listed; TYPE-03, TYPE-04, TYPE-05, ENG-02, ENG-03, WAPI-05, WAPI-06 and WAPI-08 stay open and are flagged for /gsd-verify-work"
  - "The run went ahead on the maintainer's start with none of the three precondition defects fixed and with a shortened scope (about 25-30 min instead of 60), approved by him at the gate"
  - "D9 (D-33: the first renderer to start takes the persistent web profile, so the primary monitor can run ephemeral) is recorded as a design finding with a proposal, not decided here"
requirements-completed: [TYPE-02, ENG-07, WAPI-01, WAPI-02, WAPI-04, FMT-07, WAPI-09, WAPI-10]
duration: "about 35 min maintainer run (T0 to the Portal close)"
completed: 2026-11-27
status: complete
---

# Phase 4 Plan 37: Round B maintainer check Summary

**The Round B run on the maintainer's machine passed the API, engine-row, DevTools, arguments and built-in GIF steps (8 requirements marked) but found nine defects: CEF Web themes stuck on loading, white borders on both engines, a WebView2 certificate frame that still fails, black or non-reacting video-clip and url-input pages, visible YouTube player UI on both engines, silent audio bars, and the D-33 persistent-profile placement.**

## Performance

- **Duration:** about 35 min of maintainer run (Task 3), plus Task 1 and the ready gate in the earlier session
- **Run start (T0):** 2026-11-27 14:52:53 +03 (2026-11-27T11:52:53Z); the maintainer closed the Portal at about 12:26:21Z
- **Completed:** 2026-11-27
- **Tasks:** 3 (Task 1 committed, Task 2 the ready gate, Task 3 the maintainer run)
- **Files modified:** 1 created (the url-input fixture); no source code changed

## Accomplishments

- Task 1 prepared and proved unattended the release build, the scratch data folder (settings migrated read-only), the scratch library (his 8 Web and 2 GIF themes copied, fixtures, Showcase YouTube and music themes) and the fallback capture `target/round-b-fallback/portal-fallback.png`; the url-input fixture was added in `0e08ccc1`.
- The maintainer ran Part A and Part B on CEF and WebView2 with two monitors attached (slot 0 = \\.\DISPLAY1 (0,0)-(1920,1080), slot 1 = \\.\DISPLAY9 (1920,0)-(3840,1080)); every verdict is below, verbatim.
- Eight requirements passed every one of their mapped steps (see the outcome section); the 04-20 slice-1 reruns of ENG-07, WAPI-01, WAPI-02, WAPI-04 and FMT-07 are now proven on both engines.

## Ready gate (Task 2)

Reply, verbatim (Turkish): "1. sucrose kapalı / 2. no playlist / 3. ikinci ekran açık / 4. 60dakikalık neyi test edicen? kısaltalım bu süreyi / start mı önerirsin yoksa gsd-debug mu?" and after the orchestrator proposed a shortened scope and recommended starting now: "start".

English: C# Sucrose closed; no playlist; second screen (DISPLAY9) attached; asked what the 60 minutes test and to shorten it; asked whether to start or debug first; then "start".

Debug status at the gate: none of the three precondition defects had a fix (the Test Sucrose API CORS-like error, "SucrosePropertyListener is not defined", the CEF GPU process crash); no /gsd-debug session ran. The run went ahead on "start", as the plan allows. The "no playlist" answer means TYPE-05's playlist part was not checked.

## The maintainer's verdicts (verbatim)

### Part A

| Step | Reply (Turkish, verbatim) | English translation |
|---|---|---|
| A1 (CEF, 8 Web themes) | "A1) bazı web türündeki duvar kağıtları loadingde kalıyor. ayrıca ekranın çerçevesinde beyaz kenarlıklar oluyor, niye bilmiyorum" | Some web-type wallpapers stay on loading. Also white borders appear around the edges of the screen, I don't know why. |
| A2 | "A2) done" | ok |
| A3 (WebView2, 8 Web themes) | "A3) webview web türündeki duvar kağıtlarını cef'e göre loadingde kalmayıp açıyor. yine bazı duvar kağıtları açılışında border beyaz kenarlık oluyor" | Unlike CEF, WebView opens the web-type wallpapers instead of staying on loading. Again some wallpapers show a white border when they open. |
| A4 | "A4) done" | ok |
| A5 (calls, traversal, storage, seek) | "A5) storage sürekli 1 olarak geliyor, artmıyor. cef'de artıyor. seek webview de atlıyor, cef'de video hiç görünmüyor" | Storage always shows 1, it does not go up [on WebView2]; on CEF it goes up. Seek jumps on WebView; on CEF the video is not visible at all. |
| A5 follow-up | "1 evet okdu / 2 evet açıldı" | 1: yes it was ok (the property calls, compatible keys, tables and traversal were ok on both engines). 2: yes it opened (Open DevTools opened DevTools). |
| A8 | "A8) ayarı açmam işe yaramadı yine yüklenmesi" | Turning the setting on did not help, it still did not load ("yüklenmesi" is read as a typo of "yüklenmedi"; the certs frame after Allow invalid certificates + Restart now on WebView2). |
| A9 | "A9) done" | ok |

A6 (input) and A7 (fullscreen pause) were dropped by agreement at the ready gate (see Deviations); no reply exists for them.

### Part B

| Step | Reply (Turkish, verbatim) | English translation |
|---|---|---|
| B1 | "B1 evet akıcı, tekrar etmeli" | Yes smooth, it repeats (read as a pass: it keeps looping). |
| B2 | "B2 cef gif oynatmada sorun yok. webview video-clip kısm sorunlu siyah ekran görünüyor." | No problem with GIF playback on CEF. The WebView video-clip part is faulty, a black screen shows. |
| B3 | "B3 cefde url-input açılmadı, siyah ekranda kaldı. webviewde açıldı ama fare hareketlerine tepki vermiyor" | On CEF url-input did not open, it stayed on a black screen. On WebView it opened but does not react to mouse movements. |
| B4 | "B4 youtube videoları açılıyor ama title ve oynatma durakaltma ikonları görünüyor. ekranın alt kısımlarında da bir şeyler görünüyor ama taskbardan ne olduğunu anlamadım. yine ui bileşenidir" | YouTube videos open but the title and the play/pause icons are visible. Something is also visible at the bottom of the screen, but because of the taskbar I could not tell what; probably a UI element again. (The engines are not distinguished: treated as both.) |
| B5 | "B5 webviewde sorunsuz oynuyor gibi. cef için music tunnelde düzgün çalışmayan bir şeyler var. ayrıca sese göre hareket eden çubuklar bu iki temada da yok. test edemedim yani" | On WebView it seems to play fine. On CEF something is not working right in Music Tunnel. Also the sound-reactive bars are missing in both themes. So I could not test it. (Now playing in Neomorphic Music Player was not confirmed.) |
| B6 | "B6 evet oynadı ikisinde de" | Yes, it played on both. |

## Outcome by the Complete rule

The shared Round B rule applies per requirement: (1) the run finished and every proof of the requirement's last part passed, so it is listed in `requirements-completed`; (2) the run finished but a proof measured a FAIL or a step was not checked, so the front matter says `status: complete`, the requirement is left out, and the failure is recorded verbatim and flagged for /gsd-verify-work; (3) the run was skipped or did not finish, a halted SUMMARY. The run finished, so `status: complete`. By the plan's step mapping:

**Listed in requirements-completed (case 1):**

| Requirement | Mapped steps | Evidence |
|---|---|---|
| ENG-07 | A2, A4 | "A2) done", "A4) done" |
| WAPI-01, WAPI-02, FMT-07 | A5 calls on both engines | follow-up "1 evet okdu" (calls and compatible keys ok on both engines) |
| WAPI-04 | A5 tables on both engines | the same follow-up (tables and traversal ok on both engines) |
| WAPI-09 | A8 DevTools | follow-up "2 evet açıldı" (Open DevTools opened DevTools) |
| WAPI-10 | A9 | "A9) done" |
| TYPE-02 | B1 | "B1 evet akıcı, tekrar etmeli" |

A8's certificate failure is D-12 on WebView2 and is mapped to no requirement (WAPI-09's proof is the DevTools part); it is defect D3 below.

**Left out (case 2), flagged for /gsd-verify-work:**

| Requirement | Why it stays open |
|---|---|
| TYPE-03 (A1, A3) | A1 failed on CEF ("bazı web türündeki duvar kağıtları loadingde kalıyor"); A3 white border on WebView2 |
| ENG-02 (A3, A5 WebView2, B3 WebView2, B4 WebView2, B6) | A3 white border; B3 WebView2 does not react to the mouse; B4 WebView2 shows the player UI. A5 on WebView2 and B6 (D-33 proof, both profile kinds in the logs) passed |
| ENG-03 (A1, A5 CEF, B3 CEF, B4 CEF) | A1 failed; A5 CEF "video hiç görünmüyor"; B3 CEF stayed black; B4 CEF shows the player UI |
| WAPI-08 (B2, B4) | B2 video-clip as a generated page on WebView2 is black; B4 player UI visible |
| TYPE-04 (B3) | B3 failed on both engines |
| TYPE-05 (B4) | no playlist link was sent, so the playlist was not checked; the YouTube player UI is visible |
| WAPI-05 (B5 bars) | the bars are missing in both themes; the maintainer could not test |
| WAPI-06 (B5 now playing) | not confirmed |

Never listed, as the plan says: WAPI-03, WAPI-07, PROP-05, DESK-05, PLAY-01, PLAY-05, and ENG-06 (marked by 04-27).

## Defects found (for /gsd-debug and /gsd-verify-work)

1. **D1, CEF: some Web themes stay on loading** (WebView2 opens them). Continues the 04-20 CORS-like in-page failure; the pages load with status 200 but the page content does not finish.
2. **D2, both engines: a white border around the screen edges** on some Web themes when they open (A1, A3).
3. **D3, WebView2: the certs frame still fails** after Allow invalid certificates + Restart now (D-12 on WebView2; CEF passed in 04-20).
4. **D4, CEF: the seek fixture video is never visible** (WebView2 plays and seeks). Not triaged; the codec of the clip in the CEF build is one hypothesis, unverified.
5. **D5, WebView2: video-clip as a generated page** (Video Player = WebView2) shows black.
6. **D6, url-input:** CEF stays black; WebView2 opens it but mouse input does not reach it.
7. **D7, YouTube on both engines:** the title, the play/pause icons and some bottom UI are visible (hiding the player UI fails; CEF was known from 04-35, now also seen on WebView2).
8. **D8, audio:** the spectrum bars do not move in Music Tunnel and audio on both engines; Music Tunnel misbehaves on CEF; now playing was not confirmed. The engines logged `data wanted groups=[Audio]` for the music pages on both engines; nothing shows whether spectrum data reached them.
9. **D9, design (D-33), not a bug:** with two monitors the first renderer to start takes the persistent web profile, so the primary monitor can run an ephemeral one. This explains A5's "storage sürekli 1": from 12:03:59Z to 12:04:53Z Renderer-1 (DISPLAY9) held `kind=Persistent engine="WebView2"`, so DISPLAY1 (slot 0, the monitor he watched) ran an ephemeral profile and storage stayed 1; on CEF the persistent profile moved between slots at each switch, so DISPLAY1 sometimes had it and the counter rose. Proposal for the maintainer (a decision, not made here): prefer the primary monitor for the persistent profile.

## Log findings (logs copied to `target/round-b-logs/`)

Checked by grep on the six copied logs (CefRenderer-0.log, CefRenderer-1.log, Portal.2026-11-27.log, Renderer-0.2026-11-27.log, Renderer-1.2026-11-27.log, Service.2026-11-27.log):

- `panicked`: 0 in all six.
- ERROR lines: only six, all in Portal.2026-11-27.log at 12:26:21Z ("Geçersiz pencere işleci. (0x80070578)" five times and "(0x80040102)" once), the known Phase 3 Portal-close lines (STATE blocker), at the moment the maintainer closed the Portal.
- CEF close-timeout WARN "did not close in time": 0 in every log; the 2026-11-26 cef-close-shutdown-dialog fix holds and no Shut Down Windows dialog was reported.
- CEF GPU process: the CefRenderer logs show "GPU process exited unexpectedly: exit_code=-2147483645" and "The GPU process has crashed 3 time(s)" at 15:24:42 local (during B5 on CEF); the known blocker.
- Service WARN "cannot send a message to a renderer, its writer stops error=the connection is closed": 56 lines, at renderer switches (as in 04-20), and 12 lines "cannot send a role change to a renderer".
- D-33 (the B6 proof): both profile kinds are present with two monitors, for example at 12:06:42Z Renderer-0 `web profile kind=Persistent engine="WebView2" ... slot=0` and Renderer-1 `web profile kind=Ephemeral engine="WebView2" ... slot=1`.
- Cleanup: the Portal had already exited ("not found" from `taskkill.exe /IM sucrose-portal.exe /F`); `taskkill.exe /IM sucrose.exe /F` terminated 3 processes; `tasklist.exe` afterwards showed no Rust Sucrose process.

## Task Commits

1. **Task 1: Prepare the Round B run and prove it unattended** - `0e08ccc1` (test)
2. **Task 2: Ready gate** - no commit (the maintainer replied "start")
3. **Task 3: The maintainer's check** - no commit (verdicts and findings are in this SUMMARY)

The `commits: 4` in the front matter is measured from the ledger base `950867c0`: the task commit `0e08ccc1` and three planning commits (`44780311` pause, `918b2908` and `86507011` resume).

**Plan metadata:** the docs commit of this SUMMARY with STATE.md, ROADMAP.md and REQUIREMENTS.md.

## Files Created/Modified

- `apps/sucrose/tests/fixtures/web-themes/url-input/SucroseInfo.json` - a URL test theme on an interactive page for the input check
- Scratch, not committed: `target/round-b`, `target/round-b-lib`, `target/round-b-logs/` (six logs), `target/round-b-fallback/portal-fallback.png`

## Decisions Made

- Complete rule case 2 applied per requirement (listing above); the eight listed requirements are marked in REQUIREMENTS.md.
- No source was changed in this plan; every defect goes to /gsd-debug and gap closure.

## Deviations from Plan

**Shortened scope, approved by the maintainer at the ready gate (about 25-30 min instead of 60):**

- A6 (input) and A7 (fullscreen pause) were dropped: they map to no requirement marked in Round B (WAPI-07 and PLAY-05 stay partial).
- A1 and A3: a quick glance per theme (about 10 s); all 8 themes were still used.
- B4: one Showcase YouTube video per engine instead of three, no wait for the end; no playlist, so TYPE-05 was not checked.
- B5: one engine was enough.

The run also started with none of the three precondition defects fixed (the plan allows it; the Complete rule applied as written). The maintainer closed the Portal himself at the end of the run, so the Portal-close ERROR lines are the known Phase 3 behaviour.

## Issues Encountered

- The run found defects D1-D9 above; nothing was fixed in this plan.
- The final `cargo xtask ci` check fails in one test, `apps/sucrose/tests/portal.rs` `an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen` (portal.rs:745), twice in a row, with DISPLAY9 attached: the test counts "engine started" lines over all renderers and assumes one monitor. No source was changed in this plan, so it is logged in `deferred-items.md`, not fixed. `cargo xtask ci` stops at the first failing test binary; a follow-up `cargo test --workspace --no-fail-fast` showed every other binary green (sucrose lib 395 passed) and a second real-desktop `portal` test, `switching_web_themes_on_webview2_closes_each_renderer_cleanly` (portal.rs:878), failing in that run, also with two renderers (one per monitor) in its log. Both are logged in `deferred-items.md`.
- The maintainer's B4 reply does not say which engine showed the player UI; both are treated as affected.

## Known Stubs

None (the only file this plan changed is a fixture).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- 04-38 (the measurement checkpoint, ready gate) can run next, or /gsd-debug for D1-D8 first: the maintainer's choice.
- /gsd-verify-work (after the debug sessions): TYPE-03, TYPE-04, TYPE-05, ENG-02, ENG-03, WAPI-05, WAPI-06 and WAPI-08 need a rerun of their steps; TYPE-05 also needs a playlist link.
- Phase verification still waits until Round E.

## Self-Check

**PASSED.** Checked after writing: the url-input fixture exists, commit `0e08ccc1` is an ancestor of HEAD, `target/round-b-logs/` holds the six logs, `target/round-b-fallback/portal-fallback.png` exists, and `tasklist.exe` shows no Rust Sucrose process.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-27*
