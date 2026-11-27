---
status: resolved
trigger: "Quick item 261127-uos (D10) found that a closed WebView2 renderer's browser process sometimes takes ~6.2 s to exit (other closes 0.20-0.26 s); waiting for it before the next renderer opens the same profile folder makes a Web theme switch on WebView2 take ~6 s. The maintainer chose to investigate the cause instead of accepting the 8 s BROWSER_EXIT_WAIT (2026-11-27, verbatim: araştır)."
created: 2026-11-27T19:48:31.835Z
updated: 2026-11-27T20:32:25.155Z
goal: find_and_fix
symptoms_prefilled: true
---

## Current Focus

status_note: resolved 2026-11-27 on machine evidence. Fix committed as e747c7be (SmartScreen off by default on WebView2, D-041 Accepted by the maintainer). The visual desktop check (fast Web theme switching with DISPLAY9 attached, about 1 s per switch on both monitors) is pending in the combined maintainer session at the end of the quick batch; the user setting that turns SmartScreen on is a separate quick task (STATE.md Pending Todos (e))
next_action: none in this session; archived to .planning/debug/resolved/

hypothesis: CONFIRMED: on a new user data folder the first main-frame navigation sends an Edge SmartScreen URL check; a controller close while that check is in flight makes the browser's tear-down wait about 6.3 s before the browser, GPU, network and storage processes exit
bug_class: Bohrbug
reasoning_checkpoint:
  hypothesis: "The WebView2 browser process of a never-used user data folder sends an Edge SmartScreen URL check (nav-edge.smartscreen.microsoft.com) with its first main-frame navigation; when Sucrose closes the controller while that check is in flight (the first about 0.2-1 s of the start), Chromium's tear-down blocks about 6.3 s after the profile unload, so BrowserProcessExited comes about 6.3 s after the close and the renderer, which waits for it before releasing the profile folder, holds up the next renderer"
  confirming_evidence:
    - "probe 1: the 2 slow closes of 20 (6410, 6385 ms) were the only closes within about 0.2 s of a start on a new folder; the same moments on the used persistent folder took 263-268 ms"
    - "probe C/c2: a close 50 ms after start on two new folders: 4 of 4 slow (6289-6407 ms); probe D (same, msSmartScreenProtection disabled): 6 of 6 fast (226-253 ms)"
    - "chrome_debug.log of the slow browser: SmartScreen navigate request sent at the profile unload, then 6.18 s of silence on the browser main thread before CrxUpdateService stopping; the fast browser goes from unload to CrxUpdateService stopping in 166 ms"
    - "process poller: during the slow span the browser and its gpu, network and storage processes are alive, the page's renderer is already gone; nothing of ours is waited on"
  falsification_test: "with SmartScreen off, a close on a new folder within the first second would still take seconds (probe D showed 6/6 fast), or the regression test with the switch removed would still pass"
  fix_rationale: "Sucrose adds disable-features=msSmartScreenProtection to its own WebView2 switches (args::sucrose_switches, merged into the user's disable-features list), so the browser has no SmartScreen service and no check is ever in flight at a close; the browser exits in about 0.25 s on every close, and BROWSER_EXIT_WAIT can go back to the planned 1 s bound (under SHUTDOWN_GRACE 2 s and MOVE_CLOSE_GRACE 4 s). It also stops sending every wallpaper address to Microsoft and matches CEF, which has no SmartScreen"
  blind_spots: "only measured on this machine (WebView2 Runtime 158.0.4370.45, Windows build 26300, DNS through Cloudflare DoH, debug build); not measured with a URL wallpaper whose page keeps network requests open at the close; whether another Edge service (component updater, variations) can hold the tear-down the same way on a slower network; the Edge feature name is widely used (Wails disables it always) but not in an official flag list"
  candidate_causes:
    - "config: Edge SmartScreen on by default in the WebView2 browser (confirmed)"
    - "environment: network/DNS latency of the SmartScreen request (DoH requests during the stall) decides how long the stall is"
    - "data: a never-used user data folder (no SmartScreen state yet), which the ephemeral folders always are"
    - "code: our close sequence (eliminated: same code fast in 18 of 20 closes, processes waited on are the browser's own)"
  and_gate: "yes: the stall needs SmartScreen on AND a SmartScreen check in flight at the close, which in practice means a new folder AND a close within about the first second; removing the config cause (SmartScreen) removes the stall whatever the folder or timing"
next_action: add disable-features=msSmartScreenProtection to args::sucrose_switches for WebView2, update tests/args.rs, set BROWSER_EXIT_WAIT back to 1 s, add the portal.rs desktop regression test, then measure close->exit distribution and switch time

## Symptoms

expected: Switching from one Web theme to another on WebView2 is about as fast as on CEF (new page within ~1 s of the click), the next renderer gets its web profile folder (persistent on the primary monitor, ephemeral elsewhere, D-040) without "no web profile folder", and no fixed multi-second wait is needed.
actual: Commit c1ac7f05 (quick 261127-uos task 2) made the WebView2 renderer wait for BrowserProcessExited before releasing the profile lock, bounded by BROWSER_EXIT_WAIT = 8 s (plan said 1 s). Measured in apps/sucrose/tests/portal.rs switching_web_themes_on_webview2_closes_each_renderer_cleanly: "webview2 browser process exited" after 0.20-0.26 s in some closes and ~6.2 s in others (last run 6186 ms and 6373 ms; earlier 259 ms and 201 ms), including both renderers of some runs; no clean cold vs warm profile split. With a 1 s bound the exit came late, a WARN was logged and the next renderer's folder was still in use (os error 32, the original D10 symptom "no web profile folder; the page keeps nothing"). The 8 s bound exceeds SHUTDOWN_GRACE (2 s) and MOVE_CLOSE_GRACE_MS (4 s).
errors: WARN "no web profile folder; the page keeps nothing error=... (os error 32)" (before c1ac7f05); "did not exit in time" WARN with the 1 s bound.
timeline: D10 first seen 2026-11-27 in cargo test with DISPLAY9 attached; c1ac7f05 added the wait. Monitors now (the maintainer attached the virtual display again): 
reproduction: cargo test -p sucrose --test portal switching_web_themes_on_webview2_closes_each_renderer_cleanly (logs in target/tmp/portal/webview2-switch/Log), or a scratch data dir with two Web fixtures on WebView2 and two Use switches in a row; measure the time from "the service asked this renderer to close" to "webview2 browser process exited" and to the next "page loaded".

## Constraints (from the maintainer and the project)

- Desktop runs are allowed. C# Sucrose stays closed. Scratch SUCROSE_DATA_DIR under target/, WSLENV, Portal only with --no-focus, never synthesize input, stop with taskkill.exe /IM sucrose-portal.exe /F and /IM sucrose.exe /F, never kill msedgewebview2.exe by name (other apps use WebView2 on this machine), tasklist.exe empty at the end, stop apps before rebuilding.
- With DISPLAY9 attached, an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen fails on its one-monitor count until quick item 261127-uou lands (known, not this bug).
- Quick items 261127-uos (D9/D10 code committed: bd84d9af, 8668fb8b, c1ac7f05) and 261127-uot (album art, committed up to e54c0523) are waiting at their maintainer checkpoints; do not change their scope. Record the outcome in D-040 (docs/DECISIONS.md, LF) if the wait bound or the folder strategy changes.

## Evidence

- timestamp: 2026-11-27T19:52:05.099Z
  checked: knowledge base (.planning/debug/knowledge-base.md)
  found: no entry about WebView2 browser process exit or profile folder release
  implication: no known-pattern hypothesis to test first

- timestamp: 2026-11-27T19:52:05.099Z
  checked: crates/sucrose-engine-webview2/src/host.rs close_inner, crates/sucrose-engine-webview2/src/engine.rs WebView2Pump::close, crates/sucrose-engine-web/src/profile.rs
  found: close_inner removes the event tokens, registers BrowserProcessExited, calls controller.Close(), releases webview, controller3 and controller, then pumps messages until the exit event or BROWSER_EXIT_WAIT (8 s), and only then releases the environment; WebView2Pump::close drops the profile lease after host.close() returns. The WebResourceRequested handler closure holds a clone of the environment and the accelerator handler a clone of the webview (both removed before Close). The BROWSER_EXIT_WAIT comment claims 6.2 s "when the profile folder is new"
  implication: the renderer process stays alive for the whole wait; the next renderer of the slot is spawned only after the old one exited (service log: slot 1 exited 19:45:19.723, respawned 19:45:19.730), so the wait adds straight to the switch time

- timestamp: 2026-11-27T19:52:05.099Z
  checked: apps/sucrose/tests/common/mod.rs TestService::start and profile::acquire
  found: TestService removes target/tmp/portal/<name> before each run, so the persistent Web/WebView2 folder is new on every run; acquire empties the ephemeral folder at every start, so it is always new too
  implication: "new folder" cannot be what separates 0.2 s closes from 6.2 s closes (both kinds are always new in the test), contradicting the BROWSER_EXIT_WAIT comment

- timestamp: 2026-11-27T19:52:05.099Z
  checked: last run's logs target/tmp/portal/webview2-switch/Log (19:45Z)
  found: the second Use came at 19:45:19.158, 40-90 ms after "engine started" (19.119 / 19.213), before any "page loaded" line; the third Use came at 19.48 while both were still closing, so the storage theme never ran in that run; slot 1 (ephemeral) exited after 460 ms; slot 0 (persistent) had not logged an exit 0.85 s later when the test ended and the service was killed
  implication: the switch test's timing collapses Use requests and closes pages before they load; which page a renderer closes, and whether it had loaded, varies run to run, which could explain the missing pattern; a controlled probe is needed

- timestamp: 2026-11-27T19:57:10.017Z
  checked: probe run 1 (scratch test apps/sucrose/tests/zz_exit_probe.rs, 2 monitors, 10 Use steps, target/tmp/portal/exit-probe) with a read-only Win32_Process poller (target/exit-probe/procs-1.txt)
  found: 18 of 20 closes took 263-304 ms close->exit; the 2 slow ones (6410 ms, 6385 ms) were both slot 1 (DISPLAY9, ephemeral folder, emptied = new at every start) and both came within about 0.2 s of that renderer's start (16 ms after page loaded; 47 ms after engine started, before page loaded). At the same two moments slot 0 (persistent folder, already used by earlier starts of this run) closed in 263 ms and 268 ms. Slot 1 closes 4-8 s after a start on a new folder were fast (267-301 ms). Closing the storage page after 4 s or 8 s was fast; right after load it was slow only on the new folder
  implication: the slow exit needs BOTH a new (never used) user data folder AND a close in the first seconds after that browser started; page content (localStorage) and the monitor are not the trigger (slot 0 closed the same pages at the same moments fast)

- timestamp: 2026-11-27T19:57:10.017Z
  checked: the poller snapshots during the slow slot-1 close 19:54:05.866 -> 19:54:12.294
  found: the page's renderer process was gone within 0.2 s; the browser, gpu-process, network service and storage service utility processes of that folder stayed alive for the whole 6.3 s and all vanished in the same 170 ms snapshot gap at the end; no crashpad process exists (--disable-breakpad)
  implication: the browser process itself delays its own shutdown about 6.3 s after the close request; nothing of ours (renderer process, references) is involved in that span: the renderer only waits

- timestamp: 2026-11-27T19:57:10.017Z
  checked: switch time of the probe (Use request -> every renderer's next page loaded)
  found: about 0.93-0.99 s on every fast step; 7.09 s and 7.02 s on the two steps with a slow slot-1 close
  implication: a switch on WebView2 is as fast as expected except when the previous renderer's folder was new and it was closed early, which the switch test does on every first switch (fresh test folder, Use right after engine started)

- timestamp: 2026-11-27T20:00:13.332Z
  checked: probe B (fresh test folder, Use hello, close about 1.7 s after engine started, Chromium logging on through WebArguments) and probe C (same, close about 0.15 s after engine started)
  found: probe B: both new folders closed in 287 ms and 281 ms. Probe C: both new folders closed in 6289 ms (slot 0, persistent, new) and 6306 ms (slot 1, ephemeral, new); reproduced on the first try
  implication: the persistent folder is as slow as the ephemeral one when it is new; the window is narrow (well under 1.7 s after the start); reproducible on demand with a 50 ms delay

- timestamp: 2026-11-27T20:00:13.332Z
  checked: chrome_debug.log of the slow probe C browser (target/tmp/portal/exit-probe-c/Cache/WebEphemeral/WebView2/1/EBWebView/chrome_debug.log) vs the fast probe B browser
  found: fast: keep-alives reach zero and "Unloading profile Default" at 24.199, "CrxUpdateService stopping" 166 ms later, exit 0.1 s after that. Slow: the main frame navigation starts at 34.705, Edge SmartScreen's DNS check fails for the virtual origin host (SmartScreenDnsResolver -105) and a SmartScreen URL check to nav-edge.smartscreen.microsoft.com/api/browser/edge/navigate/3 is sent at 34.780, the same millisecond the profile unloads; then the browser main thread logs nothing for 6.18 s until "CrxUpdateService stopping" at 40.958; meanwhile the network service sends two DoH requests (chrome.cloudflare-dns.com) at 39.56 and 40.57. In the fast case the SmartScreen navigate, settings and telemetry requests had all been sent 1.2 s before the close
  implication: candidate: the browser's tear-down waits for an Edge SmartScreen request that is still in flight at the close (cold profile: no SmartScreen settings yet, first check goes out with the first navigation); to be tested by turning SmartScreen off (msSmartScreenProtection in --disable-features, the documented WebView2 switch; Settings8.IsReputationCheckingRequired=false in the API)

- timestamp: 2026-11-27T20:06:21.901Z
  checked: probe D (= probe C with msSmartScreenProtection appended to the default --disable-features value) three times, and probe C once more as a control, same build, same machine, minutes apart
  found: SmartScreen off: 6 of 6 closes (new folders, close 106-146 ms after engine started) exited in 226-253 ms; chrome_debug.log has 0 SmartScreen lines. Control (SmartScreen on): 2 of 2 closes exited in 6404 ms and 6407 ms (probe C before: 6289 and 6306 ms)
  implication: CONFIRMED: Edge SmartScreen is a necessary condition of the slow exit; with it off the browser exits at once even on a new folder closed right after its start

- timestamp: 2026-11-27T20:06:21.901Z
  checked: legacy C# app (src/, grep SmartScreen / ReputationChecking) and the Rust workspace
  found: neither mentions SmartScreen: the legacy WebView2 engine ran with SmartScreen on (the runtime default); CEF has no SmartScreen or Safe Browsing at all. Every WebView2 wallpaper start sends the page address (https://sucrose.wallpaper.engine/index.html, or the user's URL) to nav-edge.smartscreen.microsoft.com
  implication: turning SmartScreen off is a behavior change against legacy WebView2 (record in FEATURES.md and DECISIONS.md) but matches CEF and stops sending wallpaper addresses to Microsoft

- timestamp: 2026-11-27T20:17:21.879Z
  checked: fix applied (args::sucrose_switches adds disable-features=msSmartScreenProtection for WebView2; BROWSER_EXIT_WAIT 8 s -> 1 s) and the new desktop test apps/sucrose/tests/portal.rs a_webview2_renderer_closed_right_after_its_start_on_a_new_profile_exits_at_once (fresh folders, Use hello, Use gif right after every renderer's engine started: the close lands 200-255 ms after the start)
  found: with the fix 5 of 5 runs pass, 10 closes 221-245 ms; the renderer log shows --disable-features=...,IsolateOrigins,msSmartScreenProtection. With only the push line removed (mutant at the fix site) 3 of 3 runs FAIL: 5 of 6 closes hit "did not exit in time" after the 1 s bound
  implication: the driving test kills the fix-site mutant and the bug returns on revert, goes away on reapply

- timestamp: 2026-11-27T20:17:21.879Z
  checked: switching_web_themes_on_webview2_closes_each_renderer_cleanly 3 times with the 1 s bound; release-build probe (target/exit-probe/zz_exit_probe.rs, the probe 1 sequence of 10 switches) on WebView2 and on CEF, 2 monitors
  found: switching test 3 of 3 pass (exits 223-351 ms, no WARN). Release WebView2: 20 of 20 closes 212-236 ms; Use -> both new pages loaded 0.82-0.94 s on every step, including the two steps that took 7.0-7.1 s before (steps 7 and 9: 0.824 s and 0.857 s). Release CEF, same steps: 0.62-0.88 s (first start 1.9 s)
  implication: a WebView2 switch is now within about 0.15-0.2 s of CEF; the remaining difference is the browser exit wait itself (about 0.23 s)

- timestamp: 2026-11-27T20:17:21.879Z
  checked: cargo fmt --all, cargo clippy --workspace --all-targets -D warnings, cargo test --workspace --no-fail-fast (DISPLAY9 attached)
  found: fmt changed only tests/args.rs formatting; clippy clean; every suite passes except portal.rs an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen (panics at portal.rs:745, the known one-monitor assumption with DISPLAY9, quick item 261127-uou); line endings restored (args.rs and tests/args.rs w/crlf as before, host.rs, portal.rs, DECISIONS.md w/lf, FEATURES.md and MEASUREMENTS.md w/crlf, FEATURES.md still without a final newline)
  implication: no regression from the change

- timestamp: 2026-11-27T20:26:25.438Z
  checked: the maintainer's answer to D-041 (checkpoint response, 2026-11-27, verbatim: "bunun kapatılıp kapatılmayacağını kullanıcı seçsin. varsayılan olarak kapalı gelsin")
  found: SmartScreen stays off by default on WebView2 and the user gets a setting to turn it on; the setting is built in a separate quick task after this session; the desktop check is deferred to one combined maintainer session at the end of the quick batch
  implication: the fix as applied is the default; only the doc comments that say "always" and the D-041/D-040/FEATURES text change in this step (no code change)

- timestamp: 2026-11-27T20:32:25.155Z
  checked: final run after the D-041 wording (doc comments in args.rs and host.rs, D-041 Accepted, D-040, FEATURES rows; no code change): cargo fmt --all --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace --no-fail-fast (DISPLAY9 attached), then the two WebView2 desktop tests alone with --test-threads=1
  found: fmt and clippy clean; every suite passes except portal.rs an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen (panics at portal.rs:745, the known one-monitor assumption, quick item 261127-uou); the new early-close test and the switching test pass. In the parallel full suite the four WebView2 exits took 663-949 ms (other desktop tests start services and renderers at the same time), alone 223-227 ms; no "did not exit in time", no "no web profile folder"; the renderer switches carry msSmartScreenProtection; tasklist.exe shows no sucrose.exe or sucrose-portal.exe after the runs
  implication: no regression; under heavy parallel test load the exits come within about 50 ms of the 1 s bound, which a slower machine could cross in the full suite (a WARN, not the SmartScreen stall); noted for the maintainer, not changed (the code fix stays as verified)

## Eliminated

- hypothesis: the storage fixture's localStorage write (DOM storage delayed commit) keeps the browser process alive
  evidence: probe run 1: slot 0 closed the storage page 16 ms after its load in 263 ms; slot 1 closed hello2 (no storage) before load in 6385 ms
  timestamp: 2026-11-27T19:57:10.017Z

- hypothesis: references we keep after Close() (environment, handler clones) keep the browser alive
  evidence: the same host code closes in 263-304 ms in 18 of 20 closes; in the slow ones the poller shows the browser's own child processes (gpu, network, storage) alive, not a wait on us; a kept reference would make every close slow
  timestamp: 2026-11-27T19:57:10.017Z

## Resolution

root_cause: Edge SmartScreen in the WebView2 browser (config, on by default): on a never-used user data folder (every ephemeral folder at every start, the persistent one on its first run) the first main-frame navigation sends a SmartScreen URL check to nav-edge.smartscreen.microsoft.com; a controller close while that check is in flight (about the first second of a start, which a quick theme switch hits) makes Chromium's tear-down wait about 6.3 s after the profile unload, so BrowserProcessExited, and with it the release of the profile folder, came about 6.3 s after the close (AND-gate: SmartScreen on AND a check in flight at the close)
fix: Sucrose's own WebView2 switches (args::sucrose_switches) add disable-features=msSmartScreenProtection (args::WEBVIEW2_SMARTSCREEN_FEATURE) by default, merged into the user's disable-features list and listed in the Portal among Sucrose's own switches; BROWSER_EXIT_WAIT back to 1 s (the plan's bound, under SHUTDOWN_GRACE 2 s and MOVE_CLOSE_GRACE 4 s). D-041 Accepted 2026-11-27 (the maintainer, verbatim: "bunun kapatılıp kapatılmayacağını kullanıcı seçsin. varsayılan olarak kapalı gelsin"): off by default, a user setting to turn SmartScreen on (planned Engine.WebViewSmartScreen, default false) follows in a separate quick task; D-040 amended; docs/FEATURES.md Web and web-data rows; docs/MEASUREMENTS.md switch row
commit: e747c7be
oracle_type: specified (every closed WebView2 renderer logs "webview2 browser process exited" with elapsed_ms below the 1 s bound, no "did not exit in time", no "no web profile folder")
verification:
  target_test: { result: pass, test: "apps/sucrose/tests/portal.rs a_webview2_renderer_closed_right_after_its_start_on_a_new_profile_exits_at_once", runs: "5 of 5 in the session (closes 221-245 ms); again in the final full suite (663-949 ms under parallel load) and alone (223-227 ms)" }
  mutation_check: { result: pass, reason_if_skipped: "Stryker does not apply to Rust; done by hand", mutant_killed: "the push of the SmartScreen switch removed: 3 of 3 runs fail" }
  no_op_deletion: { result: pass, deletion_justified_by_rca: true, note: "adds a switch; the only reduction is BROWSER_EXIT_WAIT 8 s -> 1 s, justified by the measured 0.21-0.25 s exits once the cause is gone" }
  adjacent_tests: { result: pass, suites_run: ["cargo test --workspace --no-fail-fast (only the known DISPLAY9 failure an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen, quick item 261127-uou)", "sucrose-engine-web args 19/19", "switching_web_themes_on_webview2_closes_each_renderer_cleanly"] }
  revert_and_reconfirm: { result: pass, bug_returned_on_revert: true, fixed_on_reapply: true, note: "revert of the switch: 5 of 6 closes past the bound; control probe without the switch 6.29-6.41 s; reapply 10 of 10 closes 221-245 ms" }
  guardrail_verdict: accepted
  desktop_check: { result: pending, note: "the maintainer's visual check (fast Web theme switching with DISPLAY9 attached, about 1 s per switch on both monitors) is deferred to the combined maintainer session at the end of the quick batch; resolved on the machine evidence above" }
files_changed:
  - crates/sucrose-engine-web/src/args.rs
  - crates/sucrose-engine-web/tests/args.rs
  - crates/sucrose-engine-webview2/src/host.rs
  - apps/sucrose/tests/portal.rs
  - docs/DECISIONS.md
  - docs/FEATURES.md
  - docs/MEASUREMENTS.md
