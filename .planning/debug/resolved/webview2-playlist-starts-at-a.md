---
status: resolved
trigger: "WebView2 playlist starts at a random index with Shuffle Mode off (P1a-WV2, 04-39)"
created: 2026-11-28T01:44:41.568Z
updated: 2026-11-28T02:14:59.145Z
goal: find_and_fix
symptoms_prefilled: true
---

## Current Focus

hypothesis: CONFIRMED and FIXED (see Resolution); maintainer confirmed ("Kabul, commit") on the machine proofs
test: none pending
expecting: n/a
next_action: none — session resolved and archived to .planning/debug/resolved/webview2-playlist-starts-at-a.md; TYPE-05 and ENG-02 are re-marked by the orchestrator

## Symptoms

expected: With Shuffle Mode off (Engine.WallpaperShuffle = false), a YouTube playlist wallpaper (Type 4, IFrame API page, Source https://www.youtube.com/playlist?list=...) starts at the list's first video (index 0) on WebView2, as it does on CEF (4 of 4 runs in 04-39) and as the C# app does (TYPE-05 "playlist, shuffle"; ENG-02 on WebView2).
actual: 04-39 Task 1 probe P1a-WV2 (2026-11-28, release build of HEAD b5eec90f), list PLATNX_t55hdpWZS7sShzhjQIw5q6WEG5w (29 videos): run 1 (probe started 00:47:50Z) startIndex 17 ("You Won't Believe What Happens Near a Black Hole"), page global shuffleMode read false afterwards, advance 17->18 and the list loop (28->0) correct, indicesSeen [17, 18, 28, 0], scale(1). Reruns 1 (00:48:13Z) and 2 (00:48:28Z): startIndex 0 ("Dark King Crown Red Aesthetic Live Wallpaper"), advance 0->1, loop ok. Intermittent: 1 of 3 on WebView2, 0 of 4 on CEF. P1c-WV2 (Shuffle on) started at 26 as expected. Maintainer eye check B4-WV2 was with Shuffle on (random start expected) and passed; B4-CEF with Shuffle off started at the first video.
errors: none. No panicked line; no ERROR line tied to the YouTube page in target/round-b-recheck-logs (04-39 L1). The probe records are in target/round-b-recheck-probe/playlist-wv2.json, p1a-wv2*.json and their .engine.txt files (engine started engine="WebView2").
timeline: first observed in 04-39 Task 1 (2026-11-28); 04-37 had no playlist probe. YouTube on WebView2 since 04-35 (45d79dc0); the player-UI fix adfb0922 and the WebView2 browser-wait fix c1ac7f05 are in the build. Never seen on CEF.
reproduction: release build; scratch data folder target/round-b-recheck (settings migrated read-only from the maintainer's real settings; library target/round-b-lib/Library with the youtube-playlist theme); in target/round-b-recheck/Setting/Sucrose.json set Engine.YouTube = WebViewLive, Engine.WallpaperShuffle = false, Engine.WallpaperLoop = true, Engine.DeveloperPort = 9422, Library.Selected = youtube-playlist; start `target/release/sucrose.exe --no-portal` with SUCROSE_DATA_DIR="$(wslpath -w target/round-b-recheck)" and WSLENV=SUCROSE_DATA_DIR; run the CDP probe with the Windows bun.exe from the repository root: `bun.exe target/round-b-recheck-probe/yt.ts ...` (it reads http://127.0.0.1:9422/json/list and only uses Runtime.evaluate on the page's global `player`); stop with `taskkill.exe /IM sucrose.exe /F`. Restore DeveloperPort, WallpaperShuffle, WallpaperLoop, Engine.YouTube and Library.Selected afterwards (the current values: target/round-b-recheck-Sucrose.json.pre-addendum is the state after 04-39; Sucrose.migrated.json in the probe folder holds the migrated originals). Intermittent: repeat starts to see it.

## Constraints (from the maintainer and the project)

- Unattended WebView2 runs are allowed (maintainer, 2026-11-28): scratch data only, `sucrose.exe --no-portal` (no Portal, takes no focus), short YouTube wallpapers on the desktop, no synthesized mouse or keyboard input, every start stopped with `taskkill.exe /IM sucrose.exe /F` and confirmed with `tasklist.exe`. Never kill msedgewebview2.exe by name. The C# app must be closed during runs: check `tasklist.exe` for Sucrose.* first and ask the orchestrator if it runs (never kill it).
- WSL cannot reach Windows 127.0.0.1 ports: CDP probes run with the Windows `bun.exe`. Bun is the only JS runtime (no node/npx for project work).
- The maintainer's real data (%APPDATA%\Sucrose, H:\Sucrose) is read-only; target/round-b (04-38) is not touched.
- Legacy C# (src/) is read-only reference: compare how the C# app sets shuffle/loop on its YouTube page (timing relative to page load) before choosing a fix; keep the page's JavaScript behaviour compatible (D-007).
- Line endings: keep each file's endings (`git ls-files --eol` before editing; restore CRLF on w/crlf files before commits; the Windows cargo fmt rewrites CRLF to LF). Commit messages through several -m flags. Do not push.
- Before a fix commit: cargo fmt --all, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace (known flaky: portal.rs timing tests, memory_detail_counts_live_heap_blocks, service::tests::a_restart_of_another_engine_closes_nothing_and_one_during_a_library_move_is_refused; rerun once).
- Records: a fix that changes behaviour goes into docs/FEATURES.md (YouTube row; CRLF, no final newline) and, if it is a decision, docs/DECISIONS.md (LF). After the fix, the re-proof is P1a-WV2 repeated (for example 10 starts with Shuffle off on WebView2, all at index 0) plus P1a on CEF once; TYPE-05 and ENG-02 are then re-marked by the orchestrator/maintainer, not by this session.

## Eliminated

- hypothesis: the random start is WebView2-specific (WebView2 delivers setShuffle later than CEF)
  evidence: 2 instrumented CEF starts show the same order and the same ~195 ms margin (playerReady 1602/1375, load 1657/1449, setShuffle 1676/1470, first -1 1871/1665 ms); CEF is exposed to the same race, it just has not lost it yet
  timestamp: 2026-11-28T01:54:45.051Z

- hypothesis: YouTube's player itself starts the list at another index (no page code involved)
  evidence: in every traced random start the index comes from the page's own player.playVideoAt(random) in the first -1 branch, called while shuffleMode was still the default true
  timestamp: 2026-11-28T01:54:45.051Z

## Evidence

- timestamp: 2026-11-28T01:48:20.929Z
  checked: knowledge base (.planning/debug/knowledge-base.md)
  found: no entry about YouTube, playlist, shuffle or a page-call race; the closest entries are CEF-specific (shutdown dialog, web themes stuck loading)
  implication: no known-pattern candidate; investigate fresh

- timestamp: 2026-11-28T01:48:20.929Z
  checked: crates/sucrose-engine-web/src/pages/YouTubeContent.html, crates/sucrose-engine-web/src/page.rs (engine_calls/page_calls), crates/sucrose-engine-webview2/src/host.rs:975-1009, crates/sucrose-engine-cef/src/browser.rs:543-551
  found: the page declares `var shuffleMode = true, prevIndex = -1, first = true`; on the first -1 state with a playlist it calls playVideoAt(Math.floor(Math.random()*n)) when shuffleMode is true. The only writer of shuffleMode is setShuffle(e). Both engines send the engine calls (SucroseVolume, setLoop, setShuffle, SucroseStretchMode) exactly once per main-frame load, from the load-finished handler (WebView2 NavigationCompleted, CEF OnLoadEnd), via ExecuteScript; a runtime change sends setShuffle again (Command::Shuffle)
  implication: the page's start decision depends on whether an asynchronous host-side script arrives before the player's first state change; nothing in the page waits for it

- timestamp: 2026-11-28T01:48:20.929Z
  checked: legacy C# src/Shared/Engine/Sucrose.Shared.Engine.WebView/View/YouTube.xaml.cs, Event/YouTube.cs, Helper/YouTube.cs and the CefSharp twins; Helper/Source.cs (legacy page text)
  found: the legacy page has the same `var shuffleMode=!0 ... first&&-1===e.data&&(first=!1,shuffleMode)` logic. The legacy WebView engine sets SSEMI.Initialized at DOMContentLoaded (CefSharp: IsBrowserInitialized) and a 1-second GeneralTimer then calls playFirst(), setLoop, setVolume and setShuffle on EVERY tick; so the legacy app has the same latent race (setShuffle reaches the page 0-1 s after DOMContentLoaded) and only usually wins it
  implication: matching the legacy timing would not remove the race; a fix should make the page's first decision independent of when the engine's setShuffle arrives while keeping the legacy function names and semantics (D-007)

- timestamp: 2026-11-28T01:52:10.825Z
  checked: 9 WebView2 starts (release build of b5eec90f, Shuffle off) with an instrumented copy of the page in the scratch theme folder (target/debug-wv2-shuffle/wv2-1..9.json, .log.txt)
  found: 9/9 started at index 0. Every trace has the same order: onPlayerReady (loadPlaylist) at 594-2154 ms, window load 15-95 ms after it (twice 11-15 ms before it), setShuffle(false) 10-32 ms after load (engine log: page call key="shuffle" right after "page loaded"), and the first -1 state 165-250 ms after setShuffle, read with shuffleMode=false. No playVideoAt call. The margin that protects the start is only ~150-250 ms, and it depends on the window load event (which waits for the player iframe) plus the host's NavigationCompleted handling and ExecuteScript IPC
  implication: the order "setShuffle before first -1" is not guaranteed by anything; a load or host delay of ~0.2 s more than usual flips it, which matches the intermittent 1-of-3 failure in 04-39 (the probe read shuffleMode=false afterwards, i.e. setShuffle(false) did arrive, just late)

- timestamp: 2026-11-28T01:54:45.051Z
  checked: fault injection on WebView2 — instrumented page with arriving setShuffle applied 400 ms late (target/debug-wv2-shuffle/YouTubeContent.delayed.html; runs wv2-delayed-10, -11)
  found: run 10: playerReady 1672, load 1754, setShuffle(false) arrived 1797, first -1 at 1957 with shuffleMode=true → playVideoAt(21), applied setShuffle(false) at 2210, playing at index 21; run 11: same order → playVideoAt(8), startIndex 8. Afterwards shuffleMode reads false in both
  implication: H1 confirmed: a late setShuffle alone reproduces the 04-39 symptom exactly (random start, shuffleMode false afterwards)

- timestamp: 2026-11-28T01:54:45.051Z
  checked: 2 instrumented CEF starts (cef-12, cef-13), Shuffle off
  found: index 0 both; order and margin identical to WebView2 (first -1 ~195 ms after setShuffle)
  implication: the race is in the shared page, not in a WebView2-only path; the fix belongs in YouTubeContent.html so both engines get it

- timestamp: 2026-11-28T02:07:20.471Z
  checked: unit gate — cargo test -p sucrose-engine-web --test pages a_playlist_makes_its_random_first_start_only_once_the_engine_said_it_shuffles against HEAD's page and three fix-site mutants
  found: green on the fixed page; red on HEAD's page (pages.rs:545) and on each mutant (first -1 branch deciding on the default → :555; setShuffle without the pending start → :566; pending start ignoring the value → :566)
  implication: the regression test asserts the root-cause gate, not the symptom

- timestamp: 2026-11-28T02:07:20.471Z
  checked: workspace gates — cargo fmt --all (only the new test reformatted), cargo fmt --check, cargo clippy --workspace --all-targets -D warnings, cargo test --workspace
  found: fmt/clippy clean; first cargo test --workspace run exited 101 with its output truncated by the tool (failing test not identifiable); the rerun into target/debug-wv2-shuffle/test-workspace.log passed everything (exit 0), consistent with the known flaky timing tests
  implication: no neighbour broken

- timestamp: 2026-11-28T02:07:20.471Z
  checked: live re-proof, release build of sucrose + sucrose-portal with the fix (sucrose.exe 2026-11-28 05:00:33 +03:00, "shuffleTold" ×3 embedded; scratch Content/YouTubeContent.html rewritten with the fix), scratch data target/round-b-recheck, records target/debug-wv2-shuffle/fix-*.json
  found: A) 10/10 P1a-WV2 (yt.ts playlist, Shuffle off, engine="WebView2"): startIndex 0, advance 0→1, list loop 28→0, indicesSeen [0,1,28,0], no ERROR/panic. B) fixed page with setShuffle applied 400 ms late: Shuffle off 2/2 at index 0 (first -1 traced with shuffleMode=true, no playVideoAt); Shuffle on 2/2 random (27, 9), playVideoAt traced at the setShuffle(true) arrival. C) P1c-WV2 (Shuffle on, real page): startIndex 19, shuffleMode true, same advance/loop pattern as the 04-39 P1c record (26→27). D) P1a-CEF (engine="Cef"): startIndex 0, advance 0→1, loop ok. Every start stopped with taskkill.exe /IM sucrose.exe /F, tasklist showed 0 sucrose.exe after each; msedgewebview2.exe count stayed at the 6 of the baseline; no C# Sucrose.* ran
  implication: the symptom is gone on both engines, including under the exact fault that reproduced it before the fix; Shuffle on keeps its random start

- timestamp: 2026-11-28T02:07:20.471Z
  checked: cleanup
  found: target/round-b-recheck/Setting/Sucrose.json restored byte-identical to target/round-b-recheck-Sucrose.json.pre-addendum, Sucrose.json.bak restored from its backup; the temporary YouTubeContent.html copies removed from target/round-b-lib/Library/youtube-playlist (only SucroseInfo.json, preview.gif, thumbnail.jpg remain). The scratch Renderer logs got the debug-level lines of these runs (SUCROSE_LOG with engine debug)
  implication: scratch state is as after 04-39

- timestamp: 2026-11-28T02:14:59.145Z
  checked: the exit 101 of the first cargo test --workspace run (reproduced by the orchestrator at 2026-11-28 05:08-05:10 +03:00, i.e. 02:08-02:10Z)
  found: the new test a_playlist_makes_its_random_first_start_only_once_the_engine_said_it_shuffles failed in cargo test -p sucrose-engine-web with the page text "if (shufflePending) { shufflePending = false; SucroseShuffleStart(); }" (a mutant), and two apps/sucrose/tests/portal.rs WebView2 exit-timing tests failed in the workspace run. Cause of the pages failure: stale test artifacts. YouTubeContent.html had been restored after the mutation runs with its old mtime (04:54:53 +03:00), older than the artifacts built from a mutant, so cargo reused them (the include_str dependency is judged fresh by mtime). target/release/sucrose.exe (05:00:33 +03:00) embeds the correct page (grep -c of "if (shufflePending) { shufflePending = false; if (shuffleMode) { SucroseShuffleStart(); } }" = 1, of the mutant = 0), so the live proofs ran on the real fix. After touch on YouTubeContent.html and tests/pages.rs: cargo test -p sucrose-engine-web all ok (pages 37 passed incl. the new test), every pages-*.exe embeds the correct text and no mutant, and cargo test --workspace exit 0 (portal.rs passed too; its earlier failures are the known WebView2 exit-timing flakiness under load, BROWSER_EXIT_WAIT 1 s, an open finding)
  implication: the first red run was a stale-artifact artefact of the mutation testing, not a defect of the fix; lesson: after mutation testing of an include_str'd file, restore it with a fresh mtime (plain cp or touch), never cp -p, or cargo keeps the mutant's artifacts

- timestamp: 2026-11-28T02:14:59.145Z
  checked: maintainer answer to the human-verify checkpoint (presented by the orchestrator with the root cause, the page-only fix and the machine proofs; an eye check was offered as optional)
  found: "Kabul, commit" (accept, commit): the fix is accepted on the machine proofs without an eye check
  implication: confirmed fixed; commit the three files and archive the session

## Resolution

root_cause: "YouTubeContent.html decides a playlist's first start on the first -1 state with `var shuffleMode = true` as its default, while the engines (CEF and WebView2 alike) send the real value with a single `setShuffle` call from the main-frame load handler (NavigationCompleted / OnLoadEnd → ExecuteScript). The first -1 state comes ~0.2 s after loadPlaylist and only 0.15-0.25 s after that call in normal starts; on a start where load + host handling + ExecuteScript are slower, the -1 state wins and the stale default runs playVideoAt(random), so a list with Shuffle off starts at a random video (and shuffleMode reads false afterwards). Two conditions (AND-gate): the page's order-dependent default (code) and a slower-than-usual load/host path (environment, not controllable); the legacy app has the same race via its 1 s timer"
fix: "The page no longer decides on the default: it tracks whether the engine has said the shuffle value (shuffleTold). A first -1 state before that leaves the list on its first video and marks the random start pending (shufflePending); setShuffle sets shuffleMode and shuffleTold and, if a start is pending, makes the random first start only when the value is true (new SucroseShuffleStart, the legacy random index). When setShuffle comes first (the usual case) behaviour is unchanged. Legacy function names, the shuffleMode global and the engine calls stay (D-007); no engine code changed. Recorded in the YouTube row of docs/FEATURES.md; commit 62ec2f49"
verification:
  target_test: { result: pass, test: "crates/sucrose-engine-web/tests/pages.rs::a_playlist_makes_its_random_first_start_only_once_the_engine_said_it_shuffles" }
  mutation_check: { result: pass, reason_if_skipped: "Stryker does not apply (Rust test over an embedded HTML page); replaced by 3 hand-seeded fix-site mutants", mutant_killed: "3/3 (branch decides on default; setShuffle drops the pending start; pending start ignores the value) + HEAD page red" }
  no_op_deletion: { result: pass, deletion_justified_by_rca: "n/a — the inline random start moves into SucroseShuffleStart and is called on both paths; a gate and a pending path are added, nothing is short-circuited" }
  adjacent_tests: { result: pass, suites_run: ["cargo test --workspace (rerun after one truncated failing run)", "cargo clippy --workspace --all-targets -D warnings", "cargo fmt --check", "orchestrator re-run after touch of the page and the test: cargo test -p sucrose-engine-web (pages 37 passed) and cargo test --workspace exit 0"], note: "the first exit 101 was stale test artifacts from the mutation runs (page restored with its old mtime, cargo reused a mutant's build of the include_str'd page) plus the known portal.rs WebView2 exit-timing flakiness; sucrose.exe used for the live proofs embeds the correct page (grep: fix text 1, mutant 0)" }
  revert_and_reconfirm: { result: pass, bug_returned_on_revert: true, fixed_on_reapply: true, how: "HEAD page + setShuffle 400 ms late → startIndex 21 and 8 (WebView2); fixed page + same delay → 0 and 0; unit test red on HEAD page, green on the fix" }
  live: "10/10 P1a-WV2 at index 0 with advance and loop; P1a-CEF at 0; P1c-WV2 random (19); fixed page with late setShuffle: off 2/2 at 0, on 2/2 random"
  guardrail_verdict: accepted
  human_verify: "maintainer: \"Kabul, commit\" (2026-11-28), accepted on the machine proofs without an eye check"
files_changed: [crates/sucrose-engine-web/src/pages/YouTubeContent.html, crates/sucrose-engine-web/tests/pages.rs, docs/FEATURES.md]
oracle_type: derived (structural contract of the page's first-start branch and setShuffle in the Rust test); behaviour proven live by event traces, fault injection and the P1a/P1c probes
