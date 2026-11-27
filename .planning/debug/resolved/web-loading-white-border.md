---
status: resolved
trigger: "04-37 D2: a white border at the screen edges while some Web themes show their loading screen, on both web engines (CEF and WebView2); it goes away once the theme has opened."
created: 2026-11-27T14:39:59.209Z
updated: 2026-11-27T15:28:54.460Z
goal: find_and_fix
symptoms_prefilled: true
---

## Current Focus

status_note: resolved 2026-11-27 after the maintainer's desktop check (CEF frame gone, none on WebView2); fix committed as 6c6d9a3c
next_action: none (session archived; STATE.md 04-37 D2 resolved, D11 added)

## Symptoms

expected: A Web theme shows its own loading screen edge to edge, as in the C# app (CefSharp and WebView2 engines): no white frame around the screen at any time.
actual: 04-37 A1 (CEF): "ayrıca ekranın çerçevesinde beyaz kenarlıklar oluyor, niye bilmiyorum" = white borders appear around the edges of the screen. A3 (WebView2): "yine bazı duvar kağıtları açılışında border beyaz kenarlık oluyor" = again some wallpapers show a white border when they open. After the CEF fixes (cef-web-themes-stuck-loading): "yine açılışta beyaz kenarlıklar görünüyor" = white borders still show at start. Asked whether it stays: "loadingde beyaz kenarlık görünüyor, tema açılınca gidiyor galiba" = the white border shows during loading and seems to go away when the theme has opened. The maintainer did not name the themes; candidates are their 8 Web themes (scratch copies in target/round-b-lib/Library: Critter Crossing-1, Frog Rush-1, Glitch Maze-1, Interactive Rocket-1, Sky Blitz-1, Voxel Heat-1, `_` Test Sucrose API, ynGju7Rw1cDhNQdT2Y6lfcE5B Farm Weather Cycles), several of which have a loading screen.
errors: none in the logs (target/round-b-logs, target/round-b-logs-cef-fix).
timeline: seen in the 04-37 run (2026-11-27) on both engines and again after the CEF fixes (a0e22d9a). Not reported in 04-20 (CEF themes did not get past loading then).
reproduction: release build (current HEAD 757609e6 or later), scratch data dir, Web Engine CEF or WebView2, Use a maintainer Web theme with a loading screen and watch the screen edges while it loads.

## Constraints (from the maintainer and the project)

- Desktop runs are allowed (bug fixing with wallpapers opening and closing was agreed). C# Sucrose stays closed. Runs: scratch SUCROSE_DATA_DIR under target/ (a new folder; copy the needed theme folders from target/round-b-lib, they are already read-only copies; never write under H:\Sucrose or %APPDATA%\Sucrose), WSLENV, Portal only with --no-focus (or sucrose.exe --no-portal with Library.Selected set in the scratch Setting/Sucrose.json), never synthesize input, capture only with .claude/skills/port-feature/capture.ps1 (PrintWindow) or CDP Page.captureScreenshot (read-only), stop with taskkill.exe /IM sucrose-portal.exe /F and /IM sucrose.exe /F, never kill msedgewebview2.exe by name, tasklist.exe empty at the end, stop apps before rebuilding.
- The scratch theme copies hold the maintainer's per-user property values: never copy those values into this file, a commit, a fixture or a doc.
- A visual confirmation by the maintainer is a checkpoint returned to the orchestrator; prefer machine evidence (captures taken during loading, before and after the fix).

## Evidence

- timestamp: 2026-11-27T14:43:20.446Z
  checked: knowledge base (Phase 0)
  found: no entry with a white border or a loading-screen symptom; webview2-certs-video-input recorded a standards-mode body height problem in a generated page (body 0 px tall under a transform)
  implication: only a partial match; kept as a candidate for the page-level branch

- timestamp: 2026-11-27T14:43:20.446Z
  checked: embedder and surface background colours: crates/sucrose-engine-cef/src/browser.rs (BrowserSettings.background_color), crates/sucrose-engine-webview2/src/host.rs (ICoreWebView2Controller2::SetDefaultBackgroundColor), crates/sucrose-platform/src/os_windows/surface.rs (window class brush); legacy src/Live/Sucrose.Live.WebView/App.xaml.cs and src/Live/Sucrose.Live.CefSharp
  found: CEF background_color = 0xFF000000 (opaque black), WebView2 DefaultBackgroundColor = opaque black, the surface class brush is BLACK_BRUSH; the legacy WebView engine also sets DefaultBackgroundColor = Color.Black, the legacy CefSharp engine sets no background colour
  implication: the white is not the embedder's default page background or the surface brush; the orchestrator's lead does not explain it as stated

- timestamp: 2026-11-27T14:43:20.446Z
  checked: the 8 maintainer Web themes (target/round-b-lib/Library, structure and CSS only)
  found: the five Taiizor games (Critter Crossing, Frog Rush, Glitch Maze, Sky Blitz, Voxel Heat) set html,body{margin:0;width:100%;height:100%;background:dark} and show #loader {position:fixed; inset:0; 100% x 100%; dark background; z-index 6} until main.js hides it; Farm Weather Cycles shows a loading.webm video at 100% x 100vh (body margin auto, no html background); Interactive Rocket is a Construct 2 page with html,body{background:#000} and *{margin:0}; Test Sucrose API is an img 100% x 100%
  implication: once the stylesheet applies, no theme leaves a page-level white edge by its own CSS; the white must come from before the stylesheet applies, from outside the page, or from a size mismatch

- timestamp: 2026-11-27T14:43:20.446Z
  checked: target/round-b-logs-cef-fix/Renderer-0 (the CEF run after a0e22d9a)
  found: the surface is attached as a layered child of Progman (layout=Raised strategy=LayeredChild, ex_style=0x8080080 = WS_EX_NOACTIVATE|WS_EX_LAYERED|WS_EX_TOOLWINDOW, 1920x1080 at 0,0); "page loaded" comes 0.1-2 s after "engine started"; no resize or error lines
  implication: the surface itself is placed on the full monitor; whether the browser child windows cover it from the first frame is not logged

- timestamp: 2026-11-27T14:51:47.482Z
  checked: experiment 1 - release build 757609e6, scratch target/dbg-wlb, CEF, Critter Crossing; read-only probe (target/dbg-wlb/probe.ps1, probe2.ps1): window tree of the SucroseDesktopSurface every 60-100 ms and PrintWindow(PW_RENDERFULLCONTENT) of the surface and, in later runs, of Progman cropped to the surface (the composed desktop); 3 CEF runs (cap-cef-critter, -2, -3)
  found: in every CEF run the captures from the first one after the surface appears until the GPU output window covers the surface show black with a 7 px classic window frame on all four edges (row/column pixels from the edge: E3E3E3 FFFFFF then 5 x B4B4B4 on left/top; 5 x B4B4B4 then A0A0A0 696969 on right/bottom = EDGE_RAISED plus COLOR_ACTIVEBORDER, the classic WS_THICKFRAME sizing border); it vanishes in the capture after the child "Intermediate D3D Window" (GPU process) grows from 1x1 to 1920x1080, then the loader colour 0E1116 or the scene fills the screen; visible for about 1-2.3 s of each start (run 3: 1.0-3.25 s probe time)
  implication: the white border is a window-level artefact, not page CSS; its pattern is the non-client thick frame of a child window

- timestamp: 2026-11-27T14:51:47.482Z
  checked: the surface window tree during those CEF runs (probe.log)
  found: SucroseDesktopSurface 0x56000000 (no frame bits) > CefBrowserWindow 0x56010000 (no frame bits) > Chrome_WidgetWin_1 style 0x56050000 = WS_CHILD|WS_VISIBLE|WS_CLIPSIBLINGS|WS_CLIPCHILDREN|WS_THICKFRAME|WS_MAXIMIZEBOX, ex 0x00200100 (WS_EX_NOREDIRECTIONBITMAP|WS_EX_WINDOWEDGE) > Chrome_RenderWidgetHostHWND, plus later "Intermediate D3D Window" (GPU process, 1x1 then 1920x1080); every window reports client = window size once the probe sees it
  implication: Chrome_WidgetWin_1 is the only window with WS_THICKFRAME; its frame is not part of its non-client area any more when the probe looks, so the frame on screen is stale paint from an earlier non-client phase that nothing repaints until the GPU window presents

- timestamp: 2026-11-27T14:51:47.482Z
  checked: CDP (bun.exe target/dbg-wlb/cdp.ts, developer port 9555) during CEF run 3
  found: the page target appears with readyState loading, then html/body 1920x1080 background rgb(14,17,22), #loader 0 0 1920 1080 fixed with the same dark background, elementFromPoint at the edges is the page's own elements; the page never paints white
  implication: page-level causes (default body margin, white element) are excluded for Critter Crossing on CEF

- timestamp: 2026-11-27T14:51:47.482Z
  checked: experiment 2 - same build and theme on WebView2 (cap-wv2-critter)
  found: no frame in any capture; the WebView2 windows have no WS_THICKFRAME (Chrome_WidgetWin_0 0x46010000 -> 0x56010000 in sucrose.exe, Chrome_WidgetWin_1 0x46000000 -> 0x56000000 and Intermediate D3D Window in msedgewebview2.exe); black until the D3D window appears, then the loader 0E1116 edge to edge, then the scene
  implication: H1 explains CEF only; the WebView2 report needs another theme or another cause

- timestamp: 2026-11-27T15:28:54.460Z
  checked: maintainer desktop check by the orchestrator (release sucrose.exe built from the uncommitted fix at 18:16:42 +03; data dir target/round-b with the 8 maintainer Web themes and the YouTube Showcase themes; Engine.Web, Engine.Url and Engine.YouTube = CefSharpLive; Portal with focus from 2026-11-27 18:17:47 +03, one monitor; logs target/round-b-logs-d2; stopped with both taskkill commands, tasklist empty)
  found: maintainer verdict "çerçeve artık görünmüyor" (the frame no longer shows: CEF Web themes and a YouTube wallpaper) and "webview2 de de çerçeve yok" (no frame on WebView2 either)
  implication: the fix is confirmed on CEF in the maintainer's real workflow; WebView2 shows no frame, which matches the probe (its windows have no WS_THICKFRAME) and the inference that the WebView2 part of the 04-37 report came from the CEF YouTube wallpapers that ran between the WebView2 Web themes (Engine.YouTube stayed on CEF in the 04-37 A3 run)

- timestamp: 2026-11-27T15:28:54.460Z
  checked: closeout gates from the repo root (the last green run could not be shown to postdate the 18:14:53 browser.rs edit): cargo fmt --all --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace
  found: fmt clean, clippy clean, 1678 passed / 0 failed / 29 ignored in 56 suites, including the three browser::imp::tests regression tests
  implication: the committed tree is green

## Resolution

root_cause: "CEF only (every CEF wallpaper). CEF's windowed browser (cef set_as_child: WS_VISIBLE) hosts Chromium's widget window Chrome_WidgetWin_1 with WS_THICKFRAME (CefWindowDelegateView SetCanResize(true)); the widget removes its non-client area through WM_NCCALCSIZE but keeps the style, so Windows paints the classic 7 px sizing frame (EDGE_RAISED + COLOR_ACTIVEBORDER) of that child window into the layered desktop surface - at creation while the window is visible, and again later through DefWindowProc on non-client messages (WM_NCACTIVATE-like: title, size and client size unchanged); the widget never paints its client area with GDI, so the frame stays on screen until the GPU process's Intermediate D3D Window presents the page's first frame (1-3 s, longer on a cold start). AND-gate: the visible paint (code) plus the gap before the GPU window covers it (environment)."
fix: "crates/sucrose-engine-cef/src/browser.rs: wallpaper_window_info creates the browser window without WS_VISIBLE; after browser_host_create_browser_sync, show_browser_window takes WS_THICKFRAME off Chrome_WidgetWin_1 (drop_sizing_frame: SetWindowLongPtrW + SetWindowPos SWP_FRAMECHANGED|SWP_NOMOVE|SWP_NOSIZE|SWP_NOZORDER|SWP_NOACTIVATE), then ShowWindow(SW_SHOWNOACTIVATE), logging a warning if the widget still has a non-client frame. Docs: docs/FEATURES.md Web row, docs/ARCHITECTURE.md CEF row. Committed as 6c6d9a3c (fix(cef): create the wallpaper browser hidden and drop the widget's sizing frame)."
oracle_type: "derived (window-style contract: the browser window info has no WS_VISIBLE; a WS_THICKFRAME child has no style bit and no non-client size after drop_sizing_frame) plus the desktop probe as the specified oracle (no captured edge pixel run of the classic frame)"
verification:
  target_test: { result: pass, note: "unit: the_wallpaper_browser_window_is_made_hidden_as_a_child_of_the_surface, only_the_sizing_frame_is_taken_off_a_window_style, a_widget_without_its_sizing_frame_has_no_frame_left_to_paint; desktop probe: 0 framed captures in 10/10 + 2/2 CEF starts" }
  mutation_check: { result: pass, reason_if_skipped: "no Stryker for Rust; 2 manual mutants at the fix sites", mutant_killed: true }
  no_op_deletion: { result: pass, deletion_justified_by_rca: true, note: "adds behaviour (hidden creation, style change, show); removes no logic" }
  adjacent_tests: { result: pass, suites_run: ["cargo test --workspace: 1678 passed, 0 failed (re-run at closeout)", "cargo clippy --workspace --all-targets -D warnings (re-run at closeout)", "cargo fmt --all --check"] }
  revert_and_reconfirm: { result: pass, bug_returned_on_revert: true, fixed_on_reapply: true, note: "revert 27/27 and 3/40 framed captures, reapply 0/25 and 0/40" }
  guardrail_verdict: accepted
human_verification: "maintainer, 2026-11-27 18:17 +03 (release build of the fix, target/round-b, Web/Url/YouTube on CefSharpLive, logs target/round-b-logs-d2): CEF confirmed, the frame no longer shows on the Web themes or the YouTube wallpaper; WebView2 shows no frame either"
webview2_part: "not a WebView2 defect: the WebView2 windows have no WS_THICKFRAME and no probe capture showed a frame; the WebView2 half of the 04-37 report is explained by the CEF YouTube wallpapers that ran between the WebView2 Web themes (Engine.YouTube stayed on CEF in the 04-37 A3 run), and the maintainer saw no frame on WebView2 in the confirmation run"
commit: 6c6d9a3c
files_changed: [crates/sucrose-engine-cef/src/browser.rs, docs/FEATURES.md, docs/ARCHITECTURE.md]

## Reasoning Checkpoint (history)

reasoning_checkpoint:
  hypothesis: "CEF only: the wallpaper browser window is created visible (cef set_as_child adds WS_VISIBLE), so while Chromium creates its Chrome_WidgetWin_1 child (WS_THICKFRAME; its first WM_NCCALCSIZE keeps the default sizing frame because CEF's remove_standard_frame only applies to later frame changes) Windows paints the classic 7 px thick frame into the layered desktop surface; Chromium then drops the non-client area but never paints its client area with GDI, so the stale frame pixels stay on screen until the GPU process's Intermediate D3D Window presents the first page frame (1-3 s while the wallpaper loads)"
  confirming_evidence:
    - "PrintWindow captures (surface and composed Progman) in 3 CEF runs: black with a 7 px border whose pixels are EDGE_RAISED (E3E3E3 FFFFFF / A0A0A0 696969) plus 5 px COLOR_ACTIVEBORDER B4B4B4 = the classic WS_THICKFRAME sizing border; gone in the first capture after the Intermediate D3D Window reaches 1920x1080"
    - "window tree: Chrome_WidgetWin_1 0x56050000 is the only window with WS_THICKFRAME; all windows report client == window size while the frame is still captured, so the frame is stale paint, not a live non-client area"
    - "WebView2 (4 themes): its windows have no WS_THICKFRAME and no capture shows a frame; CDP shows the CEF page itself never paints white"
  falsification_test: "build with the wallpaper browser window created without WS_VISIBLE and shown right after browser_host_create_browser_sync returns; if the same probe still captures the 7 px B4B4B4 frame on CEF, the creation-time paint is not the source and H1 is wrong"
  fix_rationale: "a hidden parent makes Chromium's widget invisible during its creation, so the non-client frame is never painted; when the window is shown after creation the widget's non-client area is already 0 (verified by a debug log of the widget's frame at show time), so nothing paints a frame and the surface stays black until the page's first GPU frame; this removes the source of the pixels instead of covering them"
  blind_spots: "whether the widget's non-client area is always 0 when create_browser_sync returns (Chromium-internal timing; logged); whether a browser created in a hidden window starts rendering normally once shown (checked by the same run: page loaded and scene visible); multi-monitor/DPI (DISPLAY9 is not attached now); the maintainer's WebView2 report is not reproduced: in the 04-37 A3 log the YouTube wallpapers between the WebView2 Web themes still ran on CEF"
  candidate_causes:
    - "code: the CEF host window is created visible (set_as_child style includes WS_VISIBLE) around a Chromium widget that has WS_THICKFRAME at creation"
    - "environment: the layered desktop surface keeps the stale GDI pixels, and the GPU process window needs 1-3 s before its first present"
    - "config: embedder background colour (eliminated: CEF and WebView2 paint black, surface brush black)"
    - "data: theme CSS / default body margin (eliminated: CDP shows full-viewport dark loaders, no white)"
  and_gate: "yes: the frame needs both the visible creation of the WS_THICKFRAME widget (code) and a gap before the GPU window covers it (environment); only the first is ours to change, and removing it removes the paint regardless of the gap"
