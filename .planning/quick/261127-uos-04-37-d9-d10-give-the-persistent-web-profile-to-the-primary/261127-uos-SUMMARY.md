---
phase: quick-261127-uos
plan: 01
subsystem: web-engines
tags: [web-profile, webview2, cef, multi-monitor, d9, d10, d-040, d-041]
status: complete
requires:
  - 04-CONTEXT D-33 and D-038 (profile rule)
provides:
  - "The primary monitor's renderer holds the persistent web profile on CEF and WebView2; every other renderer runs ephemeral and never opens the lock (D9, D-040)"
  - "A WebView2 renderer keeps its profile lease until its browser process exited or a 1 s bound passed; a folder still in use is emptied with retries (D10, D-040)"
affects: [sucrose-engine-web, sucrose-engine-cef, sucrose-engine-webview2, sucrose-ipc, sucrose service]
key-files:
  modified:
    - crates/sucrose-engine-web/src/profile.rs
    - crates/sucrose-engine-web/tests/profile.rs
    - crates/sucrose-ipc/src/protocol.rs
    - apps/sucrose/src/topology.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - crates/sucrose-engine-cef/src/browser.rs
    - crates/sucrose-engine-webview2/src/engine.rs
    - crates/sucrose-engine-webview2/src/host.rs
    - docs/DECISIONS.md
    - docs/FEATURES.md
    - .planning/phases/04-m3-engines-and-the-wallpaper-api/04-CONTEXT.md
decisions:
  - "D-040: the primary monitor keeps the persistent web profile; a WebView2 renderer waits for its browser process (bound 1 s) before it releases its profile"
  - "D-041 (from the debug session webview2-browser-exit-slow): WebView2 runs without Edge SmartScreen by default, which removed the about 6.2 s browser exits"
metrics:
  completed: 2026-11-27
commits: 12
plan_head_before: d1d76bd7e4b4249b095a22fcd519d1e8c97ecc06
plan_head_after: 93e3f099b1bbdc7c1d7eb826d264a7cb143b313f
actuals:
  tokens: 23300
  tasks: 3
  commits: 12
---

# Phase quick-261127-uos Plan 01: D9/D10 persistent web profile on the primary monitor Summary

The persistent web profile now belongs to the primary monitor's renderer on both web engines (every other monitor runs ephemeral and never opens the lock), and a closed WebView2 renderer keeps its profile until its browser process has exited, so quick theme switches on two monitors no longer lose a profile folder.

## What was built

- **Task 1 (bd84d9af, 8668fb8b): D9 end to end.** The service computes `web_profile_owner` (the primary monitor, else the lowest monitor id) and sends `Assignment.web_profile` (PROTOCOL_VERSION bumped for it). `profile::acquire` takes the wanted `ProfileKind`: a renderer told Persistent still has to win the OS lock (bounded wait `START_WAIT` = 1 s, then ephemeral), one told Ephemeral never touches the lock. A monitor snapshot that moves the primary flag rewrites the stored assignments so the next start of each slot follows the new primary. 8668fb8b set the profile owner on the `Use` start path as well (the first version only set it on the snapshot path).
- **Task 2 (c1ac7f05): D10.** The WebView2 host registers `BrowserProcessExited`, closes the controller and pumps messages until the browser process exited (or `BROWSER_EXIT_WAIT`) before the profile lease is dropped; it logs `webview2 browser process exited elapsed_ms=...`. An ephemeral folder still held by a process of the ended engine (os error 32) is emptied with retries for up to `START_WAIT`. Docs: D-040 in docs/DECISIONS.md (D-038 text unchanged), docs/FEATURES.md rows, 04-CONTEXT D-33.
- **Task 3 (maintainer checkpoint, machine part): two-monitor evidence on HEAD.** Done here after the maintainer attached DISPLAY9 (see below). The optional eye check (storage page 1, 2, 3 on DISPLAY1 and 1 on DISPLAY9) is deferred to the coordinator's final maintainer session.
- **Cleanup commit (93e3f099):** two stale "first instance" comments in `crates/sucrose-engine-cef/src/browser.rs` now say the persistent profile belongs to the primary monitor's renderer.

## Measured evidence (HEAD 93e3f099, debug build, DISPLAY1 primary 0,0 1920x1080 + DISPLAY9 1920,0 1920x1080)

**Browser exit times (close to BrowserProcessExited):**

| Build | Exit times |
|---|---|
| c1ac7f05 with the 8 s bound, before e747c7be | 0.20-0.30 s in most closes, 6.2-6.4 s in the closes that landed in the first second of a start on a never-used folder (6186, 6373, 6410, 6385 ms; the debug session found 18 of 20 at 263-304 ms) |
| HEAD (SmartScreen off by default, 1 s bound) | switching test: 281 ms (DISPLAY1, slot 0) and 181 ms (DISPLAY9, slot 1); the debug session's release runs: 20 of 20 closes 212-236 ms |

**2a. WebView2 switching test with two monitors:** `cargo test -p sucrose --test portal switching_web_themes_on_webview2_closes_each_renderer_cleanly` passes (1 passed, 1.61 s; it failed on 2026-11-27 at 12:37:41Z with the slot-1 profile WARN, os error 32). In `Log/Renderer-*.log`: `no web profile folder` count 0 in both, no `did not exit in time`, no `os error 32`; every `kind=Persistent` line names DISPLAY1 (slot 0), every `kind=Ephemeral` line names DISPLAY9 (slot 1); each closed renderer logged `webview2 browser process exited` (elapsed_ms 281 and 181). The second renderer pair of the test was ended by the test's own service kill, so it logs its profile line only.

**2b. D9 on WebView2** (`--engine Web=WebViewLive`, three runs of 15 s, 3 s apart, `target/uos-2mon-wv2`): the service logged `renderers started count=2 monitors=[DISPLAY1, DISPLAY9] primary=DISPLAY1 web_profile_owner=DISPLAY1` in each run.

| Run | DISPLAY1 renderer (slot 0) | DISPLAY9 renderer (slot 1) |
|---|---|---|
| 1 | kind=Persistent, /__storage/1 | kind=Ephemeral, /__storage/1 |
| 2 | kind=Persistent, /__storage/2 | kind=Ephemeral, /__storage/1 |
| 3 | kind=Persistent, /__storage/3 | kind=Ephemeral, /__storage/1 |

**2c. D9 on CEF** (`--engine Web=CefSharpLive`, same runs, `target/uos-2mon-cef`): same service line each run; slot 0 kind=Persistent on DISPLAY1 (path `Web/Cef`), slot 1 kind=Ephemeral on DISPLAY9 (path `Cache/WebEphemeral/Cef/1`) in all three runs. CEF does not log page requests, so the counts come from the profiles' localStorage write-ahead logs: the persistent profile holds the values 1, 2, 3 (three records of `sucrose-fixture-starts`), the ephemeral folder (emptied at every start) holds a single record, 1.

**2d. Cleanup:** every run was stopped with `taskkill.exe /IM sucrose.exe /F` (never msedgewebview2.exe by name); `tasklist.exe` shows no sucrose.exe or sucrose-portal.exe at the end. Logs are in `target/uos-logs/` (wv2-*, cef-*, switch-*). `cargo test --workspace` was not run with DISPLAY9 attached (known an_engine_change failure until 261127-uou).

## Decisions

- **D-040** (docs/DECISIONS.md, Accepted 2026-11-27): the primary monitor keeps the persistent web profile (CEF and WebView2); a WebView2 renderer waits for its browser process, bound 1 s (amended after the debug session), before it releases its profile; D-038 text unchanged.
- **D-041** (docs/DECISIONS.md, from the debug session): WebView2 runs without Edge SmartScreen by default; a user setting to turn it on follows in a separate quick task.

## Deviations from Plan

1. **[Rule 1 - Bug] The `Use` start path did not set the profile owner (8668fb8b).** Found in the one-monitor machine check after task 1: a start through `Use` left `web_profile_owner` unset, so the primary was not told Persistent. Fix: set it on that path too. No unit test was added for it (the path needs a running service); the two-monitor runs above exercise it through the service start (`web_profile_owner=DISPLAY1` logged) and the desktop tests of the switching flow cover `Use`.
2. **[Plan change] The browser exit bound went 1 s, then 8 s, then 1 s again.** c1ac7f05 set `BROWSER_EXIT_WAIT` to 8 s because exits took about 6.2 s in some closes. The maintainer chose to investigate ("araştır") instead of accepting 8 s; the debug session `webview2-browser-exit-slow` found Edge SmartScreen as the cause and e747c7be turned SmartScreen off by default (D-041) and set the bound back to the plan's 1 s. The 8 s bound never ships; D-040 was amended (5cf0d11c).
3. **[Cleanup] Stale "first instance" comments** at `crates/sucrose-engine-cef/src/browser.rs` (the `root_cache_path` doc and the settings comment) reworded in 93e3f099, keeping CRLF.
4. **[Scope note] The eye check of Task 3 is deferred** to the coordinator's final maintainer session; the machine evidence above is complete without it.

## Line-ending check

`git ls-files --eol` on every file of this item after the last commit: CRLF files stay w/crlf (apps/sucrose/src/render.rs, service.rs, topology.rs; crates/sucrose-engine-cef/src/browser.rs and engine.rs; crates/sucrose-engine-web/src/profile.rs and tests/profile.rs; crates/sucrose-ipc/src/protocol.rs; docs/FEATURES.md), LF files stay w/lf (crates/sucrose-engine-webview2/src/engine.rs and host.rs, docs/DECISIONS.md, 04-CONTEXT.md); index is i/lf everywhere. No whole-file diffs.

## Commits

Own commits of this item: bd84d9af (D9), 8668fb8b (Use path), c1ac7f05 (D10), 93e3f099 (comments). The measured `commits: 12` is the ledger count `d1d76bd7..HEAD` on the shared checkout; it also contains the interleaved debug-session commits (e747c7be, 5cf0d11c) and the items 261127-uot (5de6cf69, deb5dca9, e54c0523) and 261127-uov (faf94eb8, e048ba46, a4048b8f).

## Known Stubs

None.

## Threat Flags

None. The threats of the plan (T-uos-01 to T-uos-03, T-uos-05) are covered: the OS lock is still the only way to the persistent folder, the waits are bound at 1 s, the emptied path is unchanged, and no process is killed by name except sucrose.exe and sucrose-portal.exe.

## Self-Check: PASSED

- Commits bd84d9af, 8668fb8b, c1ac7f05, 93e3f099 exist on HEAD.
- Evidence logs exist in target/uos-logs/; tasklist.exe shows no Sucrose process.
