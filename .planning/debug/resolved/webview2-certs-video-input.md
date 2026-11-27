---
status: resolved
trigger: "WebView2 defects from the 04-37 maintainer run: D3 the certs fixture still fails after Allow invalid certificates + Restart now; D5 the video-clip GIF/Video generated page is black when Video Player = WebView2; D6 (WebView2 half) the url-input URL theme opens on WebView2 but does not react to the mouse."
created: 2026-11-27T13:52:23.485Z
updated: 2026-11-27T14:37:46.744Z
goal: find_and_fix
symptoms_prefilled: true
---

## Current Focus

hypothesis: closed - D3, D5 and the WebView2 half of D6 confirmed and fixed (see Resolution); fixes cf8f7d6a, 44758204, 747bbf92; maintainer-confirmed 2026-11-27
test: none (session resolved)
expecting: n/a
next_action: none - session archived to .planning/debug/resolved/, knowledge-base entry added, .planning/STATE.md updated
bug_class: Bohrbug

## Symptoms

expected: (D3) With Web Engine = WebView2, the certs fixture's iframe (an expired-certificate page on badssl.com) fails by default and loads after Settings "Allow invalid certificates" is turned on and "Restart now" is chosen (D-12; CEF passed this in 04-20). (D5) With Video Player = WebView2, the video-clip fixture plays as a generated web page (VideoContent.html) like it does on mpv (WAPI-08). (D6) With Url Engine = WebView2, the url-input theme (https://paveldogreat.github.io/WebGL-Fluid-Simulation/) reacts to mouse moves on the desktop (input forwarding, TYPE-04 / WAPI-07), as the legacy app does.
actual: (D3, step A8) "ayarı açmam işe yaramadı yine yüklenmesi" = turning the setting on did not help, the frame still did not load. (D5, step B2) "webview video-clip kısm sorunlu siyah ekran görünüyor" = the WebView video-clip part is faulty, a black screen shows. (D6, step B3) "webviewde açıldı ama fare hareketlerine tepki vermiyor" = on WebView it opened but does not react to mouse movements. Not checked in 04-37: input on Web themes under WebView2 (step A6 was dropped from the shortened run) and mouse on url-input under CEF.
errors: No Rust ERROR or panic in target/round-b-logs (04-37) or target/round-b-logs-cef-fix (13:28-13:35Z run). The video page logs "page loaded url=\"https://sucrose.wallpaper.engine/VideoContent.html?src=clip.mp4\" status=200" on WebView2. Page console messages are not captured on WebView2 either.
timeline: WebView2 was added in 04-22..04-27 (Round B). First maintainer check 04-37 (2026-11-27, release build 0e08ccc1). The CEF fixes of cef-web-themes-stuck-loading (a0e22d9a) do not touch WebView2. On WebView2 the seek fixture plays and seeks (so Range on the virtual origin works there), Web themes open, DevTools opens.
reproduction: Release build target/release (current at cb25c4b1; rebuild if older), scratch data dir, Web/Url/Video engine WebView2; Use certs (after enabling Allow invalid certificates + Restart now), video-clip (Video Player = WebView2), url-input (move the mouse over the desktop). Fixtures in apps/sucrose/tests/fixtures/web-themes (certs, url-input) and the media fixtures (video-clip) from cargo xtask synth-library --media-fixtures.

## Constraints (from the maintainer and the project)

- Desktop runs are allowed (the maintainer agreed to bug fixing with wallpapers opening and closing). C# Sucrose stays closed. Follow CLAUDE.md for runs: scratch SUCROSE_DATA_DIR under target/ (not target/round-b, which holds the maintainer's scratch settings), WSLENV, Portal only with --no-focus, never synthesize mouse or keyboard input (for D6 the input must be proven without synthetic input: read the input pipeline, the forwarding code and logs, CDP Input.* is synthetic too and not allowed on the desktop; the maintainer confirms the mouse visually), capture only with .claude/skills/port-feature/capture.ps1, stop with taskkill.exe /IM sucrose-portal.exe /F and /IM sucrose.exe /F, never kill msedgewebview2.exe by name, confirm with tasklist.exe, stop apps before rebuilding.
- Maintainer data read-only (H:\Sucrose, %APPDATA%\Sucrose, src/).
- Visual confirmation by the maintainer is a checkpoint returned to the orchestrator; the orchestrator may batch several sessions' visual checks into one short maintainer run, so prefer strong machine evidence and say exactly what the maintainer must look at.

## Evidence

- timestamp: 2026-11-27T13:56:45.519Z
  checked: Phase 0 knowledge base (.planning/debug/knowledge-base.md: cef-close-shutdown-dialog, cef-web-themes-stuck-loading)
  found: no entry about WebView2 certificates, generated video pages or WebView2 input; the CEF entry's pattern (compare with what the reference implementation does, read the engine's own logs) applies as method only
  implication: no known-pattern candidate; investigate each defect from scratch

- timestamp: 2026-11-27T13:56:45.519Z
  checked: D3 - target/round-b-logs Renderer-0/1 (04-37 run), the certs runs 12:06:42Z-12:08:45Z
  found: with allow_invalid_certificates=false (12:06:42, 12:07:18) no certificate line; after Restart now the renderers log allow_invalid_certificates=true and "certificate error allowed host=expired.badssl.com" (slot 1 at 12:07:47.56, slot 0 at 12:07:58.22 and again in the next renderer at 12:08:28.21); the main-frame NavigationCompleted of the certs page comes 1.5 s to 23 s after the start (the iframe holds the load event), e.g. slot 0 12:08:24 start, cert allowed 12:08:28, page loaded 12:08:45
  implication: the setting reaches the restarted renderer and the handler is registered and called for the iframe's request and sets ALWAYS_ALLOW; the failure is after that decision, not a missing switch or handler

- timestamp: 2026-11-27T13:56:45.519Z
  checked: D3 - legacy src/Shared/Engine/Sucrose.Shared.Engine.WebView/Event/Web.cs (and Url/Gif/Video/YouTube.cs), Live/Sucrose.Live.WebView/App.xaml.cs
  found: the legacy WebView engine registers ServerCertificateErrorDetected unconditionally and sets Action = AlwaysAllow; it passes no --ignore-certificate-errors; CEF (Rust) appends --ignore-certificate-errors when allowed (browser.rs)
  implication: our WebView2 path is the legacy mechanism, gated by the setting (D-12); whether the legacy app ever loaded a bad-certificate iframe is unknown

- timestamp: 2026-11-27T13:56:45.519Z
  checked: D5 - target/round-b-lib/Library/video-clip/clip.mp4 vs seek/clip.mp4; crates/sucrose-engine-web/src/pages/VideoContent.html; the seek fixture page; the 04-37 log of the video run (12:20:26Z)
  found: both clips are the same 6,603,416-byte file and WebView2 plays and seeks the seek copy; the seek page's video has the muted attribute and is 30vw wide, VideoContent's video has no muted attribute (autoplay with sound), is position:fixed full screen with z-index 9999 and object-fit cover, and gets the built-in WebView property calls; the page loaded with status 200 on both slots; no origin request lines at INFO; the WebView2 switches include --autoplay-policy=no-user-gesture-required and --disable-gpu-compositing
  implication: not a codec or serving difference of the file itself; the page, its calls, autoplay with sound or compositing of a full-screen video are the candidates

- timestamp: 2026-11-27T13:56:45.519Z
  checked: D6 - target/round-b-logs (url-input on WebView2, 12:21:51Z and 12:22:23Z, both slots), apps/sucrose/src/render.rs, crates/sucrose-engine-webview2/src/engine.rs and host.rs, legacy Helper/Handle.cs and Extension/Interaction.cs
  found: "raw input registered mouse=true keyboard=true" for every url-input renderer, so input is collected and forwarded (PageKind::Url takes input); WebView2Pump::post_input posts the legacy messages to Host::post_to_widget, whose find_widget returns the first Chrome_WidgetWin_0 under the surface (else the first Chrome_WidgetWin_1); the legacy GetInputHandle takes FindWindowEx(control, "Chrome_WidgetWin_0") and then posts to FindWindowEx(that, "Chrome_WidgetWin_1"), i.e. the Chrome_WidgetWin_1 child of Chrome_WidgetWin_0, falling back to the control window only when there is no Chrome_WidgetWin_0
  implication: differential: we post to the parent widget the legacy app steps over; prime D6 candidate, to be confirmed on the real window tree

- timestamp: 2026-11-27T14:03:17.687Z
  checked: D3 reproduction - release build (16:43 +03, = a0e22d9a code), scratch target/wv2-debug/data (synth-library --web-fixtures --media-fixtures, Web/Url/Video = WebViewLive, DeveloperPort 9222, AllowInvalidCertificates true, fresh persistent profile, one monitor, no DevTools), certs selected; CDP (bun.exe on Windows: WSL cannot reach 127.0.0.1:9222) Page.getFrameTree, Runtime.evaluate, Network events, Page.captureScreenshot
  found: the frame loads: frame URL https://expired.badssl.com/, its document title expired.badssl.com, body background rgb(255, 0, 0), screenshot target/wv2-debug/probe/certs-after.png shows the red badssl page; Network shows the Document 200 with securityState insecure; but the main frame's NavigationCompleted came 9.8 s after the start (cert decision logged at +1.2 s, page loaded at +9.8 s)
  implication: the setting works on WebView2 on this build; D3 as "never loads" does not reproduce; the timing is the suspicious part

- timestamp: 2026-11-27T14:03:17.687Z
  checked: D3 timing - a fresh invalid-certificate host in the same renderer (CDP sets the iframe src to https://self-signed.badssl.com/, frame tree, content and screenshot sampled at 1.5/4/8/14/22 s), plus curl.exe -k timings from Windows
  found: request at +6 ms; "certificate error allowed host=self-signed.badssl.com" at +2.28 s; the request is actually sent (requestWillBeSentExtraInfo) only at +11.99 s and answered at +12.17 s; until then the frame keeps showing the previous page (samples at 1.5, 4 and 8 s); curl.exe -k: TLS 1.7 s, whole request 1.9 s (self-signed, expired), 5.2 s (wrong.host)
  implication: after ALWAYS_ALLOW, WebView2 holds the frame's request about 9.7 s before it goes out, far longer than the network needs; with the slow badssl TLS the frame appears 10-20 s after a start (maintainer run: 12:08:24 start, decision 12:08:28, page loaded 12:08:45, i.e. 21 s); a person checking within that time sees the frame "not load". Candidate D3 cause: the event path (ServerCertificateErrorDetected + ALWAYS_ALLOW) instead of CEF's --ignore-certificate-errors; to test: same run with --ignore-certificate-errors in WebArguments

- timestamp: 2026-11-27T14:04:20.441Z
  checked: D3 experiment - the same build, data dir and certs fixture with one change: --ignore-certificate-errors added to Engine.WebArguments (the switch CEF gets for Allow invalid certificates); CDP sets the iframe to a third invalid host (https://untrusted-root.badssl.com/)
  found: WebView2 honors the switch: no "certificate error allowed" line at all (the error never happens); the certs page's main frame loaded 1.15 s after the start (9.8 s with the event path); the untrusted-root frame request went out at +0.85 s and was answered at +1.02 s (vs +11.99 s / +12.17 s for self-signed through the event path); the frame shows the red badssl page from the first sample (1.5 s)
  implication: confirmed D3 mechanism: the ServerCertificateErrorDetected/ALWAYS_ALLOW path delays every allowed frame by about 10 s on WebView2, the --ignore-certificate-errors switch removes the delay; fix direction: give WebView2 the same switch as CEF when Allow invalid certificates is on (Sucrose's own switch, never stored, D-17), keep validation (no switch, no handler) when it is off

- timestamp: 2026-11-27T14:06:44.615Z
  checked: D5 reproduction - video-clip (real 6.6 MB clip copied from target/round-b-lib) on WebView2, same scratch data dir; CDP Runtime.evaluate of the video element, Media domain, Page.captureScreenshot; ffprobe of the clip
  found: clip = VP9 3840x2160 60 fps, no audio, moov at the end, first frames not black (YAVG 139); in the page the video plays: paused false, readyState 4, currentTime 2.72 -> 5.72 in 3 s, decoded frames 519 -> 703, dropped 0, no error, Media log "Selected VpxVideoDecoder", autoplay_decision was_blocked false (policy no-user-gesture-required), Range requests served 206 (0-, tail, 3145728-); yet the CDP screenshot (target/wv2-debug/probe/video.png) is entirely black; layout: video rect 1920 x 0, body rect 1920 x 0, body computed transform matrix(1,0,0,1,0,0) (style scale(1) from the "scale" property call, value 0), document.compatMode CSS1Compat
  implication: not autoplay, codec, serving or compositing: the video element is 0 px tall

- timestamp: 2026-11-27T14:06:44.615Z
  checked: D5 live experiments through CDP Runtime.evaluate on the same page: (1) body.style.transform = '' ; (2) a style html,body{height:100%} with body transform scale(1) back, then SucrosePropertyListener('scale', {value: 5})
  found: (1) video rect becomes 1920 x 1080 and the screenshot shows the clip (target/wv2-debug/probe/videolayout-after.png); (2) video and body 1920 x 1080 at scale(1); at value 5 (scale 1.5) both are (-480,-270) 2880 x 1620, a centred zoom
  implication: confirmed mechanism: a transformed body becomes the containing block of the position:fixed video; in standards mode body has no height (its only content is out of flow), so height:100% is 0

- timestamp: 2026-11-27T14:06:44.615Z
  checked: D5 legacy page (src/Shared/Engine/Sucrose.Shared.Engine/Helper/Source.cs GetVideoContent, WebView Helper/Video.cs Load) vs crates/sucrose-engine-web/src/pages/VideoContent.html; both built-in property sets (properties-webview.json, properties-cef.json)
  found: the legacy page has no doctype (quirks mode, where html and body fill the viewport), the same "scale" case (body.style.transform) and the same fixed full-size video style (set by Load()); our port starts with <!DOCTYPE html> (standards mode) and has no html/body height; both built-in sets carry "scale" (default 0), so the call runs on every start on both engines; GifContent.html has no doctype (quirks, as legacy); YouTubeContent.html applies scale to the player iframe's document, not to the outer body
  implication: D5 root cause: the doctype added in the port switched VideoContent.html to standards mode, so the default "scale" call collapses the video to 0 px; CEF is affected the same way (untested there in 04-37); fix: html,body{height:100%} in VideoContent.html

- timestamp: 2026-11-27T14:11:53.921Z
  checked: D6 window tree under the renderer's surface (target/wv2-debug/probe/tree.ps1: read-only FindWindowEx walk, GetWindowThreadProcessId) with url-input and with certs on WebView2, and the renderer's debug log
  found: SucroseDesktopSurface (pid = the sucrose renderer) -> Chrome_WidgetWin_0 (pid = the same sucrose renderer, WebView2's host window in our process) -> Chrome_WidgetWin_1 (pid = msedgewebview2 browser process, titled with the page title) -> Chrome_RenderWidgetHostHWND "Chrome Legacy Window" (browser process) and Intermediate D3D Window (GPU process); the log says "input target found class=Chrome_WidgetWin_0"
  implication: we post the input messages to a window of our own process; the legacy target (the Chrome_WidgetWin_1 child of Chrome_WidgetWin_0) is the browser process's widget

- timestamp: 2026-11-27T14:11:53.921Z
  checked: D6 passive delivery check, no synthetic input: url-input on WebView2 with InputType OnlyMouse (no keystrokes forwarded) and InputDesktop off; CDP Runtime.evaluate installs capture-phase window listeners that only count mousemove/mousedown/mouseup (no key content); the real cursor sampled read-only with GetCursorPos every 2 s for 60 s (target/wv2-debug/probe/cursor.ps1)
  found: the maintainer's real cursor took 24 distinct positions in the 60 s window (e.g. 1500,529 -> 1596,299 -> 317,323), the renderer had raw input registered and looked the target up on the first forwarded event, and the page counted 0 mousemove, 0 mousedown, 0 mouseup over that window (and 0 in the 60 s before it)
  implication: confirmed: messages posted to Chrome_WidgetWin_0 never reach the page

- timestamp: 2026-11-27T14:11:53.921Z
  checked: D6 the url-input page's own script (https://paveldogreat.github.io/WebGL-Fluid-Simulation/script.js, fetched read-only)
  found: canvas mousemove returns at once unless pointer.down (line 1029: if (!pointer.down) { return; }); mousedown starts a splat, window mouseup ends it; keydown space adds random splats, P pauses
  implication: second D6 condition: on this page a plain mouse move never changes anything in any browser, so the maintainer's "does not react to mouse movements" would stay true even with correct routing; the visual check must drag (press and move); the fixture description ("follows the pointer") is misleading

- timestamp: 2026-11-27T14:19:15.504Z
  checked: red phase of the three regression tests before the fixes (cargo test -p sucrose-engine-webview2 --lib; cargo test -p sucrose-engine-web --test pages), input_widget first written with the old choice (first Chrome_WidgetWin_0, else first Chrome_WidgetWin_1)
  found: FAILED engine::tests::allow_invalid_certificates_gives_webview2_the_switch_cef_gets_and_only_then (no ignore-certificate-errors in the arguments); FAILED host::tests::input_goes_to_the_browser_widget_inside_webview2s_host_window ("never the host window itself": got the host window) and host::tests::a_host_window_without_its_browser_widget_yet_is_no_target_and_the_widget_alone_is_one; FAILED pages a_page_whose_scale_property_transforms_its_body_keeps_the_body_as_tall_as_the_viewport ("VideoContent.html: html and body must be height:100%"; GifContent.html passes)
  implication: each test reproduces its defect; after the fixes all pass (webview2 lib 31 passed; sucrose-engine-web all test binaries ok)

- timestamp: 2026-11-27T14:19:15.504Z
  checked: machine verification on the rebuilt release (cargo build --release -p sucrose -p sucrose-portal, 1 m 51 s), same scratch data dir, WebArguments back to the defaults
  found: D3 - the renderer's switches end with "--remote-allow-origins=* --ignore-certificate-errors", no "certificate error allowed" line, certs page loaded 2.7 s after the start, a fresh wrong.host.badssl.com frame answered at about 1.0 s and showed the red page from the first sample; with AllowInvalidCertificates false the switch is absent (count 0) and the frame is chrome-error://chromewebdata/ (unreachableUrl https://expired.badssl.com/, net::ERR_CERT_DATE_INVALID); D5 - video rect 1920 x 1080 with body transform scale(1), playing (2.88 -> 5.88 s), screenshot target/wv2-debug/probe/video-fixed.png shows the clip; D6 - log "input target found class=Chrome_WidgetWin_1", the page's capture listeners counted 3792 trusted events (3659 mousemove, 69 mousedown, 64 mouseup) from the maintainer's real mouse while GetCursorPos showed 29 distinct positions in 60 s (0 events before the fix in the same setup)
  implication: all three fixes work on the release build; D5 now also shows 341 dropped frames of 715 for this 4K60 VP9 clip under software decode (VpxVideoDecoder) once it is actually painted - a performance note outside this session, not a defect of the fix

- timestamp: 2026-11-27T14:26:45.868Z
  checked: guardrail - manual mutants (no Stryker/cargo-mutants in this workspace): A "always add ignore-certificate-errors", B "fall back to the host window when it holds no widget", C "html{height:100%} only"; full checks: cargo fmt --all, cargo clippy --workspace --all-targets -D warnings, cargo test --workspace --no-fail-fast
  found: all three mutants killed (A: allow_invalid_certificates_gives_webview2_the_switch_cef_gets_and_only_then FAILED; B: a_host_window_without_its_browser_widget_yet_is_no_target_and_the_widget_alone_is_one FAILED; C: a_page_whose_scale_property_transforms_its_body_keeps_the_body_as_tall_as_the_viewport FAILED); files restored from HEAD (EOL put back to LF); clippy clean; workspace: 56 test binaries ok, 0 failed (one monitor, portal.rs 31 passed 8 ignored)
  implication: the tests assert the root causes, not the symptoms; no neighbour broke

- timestamp: 2026-11-27T14:37:46.744Z
  checked: maintainer's desktop check, run by the orchestrator on the release build at HEAD 747bbf92 (data dir target/wv2-debug/data with the debugger's earlier Log moved aside, Portal started with focus at 2026-11-27 17:27:53 +03, one monitor; stopped with taskkill.exe /IM sucrose-portal.exe /F and /IM sucrose.exe /F, tasklist.exe empty; log of the run in target/wv2-debug/data/Log)
  found: maintainer verdict (verbatim) "D3 done", "D5 evet video oynatılıyor, takılmalar çok hissedilmiyor fakat bir ağırlık avr gibi hafiften" (yes, the video plays; the stutter is not very noticeable but there is a slight heaviness), "D6 done"
  implication: D3, D5 and the WebView2 half of D6 are confirmed fixed on the desktop; the slight heaviness on D5 is the software-decode observation already in notes_outside_scope (4K60 VP9 via VpxVideoDecoder, about 340 of 715 frames dropped in 3 s), recorded in .planning/STATE.md as an open item for the Round D performance work and the 04-38 measurement, not a reopened defect

## Eliminated

- hypothesis: "D5: unmuted autoplay is refused on WebView2, so the clip stays on its first frame"
  evidence: "Media log autoplay_decision was_blocked false (policy no-user-gesture-required); paused false, currentTime advancing"
  timestamp: 2026-11-27T14:26:57.789Z

- hypothesis: "D5: the codec or the serving of clip.mp4 fails on WebView2 (VP9 4K, moov at the end, Range)"
  evidence: "VpxVideoDecoder selected, readyState 4, 184 frames decoded in 3 s, Range 206 replies for 0-, the tail and 3145728-; the same file plays in the seek fixture"
  timestamp: 2026-11-27T14:26:57.789Z

- hypothesis: "D5: the full-screen video is lost in compositing (hardware overlay or --disable-gpu-compositing under the layered surface)"
  evidence: "the CDP screenshot (the compositor's own output) was black too, and the video element measured 1920 x 0; clearing the body transform showed the clip with the same switches"
  timestamp: 2026-11-27T14:26:57.789Z

- hypothesis: "D3: the Allow invalid certificates setting or its handler does not reach the restarted WebView2 renderer"
  evidence: "04-37 logs and this session's runs show allow_invalid_certificates=true and 'certificate error allowed host=...' for the frame; the frame did load, only about 10 s late"
  timestamp: 2026-11-27T14:26:57.789Z

## Resolution

root_cause: "D3: with Allow invalid certificates on, WebView2 accepted an invalid certificate only through the legacy ServerCertificateErrorDetected handler (ALWAYS_ALLOW); WebView2 holds every request allowed that way about 10 s before it goes out, so with badssl.com's slow TLS (1.7-5 s) the certs frame appeared 10-21 s after a start and was judged not to load; CEF gets --ignore-certificate-errors and loads at network speed; D5: VideoContent.html was ported with <!DOCTYPE html> (standards mode); the built-in scale property call (default 0, every start) sets body.style.transform, the transformed body becomes the containing block of the position:fixed video, body is 0 px tall in standards mode, so the video was 1920 x 0 (black) on both web engines; the legacy page had no doctype (quirks mode filled the viewport); D6: find_widget posted input to the first Chrome_WidgetWin_0 under the surface, WebView2's host window in Sucrose's own process, which never hands posted messages to the page (the legacy app and docs/legacy/engines.md use its Chrome_WidgetWin_1 child, the browser process's widget); AND the url-input page only reacts while the pointer drags, so plain moves never show anything on it"
fix: "D3 (cf8f7d6a): WebView2 browser_arguments add ignore-certificate-errors when AllowInvalidCertificates is on (never when off; not twice when the user lists it), the handler stays as the legacy fallback; D5 (44758204): html,body{height:100%} in VideoContent.html (doctype kept); D6 (747bbf92): input_widget (pure, tested) picks the Chrome_WidgetWin_1 inside the first Chrome_WidgetWin_0, none while it is missing, else the first Chrome_WidgetWin_1; the url-input fixture description says the page draws while the pointer drags; docs/FEATURES.md rows 73, 74, 77 updated in the matching commits"
verification:
  target_test: { result: pass, tests: ["sucrose-engine-webview2 engine::tests::allow_invalid_certificates_gives_webview2_the_switch_cef_gets_and_only_then", "sucrose-engine-web tests/pages.rs a_page_whose_scale_property_transforms_its_body_keeps_the_body_as_tall_as_the_viewport", "sucrose-engine-webview2 host::tests::input_goes_to_the_browser_widget_inside_webview2s_host_window", "host::tests::a_host_window_without_its_browser_widget_yet_is_no_target_and_the_widget_alone_is_one"], red_before_fix: true }
  mutation_check: { result: pass, tool: "manual mutants (no Stryker or cargo-mutants configured)", mutant_killed: "3/3 (always-add switch; host-window fallback; html-only height)" }
  no_op_deletion: { result: pass, deletion_justified_by_rca: n/a (all three fixes add behaviour; the old widget search is replaced, not removed) }
  adjacent_tests: { result: pass, suites_run: ["cargo test --workspace --no-fail-fast: 56 binaries ok, 0 failed", "cargo clippy --workspace --all-targets -D warnings: clean", "cargo fmt --all"] }
  revert_and_reconfirm: { result: pass, bug_returned_on_revert: true, fixed_on_reapply: true, how: "unit tests red on the pre-fix code and on the mutants; release binary before the fixes vs after, same scratch data and fixtures: D3 frame 12.2 s -> 1.0 s (no certificate event), off still refused; D5 video 1920x0 black -> 1920x1080 showing the clip; D6 0 page mouse events with 24 real cursor positions -> 3792 trusted events with 29 positions" }
  guardrail_verdict: accepted
  maintainer_check: { result: confirmed, run: "2026-11-27 17:27 +03, release build at HEAD 747bbf92, one monitor, scratch data dir target/wv2-debug/data", items: "D3 done; D5 the video plays (slight heaviness = the software-decode observation, out of scope); D6 done" }
files_changed: [crates/sucrose-engine-webview2/src/engine.rs, crates/sucrose-engine-webview2/src/host.rs, crates/sucrose-engine-web/src/pages/VideoContent.html, crates/sucrose-engine-web/tests/pages.rs, apps/sucrose/tests/fixtures/web-themes/url-input/SucroseInfo.json, docs/FEATURES.md]
commits: [cf8f7d6a D3, 44758204 D5, 747bbf92 D6]
oracle_type: "D3 specified (D-12: the setting accepts invalid certificates, off validates; CEF's mechanism), D5 derived (CSS containing-block rule: a transformed ancestor holds fixed descendants), D6 specified (legacy window chain, docs/legacy/engines.md section 5)"
notes_outside_scope:
  - "the 4K60 VP9 video-clip decodes in software on WebView2 (VpxVideoDecoder, not a platform decoder) and, now that it is painted, drops about 340 of 715 frames in 3 s; performance item, not part of D5; the maintainer noticed it as a slight heaviness in the 2026-11-27 check; recorded in .planning/STATE.md for the Round D performance work and the 04-38 measurement"
  - "video-clip on CEF was not run in this session; the same page and the same scale call apply, so the D5 fix covers CEF by construction"
  - "the ten-second stall of WebView2's ALWAYS_ALLOW path itself is inside the runtime (not attributed further); with the switch the event no longer fires"
  - "STATE.md (04-37 defect status line and Next step) and the knowledge-base entry were updated in the resolve step after the maintainer's check (2026-11-27)"
  - "scratch material: target/wv2-debug (data dir with fixtures and the real clip, probe/cdp.ts, tree.ps1, cursor.ps1, screenshots); never commit, can be deleted"
