---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 20
subsystem: engines
tags: [cef, web-wallpapers, maintainer-checkpoint, slice-1, defects]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "Plans 04-01..04-19: engine trait and detection, CEF bundled, virtual origin, JavaScript API, system data, DevTools, certificates, input forwarding"
provides:
  - "The certs fixture Web theme (badssl expired-certificate iframe)"
  - "The recorded slice-1 maintainer run: verdicts, T0, log findings and the defects it found"
  - "Release build and scratch run prepared and proven unattended (smoke run, fallback capture)"
affects: [04-21, gap-closure, verify-work, debug, round-b]
plan_head_before: 5a17b108111240167ffd5518abc6862c11b08027
plan_head_after: 59dca42a6bc58c500476cb5459c5332ce68539a3
actuals:
  tokens: 380
  tasks: 3
  commits: 1
tech-stack:
  added: []
  patterns:
    - "Checkpoint verdicts recorded verbatim; the Complete rule leaves a requirement open when any of its steps failed or was skipped"
key-files:
  created:
    - apps/sucrose/tests/fixtures/web-themes/certs/SucroseInfo.json
    - apps/sucrose/tests/fixtures/web-themes/certs/index.html
  modified: []
key-decisions:
  - "Complete rule case 2 applies: step 1 failed and steps 5, 6, 8 were skipped because of it, so no completing requirement is listed and none is marked Complete"
  - "The Shut Down Windows dialog after every theme switch is recorded as a high-severity blocker for /gsd-debug (cause hypothesis only, not verified, no source changed in this plan)"
requirements-completed: []
duration: "about 30 min of agent time plus the maintainer's wait (checkpoint)"
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 20: Slice-1 maintainer check Summary

**The slice-1 run on the maintainer's machine found a high-severity defect (the Windows "Shut Down Windows" dialog opens after every theme switch because every CEF browser close times out) and web themes that fail inside the page, so no completing requirement is marked Complete.**

## Performance

- **Duration:** about 30 min of agent time plus the maintainer's wait
- **Started:** 2026-11-26T17:38Z (after 04-19)
- **Completed:** 2026-11-26T18:08Z
- **Tasks:** 3 (Task 1 committed, Task 2 the ready gate, Task 3 the maintainer run)
- **Files modified:** 2 created (the certs fixture); no source code changed

## Accomplishments

- Prepared and proved unattended a release build at `5a17b108` (4 m 14 s, CEF 156.0.31 / Chromium 156.0.8078.100), a scratch data folder migrated read-only from the maintainer's settings, and a scratch library with his 8 Web themes plus the fixtures.
- Smoke run logged `2026-11-26T17:49:05.429601Z INFO sucrose_engine_cef::browser::imp: page loaded url="https://sucrose.wallpaper.engine/CritterCrossing.html" status=200` and `engine chosen kind=Web configured=CefSharpLive engine=Cef fell_back=false`; `tasklist.exe /m libcef.dll /fi "PID eq 32884"` for the service pid answered `INFO: No tasks are running which match the specified criteria.` (D-32 holds: the service has no libcef.dll).
- Ran the maintainer's check (T0 = 2026-11-26 20:57:09 +03, 17:57:09Z) and recorded his verdicts, the log findings and the defects below.

## Task 1 facts

- The 8 Web theme folders copied from `H:\Sucrose` (read-only source): Critter Crossing-1, Frog Rush-1, Glitch Maze-1, Interactive Rocket-1, Sky Blitz-1, Voxel Heat-1, `_` ("Test Sucrose API", the only one with SucroseCompatible.json), ynGju7Rw1cDhNQdT2Y6lfcE5B ("Farm Weather Cycles").
- The scratch library holds 21 theme folders; the seek clip was copied from theme m0vjKmcc19CTTixeQvkYqHU3t.
- Fallback capture: `target/round-a-fallback/portal-fallback.png` (1000x1040, row alert and toast; the window was enlarged with an inline SetWindowPos outside the repository).
- The migration start briefly played the maintainer's real selected Video theme on DISPLAY1 for about 12 s (nothing written to real data). Only DISPLAY1 was attached (DISPLAY9 off), so the run was single-monitor.

## The maintainer's verdicts (verbatim)

Reply (Turkish, verbatim):

> her temayı seçtikten snra windowsu kapat penceresi çıkıyor [Image #2]
> 1. bazı web temaları açılmıyor hata veriyor. cors hatası falan veren var testsucrose api gibi
> 2. ok
> 3. sadece cef var, webview yok listede
> 4. ok
> 5. skip çünkü web temaları düzgün çalışmıyor, açılmıyor
> 6. skip çünkü web temaları düzgün çalışmıyor, açılmıyor
> 7. ok
> 8. skip çünkü web temaları düzgün çalışmıyor, açılmıyor

English translation: "After selecting each theme, the Shut Down Windows dialog appears [screenshot]. 1. Some web themes do not open and give errors; some give something like a CORS error, e.g. Test Sucrose API. 2. ok. 3. Only CEF is there, WebView is not in the list. 4. ok. 5. skip because the web themes do not work properly / do not open. 6. skip (same reason). 7. ok. 8. skip (same reason)."

The screenshot is `target/round-a-logs/maintainer-shutdown-dialog.png`: the Windows "Windows'u Kapat" (Shut Down Windows) dialog, "Bilgisayarın ne yapmasını istiyorsunuz? - Bilgisayarı kapat", in front of the Rust Portal library.

| Step | Subject | Verdict | Counts toward |
|------|---------|---------|---------------|
| 1 | The 8 Web themes under CEF | Failed (some do not open, errors, CORS-like, e.g. Test Sucrose API) | TYPE-03 |
| 2 | Wallpaper page: CEF ready, switch off, arguments editor | ok | TYPE-03 |
| 3 | Fallback capture | Not confirmed: the reply says only CEF is in the Web engine list, WebView2 is not listed | ENG-07 |
| 4 | Open DevTools | ok | (partial requirements only) |
| 5 | calls, traversal, storage, seek | Skipped (web themes do not work) | WAPI-01, WAPI-02, WAPI-04, FMT-07 |
| 6 | input | Skipped (same reason) | partial only |
| 7 | certs | ok | (D-12, partial requirements only) |
| 8 | fullscreen pause | Skipped (same reason) | partial only |

## Outcome by the Complete rule

Case 2 applies (the run finished with a failed step): `status: complete` with `requirements-completed: []`. Step 1 failed, steps 5, 6 and 8 were skipped because of it, step 3 was not confirmed (the reply is not "ok" and does not confirm the capture), and steps 2, 4 and 7 were ok. By the mapping (TYPE-03: steps 1-2; ENG-07: step 3 and the fallback capture; WAPI-01, WAPI-02, FMT-07: step 5 calls; WAPI-04: step 5 tables) no completing requirement passed all its checks, so TYPE-03, ENG-07, WAPI-01, WAPI-02, WAPI-04 and FMT-07 stay Pending in REQUIREMENTS.md. The partial requirements (ENG-03, ENG-06, WAPI-03, WAPI-07, WAPI-09, WAPI-10, DESK-05, PLAY-01, PLAY-05) are not marked, as the plan says.

## Log findings (logs copied to `target/round-a-logs/`)

Checked by hand: `panicked` count is 0 in all four logs (CefRenderer-0.log, Portal.2026-11-26.log, Renderer-0.2026-11-26.log, Service.2026-11-26.log); there is no ERROR line in the Rust logs (the CEF log has Chromium ERROR lines, below).

- **Every CEF browser close times out (12 of 12).** Each theme switch logs `INFO sucrose::render: the service asked this renderer to close` and 5 s later `WARN sucrose_engine_cef::browser::imp: the CEF browser did not close in time, CEF shuts down anyway`. First pair: Renderer-0.2026-11-26.log line 14 (`2026-11-26T17:57:33.044786Z`) and line 15 (`2026-11-26T17:57:38.047458Z`); last pair: lines 188-189 (`18:04:38.117712Z` / `18:04:43.125261Z`). The twelve pairs are at lines 14, 29, 45, 60, 75, 90, 105, 125, 140, 155, 173, 188.
- **Every page reached load.** The Renderer log has 15 `page loaded url="https://sucrose.wallpaper.engine/..." status=200` lines (CritterCrossing.html, Farm.html, Bing.html, VoxelHeat.html, SkyBlitz.html, index.html, FrogRush.html, ...), all status=200. So the "does not open / CORS" failures happen inside the pages (subresources or fetch/XHR to the virtual origin); page console messages are not in the Rust logs. The cause is not yet known.
- **Known GPU crash (STATE blocker since 04-02).** CefRenderer-0.log: `GPU process exited unexpectedly: exit_code=-2147483645` three times (`The GPU process has crashed 3 time(s)`, 21:04:45-46 local), then `ContextResult::kFatalFailure: Failed to create shared context for virtualization.` Software rendering carries on.
- **Service log:** two WARN lines `cannot send a message to a renderer, its writer stops error=the connection is closed slot=0` (Service.2026-11-26.log lines 59 and 192, at 17:58:12Z and 18:04:00Z), next to renderer switches.
- No Rust Sucrose process remained after the run (`tasklist.exe` showed none; the apps were stopped with the two taskkill commands, 7 processes).

## Defects found in the maintainer run

1. **Shut Down Windows dialog after every theme selection (high severity).** The maintainer sees the Windows "Shut Down Windows" dialog (Bilgisayari kapat) after selecting each theme (screenshot above). It matches in time the 12 close timeouts in the log. Hypothesis, recorded and NOT verified: CEF's CloseBrowser path sends WM_CLOSE to the browser's root ancestor window; with the browser hosted under the desktop's Progman/WorkerW the root is Progman, and WM_CLOSE to Progman opens the Shut Down Windows dialog. One click on "Tamam" would shut the PC down. This is a blocker added to STATE.md.
2. **Web themes fail inside the page.** Some of the 8 Web themes do not open and show errors; the maintainer names "Test Sucrose API" (the theme with SucroseCompatible.json) with a CORS-like error. Every page logged `status=200`, so the failure is in what the page then requests; cause unknown (candidates to check, none verified: the virtual origin's CORS and fetch/XHR handling, a request the scheme handler refuses, the iframe/cross-origin rules).
3. **WebView2 is absent from the Web engine list.** The reply says only CEF is listed. WebView2 is a Round B deliverable (04-CONTEXT D-31), so this is expected for Round A; it is recorded verbatim and step 3 is not counted as passed.

## Flagged for /gsd-verify-work and /gsd-debug

- /gsd-debug: the Shut Down Windows dialog on every theme switch (CEF browser close timeout, hypothesis above), and the web themes that fail inside the page including the CORS-like error of "Test Sucrose API". Both need gap closure before TYPE-03, WAPI-01, WAPI-02, WAPI-04 and FMT-07 can be checked again.
- /gsd-verify-work: TYPE-03 (step 1 failed), ENG-07 (step 3 not confirmed: only CEF listed; the fallback capture `target/round-a-fallback/portal-fallback.png` was not reviewed by the maintainer), WAPI-01, WAPI-02, WAPI-04, FMT-07 (step 5 skipped), and the skipped steps 6 (input) and 8 (fullscreen pause: WAPI-07, PLAY-05 Round A parts) all need a rerun after the fixes.
- Steps ok: 2 (Wallpaper page), 4 (Open DevTools), 7 (certificates reject by default and accept after Allow invalid certificates + Restart now).
- The CEF GPU process crash (STATE blocker) is still open.

## Task Commits

1. **Task 1: Prepare the slice-1 run and prove it unattended** - `59dca42a` (test)
2. **Task 2: Ready gate** - no commit (the maintainer replied "start")
3. **Task 3: The maintainer's check** - no commit (verdicts and findings are in this SUMMARY)

**Plan metadata:** the docs commit of this SUMMARY with STATE.md and ROADMAP.md.

## Files Created/Modified

- `apps/sucrose/tests/fixtures/web-themes/certs/SucroseInfo.json` - the certs fixture theme info
- `apps/sucrose/tests/fixtures/web-themes/certs/index.html` - an iframe to https://expired.badssl.com/ with the expected result with the switch off and on
- Scratch, not committed: `target/round-a`, `target/round-a-lib`, `target/round-a-logs/` (four logs and the maintainer's screenshot), `target/round-a-fallback/portal-fallback.png`

## Decisions Made

- Complete rule case 2 with an empty `requirements-completed`; no requirement is marked Complete in REQUIREMENTS.md.
- The shutdown-dialog defect is a STATE.md blocker; no source was changed in this plan (the plan's scope is the check, the fix belongs to /gsd-debug and gap closure).

## Deviations from Plan

None - plan executed exactly as written. Two things went beyond the plan text without changing it: the first executor used an inline SetWindowPos (outside the repository) to enlarge the Portal window for the fallback capture, and the migration start played the maintainer's real Video theme on DISPLAY1 for about 12 s (nothing was written to real data).

## Issues Encountered

- The maintainer's check failed at step 1 and three further steps were skipped for that reason; see "Defects found in the maintainer run".
- Only DISPLAY1 was attached (DISPLAY9 off), so the DISPLAY9 part of the run and the per-monitor storage step were not exercised.

## Known Stubs

None (the only files this plan changed are two fixture files).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 04-21 can proceed in the plan order, but the Round A completions are open: gap closure for the two defects and a rerun of steps 1, 3, 5, 6 and 8 are needed before TYPE-03, ENG-07, WAPI-01, WAPI-02, WAPI-04 and FMT-07 can be marked.
- Blocker: the Shut Down Windows dialog on every theme switch (one click can shut the PC down).

## Self-Check

**PASSED.** Checked after writing: the two certs fixture files exist, commit 59dca42a is an ancestor of HEAD, target/round-a-logs holds the four logs and the screenshot, target/round-a-fallback/portal-fallback.png exists, and `tasklist.exe` shows no Rust Sucrose process.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*
