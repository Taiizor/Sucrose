---
phase: quick-261127-uov
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/sucrose-ipc/src/protocol.rs
  - apps/sucrose/src/service.rs
  - crates/sucrose-engine-cef/src/browser.rs
  - crates/sucrose-engine-cef/src/engine.rs
  - crates/sucrose-engine-webview2/src/host.rs
  - crates/sucrose-engine-webview2/src/engine.rs
  - docs/FEATURES.md
autonomous: true

must_haves:
  truths:
    - "A Web wallpaper on CEF started with General.UserAgent `SucroseProbe/1.0 (uov check)` reports exactly that text as navigator.userAgent and sends it as the User-Agent header of its requests (CDP probe on the renderer's debugging port)"
    - "The same holds for a Web wallpaper on WebView2"
    - "With the migrated default General.UserAgent both engines report `Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit` (legacy parity: CefSettings.UserAgent and CoreWebView2.Settings.UserAgent), and the YouTube fixture still reaches the playing state on both engines"
    - "An empty or whitespace-only General.UserAgent, or one with a control character, leaves each engine's own user agent (navigator.userAgent contains Chrome/ and no Sucrose/); the control-character case logs one WARN that names only the length of the value"
    - "A player job (mpv, GIF) carries no user agent, and a restart (Restart now, an engine change) hands the user agent saved now to the assignment of every slot"
    - "docs/FEATURES.md no longer lists the user agent as missing in the YouTube row, and the Web row records how General.UserAgent is applied, the blank and control-character rule, and the observed precedence of a user `user-agent` switch; every edited file keeps the line endings it had before the edit"
  artifacts:
    - path: crates/sucrose-ipc/src/protocol.rs
      provides: "WebOptions.user_agent, the last field, empty by default"
      contains: "pub user_agent: String"
    - path: apps/sucrose/src/service.rs
      provides: "web_user_agent decision; job_for and plan_for carry General.UserAgent into WebOptions"
      contains: "fn web_user_agent"
    - path: crates/sucrose-engine-cef/src/browser.rs
      provides: "CefStart.user_agent and SettingsValues.user_agent into the CEF Settings.user_agent"
      contains: "user_agent: CefString::from"
    - path: crates/sucrose-engine-webview2/src/host.rs
      provides: "HostStart.user_agent applied with ICoreWebView2Settings2::SetUserAgent before Navigate"
      contains: "SetUserAgent"
    - path: docs/FEATURES.md
      provides: "Web and YouTube rows updated for the user agent"
      contains: "261127-uov"
  key_links:
    - from: apps/sucrose/src/service.rs
      to: crates/sucrose-ipc/src/protocol.rs
      via: "job_for sets WebOptions.user_agent from settings.general.user_agent through web_user_agent"
      pattern: "settings.general.user_agent"
    - from: crates/sucrose-engine-cef/src/engine.rs
      to: crates/sucrose-engine-cef/src/browser.rs
      via: "CefStart.user_agent -> settings_values -> Settings.user_agent before cef::initialize"
      pattern: "user_agent: web.user_agent.clone()"
    - from: crates/sucrose-engine-webview2/src/engine.rs
      to: crates/sucrose-engine-webview2/src/host.rs
      via: "HostStart.user_agent -> ICoreWebView2Settings2::SetUserAgent before webview.Navigate"
      pattern: "user_agent: web.user_agent.clone()"
---

<objective>
Apply the `General.UserAgent` setting on both web engines, as the legacy C# app does: CefSharp sets `CefSettings.UserAgent = General.UserAgent` (`src/Live/Sucrose.Live.CefSharp/App.xaml.cs` line 335) and WebView2 sets `CoreWebView2.Settings.UserAgent = General.UserAgent` in every initialization handler (`src/Shared/Engine/Sucrose.Shared.Engine.WebView/Event/Web.cs` line 119, and Gif, Video, YouTube, Url). The setting is already migrated (`settings.general.user_agent`, default `sucrose_config::model::DEFAULT_USER_AGENT`), but no web engine reads it; the docs/FEATURES.md YouTube row lists it under "Not yet" and STATE.md Pending Todos (d) asks for it.

Purpose: parity (CLAUDE.md non-negotiable 1). Web themes and sites that look at the user agent see what they saw under the C# app.
Output: the user agent travels service -> `WebOptions` -> CEF settings and WebView2 settings, with tests, CDP evidence on both engines, and the FEATURES rows updated.

Discretion choices (no CONTEXT.md for this item; recorded here and in FEATURES): the service decides the value once per job; an empty or blank value, or one with a control character, means "the engine's own user agent" (the control-character case warns with the length only); player jobs carry none; a failed WebView2 SetUserAgent is a WARN, not a start failure.
</objective>

<execution_context>
@~/.claude/gsd-core/workflows/execute-plan.md
@~/.claude/gsd-core/templates/summary.md
</execution_context>

<context>
@CLAUDE.md
@.planning/STATE.md
@.claude/rules/rust.md
@.claude/rules/platform-ffi.md
@.claude/rules/docs.md
@docs/legacy/engines.md
@crates/sucrose-ipc/src/protocol.rs
@apps/sucrose/src/service.rs
@crates/sucrose-engine-cef/src/browser.rs
@crates/sucrose-engine-cef/src/engine.rs
@crates/sucrose-engine-webview2/src/host.rs
@crates/sucrose-engine-webview2/src/engine.rs

Read only the parts named below; the files are large (service.rs 6859 lines, browser.rs 1526, host.rs 1342).

- docs/FEATURES.md: line 15 (the "Web (local HTML)" row, it ends with "WebView2 never took the focus. | 🟨 |") and line 17 (the YouTube row). The file is CRLF and has no final newline.
- docs/legacy/engines.md: line 429 (CEF settings, `UserAgent = General.UserAgent`) and lines 433-436 (WebView2 environment, "UserAgent overridden").
- The CDP recipe that worked in this repository: .planning/debug/resolved/webview2-certs-video-input.md line 60 (scratch data dir from `cargo xtask synth-library --web-fixtures`, DeveloperPort 9222, CDP from bun.exe on Windows because WSL cannot reach 127.0.0.1:9222) and .planning/debug/resolved/keyboard-input-doubled.md line 75 (`sucrose.exe --no-portal`, taskkill, tasklist.exe empty afterwards).

<interfaces>
Current shapes the executor needs (line numbers at d1d76bd7):

crates/sucrose-ipc/src/protocol.rs (w/crlf), sent with postcard (positional), the service and the renderer are the same binary:
- line 175: pub struct WebOptions { allow_invalid_certificates: bool, arguments: Vec<Switch>, hardware_acceleration: bool, developer_mode: bool, developer_port: u16, shuffle: bool, theme_type: String }
- line 196: impl Default for WebOptions (theme_type "Dark", shuffle true)
- line 420: test a_web_assignment_survives_the_wire builds WebOptions as a full literal (no struct update), so it must name the new field

apps/sucrose/src/service.rs (w/crlf; binary crate `sucrose`, unit tests run with cargo test -p sucrose --bin sucrose):
- line 4009: fn plan_for(location: &Path, selected: &str, engine: &Engine, pause: PauseKind, theme_mode: ThemeMode, engines: &EngineReport) -> Result<Job, String>, ends with job_for(&theme, engine, pause, theme_mode, engines)
- line 4122: fn job_for(theme: &SelectedTheme, engine: &Engine, pause: PauseKind, theme_mode: ThemeMode, engines: &EngineReport) -> Result<Job, String>; its WebOptions literal is WebOptions { theme_type: legacy_theme_type(theme_mode).to_owned(), ..web_options(engine, chosen) } with chosen: EngineId
- line 4184: fn web_options(engine: &Engine, chosen: EngineId) -> WebOptions (full literal at 4192)
- callers that have `settings` in scope: selected_job (job_for, ~687), the Use path (job_for, ~3533), restart_wallpapers (plan_for, ~3705), plan_from (plan_for, ~3944), the engine-change restart (plan_for, ~4081); test caller url_is_playable_and_its_job_carries_the_address_as_the_source (job_for, ~6117)
- line 6535: test a_restart_hands_the_new_engine_and_web_options_to_the_assignments_of_every_slot (sets job.web fields, checks the assignment)
- Fleet::adopt_job already copies job.web into every assignment (assignment.web.clone_from(&job.web)), so a restart carries the new field without a change there

crates/sucrose-engine-cef/src/browser.rs (w/crlf):
- line 53: pub struct CefStart { log_file, subprocess_path, wake, on_loaded, page_calls, roots, profile, allow_invalid_certificates, arguments, sucrose, developer_mode }
- line 130: pub struct SettingsValues { root_cache_path, cache_path, ignore_certificate_errors, user_data_dir }
- line 147: pub fn settings_values(profile: Option<&Path>, allow_invalid_certificates: bool) -> Result<SettingsValues, CefError>; three test callers (certificate test, user-data-dir test)
- line 899: imp CefHost::start builds cef::Settings { no_sandbox, external_message_pump, ..., root_cache_path, cache_path, persist_session_cookies: 1, remote_debugging_port, ..Settings::default() } before cef::initialize; the "chromium switches" tracing::info! ends at line 891
- cef 156.7.0+156.0.31 Settings has `user_agent: CefString` (an empty value is CEF's default user agent, CEF docs: also configurable with the "user-agent" switch)

crates/sucrose-engine-cef/src/engine.rs (w/crlf): line 214 `let start = CefStart { ... }` in pub fn start, with `let web = &assignment.web;`

crates/sucrose-engine-webview2/src/host.rs (w/lf):
- line 24: pub struct HostStart { roots, user_data_folder, browser_arguments, developer_mode, allow_invalid_certificates, page_calls, on_loaded }
- line 243: the webview2_com import list (ICoreWebView2Settings3 is imported at line 254)
- line 712: imp Host::open: settings = webview.Settings(), the Set* calls, the ICoreWebView2Settings3 block (line 771), then the accelerator, certificate, resource, navigation and download handlers, and last webview.Navigate (line 925); all inside one unsafe block with a SAFETY comment
- webview2-com 0.39.1: ICoreWebView2Settings2::SetUserAgent(&self, value: impl Param<PCWSTR>) -> windows_core::Result<()>; the imp module has fn wide(text: &str) -> Vec<u16>

crates/sucrose-engine-webview2/src/engine.rs (w/lf): line 256 `let host_start = HostStart { ... }`, the "chromium switches" tracing::info! ending at line 236
</interfaces>
</context>

<tasks>

<task type="tracer" tdd="true">
  <name>Task 1: General.UserAgent reaches a CEF wallpaper end to end (service, wire, CEF settings)</name>
  <files>crates/sucrose-ipc/src/protocol.rs, apps/sucrose/src/service.rs, crates/sucrose-engine-cef/src/browser.rs, crates/sucrose-engine-cef/src/engine.rs</files>
  <precondition>The CEF dist is fetched for the target dir in use (the main checkout has target/release/libcef.dll); in a fresh worktree a build that stops with "CEF is not fetched" needs cargo xtask fetch-engines --engine cef (pinned and hash-verified, D-028) before going on. The legacy C# Sucrose is not running (if it is, stop and ask; never close it yourself). One monitor (DISPLAY9 detached).</precondition>
  <behavior>
    - web_user_agent(EngineId::Cef, DEFAULT_USER_AGENT) is DEFAULT_USER_AGENT; web_user_agent(EngineId::WebView2, "My Agent 1.0") is "My Agent 1.0", exactly as stored (not trimmed)
    - web_user_agent(EngineId::Cef, "") and web_user_agent(EngineId::Cef, "   ") are "" (the engine's own)
    - "Bad\r\nX-Injected: 1", "A\u{0}B" and "tab\tin" give "" (any char::is_control)
    - web_user_agent(EngineId::Mpv, DEFAULT_USER_AGENT) is "" (a player has no user agent; the same for the GIF engine)
    - job_for on the Url theme of the existing Url test with the user agent "Probe/1" gives job.web.user_agent == "Probe/1"
    - the restart test with job.web.user_agent "Probe/1" finds "Probe/1" in the slot's assignment
    - a Web Assignment whose WebOptions has a user agent survives postcard unchanged
    - settings_values(Some(folder), false, "Probe/1").user_agent is "Probe/1"; settings_values(None, false, "").user_agent is ""
  </behavior>
  <action>
Write the tests of the behavior block first and watch them fail (RED), then implement (GREEN). Before the first edit, note `git ls-files --eol` of the four files (all w/crlf in the main checkout).

1. crates/sucrose-ipc/src/protocol.rs: add `pub user_agent: String` as the LAST field of WebOptions (postcard is positional; appending leaves the other fields where they are). Doc comment: General.UserAgent for the web engines, the User-Agent header and navigator.userAgent of every page and frame, read when the job is made, so a change applies at the next start; empty keeps the engine's own (legacy: CefSettings.UserAgent and CoreWebView2.Settings.UserAgent). Default gives String::new(); add "no user agent of its own" to the Default doc comment. In a_web_assignment_survives_the_wire set user_agent to the legacy default text written as a literal: sucrose-ipc does not depend on sucrose-config and must not start to.

2. apps/sucrose/src/service.rs: add fn web_user_agent(chosen: EngineId, configured: &str) -> String next to web_options, with a doc comment. EngineId::Cef and EngineId::WebView2 get the value as stored; every other engine gets an empty string. An empty or whitespace-only value gives an empty string, which both engines read as their own user agent (an empty CefSettings.UserAgent and an empty WebView2 put_UserAgent did the same in the legacy app). A value with any control character (char::is_control, so CR, LF, NUL and tab) gives an empty string and one tracing::warn! with the field len = configured.len() and the message "the user agent has a control character, the web engines keep their own"; never log the refused text itself (T-uov-01, T-uov-02). Add a user_agent: &str parameter to job_for right after theme_mode and set user_agent: web_user_agent(chosen, user_agent) in its WebOptions literal next to theme_type. Give the full literal in web_options user_agent: String::new() (job_for fills it, as it does theme_type). Add the same parameter to plan_for right after theme_mode (seven parameters, within clippy's too_many_arguments limit) and pass it on to job_for. Pass &settings.general.user_agent at the five callers that have settings (selected_job, the Use path, restart_wallpapers, plan_from, the engine-change restart). The existing Url test passes "Probe/1" and asserts it on the job. Add the behavior tests with names that contain user_agent (for example web_jobs_carry_the_general_user_agent_and_player_jobs_none, a_blank_user_agent_or_one_with_a_control_character_leaves_the_engines_own, a_url_job_carries_the_user_agent_of_the_general_section), import DEFAULT_USER_AGENT from sucrose_config::model in the test module, and extend a_restart_hands_the_new_engine_and_web_options_to_the_assignments_of_every_slot with job.web.user_agent and an assertion on assignment.web.user_agent. Add General.UserAgent to the doc comments of job_for and plan_for where they list what a job reads. No unwrap or expect outside tests.

3. crates/sucrose-engine-cef/src/browser.rs: CefStart gets pub user_agent: String (doc: General.UserAgent as the service decided it; empty keeps CEF's own). SettingsValues gets pub user_agent: String (doc: the user agent CEF reports, empty for CEF's own); settings_values gets a third parameter user_agent: &str and copies it, so the CEF settings values stay testable in one place (the decision itself is the service's). Update the existing settings_values test calls and add the_user_agent_of_the_job_is_what_cef_gets_and_empty_keeps_cefs_own. In imp CefHost::start pass start.user_agent to settings_values, set user_agent: CefString::from(values.user_agent.as_str()) in the Settings literal (the legacy CefSettings.UserAgent; an empty CefString is CEF's default user agent), and add user_agent = ?values.user_agent to the "chromium switches" info line (Debug formatting escapes anything unusual). Do not touch HostApp or the user's switches: D-17 passes them as before, a user-agent switch included.

4. crates/sucrose-engine-cef/src/engine.rs: add user_agent: web.user_agent.clone() to the CefStart literal in start.

5. Line endings: after the edits and after any cargo fmt, convert only these four files back to the endings noted before the edit (perl or a Bun one-off; the REPL's file reads drop CR and cargo fmt through the wrapper writes LF), then check git ls-files --eol again.

6. End-to-end check on CEF (desktop run rules of CLAUDE.md; never synthesize input):
   a. Stop any sucrose.exe or sucrose-portal.exe you started (Windows locks running executables), then cargo build --release -p sucrose.
   b. cargo xtask synth-library --out target/uov-check/cef --count 1 --web-fixtures --select hello --engine Web=CefSharpLive --engine Url=CefSharpLive --engine YouTube=CefSharpLive --force. In its Setting/Sucrose.json set General.UserAgent to "SucroseProbe/1.0 (uov check)" and Engine.DeveloperPort to 9222 (DeveloperMode stays off, so no DevTools window opens). Edit the JSON with a script (Bun or python3), not by hand.
   c. Write the CDP probe target/uov-check/probe/ua.ts (scratch under target/, never committed; Bun built-ins fetch and WebSocket only, no package install). It waits up to 30 s for a target of type "page" on 127.0.0.1 ports 9222 to 9229 (/json/list), and for each page: Runtime.evaluate of navigator.userAgent with returnByValue; Network.enable, then Runtime.evaluate with awaitPromise of a fetch of location.href with cache "no-store", reading that request's User-Agent from Network.requestWillBeSent or, when it is not there, Network.requestWillBeSentExtraInfo. Flags: --expect TEXT exits 1 unless every page's navigator.userAgent and header equal TEXT; --engine-own exits 1 unless the value contains "Chrome/" and does not contain "Sucrose/"; --youtube also evaluates player.getPlayerState() and player.getCurrentTime() twice, 3 s apart, and exits 1 unless the state is 1 and the time advanced. It prints one JSON line per page (port, url host and path, the values) and never sends Input.* or changes the page beyond that one fetch.
   d. From WSL: SUCROSE_DATA_DIR=<absolute path of target/uov-check/cef> WSLENV=SUCROSE_DATA_DIR/p target/release/sucrose.exe --no-portal, started in the background; check the renderer log for the "chromium switches" line with the probe user agent; run bun.exe target/uov-check/probe/ua.ts --expect "SucroseProbe/1.0 (uov check)" (bun.exe, on Windows: WSL cannot reach the debugging port); then taskkill.exe /IM sucrose.exe /F (never kill msedgewebview2.exe by name) and check that tasklist.exe lists no sucrose.exe.

Fallback, only when step 6 shows CEF's own user agent while the log line shows the probe value (CEF 156 would then ignore Settings.user_agent): keep the Settings field, and in HostApp's on_before_command_line_processing for the browser process append the user-agent switch with the value when it is not empty and the command line has no user-agent switch yet (the pattern of USER_DATA_DIR_SWITCH: a documented pub const for the name and a has_switch check); rebuild, probe again, and record the fallback in the SUMMARY.
  </action>
  <verify>
    <automated>cargo test -p sucrose-ipc a_web_assignment_survives_the_wire && cargo test -p sucrose-engine-cef user_agent && cargo test -p sucrose --bin sucrose -- user_agent a_restart_hands_the_new_engine_and_web_options && test "$(grep -c -F 'pub user_agent: String' crates/sucrose-ipc/src/protocol.rs)" -ge 1 && test "$(grep -c -F 'settings.general.user_agent' apps/sucrose/src/service.rs)" -ge 5 && test "$(grep -c -F 'user_agent: CefString::from' crates/sucrose-engine-cef/src/browser.rs)" -ge 1 && test "$(grep -c -F 'user_agent: web.user_agent.clone()' crates/sucrose-engine-cef/src/engine.rs)" -ge 1 && bun.exe target/uov-check/probe/ua.ts --expect "SucroseProbe/1.0 (uov check)"</automated>
  </verify>
  <acceptance_criteria>
    - grep -c -F 'pub user_agent: String' crates/sucrose-ipc/src/protocol.rs [HEAD=0 EXPECT>=1]
    - grep -c -F 'settings.general.user_agent' apps/sucrose/src/service.rs [HEAD=0 EXPECT>=5]
    - grep -c -F 'fn web_user_agent' apps/sucrose/src/service.rs [HEAD=0 EXPECT>=1]
    - grep -c -F 'user_agent: CefString::from' crates/sucrose-engine-cef/src/browser.rs [HEAD=0 EXPECT>=1]
    - grep -c -F 'user_agent: web.user_agent.clone()' crates/sucrose-engine-cef/src/engine.rs [HEAD=0 EXPECT>=1]
    - git ls-files --eol crates/sucrose-ipc/src/protocol.rs apps/sucrose/src/service.rs crates/sucrose-engine-cef/src/browser.rs crates/sucrose-engine-cef/src/engine.rs shows the w/ state noted before the edit for each file [HEAD: 4 x w/crlf in the main checkout]
    - The CEF probe prints navigator.userAgent and the request header "SucroseProbe/1.0 (uov check)" and exits 0; tasklist.exe lists no sucrose.exe afterwards
  </acceptance_criteria>
  <done>The new tests pass; a CEF Web wallpaper started from the scratch settings reports the configured user agent in navigator.userAgent and in its request header, proven by the CDP probe, and the run was closed.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: WebView2 sets the same user agent before its first navigation</name>
  <files>crates/sucrose-engine-webview2/src/host.rs, crates/sucrose-engine-webview2/src/engine.rs</files>
  <precondition>Task 1 is committed (WebOptions.user_agent exists) and the probe target/uov-check/probe/ua.ts from Task 1 is on disk; the WebView2 Runtime is installed (user prerequisite).</precondition>
  <behavior>
    - user_agent_override("") is None (the call is skipped and WebView2 keeps its own)
    - user_agent_override("SucroseProbe/1.0 (uov check)") is Some with the same text, never trimmed or changed
  </behavior>
  <action>
Write the behavior test first (RED), then implement (GREEN). Note git ls-files --eol of both files before the first edit (w/lf in the main checkout).

1. crates/sucrose-engine-webview2/src/host.rs: HostStart gets pub user_agent: String (doc: General.UserAgent as the service decided it; empty keeps WebView2's own). Add #[must_use] pub fn user_agent_override(user_agent: &str) -> Option<&str> in the cross-platform part of the file, with a doc comment: None for an empty value, so the COM call is skipped instead of relying on put_UserAgent's empty-string rule, otherwise the value as given (the service already refused blank and control-character values). Add the test an_empty_user_agent_keeps_webview2s_own_and_a_value_is_set_as_given to the existing tests module. In imp Host::open, right after the ICoreWebView2Settings3 block and before the handler registrations and the Navigate call: when user_agent_override(&start.user_agent) gives Some(agent), cast settings to ICoreWebView2Settings2 (add it to the webview2_com::Microsoft::Web::WebView2::Win32 import list next to ICoreWebView2Settings3) and call SetUserAgent with a PCWSTR over wide(agent), keeping the buffer in a local that outlives the call. Ok logs tracing::info!(user_agent = ?agent, "user agent"); a failed cast or call logs tracing::warn!(%error, "cannot set the user agent, the page gets WebView2's own") and the start goes on, so the wallpaper still plays. The legacy app set it in the initialization handler of every page type (Web, Gif, Video, YouTube, Url); the Rust host is shared by all of them, so this one call covers them all. The call sits inside the existing unsafe block; extend its SAFETY comment to say the UTF-16 buffer outlives the call. The non-Windows stub needs no change.

2. crates/sucrose-engine-webview2/src/engine.rs: add user_agent: web.user_agent.clone() to the HostStart literal in start and user_agent = ?web.user_agent to the "chromium switches" info line.

3. Keep both files on the line endings noted before the edit.

4. End-to-end check on WebView2, as Task 1 step 6 with these differences: stop sucrose.exe first, rebuild with cargo build --release -p sucrose; the scratch folder is target/uov-check/webview2 from cargo xtask synth-library --out target/uov-check/webview2 --count 1 --web-fixtures --select hello --engine Web=WebViewLive --engine Url=WebViewLive --engine YouTube=WebViewLive --force, with the same two Sucrose.json edits (General.UserAgent "SucroseProbe/1.0 (uov check)", Engine.DeveloperPort 9222); the renderer log shows the "user agent" info line; bun.exe target/uov-check/probe/ua.ts --expect "SucroseProbe/1.0 (uov check)" exits 0; close with taskkill.exe /IM sucrose.exe /F and check tasklist.exe.
  </action>
  <verify>
    <automated>cargo test -p sucrose-engine-webview2 user_agent && test "$(grep -c -F 'SetUserAgent' crates/sucrose-engine-webview2/src/host.rs)" -ge 1 && test "$(grep -c -F 'ICoreWebView2Settings2' crates/sucrose-engine-webview2/src/host.rs)" -ge 2 && test "$(grep -c -F 'user_agent: web.user_agent.clone()' crates/sucrose-engine-webview2/src/engine.rs)" -ge 1 && bun.exe target/uov-check/probe/ua.ts --expect "SucroseProbe/1.0 (uov check)"</automated>
  </verify>
  <acceptance_criteria>
    - grep -c -F 'SetUserAgent' crates/sucrose-engine-webview2/src/host.rs [HEAD=0 EXPECT>=1]
    - grep -c -F 'ICoreWebView2Settings2' crates/sucrose-engine-webview2/src/host.rs [HEAD=0 EXPECT>=2] (import and cast)
    - grep -c -F 'fn user_agent_override' crates/sucrose-engine-webview2/src/host.rs [HEAD=0 EXPECT>=1]
    - grep -c -F 'user_agent: web.user_agent.clone()' crates/sucrose-engine-webview2/src/engine.rs [HEAD=0 EXPECT>=1]
    - git ls-files --eol of both files shows the w/ state noted before the edit [HEAD: 2 x w/lf in the main checkout]
    - The WebView2 probe prints navigator.userAgent and the request header "SucroseProbe/1.0 (uov check)" and exits 0; tasklist.exe lists no sucrose.exe afterwards
  </acceptance_criteria>
  <done>The test passes; a WebView2 Web wallpaper reports the configured user agent in navigator.userAgent and in its request header, proven by the CDP probe, and the run was closed.</done>
</task>

<task type="auto">
  <name>Task 3: Legacy default, YouTube, blank value and switch precedence on both engines; FEATURES rows; full checks</name>
  <files>docs/FEATURES.md</files>
  <precondition>Tasks 1 and 2 are committed; the release build and both scratch folders (target/uov-check/cef, target/uov-check/webview2) and the probe exist; network access for the YouTube fixture.</precondition>
  <action>
1. Desktop matrix with the release build of Task 2, one engine at a time, each run started and closed as in Task 1 step 6 (edit Sucrose.json only while sucrose.exe is stopped). Run each item on CEF (target/uov-check/cef) and on WebView2 (target/uov-check/webview2):
   a. Legacy default: General.UserAgent set to "Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit" (DEFAULT_USER_AGENT), hello selected; probe --expect with that text.
   b. YouTube: Library.Selected "youtube" (the fixture plays the Showcase video), the same default user agent; probe --youtube --expect with the default text. If an engine does not reach state 1 with the time advancing, record the states and the probe output in the SUMMARY, do not change the user agent rule, and stop the plan there with that evidence: the legacy app plays YouTube with this agent, so a refusal is a finding for the maintainer, not something to work around in this item.
   c. Blank: hello selected, General.UserAgent "" ; probe --engine-own.
   d. Precedence, for the docs only: hello selected, General.UserAgent "SettingProbe/1"; on CEF Engine.CefArguments is the legacy default map (args::legacy_defaults_cef in crates/sucrose-engine-web/src/args.rs) plus "user-agent": "SwitchProbe/2"; on WebView2 Engine.WebArguments is the legacy default list (args::legacy_defaults_web) plus "--user-agent=SwitchProbe/2"; probe without --expect and record which value each engine reports. Keep whatever happens: the legacy CefSharp and WebView2 hosts combined the same two mechanisms, so the result is parity either way and is only written down.
   Put the probe's JSON lines (values, hosts, paths; nothing personal) in the SUMMARY. Close everything you started; tasklist.exe lists no sucrose.exe at the end.

2. docs/FEATURES.md (CRLF, no final newline; change only lines 15 and 17, keep both as single table rows):
   - Web (local HTML) row: insert one paragraph just before the row's closing status cell (after "WebView2 never took the focus."): "User agent (261127-uov, 2026-11-27): General.UserAgent (legacy default `Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit`) is applied on both engines as in the legacy app (CefSettings.UserAgent, CoreWebView2.Settings.UserAgent)", then: CEF gets it through its user_agent setting (name the switch too if the Task 1 fallback was needed), WebView2 through ICoreWebView2Settings2::SetUserAgent before the first navigation; it is the User-Agent header and navigator.userAgent of every page and frame (Web, URL, YouTube and the generated Gif and Video pages); it is read when the job is made, so a change applies at the next start (Restart now, the next Use); new: an empty or blank value, or one with a control character, keeps the engine's own user agent (the control-character case with a WARN that gives only the length; the legacy app passed such a value on), a failed SetUserAgent is a WARN and the page plays with WebView2's own, and players get none; with a user-agent switch in the user's CefArguments or WebArguments the observed result of step 1d per engine; checked through CDP on both engines (the probe value, the legacy default, a blank value).
   - YouTube row: in the sentence about General.UserAgent, replace the clause that says the Rust engines do not use it with: both engines apply it since 261127-uov (see the Web row), and the YouTube fixture plays with it on both engines (CDP, 2026-11-27); keep the parenthetical that it does not decide whether YouTube serves the control layer. In the row's closing "Not yet" list remove the user agent item so two items remain (clock following of two monitors, process-level pause). The status stays 🟨.
   - Restore CRLF and the missing final newline exactly as before (check git ls-files --eol and the last byte).

3. No docs/MEASUREMENTS.md row: the change sets one string when an engine starts; say so in the SUMMARY. No docs/DECISIONS.md entry: this is a parity port and the new blank and control-character rule is recorded in FEATURES. Leave .planning/STATE.md (Pending Todos (d)) to the orchestrator. docs/ARCHITECTURE.md does not describe the web options, so it needs no change.

4. Full checks: cargo fmt --all, then restore CRLF on any CRLF file it rewrote to LF (the four Rust files of Task 1); cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace with one monitor (with DISPLAY9 attached two apps/sucrose/tests/portal.rs desktop tests fail until item 261127-uou lands; that is not this item's failure, note it if it happens). Finally git ls-files --eol for all seven files matches the states noted before the edits.
  </action>
  <verify>
    <automated>test "$(grep -c -F 'the Rust web engines do not apply it yet' docs/FEATURES.md)" -eq 0 && test "$(grep -c -F 'process-level pause (Round D), the user agent setting' docs/FEATURES.md)" -eq 0 && test "$(grep -c -F 'General.UserAgent' docs/FEATURES.md)" -ge 2 && test "$(grep -c -F '261127-uov' docs/FEATURES.md)" -ge 2 && test "$(tail -c 1 docs/FEATURES.md)" != "" && git ls-files --eol docs/FEATURES.md > target/uov-check/features-eol.txt && grep -q 'w/crlf' target/uov-check/features-eol.txt && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace</automated>
  </verify>
  <acceptance_criteria>
    - grep -c -F 'the Rust web engines do not apply it yet' docs/FEATURES.md [HEAD=1 EXPECT==0]
    - grep -c -F 'process-level pause (Round D), the user agent setting' docs/FEATURES.md [HEAD=1 EXPECT==0]
    - grep -c -F 'General.UserAgent' docs/FEATURES.md [HEAD=1 EXPECT>=2] (lines: the Web row and the YouTube row)
    - grep -c -F '261127-uov' docs/FEATURES.md [HEAD=0 EXPECT>=2]
    - w/crlf in the output of git ls-files --eol docs/FEATURES.md [HEAD=present EXPECT==present GUARD MUTATION: cargo fmt through the wrapper or a REPL write that saves the file with LF endings], and the file still ends without a newline (last byte "." at HEAD)
    - The SUMMARY holds the probe output of 1a to 1d for both engines: the legacy default text on both, the YouTube state 1 with the time advancing on both, an engine-own value (Chrome/, no Sucrose/) for the blank case on both, and the precedence result per engine
    - cargo fmt --all --check, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace pass
  </acceptance_criteria>
  <done>Both engines are proven to send the legacy default, a custom value and their own agent for a blank value, YouTube plays with the default on both, the switch precedence is recorded, the FEATURES rows say so, and the workspace checks pass.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Setting/Sucrose.json -> service | General.UserAgent is user-editable text in a local file (the Portal shows no field for it); it becomes an HTTP header value |
| service -> renderer pipe | WebOptions.user_agent travels in the Assignment over the owner-restricted named pipe (02-05), postcard encoded, same binary on both ends |
| web engine -> remote sites | every page and frame of a wallpaper sends the user agent to the sites it contacts |
| renderer -> log files | the applied value is logged in the renderer and service logs |

## STRIDE Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation Plan |
|-----------|----------|-----------|----------|-------------|-----------------|
| T-uov-01 | Tampering | web_user_agent in apps/sucrose/src/service.rs | medium | mitigate | A value with any control character (CR, LF, NUL, tab) is refused and both engines keep their own user agent, so no header injection or truncated string reaches CefString or the UTF-16 buffer; covered by the Task 1 unit test; Chromium's own header-value check on the CEF side stays as a second line |
| T-uov-02 | Information Disclosure | service and renderer logs | low | mitigate | The applied value is logged with Debug formatting (?), so control characters cannot forge log lines; a refused value is logged by length only |
| T-uov-03 | Information Disclosure | user agent sent to remote sites | low | accept | Legacy parity and the user's own setting; the default text names the product and the OS family only, no personal data |
| T-uov-04 | Denial of Service | a very long user agent | low | accept | The file is the user's own; a site may answer 431, the engine and the wallpaper keep running; the legacy app had no cap either |
| T-uov-05 | Spoofing | WebOptions over the renderer pipe | low | accept | The pipe is already restricted to the owner SID (02-05) and the renderer applies only what the service sent; the field is appended last, so the other fields keep their wire positions |
| T-uov-06 | Tampering | CDP probe on the debugging port during the checks | low | mitigate | DeveloperPort is set only in scratch folders under target/uov-check, the probe is read-only except one same-origin fetch, sends no Input.* and runs only while the executor's own sucrose.exe runs; the maintainer's settings are never touched |
| T-uov-SC | Tampering | npm/pip/cargo installs | high | mitigate | No package install and no new crate in this plan (T-04-SC); the probe uses Bun built-ins only; CEF comes only from the existing pinned, hash-verified cargo xtask fetch-engines (D-028) |
</threat_model>

<verification>
- cargo test -p sucrose-ipc, -p sucrose-engine-cef, -p sucrose-engine-webview2 and cargo test -p sucrose --bin sucrose pass with the new user_agent tests.
- CDP probe on a real desktop run: CEF and WebView2 each report the configured value, the legacy default, and their own agent for a blank value, in navigator.userAgent and in the request header; the YouTube fixture plays on both with the default.
- cargo fmt --all --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace pass.
- git ls-files --eol of the seven files equals the states noted before the edits; docs/FEATURES.md still ends without a newline.
- No process the executor started is left running (tasklist.exe shows no sucrose.exe or sucrose-portal.exe).
</verification>

<success_criteria>
- Both web engines apply General.UserAgent like the legacy app, proven on the real engines.
- The new rule (blank or control-character value keeps the engine's own) and the switch precedence are written in docs/FEATURES.md; the YouTube row no longer lists the user agent as missing.
- No new crate, no unsafe outside the engine crates, no change under src/.
</success_criteria>

<output>
Create `.planning/quick/261127-uov-apply-the-general-useragent-setting-on-both-web-engines-cef/261127-uov-SUMMARY.md` when done, with the probe output of every run, the line-ending notes, whether the Task 1 fallback was needed, and the statement that no measurement row is needed.
</output>
