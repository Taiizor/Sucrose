---
phase: quick-261127-uou
plan: 01
type: execute
wave: 2
depends_on: ["261127-uos"]
files_modified:
  - apps/sucrose/tests/portal.rs
  - .planning/phases/04-m3-engines-and-the-wallpaper-api/deferred-items.md
autonomous: false

must_haves:
  truths:
    - "With one monitor, cargo test -p sucrose --test portal passes as before and its real-desktop tests run instead of skipping (no '<test>: skipped,' line with --nocapture)"
    - "With the virtual monitor DISPLAY9 attached (two monitors), cargo test -p sucrose --test portal passes twice in a row, and the scratch logs of engine-restart and webview2-switch hold a Renderer-1 log with engine starts next to Renderer-0; cargo test --workspace passes in the same session"
    - "The wait for engine starts in portal.rs ends only when the renderer of every monitor slot (one renderer per connected monitor, the count Platform::monitors() reports) logged at least the asked number of starts; no test counts engine starts over the joined logs of all renderers"
    - "switching_web_themes_on_webview2_closes_each_renderer_cleanly still fails on any WARN in any slot's renderer log; the assertion is not filtered, allow-listed or dropped (the D10 WARN is a product defect fixed by 261127-uos)"
    - "Pure tests reproduce the 04-37 failure shape without a display: two slots with one start each satisfy a count of one but not of two, and one slot with two starts does not satisfy two expected slots"
    - "The three ignored desktop tests that counted engine starts (restart_now_restarts_only_the_affected_wallpapers, saved_webview2_arguments_reach_the_restarted_renderer, a_gif_engine_change_moves_the_running_gif_to_the_built_in_renderer) assert per slot and pass on one monitor and on two"
  artifacts:
    - path: apps/sucrose/tests/portal.rs
      provides: "per-slot renderer log helpers (slot_of, logs_by_slot, renderer_logs_by_slot, starts, every_slot_started, monitor_slots, wait_for_engine_starts_on_every_slot), their pure tests, and every engine-start assertion per slot"
      contains: "fn renderer_logs_by_slot("
    - path: .planning/phases/04-m3-engines-and-the-wallpaper-api/deferred-items.md
      provides: "the two portal.rs entries closed with the commit and the two-monitor run, or marked pending when the run was skipped"
      contains: "261127-uou"
  key_links:
    - from: apps/sucrose/tests/portal.rs
      to: crates/sucrose-platform/src/lib.rs
      via: "Platform::monitors() length = the number of renderer slots a Use starts (apps/sucrose/src/service.rs spawns for with_area(platform.monitors(), screen))"
    - from: apps/sucrose/tests/portal.rs
      to: apps/sucrose/src/main.rs
      via: "renderer log file name: sucrose_log::init(\"Renderer-{slot}\") writes <data>/Log/Renderer-<slot>.<YYYY-MM-DD>.log, appended by every renderer of that slot"
    - from: apps/sucrose/tests/portal.rs
      to: apps/sucrose/src/render.rs
      via: "the line 'engine started engine=\"<Cef|WebView2|Mpv|BuiltInGif>\"' (render.rs:174), which carries no slot field"
---

<objective>
Make the real-desktop tests of apps/sucrose/tests/portal.rs count engine starts per renderer slot instead of over the joined logs of every renderer, so `cargo test --workspace` passes with the virtual monitor DISPLAY9 attached exactly as it does with one monitor, without weakening any assertion.

Purpose: closing 04-37 with DISPLAY9 attached, `an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen` failed twice (portal.rs:745): the first Use already logs two engine starts (one CEF renderer per monitor), so the wait for the restart returned before the WebView2 renderers existed. `switching_web_themes_on_webview2_closes_each_renderer_cleanly` read its logs at the wrong moment for the same reason and failed on the real D10 WARN of slot 1 (os error 32), which item 261127-uos fixes in the product (deferred-items.md; STATE.md Pending Todos (c)). The maintainer keeps DISPLAY9 for multi-monitor work (memory: maintainer virtual display), so the gate has to hold with it. This item depends on 261127-uos: the switching test keeps its no-WARN check, so it can pass with two monitors only once the D10 fix is in HEAD.

Output: per-slot log helpers with pure tests, every engine-start assertion of portal.rs per slot, a one-monitor run and a two-monitor run on the maintainer's machine, the portal.rs entries of deferred-items.md closed.
</objective>

<execution_context>
@~/.claude/gsd-core/workflows/execute-plan.md
@~/.claude/gsd-core/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@CLAUDE.md
@.claude/rules/rust.md
@.planning/phases/04-m3-engines-and-the-wallpaper-api/deferred-items.md
@apps/sucrose/tests/portal.rs
@apps/sucrose/tests/common/mod.rs

Evidence of the failure: target/ci-0437-nf.log lines 455-540 (untracked; two slots, slot 0 DISPLAY1 persistent, slot 1 DISPLAY9 ephemeral, both renderers still on CEF when the WebView2 check ran) and .planning/phases/04-m3-engines-and-the-wallpaper-api/04-37-SUMMARY.md line 182.

Interfaces the executor needs (read from the code at d1d76bd7):
- Renderer log files: apps/sucrose/src/main.rs:72 calls sucrose_log::init with "Renderer-{slot}"; crates/sucrose-log/src/lib.rs rolls DAILY with prefix "Renderer-<slot>" and suffix "log", so every renderer process of one slot appends to <data>/Log/Renderer-<slot>.<YYYY-MM-DD>.log (a run across midnight gives two files for one slot). Other files in the same folder: CefRenderer-<slot>.log (CEF's own log, not a renderer log) and Service.<date>.log.
- The start line: apps/sucrose/src/render.rs:174 logs "engine started" with the field engine="Cef" | "WebView2" | "Mpv" | "BuiltInGif". It has no slot field; each renderer also logs "assignment received slot=<n> monitor=<id>". The slot is taken from the file name.
- One renderer per connected monitor: apps/sucrose/src/service.rs:1114-1118 and :3571 start a renderer for each entry of with_area(platform.monitors(), screen). The test process reaches the same list with sucrose_platform::current().monitors() (crates/sucrose-platform/src/lib.rs:395, primary first, empty where the OS cannot enumerate); common/mod.rs already calls sucrose_platform::current().capabilities(), so no import or dependency is new (methods of a dyn Platform need no trait import).
- Skips: common::service_runs and common::skipped print "<test>: skipped, <why>" on stderr and the test then passes. A run where libcef.dll or libmpv-2.dll is missing next to target/debug/sucrose.exe (or the WebView2 Runtime is missing) passes without exercising anything, so every run below uses --nocapture and checks for that line.
- Current callers of the old joined-count wait helper (portal.rs:700-715): an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen (737, 744), a_gif_engine_change_moves_the_running_gif_to_the_built_in_renderer (776, 784, ignored), switching_web_themes_on_webview2_closes_each_renderer_cleanly (856, 858, 861), saved_webview2_arguments_reach_the_restarted_renderer (1003, 1072, ignored). restart_now_restarts_only_the_affected_wallpapers (887-984, ignored) has its own local wait closure with the same joined count, plus two joined "exactly one start" checks; saved_webview2_arguments_reach_the_restarted_renderer has one more joined "exactly one start" check (1064) and a joined count of chromium switches lines (1075). 261127-uos lands first and may add callers; convert those too.
- Left unchanged on purpose (they hold with any number of monitors): use_plays_a_web_theme and a_loop_change_reaches_the_running_renderer (presence of one start), open_devtools_roundtrip (asks the monitor named first in the runtime state and checks that a renderer logged the DevTools open).
</context>

<assumption_delta_decision>
Primary noun: the renderer slot (one per connected monitor, as the service spawns them), no longer the joined text of all renderer logs.
Decision: promote. The wait helper returns the logs by slot and waits for every monitor's slot; every count assertion reads one slot's log; no count over the joined logs remains. The joined renderer_logs stays only for presence checks that hold with one or many monitors.
Invariant test: every_monitor_slot_has_to_reach_the_count (two slots with one start each never satisfy a count of two), added in Task 1.
</assumption_delta_decision>

## Run rules (every task)

- Desktop runs are allowed for verification. The tests start sucrose.exe on scratch folders under the cargo target directory (CARGO_TARGET_TMPDIR, <target>/tmp/portal/<name>); never run a test that touches the maintainer's real data: use_switches_between_video_themes (drives the real service of SUCROSE_DATA_DIR) and a_loop_change_reaches_the_running_renderer (migrates the real legacy settings) are never run, and ignored tests are only run by name with --ignored, never the whole binary with --include-ignored.
- After every run: tasklist.exe /FI "IMAGENAME eq sucrose.exe" shows no process; otherwise taskkill.exe /IM sucrose-portal.exe /F and taskkill.exe /IM sucrose.exe /F. Never kill msedgewebview2.exe by name. No Portal is opened by any test (--no-portal).
- Engines: before the first desktop run, the target directory's debug folder must hold libcef.dll and libmpv-2.dll (present in the main checkout); in a worktree with its own target directory run cargo xtask fetch-engines once (pinned, hash-verified, D-028). The WebView2 Runtime is user-installed and present on this machine.
- Line endings: apps/sucrose/tests/portal.rs and the deferred-items.md file are w/lf and stay LF. Before cargo fmt --all, save the list of w/crlf files from git ls-files --eol; afterwards, any file outside files_modified that turned w/lf gets CRLF back with sed -i 's/\r\?$/\r/' <file> (memory: gsd-tools-lf-rewrite; check tail -c 2 afterwards) and must show no content diff. Do not edit STATE.md: the batch orchestrator records the quick task.
- Commits: subject "test(quick-261127-uou): ..." for test code, "docs(quick-261127-uou): ..." for the planning file; pass the body with several -m flags or git commit -F, never escaped newlines. No dependency or manifest change: git diff --exit-code HEAD -- Cargo.toml Cargo.lock apps/sucrose/Cargo.toml stays clean.
- A known flaky test under build load (memory: portal.rs timing tests, memory_detail_counts_live_heap_blocks, os error 5 in the cef-dll-sys build script) is rerun once and recorded in the SUMMARY; anything that fails twice is reported, not retried further.

<tasks>

<task type="tracer" tdd="true">
  <name>Task 1: Per-slot renderer log helpers and the engine-change test on them, end to end on the real service</name>
  <files>apps/sucrose/tests/portal.rs</files>
  <read_first>apps/sucrose/tests/portal.rs lines 1-30 (imports), 540-600 (renderer_logs, use_plays_a_web_theme), 690-755 (webview2_is_ready, the old wait helper, an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen); apps/sucrose/tests/common/mod.rs lines 30-50 (service_runs, skipped); target/ci-0437-nf.log lines 455-540 (the two-slot failure; untracked, in the main checkout only, skip it when absent)</read_first>
  <behavior>
    - slot_of("Renderer-0.2026-11-27.log") is Some(0) and slot_of("Renderer-12.2026-11-28.log") is Some(12); slot_of("CefRenderer-0.log"), slot_of("Service.2026-11-27.log"), slot_of("Renderer-x.2026-11-27.log") and slot_of("Renderer-.log") are None
    - logs_by_slot over the pairs given out of order (Renderer-1.2026-11-28.log "b2", Service.2026-11-27.log "svc", Renderer-0.2026-11-27.log "a0", CefRenderer-0.log "cef", Renderer-1.2026-11-27.log "b1") has exactly the slots 0 and 1; slot 0 holds a0 only, slot 1 holds b1 then b2 (file-name order, which is date order); a file text without a final line break still ends in one in the result, so the lines of two files never merge
    - starts over one slot's log with a Cef start, two other lines and a WebView2 start gives exactly those two start lines, Cef first
    - every_slot_started: two slots with one start each gives true for (slots 2, count 1) and false for (2, 2), the 04-37 shape where the joined count already read 2; one slot with two starts gives false for (2, 1), a second monitor whose renderer has not started, and true for (1, 2); an empty map gives false for (1, 1)
  </behavior>
  <action>
RED: add two pure tests next to renderer_logs, named renderer_logs_are_grouped_by_the_slot_in_their_file_name and every_monitor_slot_has_to_reach_the_count, covering the behavior list with literal file names and log text (no service, no service_runs guard, so they run on every OS in CI). Run them and see them fail to compile; commit "test(quick-261127-uou): add failing tests for per-slot renderer logs".

GREEN, all in portal.rs next to renderer_logs, each with a one-line doc comment saying what it is for:
- slot_of(file_name: &str) -> Option<u32>: the slot of a renderer log file; the name has to start with "Renderer-" (so CefRenderer- and Service. never match) and the text up to the next "." has to parse as u32.
- logs_by_slot(files: Vec<(String, String)>) -> BTreeMap<u32, String>: (file name, text) pairs to the text of each slot, the files of one slot joined in file-name order, each ending in a line break.
- renderer_logs_by_slot(service: &TestService) -> BTreeMap<u32, String>: reads <service.dir>/Log the way renderer_logs does today (an empty map while the folder does not exist; unreadable entries skipped) and hands the pairs to logs_by_slot.
- starts(log: &str) -> Vec<&str>: the engine-start lines of one slot's log, in log order.
- every_slot_started(by_slot: &BTreeMap<u32, String>, slots: usize, count: usize) -> bool: at least slots slots have at least count starts each.
- monitor_slots() -> usize: sucrose_platform::current().monitors().len() with a floor of 1; its doc comment says the service starts one renderer per connected monitor (apps/sucrose/src/service.rs), so this is how many slots a Use starts.
- wait_for_engine_starts_on_every_slot(service: &TestService, count: usize) -> BTreeMap<u32, String>: the same 30 s deadline and 250 ms poll as the old helper, ending when every_slot_started(by_slot, monitor_slots(), count) holds and returning that map; the timeout panic names the number of slots expected, count, the start count of every slot seen and the joined logs. Doc comment: waits until the renderer of every monitor slot logged at least count engine starts.
- renderer_logs keeps its signature and meaning (the joined text, for presence checks) but is built from renderer_logs_by_slot, so one function reads the log folder.
- Add BTreeMap to the std import (use std::{collections::BTreeMap, path::PathBuf}).

Convert an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen to the new wait: after the Use, wait for one start on every slot and assert that every slot's first start names engine="Cef" (message names the slot and its log); after the WebView2 change, wait for two starts on every slot and assert that every slot has at least one start after its first and that all of those name engine="WebView2" (message: the renderer of slot N after the change plays on WebView2, with that slot's log). Extend its doc comment: with several monitors every monitor's renderer restarts on the engine chosen. The other callers keep the old helper until Task 2, so this commit compiles and the rest of the binary behaves as before.

Run the tracer end to end on one monitor (DISPLAY9 is detached now) and commit "test(quick-261127-uou): count engine starts per renderer slot". The old helper still has callers, so no dead-code warning appears; clippy must stay clean.
  </action>
  <verify>
    <automated>cargo test -p sucrose --test portal -- --nocapture renderer_logs_are_grouped_by_the_slot_in_their_file_name every_monitor_slot_has_to_reach_the_count an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen > target/uou-t1.log 2>&1 && grep -q "test result: ok. 3 passed" target/uou-t1.log && ! grep -q ": skipped," target/uou-t1.log && cargo clippy -p sucrose --all-targets -- -D warnings</automated>
  </verify>
  <acceptance_criteria>
    - grep -c 'fn renderer_logs_by_slot(' apps/sucrose/tests/portal.rs is 1 [HEAD=0 EXPECT==1 @d1d76bd7]
    - grep -c 'fn wait_for_engine_starts_on_every_slot(' apps/sucrose/tests/portal.rs is 1 [HEAD=0 EXPECT==1 @d1d76bd7]
    - grep -c '\.monitors()' apps/sucrose/tests/portal.rs is at least 1 [HEAD=0 EXPECT>=1 @d1d76bd7]
    - grep -c 'fn every_monitor_slot_has_to_reach_the_count' apps/sucrose/tests/portal.rs is 1 [HEAD=0 EXPECT==1 @d1d76bd7]
    - The automated verify above passed: target/uou-t1.log has "test result: ok. 3 passed" and no skip line (a guard against a vacuous pass when an engine is missing, not a change detector)
    - <target>/tmp/portal/engine-restart/Log holds a Renderer-0.*.log with an engine="Cef" start followed by an engine="WebView2" start
    - The RED commit exists before the GREEN commit (git log --format=%s -n 2)
    - git ls-files --eol apps/sucrose/tests/portal.rs still shows w/lf
  </acceptance_criteria>
  <done>The per-slot helpers exist with their pure tests, the engine-change test waits for every monitor slot and passes on the real service with one monitor without skipping, and both commits are in.</done>
</task>

<task type="auto">
  <name>Task 2: Every other engine-start count of portal.rs per slot; the old joined-count wait removed; one-monitor gate green</name>
  <files>apps/sucrose/tests/portal.rs</files>
  <read_first>apps/sucrose/tests/portal.rs lines 755-1080 (a_gif_engine_change_moves_the_running_gif_to_the_built_in_renderer, switching_web_themes_on_webview2_closes_each_renderer_cleanly, restart_now_restarts_only_the_affected_wallpapers, saved_webview2_arguments_reach_the_restarted_renderer) and the helpers Task 1 added; grep the file for the old helper's name and for every count of engine-start lines (261127-uos may have added a caller)</read_first>
  <action>
Baseline first: on one monitor, run the three ignored tests by name before changing them (cargo test -p sucrose --test portal -- --ignored --nocapture --test-threads=1 restart_now_restarts_only_the_affected_wallpapers saved_webview2_arguments_reach_the_restarted_renderer a_gif_engine_change_moves_the_running_gif_to_the_built_in_renderer > target/uou-t2-baseline.log 2>&1). A failure or skip there whose cause is not the joined count is recorded in the SUMMARY (and as a new entry in deferred-items.md in Task 3) and not fixed here; its counting is still converted.

Convert, keeping every existing check and its meaning for one slot:
- switching_web_themes_on_webview2_closes_each_renderer_cleanly: wait for 1, then 2, then 3 starts on every slot (the same three Uses); on every slot at least two "webview2 closed" lines, every start names engine="WebView2", and the slot's log has no " WARN " (the message names the slot and shows its log). The no-WARN check stays over every slot's whole log with no filter or allow-list: a WARN there is a product defect (the D10 one is fixed by 261127-uos), never something this test tolerates. Doc comment: with several monitors every monitor's renderer is checked.
- a_gif_engine_change_moves_the_running_gif_to_the_built_in_renderer: every slot's first start names engine="Mpv"; after the built-in choice, wait for two starts on every slot; on every slot exactly one engine="BuiltInGif" start and exactly one engine="Mpv" start. The report and fallback checks stay as they are.
- restart_now_restarts_only_the_affected_wallpapers: replace its local wait closure with the new wait; every slot's log names allow_invalid_certificates=false after the first start; after the saved switch and again after the mpv restart and the 2 s pause, every one of the monitor_slots() slots has exactly one start; after the CEF restart, wait for two starts on every slot and every slot's log names allow_invalid_certificates=true. The runtime-state check stays.
- saved_webview2_arguments_reach_the_restarted_renderer: the skip when WebView2 did not start holds when any slot's first start does not name engine="WebView2"; the no-Translate check runs over every slot's log; after the save every slot has exactly one start; after the WebView2 restart, wait for two starts on every slot, every slot has exactly two "chromium switches" lines and its second one names Translate and disable-features=.
- Delete the old joined-count wait helper (portal.rs:700-715 at d1d76bd7) once nothing calls it, and convert any remaining count of engine-start or chromium-switches lines over the joined renderer logs, including callers added by 261127-uos. Presence checks over the joined logs (use_plays_a_web_theme, a_loop_change_reaches_the_running_renderer, open_devtools_roundtrip) stay as they are.

Then the gate on one monitor: cargo fmt --all (line-ending rule above), cargo clippy --workspace --all-targets -- -D warnings, the portal suite with --nocapture, the three ignored tests again by name, cargo test --workspace. Commit "test(quick-261127-uou): assert every engine start per renderer slot".
  </action>
  <verify>
    <automated>cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test -p sucrose --test portal -- --nocapture > target/uou-t2-portal.log 2>&1 && grep -q "test result: ok" target/uou-t2-portal.log && ! grep -q ": skipped," target/uou-t2-portal.log && cargo test -p sucrose --test portal -- --ignored --nocapture --test-threads=1 restart_now_restarts_only_the_affected_wallpapers saved_webview2_arguments_reach_the_restarted_renderer a_gif_engine_change_moves_the_running_gif_to_the_built_in_renderer > target/uou-t2-ignored.log 2>&1 && grep -q "test result: ok. 3 passed" target/uou-t2-ignored.log && ! grep -q ": skipped," target/uou-t2-ignored.log && cargo test --workspace</automated>
  </verify>
  <acceptance_criteria>
    - grep -v '^\s*//' apps/sucrose/tests/portal.rs | grep -c 'matches("engine started").count()' is 0 [HEAD=5 EXPECT==0 @d1d76bd7]    - grep -v '^\s*//' apps/sucrose/tests/portal.rs | grep -c 'fn wait_for_engine_starts(' is 0 [HEAD=1 EXPECT==0 @d1d76bd7]
    - grep -v '^\s*//' apps/sucrose/tests/portal.rs | grep -c 'let started = |count: usize|' is 0 [HEAD=1 EXPECT==0 @d1d76bd7]
    - grep -c 'wait_for_engine_starts_on_every_slot(&service' apps/sucrose/tests/portal.rs is at least 11 [HEAD=0 EXPECT>=11 @d1d76bd7] (2 engine change, 2 GIF, 3 switching, 2 restart now, 2 WebView2 arguments, plus any 261127-uos caller)
    - grep -c '" WARN "' apps/sucrose/tests/portal.rs is at least 1 [HEAD=1 EXPECT>=1 @d1d76bd7 GUARD MUTATION: deleting or filtering the no-WARN assertion of switching_web_themes_on_webview2_closes_each_renderer_cleanly turns it to 0] (the no-WARN check is kept)
    - target/uou-t2-ignored.log has "test result: ok. 3 passed" and no ": skipped," line, or the SUMMARY names the baseline failure from target/uou-t2-baseline.log that is not caused by counting
    - git diff --exit-code HEAD -- Cargo.toml Cargo.lock apps/sucrose/Cargo.toml exits 0; git ls-files --eol apps/sucrose/tests/portal.rs shows w/lf; no other file changed its line endings
  </acceptance_criteria>
  <done>No count over the joined renderer logs remains in portal.rs; fmt, clippy, the portal suite, the three converted ignored tests and cargo test --workspace are green on one monitor with no skipped desktop test; the commit is in.</done>
</task>

<task type="checkpoint:human-action" gate="blocking">
  <name>Task 3: Two-monitor proof: the maintainer attaches DISPLAY9, the portal suite and the workspace pass</name>
  <files>.planning/phases/04-m3-engines-and-the-wallpaper-api/deferred-items.md</files>
  <read_first>.planning/phases/04-m3-engines-and-the-wallpaper-api/deferred-items.md (LF; the two portal.rs entries and the orchestrator reading); target/uou-t2-portal.log; the Run rules above</read_first>
  <action>
Executor, before presenting: Task 2 is committed; tasklist.exe shows no sucrose.exe and no sucrose-portal.exe (stop them with the two taskkill commands if any); nothing is started while waiting. Present the instructions in Turkish (the maintainer reads checkpoints in Turkish) and wait. A "skip" closes the plan with a halted SUMMARY: the two-monitor proof stays open, and the two portal.rs entries of deferred-items.md get a line saying the per-slot counting landed in the Task 2 commit and the two-monitor run is pending.

Continuation after "attached":
1. Confirm two monitors: powershell.exe -NoProfile -Command "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.Screen]::AllScreens.Count" prints 2; if it prints 1, present the gate again.
2. cargo test -p sucrose --test portal -- --nocapture > target/uou-2mon-1.log 2>&1, then the same into target/uou-2mon-2.log: both exit 0 with "test result: ok" and no skip line of a desktop test (the Run rules check). After each run, <target>/tmp/portal/engine-restart/Log holds Renderer-0.*.log and Renderer-1.*.log with two engine starts each (Cef, then WebView2), and <target>/tmp/portal/webview2-switch/Log holds both with three WebView2 starts each and no WARN.
3. cargo test -p sucrose --test portal -- --ignored --nocapture --test-threads=1 restart_now_restarts_only_the_affected_wallpapers saved_webview2_arguments_reach_the_restarted_renderer a_gif_engine_change_moves_the_running_gif_to_the_built_in_renderer > target/uou-2mon-ignored.log 2>&1: exit 0, 3 passed, no skip line (a baseline failure recorded in Task 2 stays recorded, not fixed).
4. cargo test --workspace > target/uou-2mon-workspace.log 2>&1: exit 0.
5. tasklist.exe shows no sucrose.exe (taskkill otherwise).
6. On a failure: a count or timeout failure in portal.rs is this plan's scope: fix it in portal.rs, rerun steps 2-4 and commit "test(quick-261127-uou): ..."; a WARN in the switching test means the D10 fix of 261127-uos does not hold with two monitors: quote the WARN line verbatim in the SUMMARY and report it for /gsd-debug, never weaken the check; any other failure is quoted and reported.
7. When steps 2-4 passed: in deferred-items.md set the portal.rs failure entry and the orchestrator-reading entry to status: resolved, each with one line naming the Task 2 commit, the run date and both slots (\\.\DISPLAY1 and \\.\DISPLAY9); leave the flaky service-test entry and any 261127-uos text as they are; keep LF. Commit "docs(quick-261127-uou): close the two-monitor portal.rs entries".
8. Tell the maintainer (in Turkish) the result and that DISPLAY9 can be detached again if he wants; the SUMMARY quotes the "test result" lines of the four logs and the monitor count.
  </action>
  <instructions>Please attach the virtual display DISPLAY9 (VDD by MTT) so that Windows has two screens (Settings > System > Display shows two), then reply "attached". You do not need to watch anything: I run the portal tests twice, three desktop tests by name and the whole test suite; wallpapers from scratch folders appear on both screens for a few minutes and close by themselves. Please do not start Sucrose (the Rust build) during the run. Reply "skip" to leave the two-monitor check for later.</instructions>
  <verification>Windows reports 2 screens; target/uou-2mon-1.log and target/uou-2mon-2.log each have "test result: ok" and no ": skipped," line; the engine-restart and webview2-switch scratch logs hold a Renderer-1 log beside Renderer-0 with the expected starts; target/uou-2mon-ignored.log has "test result: ok. 3 passed"; target/uou-2mon-workspace.log ends green; tasklist.exe shows no sucrose.exe; git ls-files --eol on deferred-items.md shows w/lf.</verification>
  <verify>
    <automated>grep -q "test result: ok" target/uou-2mon-1.log && grep -q "test result: ok" target/uou-2mon-2.log && ! grep -q ": skipped," target/uou-2mon-1.log && ! grep -q ": skipped," target/uou-2mon-2.log && ls target/tmp/portal/engine-restart/Log/Renderer-1.*.log target/tmp/portal/webview2-switch/Log/Renderer-1.*.log && grep -q "test result: ok. 3 passed" target/uou-2mon-ignored.log && grep -c "261127-uou" .planning/phases/04-m3-engines-and-the-wallpaper-api/deferred-items.md</automated>
  </verify>
  <acceptance_criteria>
    - The maintainer replied "attached" and the four two-monitor logs are green as listed in verification, or the skip is recorded verbatim in a halted SUMMARY
    - grep -c "261127-uou" .planning/phases/04-m3-engines-and-the-wallpaper-api/deferred-items.md is at least 2 [HEAD=0 EXPECT>=2 @d1d76bd7] (both portal.rs entries name this item, resolved or pending)
    - The SUMMARY quotes the "test result" lines, the monitor count and the maintainer's replies; no WARN was filtered to make a run pass
  </acceptance_criteria>
  <resume-signal>Reply "attached" when DISPLAY9 is on (or "skip").</resume-signal>
  <done>With two monitors the portal suite passes twice, the three converted ignored tests and cargo test --workspace pass, both portal.rs entries of deferred-items.md are resolved, nothing of Sucrose runs; or the skip is recorded and the entries say the two-monitor run is pending.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| test harness to real desktop | portal.rs starts the real sucrose.exe, which puts wallpapers on the maintainer's monitors from scratch folders under the target directory |
| test assertions to the CI gate | a weakened or vacuous assertion lets a product defect (such as the D10 profile-folder lock) pass cargo test --workspace |

## STRIDE Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation Plan |
|-----------|----------|-----------|----------|-------------|-----------------|
| T-uou-01 | Tampering | switching_web_themes_on_webview2_closes_each_renderer_cleanly no-WARN check | medium | mitigate | The check stays over every slot's whole log with no filter or allow-list (Task 2); the acceptance grep keeps '" WARN "' at 1 or more; a WARN at the two-monitor run is reported as a 261127-uos/D10 regression, never tolerated (Task 3 step 6) |
| T-uou-02 | Repudiation | desktop tests that skip and still pass | medium | mitigate | Every run uses --nocapture and requires no ": skipped," line; the two-monitor run also requires a Renderer-1 log in the engine-restart and webview2-switch scratch folders (Tasks 1-3); engines are fetched in a worktree with its own target |
| T-uou-03 | Information Disclosure | tests that read or drive the maintainer's real data (use_switches_between_video_themes, a_loop_change_reaches_the_running_renderer) | medium | mitigate | Ignored tests run only by exact name with --ignored, never --include-ignored on the whole binary; all runs use scratch folders under the target directory (Run rules) |
| T-uou-04 | Denial of Service | sucrose.exe or renderers left on the desktop after a run | low | mitigate | tasklist.exe after every run, taskkill.exe /IM sucrose-portal.exe /F and /IM sucrose.exe /F when needed, never msedgewebview2.exe by name (Run rules, Task 3 step 5) |
| T-uou-SC | Tampering | cargo dependencies | low | mitigate | No crate, feature or manifest change (sucrose-platform is already a dependency of apps/sucrose); git diff --exit-code HEAD -- Cargo.toml Cargo.lock apps/sucrose/Cargo.toml stays clean (Task 2); no package-manager install task exists |
</threat_model>

<verification>
- Task 1 and Task 2 automated commands green on one monitor, with no skipped desktop test.
- No count over the joined renderer logs remains in portal.rs (Task 2 acceptance greps); the no-WARN check is still there.
- Task 3: with DISPLAY9 attached, the portal suite passes twice, the three converted ignored tests and cargo test --workspace pass, and both slots appear in the scratch logs.
- Line endings unchanged (portal.rs and deferred-items.md w/lf; no CRLF file turned LF); no manifest or lock change.
</verification>

<success_criteria>
- cargo test --workspace passes with one monitor and with DISPLAY9 attached, with no change to product code in this item.
- Every engine-start assertion in apps/sucrose/tests/portal.rs reads one slot's log, and the wait ends only when every monitor's renderer reached the count.
- The switching test still fails on any WARN; with the 261127-uos fix it passes on two monitors.
- deferred-items.md no longer lists the two portal.rs failures as open (or says the two-monitor run is pending after a skip).
</success_criteria>

<output>
Create `.planning/quick/261127-uou-apps-sucrose-tests-portal-rs-real-desktop-tests-that-assume/261127-uou-SUMMARY.md` when done
</output>
