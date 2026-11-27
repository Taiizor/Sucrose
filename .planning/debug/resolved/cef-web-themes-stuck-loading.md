---
status: resolved
trigger: "CEF: some of the maintainer's Web themes stay on their loading screen (04-37 D1; WebView2 opens the same themes). Continues the 04-20 in-page failure: Test Sucrose API shows a CORS-like error and Critter Crossing reports SucrosePropertyListener is not defined, although every page loads with status 200. Possibly the same CEF-only root cause as 04-37 D4 (seek video invisible on CEF), D6 (url-input black on CEF) and Music Tunnel misbehaving on CEF."
created: 2026-11-27T12:49:08.020Z
updated: 2026-11-27T13:49:32.482Z
goal: find_and_fix
symptoms_prefilled: true
---

## Current Focus

hypothesis: RESOLVED - three CEF-only causes (manifest, --user-data-dir, Skip), confirmed by the maintainer's desktop check 2026-11-27 16:28 and by the machine re-check after the switch to linker embedding
test: done (see Evidence and Resolution)
expecting: -
next_action: none; session archived to .planning/debug/resolved/, KB entry added, STATE.md updated (fix commit a0e22d9a)
bug_class: Bohrbug (deterministic, reproduced 2/2)

reasoning_checkpoint:
  hypothesis: "(A) sucrose.exe, which CEF also runs as its GPU subprocess, has no application manifest declaring Windows 10 support, so the GPU process hits a Chromium CHECK (exit 0x80000003) on the first command buffer, Chromium disables GL after 3 crashes and every WebGL page fails - three.js themes stay on their loaders, Critter Crossing never installs SucrosePropertyListener, url-input and Music Tunnel stay black. (B) Chrome's BasicStartupComplete strips --disable-web-security because CEF only sets the user data dir through PathService, never as --user-data-dir on the command line, so CEF enforces CORS (Test Sucrose API's Bing fetch). (C) CEF's StreamReaderURLLoader calls CefResourceHandler::Skip(first_byte_position) for any Range starting past 0 and fails the request when Skip is not implemented; OriginHandler has no skip, so moov-at-end videos never get metadata and seeks fail."
  confirming_evidence:
    - "X1/X2 (2/2 each): the same exe crashes the GPU process and gives webgl null without a manifest, and gives WebGL 2.0 with no crash when only an external Win10 compatibility manifest is added"
    - "Range probe: bytes=0-7 is a 206 with the right bytes; bytes=6600544-6600551, bytes=100- and bytes=7000000- all fail with TypeError: Failed to fetch; CEF source shows Skip is only called for first_byte_position > 0"
    - "Test Sucrose API: default switches -> 'blocked by CORS policy' + debug.log 'Web security may only be disabled...'; the same run with user-data-dir=<root cache path> added to CefArguments -> no CORS error and no debug.log line; CEF source: OnBeforeCommandLineProcessing runs before ChromeMainDelegate::BasicStartupComplete, OverrideUserDataDir only uses PathService"
  falsification_test: "after the fix, target/release/sucrose.exe (no external manifest, default switches) must show no GPU crash and webgl ok; Range probes must return 206/416 with the expected first bytes; the Test Sucrose API run must show no CORS error. Any of these failing disproves the corresponding cause."
  fix_rationale: "(A) embeds the manifest CEF's own sample executables carry (cef_variables.cmake /MANIFEST:NO + ADD_WINDOWS_MANIFEST compatibility.manifest), which is what X2 proved sufficient; (B) puts on the command line the value Chromium's check requires, equal to root_cache_path, which CEF prefers anyway (GetUserDataPath) so the profile folder does not change; (C) implements the loader contract: the planner already positioned the stream at the range start, Skip reports the requested offset reached (moving forward when the stream starts earlier) and the response length follows the stream."
  blind_spots: "the GPU CHECK site itself is unknown (official build, no message, no dump); the manifest also changes GetVersionEx/VerifyVersionInfo answers for mpv and WebView2 code in the same process (mpv d3d11 may pick the flip model) - needs a Video run; CEF's documented 8 MiB main-thread stack (/STACK:0x800000) is also missing from sucrose.exe but is not proven to cause anything here; Local Network Access prompt for loopback fetches is ignored by CEF (not part of these symptoms)"
  candidate_causes:
    - "build/environment: missing application manifest on the CEF subprocess executable (confirmed)"
    - "config: Chromium switch stripped by Chrome's startup check (confirmed)"
    - "code: CEF resource handler contract (Skip) not implemented (confirmed)"
    - "code: page calls injected at OnLoadEnd too early (eliminated: Critter installs the listener after WebGLRenderer, the failure is the WebGL exception)"
    - "environment: NVIDIA driver / GPU instability (eliminated: same machine, same driver, no crash with the manifest)"
  and_gate: "no - each symptom needs only its own cause: WebGL symptoms need only (A); the Bing CORS error only (B); the seek/moov video only (C). The maintainer's 'stuck loading' themes are WebGL themes, so (A) alone explains them."

## Symptoms

expected: Each of the maintainer's 8 Web themes (copied read-only into target/round-b-lib/Library: Critter Crossing-1, Frog Rush-1, Glitch Maze-1, Interactive Rocket-1, Sky Blitz-1, Voxel Heat-1, `_` "Test Sucrose API" (the only one with SucroseCompatible.json), ynGju7Rw1cDhNQdT2Y6lfcE5B "Farm Weather Cycles") plays on CEF as it did in the C# app (CefSharp) and as it does on WebView2 in our build: the page leaves its loading screen, its scripts run, and the Sucrose JavaScript API (SucrosePropertyListener calls, compatible keys) works.
actual: On CEF some themes stay on their loading screen (04-37 step A1, 2026-11-27, maintainer: "bazı web türündeki duvar kağıtları loadingde kalıyor"); the same themes open on WebView2 (A3: "webview web türündeki duvar kağıtlarını cef'e göre loadingde kalmayıp açıyor"). The maintainer did not name which themes stick. In 04-20 (2026-11-26) "Test Sucrose API" showed a CORS-like error ("cors hatası falan veren var testsucrose api gibi") and Critter Crossing reported "SucrosePropertyListener is not defined". Related CEF-only symptoms of the 04-37 run that may share the cause: the seek fixture video is never visible on CEF while WebView2 plays and seeks it (A5); the url-input URL theme stays black on CEF (B3; on WebView2 it opens); Music Tunnel misbehaves on CEF (B5). The CEF GPU process crashes 3 times at the start of a CEF wallpaper ("GPU process exited unexpectedly: exit_code=-2147483645", "Failed to create shared context for virtualization"), then Chromium renders in software.
errors: Every page logs "page loaded url=\"https://sucrose.wallpaper.engine/...\" status=200" in the Renderer log. No Rust ERROR and no panic. Page console messages are not in the Rust logs (nothing captures them), so the in-page errors are known only from what the maintainer saw. CefRenderer-*.log holds the GPU crash lines.
timeline: First seen in the 04-20 maintainer run (release build 5a17b108, CEF 156.0.31 / Chromium 156.0.8078.100), again in the 04-37 run (release build 0e08ccc1, 2026-11-27). Never seen working on CEF for these themes. WebView2 (added in 04-22..04-27) opens them.
reproduction: Release build target/release (built at 0e08ccc1; code unchanged since), scratch data dir (target/round-b holds the scratch settings migrated read-only from the maintainer's; Web Engine CEF), Use one of the maintainer themes in target/round-b-lib/Library (for example `_` Test Sucrose API or Critter Crossing-1) -> the page stays on loading or shows the in-page error. Evidence: .planning/phases/04-m3-engines-and-the-wallpaper-api/04-37-SUMMARY.md, 04-20-SUMMARY.md; logs in target/round-b-logs (04-37) and target/round-a-logs (04-20).

## Constraints (from the maintainer and the project)

- Desktop runs are allowed for this session: the maintainer said "evet başla" after being told wallpapers will open and close on the desktop while bugs are fixed. The C# Sucrose app stays closed. Runs follow CLAUDE.md: a scratch SUCROSE_DATA_DIR under target/ (never %APPDATA%\Sucrose), WSLENV=SUCROSE_DATA_DIR, the Portal only with --no-focus, never synthesize mouse or keyboard input, capture windows only with .claude/skills/port-feature/capture.ps1, stop with taskkill.exe /IM sucrose-portal.exe /F and taskkill.exe /IM sucrose.exe /F (never kill msedgewebview2.exe by name), confirm with tasklist.exe, stop the apps before rebuilding.
- The maintainer's data is read-only (H:\Sucrose, %APPDATA%\Sucrose, src/). The scratch theme copies in target/round-b-lib hold the maintainer's per-user property values: never copy those values into this file, a commit, a test fixture or a doc.
- A visual check by the maintainer is a checkpoint returned to the orchestrator (the session manager has no AskUserQuestion here).

## Evidence

- timestamp: 2026-11-27T12:54:00.598Z
  checked: target/round-b-logs/CefRenderer-0.log and CefRenderer-1.log (last CEF run of 04-37, 15:24:41 local, a three.js theme on both monitors)
  found: GPU process exits 3 times with exit_code=-2147483645 (0x80000003, STATUS_BREAKPOINT = a Chromium CHECK/IMMEDIATE_CRASH), each right after the renderer's GpuControl.CreateCommandBuffer; after crash 3 GL is disabled (GL_VENDOR = Disabled, "Failed to create shared context for virtualization"); console lines: THREE.WebGLRenderer "A WebGL context could not be created", "Web page caused context loss and was blocked", "Uncaught (in promise) Error: Error creating WebGL context." from js/three.min.js, then "Cannot read properties of undefined (reading 'uniforms')" from js/script.js
  implication: on CEF no WebGL context can be created at all; any WebGL/three.js theme fails in its init and stays on its loading screen. The GPU crash is not a separate cosmetic defect: it is the direct cause of the stuck loading for WebGL themes.

- timestamp: 2026-11-27T12:54:00.598Z
  checked: whether page console output is observable
  found: CEF already writes page console messages to CefRenderer-<slot>.log as "INFO:CONSOLE:<line>] \"message\", source: url (line)"; the file only keeps the last run of each slot
  implication: experiments can read page errors from the CEF log without CDP; Sucrose's own Renderer log still shows none of them

- timestamp: 2026-11-27T12:54:00.598Z
  checked: target/release/debug.log (Chromium's early log before CEF's log settings apply)
  found: every CEF browser process start (two per start with two monitors) logs "chrome_main_delegate.cc:1215] Web security may only be disabled if '--user-data-dir' is also specified with a non-default value."
  implication: Chrome removes --disable-web-security from the CEF command line (legacy default switch, D-17), so CEF enforces CORS and same-origin rules that WebView2 (which honors the switch) and legacy CefSharp did not; a candidate for the Test Sucrose API "CORS-like" error

- timestamp: 2026-11-27T12:54:00.598Z
  checked: target/release/sucrose.exe (the CEF subprocess executable, D-036) for an application manifest; CEF dist contents
  found: no manifest text (no urn:schemas-microsoft-com:asm.v1) in sucrose.exe; CEF 156 minimal dist has no libEGL.dll/libGLESv2.dll (ANGLE inside libcef.dll), and ANGLE did initialise (GL_RENDERER ANGLE NVIDIA D3D11 before the first crash)
  implication: the GPU subprocess runs without the Windows 10 supportedOS compatibility entry CEF's sample executables carry; a candidate cause of the GPU CHECK crash. Missing ANGLE DLLs is ruled out.

- timestamp: 2026-11-27T12:57:23.443Z
  checked: experiment X1 (baseline): scratch library target/gpu-exp/data (synth-library, Web=CefSharpLive, default switches, hardware acceleration on), theme hello replaced by a probe page that logs PROBE lines and creates a WebGL context 3 s after start; run target/release/sucrose.exe --no-portal
  found: GPU process crashes 3 times (exit_code=-2147483645) at +1.1 s, long before the page asks for WebGL (+3 s); then "Failed to create shared context for virtualization"; the probe logs "PROBE webgl null"
  implication: the crash is not caused by WebGL or by a theme: the GPU process dies on the first command buffer any CEF page gets, and WebGL is then unavailable for every page

- timestamp: 2026-11-27T12:57:23.443Z
  checked: experiment X2: the same exe copied to target/gpu-exp/bin-manifest (libcef and resources hard-linked) with an external sucrose.exe.manifest that only declares supportedOS Windows 10 ({8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}); nothing else changed
  found: no GPU crash at all; "PROBE webgl ok WebGL 2.0 (OpenGL ES 3.0 Chromium)"; also "permission_prompt.cc:318 Implement OnShowPermissionPrompt to override default IGNORE handling of permission prompts" when the page fetched http://127.0.0.1:47123 (the fetch never completed)
  implication: the missing Windows 10 compatibility manifest of sucrose.exe (the CEF subprocess executable) is what makes the GPU process crash. Separately, a page request to a loopback address raises a Chromium permission prompt that CEF ignores (candidate for in-page network failures)

- timestamp: 2026-11-27T13:40:39.685Z
  checked: maintainer desktop check (orchestrator-run, release build with the fix, scratch target/round-b, Engine.Web and Engine.Url = CefSharpLive, one monitor) and its logs (target/round-b-logs-cef-fix)
  found: "GPU process exited" 0 times in CefRenderer-0.log (3 per start before), no panic; status=200 for Farm, CritterCrossing, FrogRush, GlitchMaze, Interactive Rocket, SkyBlitz, VoxelHeat, Test Sucrose API (data groups requested), seek and url-input pages, Tunnel.html; maintainer: themes open, the seek fixture video shows and reaches 2.99 of 5.98 (the middle), url-input fluid shows, Music Tunnel works. White border at the start still shows (04-37 D2, separate defect, out of scope). Test Sucrose API shows no on-screen data by design (its js/script.js only console.logs the values)
  implication: the fix is confirmed for this session's scope; remaining step is the maintainer's request to drop the new embed-resource build dependency if the manifest can be embedded another sensible way

- timestamp: 2026-11-27T13:48:09.264Z
  checked: the MSVC linker's own manifest embedding instead of the embed-resource build dependency (apps/sucrose/build.rs: cargo::rustc-link-arg-bin=sucrose=/MANIFEST:EMBED, /MANIFESTINPUT:<CARGO_MANIFEST_DIR>\resources\windows\sucrose.manifest.xml, /MANIFESTUAC:NO; sucrose.rc, the build-dependency and the workspace entry removed, Cargo.lock back to HEAD)
  found: x86_64 links with rust-lld.exe (.cargo/config.toml) and merges the manifest itself (no mt.exe on PATH); the PE resource directory of target/debug/sucrose.exe and target/release/sucrose.exe holds exactly one RT_MANIFEST (24), ID 1, lang 1033, 538 bytes: one trustInfo (asInvoker) plus the compatibility section with {8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}; the same three args with link.exe (scratch probe crate, --config linker="link.exe", Rich header present) embed the identical manifest, so the aarch64 and i686 targets (default link.exe) work too. tests/manifest.rs passes on debug and release; with the three link args removed the binary has no resource directory at all and the test fails ("the binary has no compatibility section in an embedded manifest"); restored, it passes
  implication: no build dependency is needed; the linker is the sensible way to embed the manifest. A non-MSVC Windows target gets a cargo warning instead (CI and releases use MSVC)

- timestamp: 2026-11-27T13:48:09.264Z
  checked: machine verification run of the rebuilt release (cargo build --release -p sucrose -p sucrose-portal) on a new scratch data dir target/cef-manifest-verify/data (synthetic probe-webgl theme only, Engine.Web = CefSharpLive, hardware acceleration on, one monitor), sucrose.exe --no-portal, stopped with taskkill.exe /IM sucrose-portal.exe /F and /IM sucrose.exe /F, tasklist.exe empty
  found: CefRenderer-0.log: "GPU process exited" 0 times, "PROBE webgl ok WebGL 2.0 (OpenGL ES 3.0 Chromium) / WebKit WebGL"; Renderer log: page loaded status=200, no ERROR, no panic; target/release/debug.log not written by the run (no "Web security may only be disabled" line)
  implication: the linker-embedded manifest gives the same result as the embed-resource build the maintainer checked

## Resolution

root_cause: (A) sucrose.exe (also CEF's subprocess executable, D-036) has no application manifest with the Windows 10 supportedOS entry, so CEF's GPU process CHECK-crashes (0x80000003) on its first command buffer, Chromium disables GL after 3 crashes and no page gets WebGL; (B) --disable-web-security (legacy default switch) is stripped by Chrome's early startup check because --user-data-dir is not on the CEF command line, so CEF enforces CORS; (C) OriginHandler does not implement CefResourceHandler::Skip, which CEF's loader calls for every Range request starting past byte 0, so those requests fail
fix: (A) apps/sucrose/build.rs has the MSVC linker embed apps/sucrose/resources/windows/sucrose.manifest.xml (asInvoker + Windows 10 supportedOS) into the sucrose binary only: cargo::rustc-link-arg-bin=sucrose=/MANIFEST:EMBED, /MANIFESTINPUT:<absolute path>, /MANIFESTUAC:NO (the manifest carries its own trustInfo); works with rust-lld.exe (x86_64) and link.exe; no build dependency; a non-MSVC Windows target gets a cargo warning; (B) SettingsValues.user_data_dir = root_cache_path (None for in-memory caches) and HostApp::on_before_command_line_processing appends --user-data-dir with it unless the user's switches name one; (C) OriginHandler implements skip(): Stream::skip_to confirms the planned range start or moves a whole-file stream forward, refusals (404/416 text) skip nothing, a target behind the stream or past its end fails with ERR_REQUEST_RANGE_NOT_SATISFIABLE; response_headers reports the bytes left after the skip; the read loop moved into Stream::read_into
verification: fix commit a0e22d9a. guardrail - (1) reproduction before: X1 baseline GPU crash + webgl null (2/2), Range probes past 0 fail, Bing CORS error; after: all pass on the rebuilt release; (2) revert check: removing the manifest brings the crash back (X1 vs X2, 2/2 each); mutants of skip, user_data_dir and the build.rs embed each make their regression test fail; (3) suite: fmt, clippy -D warnings, cargo test --workspace all green; (4) diff tied to the three causes only (no unrelated change; CEF's documented /STACK:0x800000 deliberately left out as not proven necessary); (5) oracle type: specified (CEF loader contract for Skip, Chrome's switch check, CEF's executable manifest requirement) plus derived probes (expected bytes read from the file). guardrail_verdict: accepted; maintainer desktop check 2026-11-27 16:28 confirmed (themes open, seek reaches the middle, url-input and Music Tunnel render, GPU process exits 0). After the switch to linker embedding: manifest test green on debug and release, fails with the link args removed; release run on target/cef-manifest-verify: GPU process exits 0, webgl ok; cargo fmt --check, cargo clippy --workspace --all-targets -D warnings and cargo test --workspace green (portal.rs 31 passed, 0 failed, 8 ignored, one monitor)
files_changed: [apps/sucrose/build.rs, apps/sucrose/resources/windows/sucrose.manifest.xml, apps/sucrose/tests/manifest.rs, crates/sucrose-engine-cef/src/browser.rs, crates/sucrose-engine-cef/src/scheme.rs, docs/FEATURES.md, docs/ARCHITECTURE.md, .planning/STATE.md]
oracle_type: specified
notes_outside_scope:
  - "sucrose.exe is linked with the default 1 MiB main-thread stack; CEF's cmake sets /STACK:0x800000 for x64 executables (cef_variables.cmake: 1.5 MiB minimum for CEF's main thread). Not proven to cause anything here; recommended follow-up"
  - "the 04-21 M3 CEF measurement row was taken while the GPU process crashed (software rendering); 04-38 should measure after this fix"
  - "Interactive Rocket has a SucroseProperties.json but defines no SucrosePropertyListener: its property calls throw on every engine (theme content, not engine)"
  - "H.264/AAC are not playable on CEF (no proprietary codecs in the CEF dist, as legacy CefSharp); VP8/VP9/AV1 are"
  - "CEF keeps only the last run in CefRenderer-<slot>.log; page console lines are there (INFO:CONSOLE), not in Sucrose's Renderer log"
  - "scratch material: target/gpu-exp (data dir with probe themes and copies of maintainer themes, bin-manifest copy, cdp.ts); never commit, can be deleted"
