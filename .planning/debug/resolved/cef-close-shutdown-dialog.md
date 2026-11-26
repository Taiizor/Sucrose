---
status: resolved
trigger: "CEF browser close opens the Windows Shut Down dialog on every theme switch (04-20 run, target/round-a-logs)"
created: 2026-11-26T18:23:01.279Z
updated: 2026-11-26T18:55:39.090Z
goal: find_and_fix
symptoms_prefilled: true
---

## Current Focus

bug_class: Bohrbug (deterministic: 12 of 12 closes, same timing, same log lines)
hypothesis: CONFIRMED (statically, then on the desktop) - our CEF life span handler did not override DoClose, so CEF's default (false) made CloseContents call CefBrowserPlatformDelegateNativeWin::CloseHostWindow, which does PostMessage(GetAncestor(browser_hwnd, GA_ROOT), WM_CLOSE). The browser window is a child of the desktop surface, and the surface is a WS_CHILD of Progman (raised layout, LayeredChild), so GA_ROOT is Progman and Progman answers WM_CLOSE with the Shut Down Windows dialog. Nothing destroyed the browser window during the 5 s close loop, so OnBeforeClose never came, close() timed out, and cef::shutdown() ran with a live browser and the renderer crashed (0xFFFF7003 = Crashpad kTerminationCodeNotConnectedToHandler; 2 runs 0xC0000005).
test: DONE - unit tests and mutants (static), then the maintainer's desktop run of 2026-11-26 18:46:56Z (target/round-a-logs-fix): no dialog, 14 of 14 CEF closes clean, every CEF renderer exit code 0
expecting: met - no Shut Down dialog, "the CEF browser closed" and "the renderer stopped" after every CEF close, exit code 0, no close-timeout WARN
next_action: none - session resolved; fix committed (5758edd0), session archived to .planning/debug/resolved/cef-close-shutdown-dialog.md, STATE.md blocker marked resolved; the 04-20 slice-1 maintainer check still needs its gap-closure re-run (steps 1, 5, 6, 8) through the phase workflow
reasoning_checkpoint:
  hypothesis: "Each theme switch opens Shut Down Windows because CEF answers our CloseBrowser(true) with its default windowed close (DoClose not overridden -> CloseHostWindow -> PostMessage(GetAncestor(browser, GA_ROOT), WM_CLOSE)), and GA_ROOT of a browser hosted in the desktop surface is the shell's Progman, whose WM_CLOSE handler is the Shut Down dialog; the browser then never closes because only a WM_CLOSE handler of our own top-level window would have destroyed it."
  confirming_evidence:
    - "CEF 156 (branch 8078) source: CloseHostWindow() is exactly 'HWND frameWnd = GetAncestor(window_info_.window, GA_ROOT); PostMessage(frameWnd, WM_CLOSE, 0, 0);'; CloseContents() calls it when DoClose() returns false and the browser is windowed."
    - "cef-rs 156.7.0 default ImplLifeSpanHandler::do_close returns Default::default() (0); browser.rs SpanHandler implements only on_before_close."
    - "Renderer log: 'surface attached layout=Raised strategy=LayeredChild ... order=SHELLDLL_DefView > SucroseDesktopSurface > WorkerW style=0x56000000' -> the surface is a WS_CHILD of Progman, the browser a WS_CHILD of the surface (set_as_child), so GA_ROOT(browser) = Progman."
    - "12 of 12 close requests are followed 5.0 s later by the close-timeout WARN; 'the renderer stopped' is logged 0 times; the dialog appears at each close."
    - "No code of ours sends WM_CLOSE anywhere (git grep WM_CLOSE in crates/ apps/: none); the only Progman message (0x052C) is sent at attach, also by mpv renderers that were switched many times in Phases 2-3 without a dialog."
  falsification_test: "If DoClose returned 1 and the renderer destroyed the browser window itself, and the dialog still opened on a switch, the hypothesis would be wrong (some other WM_CLOSE/close path reaches Progman). Also: if 'the renderer stopped' is still missing or the timeout WARN still appears after the fix."
  fix_rationale: "Returning 1 from DoClose for the wallpaper browser is CEF's documented way to keep it from sending the standard close notification to a top-level window that needs non-standard handling; completing the close by destroying the browser's own window (window/view hierarchy tear-down, also documented) makes CEF call WindowDestroyed -> DestroyBrowser -> OnBeforeClose, so close() ends with a closed browser before cef::shutdown(). This removes the WM_CLOSE to Progman at its source instead of hiding the dialog, and also fixes the shutdown crash that follows the timeout."
  blind_spots: "Not run on the desktop (static analysis by the maintainer's choice); the classic layout (Windows 10 to 11 23H2, GA_ROOT = top-level WorkerW) was not observed, its WM_CLOSE effect on WorkerW is inferred; whether the page's unload handlers delay DoClose was not measured (a forced tear-down after 3 s covers it); the exact crash site of 0xFFFF7003 inside cef::shutdown is not identified (no dump)."
  candidate_causes:
    - "code: CEF default DoClose (false) for a windowed browser posts WM_CLOSE to the GA_ROOT ancestor (CONFIRMED)"
    - "environment: shell layout - in the 24H2 raised layout the surface is a child of Progman, so that ancestor is Progman (CONFIRMED as the contributing condition)"
    - "config: Chromium switches / GPU crash slowing the close (ELIMINATED: the GPU crashes in CefRenderer-0.log are minutes apart from the closes, and a slow close would not open a shell dialog)"
    - "data: a theme page calling window.close() (ELIMINATED as the trigger: 12 different switches across several themes; but it is a second trigger of the same WM_CLOSE path, covered by the fix)"
  and_gate: "yes - the dialog needs both (1) CEF's default windowed close notification (DoClose false -> WM_CLOSE to GA_ROOT) and (2) a window tree whose root is a shell window (desktop surface under Progman). Either alone is harmless: an ordinary app's root is its own window; a browser whose DoClose returns 1 never posts it. The crash after the timeout needs (1)+(2) plus (3) nothing else destroying the browser window before cef::shutdown()."

## Symptoms

expected: When the maintainer selects another theme in the Portal while a Web (CEF) wallpaper is shown, the service closes the CEF renderer quietly, the CEF browser closes within its close timeout, and the new theme starts. No system dialog appears.
actual: After every theme selection the Windows "Shut Down Windows" dialog ("Windows'u Kapat" - "Bilgisayarin ne yapmasini istiyorsunuz? - Bilgisayari kapat") opens in front of the Portal (screenshot target/round-a-logs/maintainer-shutdown-dialog.png). Every CEF browser close times out: 12 of 12 switches log "INFO sucrose::render: the service asked this renderer to close" and exactly 5 s later "WARN sucrose_engine_cef::browser::imp: the CEF browser did not close in time, CEF shuts down anyway". The renderer then exits with status 0xffff7003. One click on OK (or Enter) would shut the PC down.
errors: No ERROR line and no panic in the Rust logs; the only related line is the WARN above. Service log after each switch: "a renderer disconnected", "a renderer exited pid=... slot=0 status=exit code: 0xffff7003", then "the renderer closed for the fullscreen app ended slot=0" and "the fullscreen app is gone, starting the renderer again" (side observation: the service files every switch under the fullscreen-ended path - may be a wrong label or a wrong code path, check while there); two "cannot send a message to a renderer, its writer stops error=the connection is closed slot=0" WARNs (Service log lines 59 and 192). CefRenderer-0.log has the known GPU process crash (STATE blocker since 04-02), unrelated in time.
timeline: First seen by the maintainer in the 04-20 slice-1 run (2026-11-26, release build at 5a17b108, CEF 156.0.31 / Chromium 156.0.8078.100). The maintainer does not know whether earlier CEF runs (04-02 onwards) showed it too.
reproduction: Release build, scratch SUCROSE_DATA_DIR, Web engine CEF, single monitor DISPLAY1. Select a Web theme in the Portal, then select another Web theme -> the dialog opens and the close timeout WARN is logged. All 12 logged switches were Web -> Web (service: "theme used kind=Some(Web) transition=Restart renderers=1"); Web -> non-Web and non-Web -> non-Web switches were not tried.

## Constraints (from the maintainer and the project)

- Static analysis first (maintainer's choice): find the root cause from our code (crates/sucrose-engine-cef, apps/sucrose render/service), the CEF headers/source behavior (cef_life_span_handler.h DoClose/OnBeforeClose, CloseBrowser/TryCloseBrowser, windowed vs windowless) and the legacy C# app (src/, read-only reference; how CefSharp was hosted and closed there). If a run on the maintainer's machine is needed to confirm or verify, STOP at a checkpoint and ask the maintainer first: such a run opens the Shut Down Windows dialog on their real desktop, and an accidental Enter shuts the PC down.
- Never synthesize mouse or keyboard input. Never send WM_CLOSE or any message to Progman/WorkerW/explorer windows in a probe.
- Runs use a scratch SUCROSE_DATA_DIR (from WSL also WSLENV=SUCROSE_DATA_DIR); the maintainer's real %APPDATA%\Sucrose, library and settings are read-only. Stop every process you start (taskkill) before rebuilding.
- unsafe only in sucrose-platform, sucrose-sysinfo and engine/FFI crates, each block with a // SAFETY: comment (D-011, D-039). Code must pass cargo fmt, clippy -D warnings and cargo test --workspace (cargo xtask ci).
- Behavior changes visible to users go into docs/FEATURES.md; GPUI/CEF lessons into the matching docs.

## Evidence

- timestamp: 2026-11-26T18:23:01.279Z
  checked: target/round-a-logs/Renderer-0.2026-11-26.log
  found: 12 close requests, each followed 5 s later by the close-timeout WARN (lines 14/15, 29/30, 45/46, 60/61, 75/76, 90/91, 105/106, 125/126, 140/141, 155/156, 173/174, 188/189); every page load before them is status=200.
  implication: the CEF browser never reaches its closed state on any switch; the timing of the dialog matches the close requests.
- timestamp: 2026-11-26T18:23:01.279Z
  checked: target/round-a-logs/Service.2026-11-26.log
  found: each switch is "theme used kind=Some(Web) transition=Restart renderers=1", then the renderer disconnects and exits with 0xffff7003 about 5.1 s later, and the service restarts it through the "fullscreen app ended" path.
  implication: the service-side restart works; the close itself is the failing part. The fullscreen-ended label on every switch is a separate oddity to check.

- timestamp: 2026-11-26T18:35:10.731Z
  checked: CEF 156 source, branch 8078 (libcef/browser/native/browser_platform_delegate_native_win.cc, libcef/browser/alloy/alloy_browser_host_impl.cc) and include/cef_life_span_handler.h of the pinned dist
  found: CloseBrowser(force) -> CloseContents(); CloseContents asks the life span handler DoClose(); false (the default) + windowed + window not destroyed -> platform_delegate_->CloseHostWindow() = PostMessage(GetAncestor(window_info_.window, GA_ROOT), WM_CLOSE, 0, 0). True -> nothing is sent and destruction_state_ goes back to NONE; the client must then complete the close (header: by window/view hierarchy tear-down). The CefBrowserWindow's WM_NCDESTROY calls WindowDestroyed() -> CloseBrowser(true) -> DestroyBrowser() -> OnBeforeClose() without asking DoClose again.
  implication: with a browser in the desktop window tree, CEF's default close posts WM_CLOSE to the shell's top-level window.
- timestamp: 2026-11-26T18:35:10.731Z
  checked: crates/sucrose-engine-cef/src/browser.rs (SpanHandler, CefHost::start/close) and cef-rs 156.7.0 bindings (ImplLifeSpanHandler::do_close default)
  found: SpanHandler overrides only on_before_close; cef-rs' default do_close returns 0. start() creates the browser with set_as_child(surface); close() calls close_browser(1) and pumps until OnBeforeClose or 5 s, then cef::shutdown().
  implication: every close goes through CEF's default windowed close (WM_CLOSE to GA_ROOT).
- timestamp: 2026-11-26T18:35:10.731Z
  checked: Renderer-0 log "surface attached" line and crates/sucrose-platform/src/os_windows/surface.rs
  found: layout=Raised strategy=LayeredChild, order "SHELLDLL_DefView > SucroseDesktopSurface > WorkerW", style 0x56000000 (WS_CHILD|WS_VISIBLE|WS_CLIPSIBLINGS|WS_CLIPCHILDREN): the surface is a direct child of Progman. In the classic layout it is a child of the top-level WorkerW.
  implication: GA_ROOT(browser window) = Progman (raised) or the shell's WorkerW (classic). WM_CLOSE to Progman is the Shut Down Windows dialog (the desktop's Alt+F4).
- timestamp: 2026-11-26T18:35:10.731Z
  checked: apps/sucrose/src/render.rs end of run()
  found: pump.close() (CefHost::close) runs before drop(surface); 'the renderer stopped' (the normal end) appears 0 times in the Renderer-0 log; the exit status is 0xFFFF7003 in 10 and 0xC0000005 in 2 runs. 0xFFFF7003 (-36861) is Crashpad's kTerminationCodeNotConnectedToHandler, the code a Chromium process without a crash handler exits with when it crashes.
  implication: nothing destroys the browser window during the close loop, the close never completes, and the renderer crashes in or right after cef::shutdown() with the browser still alive. The odd exit codes are part of this defect, not a separate one.
- timestamp: 2026-11-26T18:35:10.731Z
  checked: CEF popup and DevTools creation (browser_info_manager.cc CanCreateWindow, chrome_browser_delegate.cc CreateDevToolsBrowser)
  found: window.open() popups inherit the opener's client (so our SpanHandler sees them; their default window is a native top-level popup whose GA_ROOT is itself); DevTools opened with client None use CEF's default client, not ours.
  implication: the fix must answer DoClose with 1 only for the wallpaper browser (not popups), and OnBeforeClose of a popup must not mark the wallpaper browser closed.
- timestamp: 2026-11-26T18:35:10.731Z
  checked: side observations in the Service log (apps/sucrose/src/service.rs restart_with, RuleClosed, respawn_closed, Link::for_connection)
  found: (a) the Restart transition (D-26) deliberately reuses the fullscreen rule's Close/respawn bookkeeping (restart_with's doc: "the path of the fullscreen rule's Close"), so the logs "the renderer closed for the fullscreen app ended" and "the fullscreen app is gone, starting the renderer again" are printed for theme switches too; the behavior (close, wait for the exit, start again) is correct. (b) "cannot send a message to a renderer, its writer stops error=the connection is closed" is logged at the same instant as "a renderer disconnected" (2 of 12): the writer thread had a queued message when the dying renderer's pipe closed.
  implication: (a) is a mislabel, not a defect and not part of this one (follow-up: neutral wording or a close reason in RuleClosed). (b) is a benign exit race, made more likely by the 5 s hang; not a separate defect (a possible follow-up is to log it at debug once the reader has seen the disconnect).

- timestamp: 2026-11-26T18:42:10.282Z
  checked: the fix in crates/sucrose-engine-cef/src/browser.rs, unit tests, manual mutants, workspace gate
  found: cargo fmt --check 0, cargo clippy --workspace --all-targets -D warnings 0, cargo test --workspace 0 (all suites ok, sucrose-engine-cef 47 tests). Manual mutants at the fix site, each killed: M1 do_close_answer -> 0 (killed by the_wallpaper_browser_never_lets_cef_send_its_close_message), M2 close_step ignores DoClose readiness (killed by the_close_destroys_the_browser_window_once_cef_is_ready), M3 no forced tear-down (killed by the_close_destroys_the_window_anyway_when_cef_never_gets_ready_and_gives_up_at_the_timeout), M4 the DoClose override removed = the pre-fix handler (killed by imp::tests::the_life_span_handler_never_lets_cef_close_the_desktop_window, which calls the handler through its C function table as CEF does). Release build (cargo build --release -p sucrose -p sucrose-portal) done, target/release/sucrose.exe contains the new log strings.
  implication: statically the fix is complete and guarded; the desktop effect (no dialog, a clean renderer exit) still needs the maintainer's run.

- timestamp: 2026-11-26T18:54:29.911Z
  checked: maintainer desktop run with the fix (release target/release/sucrose-portal.exe + sucrose.exe built from 47c4d494 plus the uncommitted browser.rs fix, SUCROSE_DATA_DIR=target/round-a, T0 2026-11-26 18:46:56Z, single monitor DISPLAY1, raised layout / LayeredChild); logs copied to target/round-a-logs-fix/ (Renderer-0, Service, CefRenderer-0, Portal)
  found: The maintainer did the switches and answered "duzeldi" (fixed): the Shut Down Windows dialog did not appear. Service log: 17 "theme used" lines (15 transition=Restart, 2 Video re-picks with started=1). Renderer-0 log: 15 "the service asked this renderer to close"; 14 of them closed a CEF renderer and each is followed within 4-28 ms by "the CEF browser closed elapsed_ms=<4..28>" and then "the renderer stopped" (14/14); the 15th (18:48:18.574Z, Video -> Web) closed an mpv renderer that was already failing. Zero "did not close in time", zero "CEF was not ready to close the browser in time", zero "the page asked to close its window". Service log: all 14 CEF renderer exits are "status=exit code: 0" (before: 0xffff7003 / 0xc0000005 in 12/12). Switches covered: Web -> Web (13), Web -> Video (18:48:14.543Z, CEF close 7 ms, exit 0), Video -> Web, and one close with DevTools open (devtools opened 18:49:48.993Z, closed with the browser at 18:50:14.771Z in 27 ms, exit 0). Remaining WARNs: two "cannot send a message to a renderer, its writer stops" (Service log lines 59 and 219, same instant as "a renderer disconnected"), the known benign pipe-close race. CefRenderer-0.log still shows the known GPU process crash (3 times at the start of a Web wallpaper, exit_code=-2147483645; STATE blocker since 04-02), unrelated to the close.
  implication: the fix removes the WM_CLOSE to Progman at its source and the close now completes in milliseconds before cef::shutdown(), so the crash exit codes are gone too. revert_and_reconfirm at desktop level passes: before = 04-20 run at 5a17b108 (12/12 dialogs, 12/12 5 s timeouts, 0xffff7003), after = this run (0 dialogs, 14/14 clean closes, exit 0).
- timestamp: 2026-11-26T18:54:29.911Z
  checked: side observation in the same run, the Video theme the maintainer picked (Renderer-0 lines 119-189, Service lines 115-189)
  found: every mpv renderer for it ended with "playback ended with an error error=unrecognized file format" / "the video cannot be played" and exit code 4 (failed start), restarted under the StartFailed budget (6 exits with code 4); the theme comes from the synthesized scratch library (target/round-a), most likely a placeholder video file.
  implication: not part of this defect and not a CEF close problem (the CEF renderer before it closed cleanly with exit 0); the failed-start path behaved as documented in FEATURES.md (Video row). Recorded only; check the scratch library's video file if it matters for a later run.

## Eliminated

- hypothesis: our own code sends WM_CLOSE (or another close message) to Progman/WorkerW on a theme switch
  evidence: git grep WM_CLOSE over crates/ and apps/ finds nothing; the only Progman message is 0x052C (SPAWN_WORKERW) at surface attach, which mpv renderers have sent on every switch since Phase 2 without a dialog
  timestamp: 2026-11-26T18:35:10.731Z
- hypothesis: the GPU process crash or a Chromium switch makes the close slow and that is what opens the dialog
  evidence: CefRenderer-0.log's GPU crashes are at 18:04:45Z, not at the 12 close times; a slow close cannot open a shell dialog; the timeout WARN is the consequence of the browser never being destroyed
  timestamp: 2026-11-26T18:35:10.731Z
- hypothesis: a specific theme page closes itself (window.close())
  evidence: all 12 switches, across several different themes (CritterCrossing, Farm, Frog Rush, ...), behave the same; the close is started by the service's Close (render.rs logs "the service asked this renderer to close" each time). window.close() is however a second trigger of the same CEF path and is covered by the fix
  timestamp: 2026-11-26T18:35:10.731Z
- hypothesis: the legacy app shows how to close CefSharp under the desktop without the dialog
  evidence: legacy Sucrose.Live.CefSharp (CefSharp.Wpf.HwndHost, windowed) never closes the browser through CEF on a switch: Sucrose.Shared.Live/Helper/Kill.cs and Space/Helper/Live.cs Kill() end the Live process with Process.Kill; it sets no LifeSpanHandler, and CefSharp's ClientAdapter::DoClose returns false without one. There is no legacy close behavior to port
  timestamp: 2026-11-26T18:35:10.731Z

## Resolution

root_cause: CEF's default windowed close sends WM_CLOSE to the browser's top-level ancestor (CloseHostWindow: PostMessage(GetAncestor(browser, GA_ROOT), WM_CLOSE)) because our life span handler did not override DoClose; the wallpaper browser lives in the desktop surface under Progman, so that ancestor is Progman and WM_CLOSE there opens Shut Down Windows; and since nothing destroyed the browser window during the close loop, the close never completed, timed out after 5 s and the renderer crashed in cef::shutdown() (0xFFFF7003 = Crashpad kTerminationCodeNotConnectedToHandler, twice 0xC0000005)
fix: SpanHandler::do_close answers 1 for the wallpaper browser (do_close_answer; popups opened by the page keep CEF's default 0, their window is their own root), so CEF never posts its close message; CefHost::close marks closing before close_browser(1), and its loop (close_step) destroys the browser's own window (tear_down -> DestroyWindow on host.window_handle(), same thread that CEF created it on) as soon as DoClose came, or after FORCED_TEAR_DOWN_AFTER = 3 s without it, then pumps until OnBeforeClose (logs "the CEF browser closed elapsed_ms=...") or CLOSE_TIMEOUT = 5 s. A DoClose that is not part of Sucrose's close (window.close() in the page) is answered 1 and ignored with a WARN: the wallpaper stays open. on_before_close ignores popups, so a closed popup no longer marks the wallpaper browser closed. Docs: FEATURES.md Web (local HTML) row, ARCHITECTURE.md CEF subprocesses row.
verification:
  target_test: { result: pass, tests: [browser::tests::the_wallpaper_browser_never_lets_cef_send_its_close_message, browser::tests::the_close_destroys_the_browser_window_once_cef_is_ready, browser::tests::the_close_destroys_the_window_anyway_when_cef_never_gets_ready_and_gives_up_at_the_timeout, browser::imp::tests::the_life_span_handler_never_lets_cef_close_the_desktop_window] }
  oracle_type: specified (CEF's documented DoClose contract: return true to suppress the standard close notification, then complete the close by window tear-down) plus boundary neighbors (FORCED_TEAR_DOWN_AFTER - 1 ms / at, CLOSE_TIMEOUT - 1 ms / at, already closed at 0 ms, popup vs wallpaper browser)
  mutation_check: { result: pass, tool: "manual mutants (no Stryker for Rust)", mutant_killed: [M1, M2, M3, M4] }
  no_op_deletion: { result: pass, note: "the diff adds the DoClose answer, the tear-down and the close decision; the only removal is the old fixed-deadline loop, replaced by close_step" }
  adjacent_tests: { result: pass, suites_run: ["cargo test --workspace (all suites)", "cargo clippy --workspace --all-targets -D warnings", "cargo fmt --all --check"] }
  revert_and_reconfirm: { result: pass, unit_level: "M4 (pre-fix handler without the DoClose override) turns the handler test red; reapplied it is green", desktop_level: "pass - before: the 04-20 run at 5a17b108 (12/12 switches opened the Shut Down Windows dialog, 12/12 'did not close in time' after 5 s, renderer exits 0xffff7003 (10) / 0xc0000005 (2)); after: the maintainer run of 2026-11-26 18:46:56Z with the fix (target/round-a-logs-fix: no dialog, 14/14 CEF closes logged 'the CEF browser closed' in 4-28 ms and 'the renderer stopped', 0 timeout WARNs, all 14 CEF renderer exits code 0)" }
  human_verify: { result: pass, by: maintainer, date: 2026-11-26, answer: "duzeldi (fixed): the Shut Down Windows dialog no longer appeared", switches: "Web -> Web, Web -> Video, Video -> Web, one close with DevTools open" }
  guardrail_verdict: accepted
files_changed: [crates/sucrose-engine-cef/src/browser.rs, docs/FEATURES.md, docs/ARCHITECTURE.md, .planning/STATE.md]
fix_commit: 5758edd0 (fix(cef): close the wallpaper browser without WM_CLOSE to Progman)
side_observations:
  fullscreen_label: "mislabel, not a defect and not part of this one: restart_with (D-26) reuses the fullscreen rule's RuleClosed/respawn path on purpose, so its INFO lines name the fullscreen app on theme switches; follow-up candidate (/gsd-quick): neutral wording or a close reason in RuleClosed"
  writer_stops_warn: "benign exit race, not a separate defect: the service's writer had a queued message when the dying renderer's pipe closed (same instant as 'a renderer disconnected', 2 of 12); the 5 s hang made it likelier; follow-up candidate: log at debug once the reader has seen the disconnect; still seen twice (Service lines 59 and 219) in the verification run after clean closes, so it is independent of the close hang"
  exit_codes: "part of this defect: 0xFFFF7003 / 0xC0000005 were crashes after the close timeout; confirmed gone with the fix (14/14 CEF renderer exits code 0 in the maintainer run)"
  video_unrecognized_format: "seen in the verification run, unrelated: the Video theme picked from the synthesized scratch library failed in mpv with 'unrecognized file format' (renderer exit code 4, StartFailed restarts, 6 exits); most likely a placeholder video in target/round-a; recorded only"
  gpu_process_crash: "still present and unrelated: CefRenderer-0.log shows the known GPU process crash (3 times at a Web wallpaper start); the STATE blocker since 04-02 stays open"
eol_notes: "browser.rs, FEATURES.md, ARCHITECTURE.md are w/crlf and were written back as CRLF (FEATURES.md and ARCHITECTURE.md keep no final newline); cargo fmt was only run with --check, so nothing needs restoring. The debug file is git-ignored (.gitignore [Dd]ebug/), committing it needs git add -f or the gsd commit tool."
prevention:
  five_whys:
    - "Why did Shut Down Windows open? Progman received WM_CLOSE."
    - "Why did Progman get it? CEF's default windowed close posts WM_CLOSE to the browser's GA_ROOT ancestor, and under the desktop surface that ancestor is Progman (branch: environment - the 24H2 raised layout puts the surface directly under Progman)."
    - "Why did CEF use its default close? Our life span handler did not override DoClose (branch: code - cef-rs' default do_close returns 0)."
    - "Why was the default kept? The CEF host was written for an ordinary windowed child, whose top-level window is its own; the DoClose contract (cef_life_span_handler.h) was not checked against a browser hosted inside a shell window."
    - "Why did the close also crash? With DoClose false nothing but our own top-level window's WM_CLOSE handler would have destroyed the browser; there is none, so close() timed out and cef::shutdown ran with a live browser."
  why_not_caught: "no gate existed for this class: unit tests never called the life span handler through CEF's C function table and had no model of the close sequence; code review did not check the DoClose contract; the first desktop run that switched away from a CEF wallpaper was the 04-20 maintainer check, which caught it"
  recurrence_guard: "regression tests crates/sucrose-engine-cef/src/browser.rs: imp::tests::the_life_span_handler_never_lets_cef_close_the_desktop_window (calls do_close through the C table), tests::the_wallpaper_browser_never_lets_cef_send_its_close_message, tests::the_close_destroys_the_browser_window_once_cef_is_ready, tests::the_close_destroys_the_window_anyway_when_cef_never_gets_ready_and_gives_up_at_the_timeout; FEATURES Web row and ARCHITECTURE CEF row; KB entry cef-close-shutdown-dialog"
mempalace: "skipped: no MemPalace tool in this session; knowledge-base.md is the durable record"
