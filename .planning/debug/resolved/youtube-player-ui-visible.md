---
status: resolved
trigger: "04-37 D7: YouTube wallpapers show the player UI (title, play/pause icons, something at the bottom) on both web engines; the legacy app hides it. Still visible after the CEF fixes of 2026-11-27."
created: 2026-11-27T15:31:16.524Z
updated: 2026-11-27T17:05:57.338Z
goal: find_and_fix
symptoms_prefilled: true
---

## Current Focus

resolved: fix adfb0922 confirmed by the maintainer for normal playback on CEF and WebView2; pause/resume, end of video and captions machine-verified on both engines (2026-11-27)
next_action: none (session archived in .planning/debug/resolved/)
known follow-up (not this bug): the Rust web engines do not apply General.UserAgent (legacy sets it on CefSharp and WebView2); recorded in the FEATURES YouTube row as "Not yet" and in STATE.md "Small tasks after the 04-37 debug batch"

## Symptoms

expected: A YouTube wallpaper (Type 4; IFrame API page, no API key) plays the video with no player UI: no title bar, no play/pause or other controls, no YouTube logo, nothing at the bottom, on CEF and on WebView2, as the C# app does (TYPE-05 "hidden player UI"; the in-frame properties scale, filters, mirror, animation, caption should apply too).
actual: 04-37 B4 (2026-11-27): "youtube videoları açılıyor ama title ve oynatma durakaltma ikonları görünüyor. ekranın alt kısımlarında da bir şeyler görünüyor ama taskbardan ne olduğunu anlamadım. yine ui bileşenidir" = the YouTube videos open but the title and the play/pause icons are visible; something also shows at the bottom of the screen (behind the taskbar, probably UI too). Engines not distinguished in that reply (YouTube on CEF, then on WebView2). Again during the D2 check on CEF, after the CEF fixes (a0e22d9a restored --disable-web-security by adding --user-data-dir; 6c6d9a3c): "hala youtube ui bileşenleri görünmeye devam ediyor" = the YouTube UI components are still visible.
errors: 04-35 SUMMARY: CEF refused the player-frame access ("hideIframeElement" SecurityError) despite --disable-web-security, so the UI is not hidden and the in-frame properties do not apply on CEF; WebView2 frame access was not examined. (That was before --disable-web-security was actually reaching CEF; re-check.) No Rust ERROR in the logs.
timeline: YouTube wallpapers were built in 04-35 (CEF) and on WebView2 in 04-35 too (45d79dc0 "YouTube on WebView2 and the Referer fill hook"). Never seen hidden on either engine.
reproduction: release build (HEAD 4d985283 or later), scratch data dir, YouTube Engine CEF or WebView2, Use one of the Showcase YouTube themes ("Floating in Space", "Space Train", "Spirit Tree"; scratch copies are in target/round-b-lib/Library, settings in target/round-b) -> the player UI shows. Needs network access to youtube.com.

## Constraints (from the maintainer and the project)

- Desktop runs are allowed (bug fixing with wallpapers opening and closing was agreed). C# Sucrose stays closed. Runs: a new scratch SUCROSE_DATA_DIR under target/ (copy the Showcase theme folders from target/round-b-lib; never write under H:\Sucrose or %APPDATA%\Sucrose), WSLENV, Portal only with --no-focus (or sucrose.exe --no-portal with Library.Selected set in the scratch Setting/Sucrose.json), never synthesize input, captures only with .claude/skills/port-feature/capture.ps1 (PrintWindow) or CDP Page.captureScreenshot (read-only), stop with taskkill.exe /IM sucrose-portal.exe /F and /IM sucrose.exe /F, never kill msedgewebview2.exe by name, tasklist.exe empty at the end, stop apps before rebuilding.
- Docs: a change to the default Chromium switches or the web engine arguments touches D-021 (docs/DECISIONS.md, Accepted): say so and keep the legacy defaults unless the legacy app itself did otherwise; a needed decision change is a checkpoint for the maintainer.
- A visual confirmation by the maintainer is a checkpoint returned to the orchestrator; prefer machine evidence (CDP: the player iframe's DOM state, which .ytp-* elements are hidden, captures).

## Evidence

- timestamp: 2026-11-27T15:33:46.525Z
  checked: crates/sucrose-engine-web/src/pages/YouTubeContent.html vs legacy src/Shared/Engine/Sucrose.Shared.Engine/Helper/Source.cs GetYouTubeContent
  found: hideIframeElement is the legacy function verbatim (MutationObserver on iframe.contentDocument hiding every #movie_player descendant except the video container/video, keeping caption/role=status/aria-live elements and forcing them display:block z-index max). Rust only wraps the call in try/catch (console.warn "The player frame could not be reached: ...") and uses width/height 100% instead of the legacy {Width}/{Height} pixel sizes
  implication: the page logic is the legacy logic; the difference must be in whether the frame is reachable (engine/switches) or in what the YouTube DOM looks like now
- timestamp: 2026-11-27T15:33:46.525Z
  checked: crates/sucrose-engine-web/src/args.rs LEGACY_DEFAULTS, cef browser.rs on_before_command_line_processing, webview2 host.rs set_additional_browser_arguments
  found: both engines get the legacy defaults incl. disable-web-security, disable-site-isolation-trials, disable-features=...IsolateOrigins...; CEF also gets --user-data-dir since a0e22d9a; the legacy app used the same lists (Internal.cs WebArguments/CefArguments)
  implication: on paper the switches match the legacy app; whether Chromium honours them (site isolation off, frame in-process) must be observed at runtime

- timestamp: 2026-11-27T15:36:57.880Z
  checked: CEF run, release build 18:16 (= 6c6d9a3c code), target/yt-debug/data (Floating in Space, YouTube=CefSharpLive, DeveloperPort 9222), CDP probe target/yt-debug/probe/yt.ts 60 s after start
  found: the player frame (https://www.youtube.com/embed/znNllvBt_Jg) is in-process (no OOPIF target, same-renderer execution context 3); from the page contentDocument and contentWindow.document are reachable (no SecurityError); ready=true, customPlayer set, the Sucrose keyframes style is in the frame; inside #movie_player 9 of 11 descendants have inline display:none and only the VIDEO is displayed; #movie_player class has ytp-hide-controls (controls=0) and ytp-exp-bottom-control-flexbox
  implication: on CEF the frame access works and hideIframeElement does its legacy job; the visible UI is not inside #movie_player
- timestamp: 2026-11-27T15:36:57.880Z
  checked: the player frame body outside #movie_player (yt2.ts dump)
  found: the frame body has DIV#player-controls.ytPlayerControlsContainerHost (ytm-watch-player-controls: video-cover, cued-overlay play button, player-top-controls with embedded-player-video-details = video title "Floating In Space - 12 Hours..." + channel avatar, player-middle-controls with Previous / Pause / Next buttons, player-bottom-controls with the time display 1:01 / 12:00:00, yt-progress-bar, creator-endscreen with end screen cards) and DIV#bottom-sheet-wrapper, both siblings of DIV#player, i.e. outside #movie_player; at that moment (playing, ytp-autohide) the layer's containers are displayed full screen but their content was faded; the CDP screenshot showed the video only
  implication: these are exactly the reported elements (title, play/pause icons, something at the bottom = time pill/progress bar); the legacy script cannot hide them because it only walks #movie_player
- timestamp: 2026-11-27T15:36:57.880Z
  checked: navigator.userAgent in page and frame; legacy src for UserAgent; Rust crates for user_agent
  found: the Rust CEF page/frame UA is the Chromium default "Mozilla/5.0 (Windows NT 10.0; Win64; x64) ... Chrome/156.0.0.0 Safari/537.36"; the legacy app sets General.UserAgent (default "Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit") as CefSettings.UserAgent (Sucrose.Live.CefSharp App.xaml.cs:335) and CoreWebView2.Settings.UserAgent for every web type incl. YouTube (SE.WebView Event/YouTube.cs:131 etc.); in Rust general.user_agent exists only in sucrose-config (model/migrate), no engine reads it
  implication: a real parity difference in what YouTube sees; candidate cause of the different player UI

- timestamp: 2026-11-27T15:39:15.026Z
  checked: yt3.ts on CEF, Page.reload with the default Chrome 156 UA, samples at 1.5/3/5/8/12/20 s + CDP screenshots (target/yt-debug/probe/chrome-*.png)
  found: at 1.5-5 s the #player-controls layer shows (opacity 1): title "Floating In Space - 12 Hours - 4K Ultra HD 60fps" + VISUALDON + channel avatar top left, Previous / Pause / Next buttons in the centre, share button bottom left, "More videos" + YouTube logo bottom right (y 1020-1068, behind the taskbar); from ~8 s it fades out (ytp-autohide) leaving only an invisible 12x12 a11y button; chrome-3000.png shows exactly that UI over the video; inside #movie_player only the VIDEO is shown the whole time
  implication: the reported UI (title, play/pause icons, something at the bottom) is reproduced on CEF and is the #player-controls layer outside #movie_player
- timestamp: 2026-11-27T15:39:15.026Z
  checked: same with Network.setUserAgentOverride("Sucrose/2.4 (Windows NT 10.0; Wallpaper Engine) SucroseWebKit") + reload (requests carried that UA and no sec-ch-ua; YouTube then served player_embed.vflset/base.js instead of player_embed_es6)
  found: #player-controls is served and shown identically (title, Previous/Pause/Next, More videos) for the first ~5 s, then autohides
  implication: the UA does not decide whether YouTube serves the new controls layer; H1 eliminated as the cause of this symptom (the missing UA setting stays a separate parity gap)
- timestamp: 2026-11-27T15:39:15.026Z
  checked: caption containers and aria-live/role=status elements in the frame; the engine's own Light pause (SucrosePause() via Runtime.evaluate) then a look 6 s later
  found: nothing caption-like and no aria-live/role=status element outside #movie_player (#player-controls innerHTML has no "caption"); while paused (paused-mode) the layer content stays faded (only the invisible a11y button)
  implication: hiding the whole layer outside #movie_player does not touch captions (they render inside #movie_player, kept by the legacy rules); the layer is shown at the player start (and possibly on other player events), not held by the Light pause

- timestamp: 2026-11-27T15:41:38.253Z
  checked: WebView2 run (Engine.YouTube=WebViewLive, same scratch data dir, WebView2 port 9222): yt.ts and yt3.ts default
  found: the player frame is in-process and reachable on WebView2 too (contentDocument ok, Sucrose style injected, only the VIDEO displayed inside #movie_player); the frame body has the same DIV#player-controls and DIV#bottom-sheet-wrapper; after a reload the layer shows at 3-5 s (title, VISUALDON, Previous/Pause/Next "Videoyu duraklat", share, YouTube logo) and autohides by 8 s; one page exception "TypeError: Cannot read properties of null (reading 'querySelectorAll')" from the observer callback (#movie_player null while the frame was still being built)
  implication: same cause on both engines (one shared page) -> one fix and one commit; the legacy callback also throws while #movie_player does not exist yet
- timestamp: 2026-11-27T15:41:38.253Z
  checked: ancestor chain of #movie_player in the frame
  found: body > DIV#player > DIV#movie_player; the siblings along the chain are DIV#global-loader, DIV#player-controls, DIV#bottom-sheet-wrapper (plus SCRIPT/STYLE/NOSCRIPT); #movie_player has no siblings inside #player
  implication: hiding every non-script sibling along the path from #movie_player to body removes the whole new layer and keeps the player, its video and its captions

- timestamp: 2026-11-27T15:42:25.255Z
  checked: regression test crates/sucrose-engine-web/tests/pages.rs the_youtube_page_hides_the_controls_next_to_the_player_not_only_inside_it against the old page (temporarily restored) and the patched page
  found: RED with the old page (panicked "the pass hides what sits next to the player"); GREEN with the patch, the whole pages test binary 36/36 ok; the page script parses (bun new Function)
  implication: the test bites on the missing sibling pass; runtime proof still needed in both engines

- timestamp: 2026-11-27T15:45:59.707Z
  checked: patched build (release 26.7 s incremental) on WebView2: yt4.ts sampling every ~250 ms from the first moment the page target exists (start) and after Page.reload, 10-12 s each; CDP screenshots wvfix-*.png
  found: WebView2 start and reload: no element outside #movie_player displayed in any sample (#player-controls computed display none from the first sample with a player), inside #movie_player only VIDEO, video time advancing (0.6 -> 8.7 s), no page exception (the null guard removed the TypeError), customPlayer/customBody still set to the frame's #player/body (properties path intact); screenshots show the video only
  implication: the sibling pass removes the layer on WebView2
- timestamp: 2026-11-27T15:45:59.707Z
  checked: same on CEF (Engine.YouTube=CefSharpLive): start + reload; then yt5.ts, a per-animation-frame monitor installed in the YouTube frame before its scripts (Page.addScriptToEvaluateOnNewDocument) with a style/class MutationObserver on #player-controls
  found: CEF reload clean, but the start run caught #player-controls displayed (block, title/channel visible) in one sample at 1.1 s; the monitor shows why: YouTube resets the style attribute of #player-controls to "" together with its class at 730, 774 and 1273 ms after the frame starts; the pass only runs on the next childList mutation (1-13 ms later), so 2 animation frames (775, 782 ms) rendered the layer with the title at opacity 1
  implication: inline display:none alone is undone by YouTube's own style resets (attribute mutations the childList observer does not see); the hide must survive a style reset: tag the hidden elements and hide the tag through the injected Sucrose stylesheet as well

- timestamp: 2026-11-27T15:51:21.681Z
  checked: rebuilt with the marker rule; CEF (Floating in Space): yt4 start sampling + 4x yt5 per-frame monitor after reloads; WebView2: the same + one 60 s monitor; CEF with a second theme (Spirit Tree, v=DtE5Y5VGkjU): start + 2 monitors; screenshots cef2-*, wv2-*, cef-spirit-*.png
  found: 0 animation frames with #player-controls displayed and its title opaque in every monitored load (CEF ~1130-1187 frames per 9 s, WebView2 ~386-435 per 9 s and 2917 in 60 s), although YouTube still reset the controls' inline style 3 times per load (in the first second); the only "block" readings are before onReady with the layer's content not yet rendered (title opacity 0); inside #movie_player only the VIDEO (plus the 0-height video container) displays; video time advances on both engines; no page exception; screenshots show only the video (or black before the first video frame)
  implication: the fix holds on both engines, across loads and themes; the start flash found on CEF is gone
- timestamp: 2026-11-27T15:51:21.681Z
  checked: apps stopped (taskkill sucrose.exe), tasklist; remaining msedgewebview2.exe processes inspected by command line
  found: no sucrose.exe; the 6 msedgewebview2.exe belong to Windows (MicrosoftWindows.Client.CBS EBWebView profile), none to the scratch profile
  implication: desktop clean; nothing of ours left running

- timestamp: 2026-11-27T16:51:34.340Z
  checked: maintainer desktop check on the release build of adfb0922 (orchestrator run, data dir target/round-b, Showcase YouTube themes, YouTube Engine CEF then WebView2, Portal with focus, 2026-11-27 18:55:48 +03, one monitor; logs target/round-b-logs-d7)
  found: verbatim "1. hiç bir şey görünmedi. fakat altyazıları test etmedim. ayrıca duvar kağıdı durdurulduğunda ui tekrar görünüyor mu onu da test etmedim. 2. webview2 de sorunsuz" = (CEF) nothing showed; captions not tested; whether the UI shows again when the wallpaper is paused not tested; no problems on WebView2 either
  implication: normal playback is confirmed on both engines; pause/resume and captions are left to machine evidence

- timestamp: 2026-11-27T17:01:54.141Z
  checked: release build of adfb0922 (target/release/sucrose.exe 18:46:34, the page with the sibling pass and the marker rule), scratch target/yt-debug/data, Floating in Space, sucrose.exe --no-portal, CDP on 9222 driven with bun.exe (WSL cannot reach the Windows loopback); probe yt6.ts: per-animation-frame monitor in the YouTube frame that walks the rendered tree (pruning display:none and opacity < 0.05) and counts every visible element other than the video, #movie_player itself and caption elements; phases: playing 4 s, the service's Light pause script (PAUSE_SCRIPT from js.rs, which calls SucrosePause) 10 s, its RESUME_SCRIPT (SucrosePlay) 8 s, #movie_player.pauseVideo() in the frame 10 s, playVideo() 8 s, seekTo(duration - 6) through the end 22 s; CDP captures in every phase
  found: CEF: playing 573 frames/0 with UI, light-pause 1412 frames/0 with UI, light-resume 1150 frames/0 with UI, api-pause 1402 frames/0 with UI, api-play 1148 frames/0 with UI, end 3150 frames/0 with UI; WebView2: playing 171 frames/0 with UI, light-pause 1427 frames/0 with UI, light-resume 379 frames/0 with UI, api-pause 1433 frames/0 with UI, api-play 389 frames/0 with UI, end 1095 frames/0 with UI. While paused #movie_player was in paused-mode on every frame, the video element paused, player state 2 (Light pause also: SucrosePause flag set, visibilityState hidden); after resume and play the state went 3 -> 1 and the video time advanced. End: states 1 -> 0 (ENDED) -> 1 -> 3 -> 1 within 12 ms; one frame in ended-mode; the page's legacy single-video loop (setLoop forced false, then seekTo(0) + playVideo on ENDED) restarted the video from 0 (time 15.5 s 15 s later). No page exception on either engine. Captures (cefc-*, wvc-* in target/yt-debug/probe) show the video only in every phase
  implication: no player UI shows while paused, on resume, at the end of the video or after the loop restart, on either engine
- timestamp: 2026-11-27T17:01:54.141Z
  checked: positive controls in the same runs (synchronous scans inside one Runtime.evaluate, so the page's observer cannot run in between): a synthetic div with text added to the frame body; #player-controls / #bottom-sheet-wrapper with the mark and the inline display removed and put back in the same task, while playing, 3 s into a Light pause, 1.5 s after the resume, and every 200 ms for 9 s around the end of the video
  found: the scan reports the synthetic div (the monitor detects visible elements); without the hide nothing in the layer is visible while playing or paused (YouTube keeps its content faded), but 1.5 s after the resume YouTube shows 23 elements (title "Floating In Space - 12 Hours - 4K Ultra...", VISUALDON, Previous / Pause / Next video, Share, More videos, Watch on YouTube, the controls background), and around the end it shows its end screen card ("MEMORIES - 12 Hours - 4K Ultra HD", 12:00:00 badge) and after the loop restart the title again, in 38 of 38 samples; identical on CEF and WebView2; restoring the hide left 0 visible elements each time
  implication: resume and end of video are states in which YouTube does show the layer, so the 0-frame results above are the fix at work, not an idle layer; the pause/resume gap the maintainer named is closed by machine evidence

- timestamp: 2026-11-27T17:05:57.338Z
  checked: captions, probe yt7.ts on the same release build: scratch theme target/yt-debug/data/Library/Captioned-1 (a copy of Floating in Space with the source changed to the IFrame API documentation's sample video, which has an English caption track; the id stays out of the repository), the caption property turned on through the page's property call SucrosePropertyListener('caption', {type: checkbox, value: true}) (the PropertyListener template the engine uses; toggleCaptions loads the captions module and picks the navigator.language track, else the default or the first), per-frame monitor, frame DOM samples every 700 ms for 25 s, CDP captures once a caption window is shown and once a caption line is at the bottom, then the property turned off; WebView2 (navigator.language tr) twice, then CEF (en-US)
  found: WebView2: "Subtitles in 'tr' not found, switched to 'en'", track en; caption windows from 0.7 s with the spoken lines ("JEFF POSNICK: Hey, everybody.", "Welcome to this week's show of / YouTube Developers Live." ...), 1809 and 1853 of 1862 / 1890 frames with caption segments visible (48 px white on rgba(8,8,8,0.7), bottom centre, y 942-1058), 0 frames with any other UI. CEF: "Subtitles enabled in 'en'", 3563 of 3656 frames with captions, 0 with other UI. On both: .ytp-caption-window-container is inside #movie_player (#player > #movie_player > container) with the legacy inline display:block !important / z-index max, every .caption-window is inside #movie_player, no caption-class element exists outside it, so the sibling pass never touches captions; captures wvcap2-caption-line-1495.jpg and cefcap-caption-line-2854.jpg show the caption line over the video and nothing else; turning the property off removes every caption window within 6 s; no page exception. When captions are turned on YouTube also shows its caption notice ("English / Click [gear] for settings", tr: "İngilizce / Ayarlar için ... simgesini tıklayın") as a caption window at the top left for about 2-3 s; it has the caption-window class, so the legacy rules keep it exactly as the C# app does (gear icon hidden)
  implication: the caption property works on both engines and the fix leaves captions visible; the caption gap the maintainer named is closed by machine evidence; nothing to fix

## Eliminated

- hypothesis: H1 — the missing legacy General.UserAgent makes YouTube serve the new #player-controls layer
  evidence: with the Sucrose UA override (requests carry it, no client hints, a different base.js is served) YouTube still serves and shows #player-controls with the same title, Previous/Pause/Next and More videos for the first ~5 s
  timestamp: 2026-11-27T15:39:15.026Z

- hypothesis: H0 — the player frame is out of process or web security is still on, so hideIframeElement throws (the 04-35 SecurityError) and nothing is hidden
  evidence: on CEF (after a0e22d9a) and on WebView2 the YouTube frame is an in-process child (no OOPIF target), contentDocument and contentWindow.document are reachable, the Sucrose style is injected and inside #movie_player only the VIDEO is displayed
  timestamp: 2026-11-27T15:54:14.574Z

## Resolution

root_cause: "YouTube's embed now renders its player controls (title and channel, previous/play-pause/next, share, More videos, YouTube logo, time display, progress bar, end screen) in a layer #player-controls (+ #bottom-sheet-wrapper) that is a sibling of #player in the frame body, outside #movie_player; the legacy hideIframeElement (ported verbatim) only hides #movie_player's descendants, so on both engines, where the player frame is reachable, the layer showed for ~5-8 s after every start (and whenever YouTube shows its controls: on resume after a pause, at the end of the video, after the loop restart); AND YouTube resets that layer's inline style while it starts, so an inline-only hide still let 1-2 frames through on CEF"
fix: "crates/sucrose-engine-web/src/pages/YouTubeContent.html: hideIframeElement runs one pass at once and one per mutation batch; the pass keeps the legacy rules inside #movie_player, returns while #movie_player is missing (no TypeError), and calls SucroseHidePlayerSiblings, which hides every non-script sibling at each level from #movie_player up to the body with inline display:none !important plus a data-sucrose-hidden mark; the injected Sucrose style sheet starts with [data-sucrose-hidden]{display:none !important}. Commit adfb0922."
verification: |
  signal_1_regression_test: crates/sucrose-engine-web/tests/pages.rs the_youtube_page_hides_the_controls_next_to_the_player_not_only_inside_it - RED on the old page (panicked "the pass hides what sits next to the player"), GREEN on the fix; oracle_type: derived (the page structure the fix needs, from the CDP-observed DOM); boundary neighbours: the immediate pass, the missing-player guard, the style-reset mark, the legacy in-player rules kept
  signal_2_mutation: removing the sibling call or the mark makes the test fail (the RED run removed both); the marker assertion was added after the CEF flash was measured
  signal_3_revert: the unpatched release build (same scratch data, same themes) showed the layer at opacity 1 during 1.5-5 s on CEF and 3-5 s on WebView2 (title, Pause/Previous/Next, More videos, logo; chrome-3000.png); the inline-only first patch still showed 2 frames on CEF; the final build shows 0 frames; positive controls on the final build (the hide lifted for one synchronous scan) show YouTube's layer with 23 elements 1.5 s after a resume and its end screen card / title around the end of the video on both engines
  signal_4_diff: additive page change (one new function, a guard, an immediate pass, one CSS rule); no deletion of behaviour; legacy in-player rules untouched
  signal_5_suites: cargo fmt --all (no change), cargo clippy --workspace --all-targets -D warnings clean, cargo test --workspace all ok (0 failed) at adfb0922; no code changed after that
  desktop_machine_evidence_start: release build of the fix, scratch target/yt-debug/data; per-animation-frame monitor in the YouTube frame (yt5.ts): CEF Floating in Space 4 loads 0 frames shown, CEF Spirit Tree 2 loads 0, WebView2 4 loads 0 plus 60 s 0 (2917 frames); 250 ms samples from the first moment of a start: no element outside #movie_player displayed on either engine; video time advancing; no page exceptions; customPlayer/customBody still set (properties path intact); screenshots show the video only
  desktop_machine_evidence_pause_resume_end (yt6.ts, release adfb0922, Floating in Space): CEF playing 573 frames/0 with UI, light-pause 1412 frames/0 with UI, light-resume 1150 frames/0 with UI, api-pause 1402 frames/0 with UI, api-play 1148 frames/0 with UI, end 3150 frames/0 with UI; WebView2 playing 171 frames/0 with UI, light-pause 1427 frames/0 with UI, light-resume 379 frames/0 with UI, api-pause 1433 frames/0 with UI, api-play 389 frames/0 with UI, end 1095 frames/0 with UI; Light pause = the service's PAUSE_SCRIPT/RESUME_SCRIPT (SucrosePause/SucrosePlay), api = #movie_player.pauseVideo/playVideo; paused-mode and a paused video on every frame once the pause applied (the first frame of each pause phase was still playing); end: ENDED then the page's legacy single-video loop (seekTo(0) + playVideo) restarts from 0 within 12 ms, 1 frame in ended-mode; captures show the video only; no page exception
  desktop_machine_evidence_captions (yt7.ts, scratch theme Captioned-1 with a captioned public video, caption property on through SucrosePropertyListener): WebView2 (tr, fell back to en) 1853/1890 frames with caption lines, CEF (en) 3563/3656, 0 frames with other UI on both; the caption container and windows are inside #movie_player and keep the legacy display:block / z-index; captures show the caption line over the video; property off removes the windows; YouTube's caption notice (language + "Click [gear] for settings") shows top left for 2-3 s when captions turn on, kept by the legacy caption rules exactly as in the C# app
  guardrail_verdict: accepted
  maintainer_check: confirmed - release build of adfb0922, orchestrator run 2026-11-27 18:55:48 +03 (data dir target/round-b, Showcase YouTube themes, YouTube Engine CEF then WebView2, one monitor), verbatim "1. hiç bir şey görünmedi. fakat altyazıları test etmedim. ayrıca duvar kağıdı durdurulduğunda ui tekrar görünüyor mu onu da test etmedim. 2. webview2 de sorunsuz" (normal playback: nothing showed on CEF, no problems on WebView2); captions and pause/resume, which the maintainer did not test, are machine-verified above (no maintainer look needed)
files_changed:
  - crates/sucrose-engine-web/src/pages/YouTubeContent.html
  - crates/sucrose-engine-web/tests/pages.rs
  - docs/FEATURES.md
scratch: "target/yt-debug (data dir, Captioned-1 scratch theme, probes yt*.ts, monitor.js, captures); deleted at the end of the session"
