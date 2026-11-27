---
phase: quick-261127-uov
plan: 01
subsystem: web-engines
tags: [user-agent, cef, webview2, ipc, parity]
requires: []
provides:
  - WebOptions.user_agent (service -> assignment -> CEF settings and WebView2 settings)
affects: [apps/sucrose, sucrose-ipc, sucrose-engine-cef, sucrose-engine-webview2, docs/FEATURES.md]
key-files:
  modified:
    - crates/sucrose-ipc/src/protocol.rs
    - apps/sucrose/src/service.rs
    - crates/sucrose-engine-cef/src/browser.rs
    - crates/sucrose-engine-cef/src/engine.rs
    - crates/sucrose-engine-webview2/src/host.rs
    - crates/sucrose-engine-webview2/src/engine.rs
    - docs/FEATURES.md
decisions:
  - "PROTOCOL_VERSION 11: WebOptions gained a field inside Assignment (same rule as 261127-uos)"
  - "A blank or control-character General.UserAgent keeps the engine's own agent; players get none"
  - "Observed precedence of a user-agent switch: CEF switch wins, WebView2 General.UserAgent wins"
status: complete
commits: 3
plan_head_before: 5cf0d11cb9feb5d4d1353a4fba9d209319839427
plan_head_after: a4048b8f65cff1abf288fdb1032dbfa9e5c137c2
actuals:
  tasks: 3
  commits: 3
---

# Phase quick-261127-uov Plan 01: General.UserAgent on both web engines Summary

General.UserAgent now reaches CEF (`Settings.user_agent`) and WebView2 (`ICoreWebView2Settings2::SetUserAgent` before the first navigation) through a new last field `WebOptions.user_agent`, decided once per job by `web_user_agent` in the service; proven with a CDP probe on both real engines.

## Commits

- faf94eb8 feat: CEF end to end (service, wire, CEF settings), PROTOCOL_VERSION 11
- e048ba46 feat: WebView2 SetUserAgent before Navigate
- a4048b8f docs(features): Web and YouTube rows

## Behavior

- `web_user_agent(chosen, configured)`: only Cef and WebView2 get a value, as stored (not trimmed); empty/blank gives ""; any control character gives "" with one WARN carrying only the length. `job_for` and `plan_for` take the value; all five paths with settings in scope pass `&settings.general.user_agent` (selected_job, Use, restart_wallpapers, plan_from, engine-change restart). Fleet already copies `job.web` into every assignment, so a restart carries the value saved now.
- WebView2: `user_agent_override("")` is None so the COM call is skipped; a failed cast or call is a WARN and the page plays with WebView2's own agent.
- Task 1 fallback (user-agent switch in HostApp) was NOT needed: CEF 156 honours `Settings.user_agent`.

## CDP probe output (scratch probe target/uov-check/probe/ua.ts, not committed; one monitor line shown per run, the second page on port 9223 was identical)

The machine had two displays attached during the runs (two renderers, ports 9222 and 9223); both pages gave the same values.

1. Probe value `SucroseProbe/1.0 (uov check)`, hello, CEF and WebView2: navigator.userAgent and request header both equal it; exit 0 on both.
2. 1a legacy default (`Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit`), hello:
   - CEF: {"port":9222,"host":"sucrose.wallpaper.engine","path":"/index.html","navigator":"Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit","header":"Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit"}
   - WebView2: {"port":9222,"host":"sucrose.wallpaper.engine","path":"/index.html","navigator":"Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit","header":"Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit"}
3. 1b YouTube fixture with the default (state 1, time advancing):
   - CEF: {"port":9222,"host":"sucrose.wallpaper.engine","path":"/YouTubeContent.html","navigator":"Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit","header":"Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit","youtube":{"a":{"state":1,"time":25.974344011444092},"b":{"state":1,"time":28.975527963760374}}}
   - WebView2: {"port":9222,"host":"sucrose.wallpaper.engine","path":"/YouTubeContent.html","navigator":"Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit","header":"Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit","youtube":{"a":{"state":1,"time":26.028221186920167},"b":{"state":1,"time":29.031153994277954}}}
4. 1c blank value (engine's own agent: Chrome/, no Sucrose/):
   - CEF: {"port":9222,"host":"sucrose.wallpaper.engine","path":"/index.html","navigator":"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36","header":"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/156.0.0.0 Safari/537.36"}
   - WebView2: {"port":9222,"host":"sucrose.wallpaper.engine","path":"/index.html","navigator":"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/158.0.0.0 Safari/537.36 Edg/158.0.0.0","header":"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/158.0.0.0 Safari/537.36 Edg/158.0.0.0"}
5. 1d precedence (General.UserAgent `SettingProbe/1` plus a `user-agent` switch `SwitchProbe/2` in the legacy default argument list):
   - CEF: {"port":9222,"host":"sucrose.wallpaper.engine","path":"/index.html","navigator":"SwitchProbe/2","header":"SwitchProbe/2"}  (the switch wins)
   - WebView2: {"port":9222,"host":"sucrose.wallpaper.engine","path":"/index.html","navigator":"SettingProbe/1","header":"SettingProbe/1"}  (General.UserAgent wins)

Every run was closed with taskkill.exe /IM sucrose.exe /F; tasklist.exe listed no sucrose.exe afterwards (also at the end of the plan). msedgewebview2.exe processes of other apps were left alone.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical] PROTOCOL_VERSION bumped to 11**
- **Found during:** Task 1
- **Issue:** the plan did not bump the version, but a new field inside `WebOptions` (part of `Assignment`) changes the wire layout; protocol.rs documents the version as bumped for changes appending a variant cannot express (same reasoning as 261127-uos, version 10).
- **Fix:** version 11; the version test now names WebOptions.user_agent.
- **Files modified:** crates/sucrose-ipc/src/protocol.rs
- **Commit:** faf94eb8

**2. [Rule 3 - Blocking] missing import in the WebView2 host**
- **Found during:** Task 2 (compile error); added `user_agent_override` to the `use super::{...}` list of `imp`.
- **Commit:** e048ba46

Otherwise the plan ran as written. TDD note: RED was the compile failure of the new tests (no separate RED commits, since a non-compiling commit would break the build).

## Verification

- cargo test: sucrose-ipc, sucrose-engine-cef user_agent, sucrose-engine-webview2 user_agent, sucrose --bin sucrose (402 passed, including the new user_agent tests and the restart test) pass.
- cargo fmt --all --check and cargo clippy --workspace --all-targets -- -D warnings pass.
- cargo test --workspace: everything passes except apps/sucrose/tests/portal.rs `an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen` ("the second renderer plays on WebView2"). The machine had two displays attached (surface parent 3840x1080, WMI shows 2 monitors), which is the known multi-monitor failure of those desktop tests until 261127-uou lands; it is a second-renderer engine assertion unrelated to the user agent.

## Line endings

Before and after (git ls-files --eol): protocol.rs, service.rs, browser.rs (cef), engine.rs (cef), FEATURES.md w/crlf (restored after editing in LF; FEATURES.md still has no final newline, last byte "."); webview2 host.rs and engine.rs w/lf. cargo fmt --check passes on the CRLF files.

## Measurements

No docs/MEASUREMENTS.md row: the change sets one string when an engine starts. No DECISIONS.md entry (parity port; the blank and control-character rule is in FEATURES.md).

## Known Stubs

None.

## Threat Flags

None (T-uov-01/02 mitigated: control-character values are refused and only their length is logged; applied values are logged with Debug formatting).

## Self-Check: PASSED

All seven files exist, commits faf94eb8, e048ba46 and a4048b8f are ancestors of HEAD.
