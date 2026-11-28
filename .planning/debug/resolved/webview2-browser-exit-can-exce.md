---
status: resolved
trigger: "WebView2 browser exit can exceed the 1 s bound (1052 ms in 04-39 L5; ENG-02)"
created: 2026-11-28T02:32:33.557Z
updated: 2026-11-28T04:06:40.952Z
goal: find_and_fix
symptoms_prefilled: true
---

## Current Focus

status_note: resolved 2026-11-28 on machine proof (orchestrator: machine proof is enough; maintainer decision A "1,5 sn" applied). The reasoning_checkpoint below is the pre-fix one; its load claims are corrected in the Evidence (v-l1, v-l8) and in Resolution
hypothesis: CONFIRMED: the WebView2 Runtime's slower browser shutdown path (0.77-1.12 s idle machine, up to 1.25 s under the test suite) sat on the 1 s bound; 1.5 s waits it out
bug_class: Mandelbug (intermittent, environment- and timing-dependent; deterministic code path, nondeterministic entry)
reasoning_checkpoint:
  hypothesis: "The late exits are a fixed slow path (about 1 s) of the WebView2 runtime's own browser-process shutdown, entered intermittently (audio stream on a folder's first start, CPU load), and the 1 s BROWSER_EXIT_WAIT lies on that mode; Sucrose's close sequence takes no part in the extra time"
  confirming_evidence:
    - "process poller h13/k23/n6: every child (renderer, audio, network, storage, gpu) ends 0.19-0.28 s after the close in fast and slow cases alike; only the browser process stays, idle in all 52-53 threads, until 0.77-1.08 s"
    - "Chromium log g14: the slow browser's log is identical to a fast one and ends at +318 ms; the exit event comes at +1031 ms"
    - "bimodal distribution with a fixed slow mode: load l1/l2 (no audio) 1006-1084 ms; audio cases 772-1115 ms; nothing between about 410 and 770 ms"
    - "fault injection: tone page (audio, no network) reproduces it on both monitors; busy and unload-spinning pages do not; muted YouTube Followers never (0 of about 57) while unmuted Masters do (5 of about 57)"
  falsification_test: "a close sequence change in Sucrose (or a switch) that makes the slow mode vanish in the reproduction (new-folder tone or YouTube first closes, or the 16-thread load run) would show a removable cause; a slow exit far beyond 1.2 s would show the mode is not fixed"
  fix_rationale: "not a removed cause: raising BROWSER_EXIT_WAIT to 1.5 s would wait out the whole slow mode (max seen 1115 ms) with about 0.4 s margin, keep normal closes unchanged (0.2-0.35 s) and stay under SHUTDOWN_GRACE 2 s and MOVE_CLOSE_GRACE 4 s; needs the maintainer's decision (Constraints)"
  blind_spots: "what exactly the runtime waits for in its idle span (no ETW/elevation here; --trace-shutdown writes nothing in WebView2); only this machine and runtime 158.0.4370.45; the AudioServiceOutOfProcess A/B is inconclusive (0/12 vs 1/12); the release build was only seen in L5, the probes used the debug sucrose build (the browser is the same runtime binary either way)"
  candidate_causes:
    - "environment: the WebView2 runtime's internal shutdown wait (confirmed as the place of the time)"
    - "environment: CPU saturation makes the runtime take the slow path (confirmed, l1/l2)"
    - "data: a profile folder on its first start with an audio stream (strongly associated: 7 of 69 vs 0 of about 81 on used folders)"
    - "code: Sucrose's close sequence (eliminated: same code on both paths, nothing of ours runs in the idle span; busy pages do not trigger it)"
    - "config: SmartScreen (eliminated: off in every run)"
  and_gate: "yes for the trigger (the runtime's slow path AND a trigger condition: audio on a first start, or load); no for the fix: whatever triggers it, the slow path ends by about 1.1 s"
next_action: none (archived)

## Symptoms

expected: Every closed WebView2 renderer logs "webview2 browser process exited" with elapsed_ms below BROWSER_EXIT_WAIT (1 s, crates/sucrose-engine-webview2/src/host.rs:308) and never "the WebView2 browser process did not exit in time" (04-39 Edge truth of ENG-02 concurrency; apps/sucrose/tests/portal.rs:1060-1073 asserts the same), since the SmartScreen fix e747c7be (D-041).
actual: 04-39 log check L5 (2026-11-28, release build of HEAD b5eec90f, DISPLAY9 attached, maintainer driving the Portal): 83 of 84 exits took 174-461 ms (241-388 ms during C2), one took 1052 ms: "2026-11-28T01:12:22.371599Z WARN sucrose_engine_webview2::host::imp: the WebView2 browser process did not exit in time; its profile folder may still be in use elapsed_ms=1052" in Renderer-0 (DISPLAY1, persistent profile per D-040), while closing a YouTube page (Spirit Tree, replaced by Space Train at 04:12:21 local / 01:12:21Z). Earlier a 1034 ms exit was seen under a full parallel test run (STATE.md open finding "BROWSER_EXIT_WAIT 1 s is tight"). On 2026-11-28 05:08-05:10 (+03:00) the portal.rs tests switching_web_themes_on_webview2_closes_each_renderer_cleanly (portal.rs:1001) and a_webview2_renderer_closed_right_after_its_start_on_a_new_profile_exits_at_once (portal.rs:1064) failed in full cargo test runs (and the first once in `cargo test -p sucrose --test portal` alone); the next full run passed. Those runs may have used stale artifacts (a mutant YouTubeContent.html from the previous debug session's mutation testing was embedded in some test binaries until a touch), so treat them as weak evidence. Related maintainer observation C2 (04-39): on WebView2 a Web theme appears on both monitors after "a few seconds" (measured 0.68-2.36 s Use-to-page-load, median 1.09 s), not about 1 s; the browser exit was about 0.33 s in the slow Use that was split, so C2's slowness is mostly the new start and the page load, not the exit.
errors: the WARN line above; no "no web profile folder" line; after a late exit the next renderer retries the profile folder (RETRY_DELAY 2 s, host.rs) — check whether that retry happened at 01:12:22Z and what it cost.
timeline: since e747c7be (2026-11-27: SmartScreen off by default, BROWSER_EXIT_WAIT back from 8 s to 1 s). Before that, ~6.3 s exits came from SmartScreen (resolved). Seen once in 04-39 (1 of 84 closes) and once under test load.
reproduction: logs in target/round-b-recheck-logs/ (Renderer-0.2026-11-28.log, Service.2026-11-28.log around 01:12:21Z; also the addendum/ copy). Live: release build, scratch data folder (target/round-b-recheck; settings migrated read-only; library target/round-b-lib/Library with YouTube themes such as "Spirit Tree-1", "Space Train-1", "Floating in Space-1", and the web fixtures), Engine.Web/Url/YouTube = WebViewLive, start `target/release/sucrose.exe --no-portal` with SUCROSE_DATA_DIR="$(wslpath -w target/round-b-recheck)", SUCROSE_LOG=info,sucrose_engine_webview2=debug and WSLENV=SUCROSE_DATA_DIR:SUCROSE_LOG; switch themes by IPC/settings the way portal.rs tests do (no synthesized input), especially away from a playing YouTube page, and with DISPLAY9 attached (two renderers). Tests: `cargo test -p sucrose --test portal -- webview2` alone and under load. Intermittent.

## Constraints (from the maintainer and the project)

- Unattended WebView2 runs are allowed (maintainer, 2026-11-28, "Serbest"): scratch data only, `sucrose.exe --no-portal` (no Portal, takes no focus), short wallpaper switches on the desktop, no synthesized mouse or keyboard input, every start stopped with `taskkill.exe /IM sucrose.exe /F` and confirmed with `tasklist.exe`. Never kill msedgewebview2.exe by name (other applications use WebView2). The C# app must be closed during runs (at session start: no Sucrose process running); never kill a C# process — return a checkpoint if it runs.
- Do not reopen Accepted decisions (D-040 profile owner, D-041 SmartScreen off by default). Changing BROWSER_EXIT_WAIT or the retry policy is a behaviour change: if the fix is "raise the bound" rather than a removed cause, return a decision checkpoint first (the maintainer chose "investigate the cause" over "raise the bound" in the previous session); keep SHUTDOWN_GRACE (2 s) and MOVE_CLOSE_GRACE (4 s) ordering in mind.
- WSL cannot reach Windows 127.0.0.1 ports: CDP probes run with the Windows `bun.exe`. Bun is the only JS runtime.
- The maintainer's real data (%APPDATA%\Sucrose, H:\Sucrose) is read-only; target/round-b (04-38) is not touched.
- After any mutation test of a file, restore it with a fresh mtime (plain cp or touch, never cp -p): cargo otherwise keeps the mutant's artifacts (knowledge base, previous session).
- Line endings: keep each file's endings (`git ls-files --eol` first; restore CRLF on w/crlf files before commits; the Windows cargo fmt rewrites CRLF to LF). Commit messages through several -m flags. Do not push.
- Before a fix commit: cargo fmt --all, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace (known flaky under load: portal.rs timing tests, memory_detail_counts_live_heap_blocks, service::tests::a_restart_of_another_engine_closes_nothing_and_one_during_a_library_move_is_refused; rerun once and say so).
- Records: a behaviour change goes into docs/FEATURES.md (CRLF, no final newline) and, for a decision, docs/DECISIONS.md (LF); a measured timing goes into docs/MEASUREMENTS.md (CRLF) if notable. ENG-02 is re-marked by the orchestrator/maintainer afterwards, not by this session.

## Eliminated

- hypothesis: H-B as stated: YouTube page tear-down (player, beacons, media decoders, network) holds the browser
  evidence: n6: a local tone page with no network and no video makes both browsers late in the same way
  timestamp: 2026-11-28T03:16:15.664Z

- hypothesis: H-E as "a WebView2 child process (audio service, gpu) ends late and BrowserProcessExited waits for it"
  evidence: h13 and k23: every child of the late browser ended 0.24-0.28 s after the close, as in fast closes; the browser process itself stayed 0.5-0.85 s longer
  timestamp: 2026-11-28T03:11:48.886Z

- hypothesis: H-A as the main cause: CPU load stretches the tear-down
  evidence: k23: the late browser used no CPU in any thread during its extra wait; the late closes happen on an otherwise idle machine (probe runs, no Portal) and only for the unmuted Master
  timestamp: 2026-11-28T03:11:48.886Z

- hypothesis: H-C as stated: only the persistent (used) profile folder is slow at its close (profile flush of a used folder)
  evidence: f1: the late close was slot 1 on its ephemeral folder (new at that start); probe A: the late close was the persistent folder on its first use
  timestamp: 2026-11-28T02:52:02.943Z

- hypothesis: the page's renderer main thread is busy at the close (YouTube player initialisation after a slow load) and the browser waits for its unload/close acknowledgement (a Chromium unload timeout of about 1 s)
  evidence: probe D: a page busy for 6 s at the close and a page spinning 3 s in pagehide/unload both closed in 161-294 ms, like hello
  timestamp: 2026-11-28T02:49:33.467Z

- hypothesis: Edge SmartScreen check in flight at the close (the resolved session's cause)
  evidence: the L5 and probe switches carry --disable-features=...,msSmartScreenProtection (webview_smartscreen=false); the probe A late close happened with the same switches; no SmartScreen lines in the probe C logs
  timestamp: 2026-11-28T02:49:33.467Z

- hypothesis: CPU saturation sends the browser onto the same fixed ~1.05 s slow path (stated in the l1/l2 evidence and the decision checkpoint)
  evidence: v-l1 with the bound at 1.5 s: 38 of 38 closes under 16 busy threads hit the 1.5 s WARN and the ephemeral folder was still in use about 2.6 s after the close; the l1 "1006-1084 ms exits" were WARN lines at the 1 s deadline, not exit times; v-l8 (8 threads): unchanged 225-257 ms
  timestamp: 2026-11-28T04:06:06.239Z

- hypothesis: the host's own wait ends the slow path (the exit tracks the bound)
  evidence: with the bound at 1.5 s the slow-path exits still ended at 941, 958, 1057 ms (v-t1, v-y3) and 817-1251 ms in the portal tests, never at the bound
  timestamp: 2026-11-28T04:06:06.239Z

## Evidence

- timestamp: 2026-11-28T02:37:31.227Z
  checked: knowledge base (webview2-browser-exit-slow) against the L5 renderer log
  found: KB match on "did not exit in time / BROWSER_EXIT_WAIT" (root cause there: an Edge SmartScreen check in flight at the close of a never-used folder, about 6.3 s). In L5 the WebView2 switches of the late start carry --disable-features=...,msSmartScreenProtection and webview_smartscreen=false, and the persistent folder (target/round-b-recheck/Web/WebView2) had been used by about 40 earlier starts
  implication: the SmartScreen mechanism itself is not this case; the general pattern (an in-flight browser task at the close) stays a candidate

- timestamp: 2026-11-28T02:37:31.227Z
  checked: every WebView2 session of target/round-b-recheck-logs/addendum/Renderer-0/1 (WebView2 from 01:06:01Z; the earlier sessions were CEF), tabulated as slot, folder, start->load, load->close, role, page, exit
  found: 84 WebView2 closes with an exit line; 83 at 174-461 ms; the one late close (1052 ms) is slot 0 (DISPLAY1, persistent, Master = audio on, Volume 100) closing YouTubeContent.html?v=DtE5Y5VGkjU (Spirit Tree) 2597 ms after its page loaded; that start's page load took 2403 ms (slot 1, same page: 2423 ms), the slowest loads of the run (others 65-1919 ms). Slot 1 (ephemeral, Follower, muted) closed the same page at the same instant in 273 ms. The other Master YouTube closes on slot 0 were 241-461 ms (load->close 1.6-61 s)
  implication: one sample: the late close coincides with a slow start (environment busy or network slow at 01:12:16-18) and with persistent+Master; YouTube alone or the moment of the close alone is not sufficient (slot 1 same page same moment fast)

- timestamp: 2026-11-28T02:37:31.227Z
  checked: host.rs close_inner/await_browser_exit and the service/renderer log after the late close
  found: elapsed_ms counts from after controller.Close() and the releases (asked 21.287 -> WARN 22.3716, so about 32 ms before the pump); pump_until polls every 50 ms, so 1052 ms only says "not exited by the 1 s deadline", not when it exited. The next slot-0 renderer spawned 22.393, took the Persistent folder at 22.4197, its environment came up at 22.678 (no "the WebView2 environment failed, trying once more", so no RETRY_DELAY), page loaded 23.431. The switch (Use 21.271 -> both pages loaded 23.508) took 2.24 s, of which about 0.75 s was the over-long wait
  implication: the late exit had no functional consequence beyond the WARN and the extra 0.75 s; the browser was likely gone, or not in the way, by about 22.42-22.68

- timestamp: 2026-11-28T02:43:16.766Z
  checked: probe A (scratch apps/sucrose/tests/zz_exit_probe2.rs, debug build, 2 monitors, Web/Url/YouTube on WebView2, WallpaperVolume 5, the three YouTube themes of target/round-b-lib cycled, 24 Use steps, each close 2.5 s after both pages loaded; target/tmp/portal/exit-probe2-a)
  found: 46 closes: 45 at 182-341 ms (median 267); one at 1115 ms: slot 0 (persistent, new folder of this run, Master) closing Spirit Tree (DtE5Y5VGkjU) 2559 ms after its load, the first close of the run; that start's load took 1975 ms (the slowest of the run). Logged as "exited" (not the WARN): the exit came just after the 1 s deadline and the pump loop saw it on its next turn. Slot 1 closed the same page at the same moment in 293 ms
  implication: reproducible on the debug build without the Portal: about 1 in 46, again slot 0 persistent Master on Spirit Tree after a slow load; not the C2 Portal load

- timestamp: 2026-11-28T02:43:16.766Z
  checked: probe B1 (same, YT -> GIF alternation, --enable-logging --v=1 in WebArguments, chrome_debug.log of both folders saved after every step; exit-probe2-b1/chromelogs)
  found: 6 closes 341-459 ms (logging costs about 0.1 s); first close of the new persistent folder 356 ms although its load took 3146 ms. Normal shutdown in step-01-p.log: close 32.141 -> YouTube sends keepalive beacons (youtubei/v1/log_event, api/stats/qoe, api/stats/watchtime) 32.151-32.167, the audio output stream closes 32.154 (AOC::Close, 1 ms), kBrowserWindow keep-alive removed and "Unloading profile Default" 32.167, "CrxUpdateService stopping" 32.265 and 32.367, exit about 32.50
  implication: a slow load does not make every following close slow, and the first close of a new folder is not slow by itself; the normal tear-down shows unload beacons and an audio stream close at the moment of the close: candidates for what a slow close waits on. A slow close must be caught with the log on

- timestamp: 2026-11-28T02:49:33.090Z
  checked: probe C (YouTube-to-YouTube, 30 steps, Chromium VERBOSE1 log, every chrome_debug.log generation kept; exit-probe2-c/chromegens)
  found: 58 closes 238-533 ms (median 386 with logging), none past 1 s. In every generation: close -> "Unloading profile Default" 10-20 ms, then 190-340 ms to the last line; the Master's audio output stream opens 1.8-2.5 s after the browser start and closes in 1 ms at the close (AOC::Close); YouTube sends its unload beacons (log_event, stats/qoe, stats/watchtime) at every close
  implication: the slow case did not happen with logging on in 58 closes; the normal tear-down has no step that waits on the audio stream or on the beacons

- timestamp: 2026-11-28T02:49:33.090Z
  checked: probe D (fault injection: a page whose main thread runs a 6 s busy loop from 1 s after load, closed 2 s after load; a page that spins 3 s in its pagehide and unload handlers; hello between them)
  found: 18 closes 161-294 ms; the busy and unloadbusy pages close as fast as hello
  implication: the WebView2 close does not wait for the page's renderer (no unload-ack timeout of about 1 s); a busy YouTube main thread at the close cannot explain the late exit

- timestamp: 2026-11-28T02:52:02.588Z
  checked: eight short runs on new folders (exit-probe2-f1..f8: Spirit Tree, Space Train, hello; first close of each folder 2.5 s after both loads; no Chromium log)
  found: f1: slot 1 (DISPLAY9, ephemeral, Master) closing Spirit Tree 2597 ms after a 2577 ms load: WARN "did not exit in time" elapsed_ms=1011; the 31 other closes 256-369 ms. Master Spirit Tree first closes: 1 of 8 late (the one with the slowest load); Follower closes of the same page at the same instant all fast
  implication: not the persistent folder (H-C as "persistent only" is out): all three late closes so far are the Master (unmuted) closing Spirit Tree (DtE5Y5VGkjU) about 2.6 s after a slow (about 2-2.6 s) load; the role and the video, not the folder, go with the slow exit

- timestamp: 2026-11-28T02:55:50.604Z
  checked: short runs with the Chromium log (exit-probe2-g1..g14: Spirit Tree then hello on new folders, VERBOSE1, every log generation kept)
  found: g14: slot 0 (Master) closed Spirit Tree 3164 ms after a 2021 ms load and "webview2 browser process exited" came at elapsed_ms=1031 (27 other closes 325-473 ms). Its log is the same as a fast one: close +13 ms AOC::Close (audio stream closed, 1 ms), +27 ms kBrowserWindow keep-alive removed and "Unloading profile Default", +132 and +318 ms "CrxUpdateService stopping", then no line from any process; the exit event came about 0.7 s after the last line (normal closes: 50-130 ms after it)
  implication: the profile tear-down is not what is slow: the time goes after the browser main thread's last log line. WebView2 raises BrowserProcessExited only when every process of the browser (gpu, utility, renderer) has ended, so a child process that ends late, or a late step of the browser's own process exit, would explain it; the Master-only pattern (4 of 4 late closes are the unmuted Master; muted Followers closing the same page at the same instant are fast) points at the audio service utility process, which only the Master uses

- timestamp: 2026-11-28T03:11:48.604Z
  checked: short runs with a light process poller (pid list every 15-30 ms, command line once per pid; exit-probe2-h2..h13 and k1..k23; target/exit-probe2/procs-*.txt)
  found: late closes caught: h13 (slot 0 Master, 1069 ms), k8 (slot 0 Master, 1094 ms), k23 (slot 1 Master, 772 ms); 33 other closes 174-381 ms. h13: after the close (52.086) the renderer ends at +89 ms, network and audio at +177, storage +206, gpu +238 ms (the same as the other folder's processes), but the browser process (and its crashpad handler, which follows it) ends only at +1082 ms. k23: children gone by +282 ms, browser at about +880 ms; thread samples of that browser after its children were gone: 53 threads, every one waiting (UserRequest or EventPairLow), CPU time of no thread changing between samples 140 ms and 175 ms apart
  implication: the late part is the WebView2 browser process itself, idle (waiting, not computing) for 0.5-0.85 s after every child process ended; neither a child process nor CPU load explains it; H-E as "a child process ends late" is out; the browser waits on something outside its own CPU work (a timed wait, file I/O completion, or a call to another process)

- timestamp: 2026-11-28T03:11:48.604Z
  checked: Windows media sessions (GlobalSystemMediaTransportControlsSessionManager, read-only) during a Master Spirit Tree playback (exit-probe2-s1)
  found: no media session from the wallpaper's WebView2 browser or from sucrose (the only session listed belonged to an unrelated app of the maintainer; not recorded)
  implication: the Master's audible playback does not register with the System Media Transport Controls, so an SMTC tear-down cannot be what the browser waits on

- timestamp: 2026-11-28T03:16:15.383Z
  checked: audio-only fault injection: a local "tone" page (no YouTube, no network: WebAudio oscillator at gain 0.001, started at load), six short runs on new folders with the process poller (exit-probe2-n1..n6)
  found: n6: BOTH slots (Master and the muted Follower) late at the same instant: 1090 and 1091 ms. Poller: both browsers' children (renderer, audio, network, storage, gpu) gone 192 ms after the close; then both browser processes idle (52 and 53 threads, no thread gaining CPU in 22 samples) for about 0.73 s; at the same instant (+923 ms) both wake (one thread Executive, one LpcReply = an ALPC call), shut their threads down (n=1 at +1017 ms) and exit at about +1048 ms. n1..n5: 223-274 ms
  implication: YouTube, video and network are not needed: an audio stream is. The Follower is late with the tone page because WebAudio keeps an output stream even when the WebView is muted, while a muted YouTube video has no stream (Chromium suspends a muted element's audio sink): this explains "Master only". Two separate browsers waking at the same instant after the same idle points at a shared timed or external wait that starts at the close

- timestamp: 2026-11-28T03:20:26.936Z
  checked: tone/hello cycling on used folders (exit-probe2-o1: 15 tone closes per slot, 2.6 s after load, and 15 hello closes)
  found: 60 closes 156-332 ms, none late; tally of audio closes so far: first close of a new folder 7 late of 69 (f 1/8, g 1/14, h 1/12, k 2/23, n 2/12), closes on an already used folder 0 late of about 81 (A, C, o1), plus L5 (used persistent folder, 1 of about 40)
  implication: the late exit needs an audio stream AND (mostly) a profile folder on its first start; the ephemeral folders of non-primary monitors are new at every start, so they hit it most

- timestamp: 2026-11-28T03:20:26.936Z
  checked: per-app audio session store (HKCU ...\Audio\PolicyConfig\PropertyStore, read only) and the profile contents of an audio vs a no-audio folder
  found: the store keys WebView2 audio by host exe (sucrose.exe debug/release) and by msedgewebview2.exe version with the default session GUID, nothing per profile folder; an audio folder and a hello folder hold the same file set (no media device salt database, no extra files)
  implication: no per-folder audio state explains the new-folder factor; the wait is not a Windows per-app session record being created

- timestamp: 2026-11-28T03:24:50.239Z
  checked: A/B on new folders, interleaved: tone then GIF, base switches vs --disable-features=AudioServiceOutOfProcess (exit-probe2-ab1..12 a/b)
  found: base: ab1a both slots at the same time 856 and 872 ms (others 173-287); in-process audio: 24 closes 235-337 ms. Every tone event so far (n6, ab1a) hit BOTH browsers at once, with the same duration
  implication: 1 vs 0 events is not a result on its own; the simultaneous hit of two independent browsers on separate folders again points at a shared, system-wide resource rather than per-folder state. Since the new-folder runs are separated by 10-18 s of silence and the used-folder runs close every 2.5 s, "first audio after an idle period" (an audio device that powers down when idle) is an alternative to "new folder"

- timestamp: 2026-11-28T03:29:38.569Z
  checked: used folder with 20 s of silence before every tone close (exit-probe2-i1: tone 2.5 s, GIF 20 s, 10 cycles)
  found: 20 closes 218-382 ms, none late
  implication: an idle (cold) audio device before the stream is not the trigger; the late exits stay tied to the first start of a folder (used folders: 0 events in 25 tone cycles and about 51 Master YouTube closes; new folders: about 1 in 9-11 runs)

- timestamp: 2026-11-28T03:32:14.487Z
  checked: no-audio switching (hello and hello2, 20 switches, used folders) under a CPU load of 16 busy worker threads (bun.exe, 16 logical CPUs; exit-probe2-l1)
  found: 38 closes, bimodal: 23 at 210-409 ms and 15 at 1006-1084 ms; nothing in between except 404/409
  implication: load does not stretch the exit proportionally: it makes the browser take the same slow path as the audio cases, which ends 1.0-1.1 s after the close. All slow exits measured in this session (about 25: audio on new folders and CPU load) lie in 772-1115 ms. The WebView2 runtime seems to fall back to a fixed wait of about 0.9-1 s in its own shutdown when something it waits for does not arrive in time (a race that load and a new folder with audio make more likely); 1 s sits exactly on that mode

- timestamp: 2026-11-28T03:36:01.271Z
  checked: the same no-audio switching under 32 busy threads (exit-probe2-l2) and the pairing of slow exits by close instant (l1, l2)
  found: l2: 34 closes 213-265 ms, 4 at 1051-1057 ms. Of the 10 slow events under load, 9 hit BOTH browsers at the same close (pairs 1062/1061, 1033/1040, 1045/1042, 1010/1046, 1040/1037, 1050/1006, 1011/1024, 1055/1054, 1057/1051), one hit slot 0 only (1084/258); the tone events (n6, ab1a) were pairs too
  implication: the slow path is a fixed mode of about 1.0-1.1 s (heavier load does not lengthen it: max 1057 ms at 32 threads, 1084 at 16, 1115 with audio) and two independent browsers on separate folders mostly take it together, so it is decided by something shared at the close moment, inside or around the WebView2 runtime, not by Sucrose's close sequence (the same code takes 0.2-0.3 s on the normal path). No Sucrose-side switch or code change that removes it was found

- timestamp: 2026-11-28T03:49:50.678Z
  checked: verification load run with the fix (BROWSER_EXIT_WAIT 1.5 s; exit-probe2-v-l1: hello/hello2, 20 switches, 16 busy bun.exe threads for the whole run, debug build), and the raw lines of the earlier load run l1
  found: v-l1: 38 of 38 closes ended with the WARN "did not exit in time" at elapsed_ms 1501-1559, no "exited" line; Renderer-1 (ephemeral) logged "no web profile folder ... (os error 32)" at 14 of its 19 starts, about 1.05 s after the previous WARN (its emptying retries for up to 1 s), so the old browser still held the folder at least about 2.6 s after the close; steps took 2.1-16.5 s. Re-reading l1: its 15 "slow exits at 1006-1084 ms" were 14 WARN lines at the 1 s bound (elapsed_ms is the deadline plus pump latency, not an exit time) and one "exited" (1084); all came while the burn ran (steps 0-7), and the 23 fast exits came after the 60 s burn had ended
  implication: CORRECTION of the l1/l2 evidence and of the reasoning_checkpoint: CPU saturation does NOT send the browser onto the same fixed ~1.05 s path. Under 16 busy threads on 16 logical CPUs the browser exit takes longer than 2.5 s (CPU starvation; H-A holds for the load case), and no bound under SHUTDOWN_GRACE (2 s) covers it; 1.5 s does not change that case (it logs the WARN 0.5 s later). The audio/new-folder slow path (the L5 symptom) rests on real exit times (poller h13 +1082, k23 +880, n6 +1048 ms; "exited" lines A 1115, g14 1031, ab1a 856/872) and is what the 1.5 s bound is for; the tone runs on new folders must now show it logged as "exited" below 1.5 s

- timestamp: 2026-11-28T03:58:12.557Z
  checked: verification runs with the fix, no load, new folders (exit-probe2-v-t1..t24: tone then GIF; exit-probe2-v-y1..y6: Spirit Tree, Space Train, hello), debug build, both monitors
  found: 72 closes, every one logged "webview2 browser process exited", no WARN, no "no web profile folder"; 69 at 222-328 ms; 3 on the slow path: v-t1 both slots 941 and 958 ms (a pair again), v-y3 slot 1 at 1057 ms (closing YouTube); none near 1.5 s
  implication: the audio/new-folder slow path does not follow the bound (with the bound at 1.5 s it still ends at 0.94-1.06 s), so the host's wait is not what ends it; the 1.5 s bound waits it out: the 1057 ms close is the kind that the 1 s bound cut with the WARN (f1 1011, L5 1052 at HEAD). The original symptom is fixed

- timestamp: 2026-11-28T03:58:12.557Z
  checked: the same no-audio switching under 8 busy threads (half the 16 logical CPUs; exit-probe2-v-l8, 16 switches)
  found: 30 closes at 225-257 ms, no WARN, switches 0.9-1.0 s (as unloaded)
  implication: load is a saturation cliff, not a gradual stretch: with free CPUs the exit is unchanged; only a fully saturated machine (16 threads: every close past 2.5 s, switches 2-16 s) holds the browser, and no bound under SHUTDOWN_GRACE covers that; it stays the logged WARN path (expected degraded behaviour, recorded in D-040 and FEATURES.md)

- timestamp: 2026-11-28T04:04:23.539Z
  checked: adjacent tests with the fix: cargo fmt --all (no change), cargo clippy --workspace --all-targets -D warnings (clean), cargo test --workspace twice and cargo test -p sucrose --test portal three times (all green, no rerun needed); the renderer logs of the two WebView2 portal tests after each run (target/tmp/portal/webview2-early-close, webview2-switch)
  found: a_webview2_renderer_closed_right_after_its_start_on_a_new_profile_exits_at_once: both slots on the slow path in 5 of 5 runs (10 closes, 817-1251 ms, pairs 1146/1110, 1196/1226, 817/878, 1251/1196, 1099/1079), every one logged "exited", no WARN; the close comes about 0.6 s after the renderer's start, before the page loads. switching_web_themes_on_webview2_closes_each_renderer_cleanly: the first close of each slot on the slow path in 3 of 5 runs (902-1085 ms), the second close 222-527 ms
  implication: these are the closes that failed the 1 s check of the portal tests in the full runs of 05:08-05:10 (symptoms): while the parallel test suite runs, a close in the first second of a new folder's start nearly always takes the slow path, and the path stretches to 1.25 s there (1.12 s on an idle machine). The 1.5 s bound covers every slow exit seen (1251 ms the longest, about 0.25 s to spare); the margin under the test suite is smaller than the 0.4 s stated in the checkpoint

## Resolution

root_cause: the WebView2 Runtime's browser process intermittently takes a slower path of its own shutdown (every child process gone by about 0.3 s, then the browser idles until it exits 0.77-1.12 s after the close on an idle machine, up to 1.25 s while the parallel test suite runs), entered with an audio output stream on a profile folder's first start (about 1 close in 10) and, on a busy machine, by a close in the first second of a new folder's start (nearly always while the test suite runs), mostly for both monitors' browsers at once; BROWSER_EXIT_WAIT = 1 s sat on that path, so such closes logged "did not exit in time" (04-39 L5 1052 ms) or failed the portal tests' 1 s check. Not SmartScreen, not Sucrose's close sequence, not page work, not the host's wait (the path does not follow the bound). Separate case, not covered by any bound under SHUTDOWN_GRACE: a fully saturated CPU (16 busy threads on 16 logical CPUs) holds the browser more than 2.5 s
fix: BROWSER_EXIT_WAIT 1 s -> 1.5 s (Duration::from_millis(1500)) in crates/sucrose-engine-webview2/src/host.rs with its comment rewritten from this session's evidence; portal.rs early-close test asserts exits below the bound (BROWSER_EXIT_WAIT_MS = 1500) and keeps the "did not exit in time" and "no web profile folder" checks; D-040 and D-041 amended (2026-11-28), FEATURES.md row 79, MEASUREMENTS.md row (WebView2 browser process exit after a close)
verification:
  target_test: { result: pass, detail: "probe repro with the fix: 72 closes on new folders (tone x24, YouTube x6), 3 on the slow path at 941/958/1057 ms, all logged exited, no WARN, no 'no web profile folder'; portal tests a_webview2_renderer_closed_right_after_its_start_on_a_new_profile_exits_at_once and switching_web_themes_on_webview2_closes_each_renderer_cleanly green in 6 of 6 runs with slow-path closes at 817-1251 ms" }
  mutation_check: { result: pass, reason_if_skipped: "Stryker n/a (Rust); manual fix-site mutant instead", mutant_killed: "BROWSER_EXIT_WAIT back to from_secs(1): the early-close test failed at portal.rs:1068 with WARN elapsed_ms 1019/1014" }
  no_op_deletion: { result: flagged, deletion_justified_by_rca: true, detail: "the test's elapsed check moved from < 1000 to < 1500 to match the bound the maintainer chose; the WARN and missing-folder checks stay and the mutant shows they still bite; no behaviour removed" }
  adjacent_tests: { result: pass, suites_run: ["cargo fmt --all (no change; --check clean)", "cargo clippy --workspace --all-targets -- -D warnings (clean)", "cargo test --workspace x2 (green first time, no flaky rerun needed)", "cargo test -p sucrose --test portal x4 with the fix (green)"] }
  revert_and_reconfirm: { result: pass, bug_returned_on_revert: true, fixed_on_reapply: true, detail: "1 s: early-close test red (WARN 1019/1014); 1.5 s restored with a fresh mtime: portal tests green" }
  load_criterion: { result: fail_expected, detail: "the orchestrator's 'no WARN under 16 busy threads' check rested on the wrong l1/l2 reading; 38/38 closes outlast 1.5 s there (and outlasted 1 s before); 8 threads: no effect. Recorded as expected degraded behaviour in host.rs, D-040, FEATURES.md, MEASUREMENTS.md" }
  guardrail_verdict: accepted
fix_commit: f5d57b98
files_changed: [crates/sucrose-engine-webview2/src/host.rs, apps/sucrose/tests/portal.rs, docs/DECISIONS.md, docs/FEATURES.md, docs/MEASUREMENTS.md]
oracle_type: specified (the bound and the log lines the host writes)
prevention:
  five_whys_code: "why did closes WARN? the 1 s bound sat on the Runtime's slow mode -> why was 1 s chosen? the previous session measured only the fast mode (0.21-0.25 s, idle machine, 26 closes) after removing SmartScreen -> why not the slow mode? it appears in ~1 of 10 audio first closes and under load, which the 26-close sample on an idle machine could not show"
  five_whys_environment: "why did the portal tests fail only in full runs? the parallel test suite is the load that sends an early close onto the slow path (10 of 10 closes here) -> why unnoticed? the failures were attributed to stale artifacts"
  why_not_caught: "the portal test checked < 1000 ms, the same number as the bound, so it failed with the code rather than showing the margin; no test runs under load on purpose, and the slow mode is rare on an idle machine"
  recurrence_guard: "apps/sucrose/tests/portal.rs:a_webview2_renderer_closed_right_after_its_start_on_a_new_profile_exits_at_once (BROWSER_EXIT_WAIT_MS = 1500, no 'did not exit in time' WARN; it took the slow path in 10 of 10 closes during test runs, so it exercises the margin) + KB pattern: a timing bound must be measured against the tail of the distribution under load, and a WARN's elapsed_ms is the deadline, not an exit time"
