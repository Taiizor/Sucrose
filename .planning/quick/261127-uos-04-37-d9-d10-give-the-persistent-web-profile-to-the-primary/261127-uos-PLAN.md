---
phase: quick-261127-uos
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/sucrose-engine-web/src/profile.rs
  - crates/sucrose-engine-web/tests/profile.rs
  - crates/sucrose-ipc/src/protocol.rs
  - apps/sucrose/src/topology.rs
  - apps/sucrose/src/service.rs
  - apps/sucrose/src/render.rs
  - crates/sucrose-engine-cef/src/engine.rs
  - crates/sucrose-engine-webview2/src/engine.rs
  - crates/sucrose-engine-webview2/src/host.rs
  - docs/DECISIONS.md
  - docs/FEATURES.md
  - .planning/phases/04-m3-engines-and-the-wallpaper-api/04-CONTEXT.md
autonomous: false

must_haves:
  truths:
    - "With several monitors playing a Web wallpaper, only the renderer of the primary monitor (no monitor flagged primary: the lowest monitor id, as the master election) asks for the persistent profile <data_dir>/Web/<engine>; every other renderer goes straight to its ephemeral Cache/WebEphemeral/<engine>/<slot> and never opens the lock file, on CEF and on WebView2 (04-37 D9, maintainer decision 2026-11-27, amends 04-CONTEXT D-33 and the profile rule of D-038)"
    - "The OS lock still guarantees one holder: a primary renderer that finds the lock held waits at most profile::START_WAIT (1 s) and then runs ephemeral, never sharing the folder (D-33 guard kept)"
    - "After a monitor snapshot that moves the primary flag, the stored assignment of every slot asks for the kind the new primary implies, so the next start of each slot follows the current primary; a running renderer keeps its lease until it ends"
    - "A WebView2 renderer drops its profile lease only after its WebView2 browser process exited (BrowserProcessExited) or BROWSER_EXIT_WAIT (1 s) passed, and logs 'webview2 browser process exited' with the elapsed ms (04-37 D10)"
    - "An ephemeral profile folder that a process of the ended engine still holds (os error 32) is emptied with retries for up to START_WAIT instead of failing at once"
    - "With DISPLAY9 attached: switching_web_themes_on_webview2_closes_each_renderer_cleanly passes, and the storage fixture count rises run after run on the primary monitor while DISPLAY9 stays at 1, on WebView2 and on CEF"
    - "docs/DECISIONS.md has a new Accepted entry for the primary-monitor rule (D-038 text unchanged), docs/FEATURES.md and 04-CONTEXT D-33 say the primary monitor keeps the web data; every edited file keeps its line endings"
  artifacts:
    - path: crates/sucrose-engine-web/src/profile.rs
      provides: "acquire with the wanted ProfileKind and a bounded wait; START_WAIT; emptying retries"
      contains: "pub const START_WAIT"
    - path: crates/sucrose-ipc/src/protocol.rs
      provides: "Assignment.web_profile (last field), PROTOCOL_VERSION 10"
      contains: "pub web_profile: ProfileKind"
    - path: apps/sucrose/src/topology.rs
      provides: "web_profile_owner: the primary monitor, else the lowest id"
      contains: "pub fn web_profile_owner"
    - path: crates/sucrose-engine-webview2/src/host.rs
      provides: "close waits for BrowserProcessExited, bounded"
      contains: "add_BrowserProcessExited"
    - path: docs/DECISIONS.md
      provides: "new entry amending the D-038 profile rule"
      contains: "The primary monitor keeps the persistent web profile"
  key_links:
    - from: apps/sucrose/src/service.rs
      to: apps/sucrose/src/topology.rs
      via: "Fleet spawn and the snapshot refresh ask web_profile_owner which monitor owns the persistent profile"
      pattern: "web_profile_owner"
    - from: apps/sucrose/src/service.rs
      to: crates/sucrose-ipc/src/protocol.rs
      via: "Job::assignment fills Assignment.web_profile per monitor"
      pattern: "web_profile"
    - from: crates/sucrose-engine-webview2/src/engine.rs
      to: crates/sucrose-engine-web/src/profile.rs
      via: "acquire_profile passes assignment.web_profile and profile::START_WAIT"
      pattern: "START_WAIT"
    - from: crates/sucrose-engine-cef/src/engine.rs
      to: crates/sucrose-engine-web/src/profile.rs
      via: "acquire_profile passes assignment.web_profile and profile::START_WAIT"
      pattern: "START_WAIT"
    - from: crates/sucrose-engine-webview2/src/host.rs
      to: crates/sucrose-engine-webview2/src/engine.rs
      via: "WebView2Pump::close drops the lease only after Host::close returned, which now waits for the browser process"
      pattern: "drop\\(profile\\)"
---

<objective>
Two 04-37 findings in one change, as the maintainer asked (STATE.md Pending Todos (a), Decisions 2026-11-27):

- **D9 (decision, amends 04-CONTEXT D-33 and the profile rule of docs/DECISIONS.md D-038):** with several monitors playing a Web wallpaper, the persistent web profile goes to the primary monitor. Today the first renderer to lock `.sucrose-profile.lock` wins, so in the 04-37 run DISPLAY9 held `kind=Persistent` and the storage fixture on DISPLAY1 stayed at 1 ("storage sürekli 1", 04-37-SUMMARY D9). The service will choose the owner from the monitor list and tell each renderer which kind to ask for; the OS lock stays as the guard that two renderers never share a folder.
- **D10 (defect):** on a quick Web theme switch with two monitors, the new slot-1 WebView2 renderer could not empty `Cache/WebEphemeral/WebView2/1` (os error 32, sharing violation) and ran without a profile folder (WARN seen at 12:37:41Z on 2026-11-27 in `switching_web_themes_on_webview2_closes_each_renderer_cleanly`, deferred-items.md). Cause read from the evidence: the previous renderer's msedgewebview2 processes still held files after its controller closed, because the WebView2 browser process outlives the host's close. The renderer will wait (bounded) for WebView2's BrowserProcessExited before it lets go of the folder, and the allocator retries a folder that is still in use.

Purpose: the monitor the user looks at keeps its wallpaper's localStorage and cookies across theme switches and restarts, and quick switches never leave a page without a profile folder.
Output: the code change on both web engines and in the service, tests, the DECISIONS/FEATURES/04-CONTEXT records, and the two-monitor evidence from the maintainer's machine.
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
@crates/sucrose-engine-web/src/profile.rs
@crates/sucrose-engine-web/tests/profile.rs

Key facts gathered at planning time (HEAD d1d76bd7):
- `profile::acquire(web_dir, engine, ephemeral_root, slot)` (crates/sucrose-engine-web/src/profile.rs) tries the lock once and otherwise empties `<ephemeral_root>/<engine>/<slot>` with one `remove_dir_all`; any error is returned and both engines' `acquire_profile` then log a WARN and run without a profile (CEF: in-memory caches; WebView2: a scratch folder under %TEMP%).
- Callers of `acquire`: crates/sucrose-engine-cef/src/engine.rs (`acquire_profile`, ~line 160, called at ~251) and crates/sucrose-engine-webview2/src/engine.rs (`acquire_profile`, ~145, called at ~239), plus crates/sucrose-engine-web/tests/profile.rs.
- `Assignment` (crates/sucrose-ipc/src/protocol.rs ~234) carries slot, monitor, rect, source, settings, kind, engine, theme_dir, web, input; it has no primary flag. Adding a field changes the postcard layout, so PROTOCOL_VERSION goes 9 → 10, as commit 353649b4 did for `Assignment.input`. The version is asserted in a test (~line 528); Assignment literals are at protocol.rs ~266, ~401, ~421 and apps/sucrose/src/render.rs ~1094. sucrose-ipc already depends on sucrose-engine-web, and sucrose-engine-web has serde (args::Switch derives Serialize/Deserialize).
- The service gives every monitor a renderer (D-01). `Job::assignment(slot, monitor)` (apps/sucrose/src/service.rs ~2844) is called only from `Fleet::spawn` (~3287) and a test helper (~5510). Stored assignments live in the shared `assignments` map; `adopt_job` (~3787) and `apply_live` (~3809) update only job fields in place; a respawn (crash, shell restart, Use restart through `restart_with` ~3844) reuses the stored assignment. Startup applies `diff(&[], &monitors)` and sets `fleet.monitors` afterwards (~1125-1137); `apply_snapshot` (~3234) applies the actions before `self.monitors = monitors`. `topology::primary` finds the flag; `choose_master` (~2581) falls back to the lowest id when no monitor is primary.
- WebView2 close: `WebView2Pump::close` (webview2 engine.rs ~624) calls `host.close()` and then `drop(profile)`; `Host::close_inner` (host.rs ~1035) closes the controller and releases webview, controller3, controller and environment at once. `wait_for` (host.rs ~316) already pumps the thread's messages for WebView2 callbacks with `CREATE_TIMEOUT`. The renderer calls `pump.close()` after its loop returned and before the surface is dropped (render.rs ~338).
- webview2-com 0.39.1 (Cargo.lock) provides `BrowserProcessExitedEventHandler` (callback.rs: closure over `Option<ICoreWebView2Environment>`, `Option<ICoreWebView2BrowserProcessExitedEventArgs>`) and `ICoreWebView2Environment5::add_BrowserProcessExited` / `remove_BrowserProcessExited`. No new crate and no new dependency (T-04-SC).
- The service's quit grace is `SHUTDOWN_GRACE` = 2 s (service.rs ~817); after it, survivors are killed through the job object.
- The portal test scratch folder is `target/tmp/portal/<name>` (CARGO_TARGET_TMPDIR), removed only at the next start of the same test, so its `Log/Renderer-*.log` can be read after a run. The storage fixture reports its count as a `__storage/<n>` origin request in the renderer log at debug level (04-24 verify).
- Line endings in the working tree: CRLF in protocol.rs, service.rs, topology.rs, render.rs, sucrose-engine-cef/src/engine.rs, sucrose-engine-web/src/profile.rs, sucrose-engine-web/tests/profile.rs and docs/FEATURES.md (no final newline); LF in sucrose-engine-webview2/src/engine.rs and host.rs, docs/DECISIONS.md and 04-CONTEXT.md. `cargo fmt` through the wrapper writes LF.
- D9 is settled by the maintainer (2026-11-27 decision in STATE.md): no new decision checkpoint. DECISIONS.md is append-only: D-039 amended D-011 without editing it; do the same for D-038.
</context>

<assumption_delta_decision>
- Primary noun: the persistent web profile's owner is the primary monitor's renderer, chosen by the service from the monitor list (lowest id when no monitor is flagged primary).
- Decision: promote. The chosen owner replaces "whoever locks first" as the rule; the OS lock is demoted to the guard that keeps two renderers off one folder (a renderer told Persistent still has to win the lock).
- Invariant test (task 1): a renderer that asks for Ephemeral never takes the lock, so a later Persistent request always finds it free.
</assumption_delta_decision>

<tasks>

<task type="tracer" tdd="true">
  <name>Task 1: End-to-end D9: the service marks the primary monitor's renderer and both web engines ask the allocator for that kind</name>
  <files>crates/sucrose-engine-web/src/profile.rs, crates/sucrose-engine-web/tests/profile.rs, crates/sucrose-ipc/src/protocol.rs, apps/sucrose/src/topology.rs, apps/sucrose/src/service.rs, apps/sucrose/src/render.rs, crates/sucrose-engine-cef/src/engine.rs, crates/sucrose-engine-webview2/src/engine.rs</files>
  <read_first>
    - crates/sucrose-engine-web/src/profile.rs (whole file) and crates/sucrose-engine-web/tests/profile.rs (whole file, including the cross-process holder child)
    - crates/sucrose-ipc/src/protocol.rs: module docs, PROTOCOL_VERSION, Assignment (~234) and the tests that build an Assignment or assert the version (~262-530)
    - apps/sucrose/src/topology.rs: primary() and its tests (~395)
    - apps/sucrose/src/service.rs: Job::assignment (~2844), the startup block (~1110-1140), Fleet::apply_snapshot / apply / spawn (~3230-3305), adopt_job (~3787), choose_master (~2581), the fleet test helpers and assignment tests (~5430-5520)
    - crates/sucrose-engine-cef/src/engine.rs ~150-175 and ~251; crates/sucrose-engine-webview2/src/engine.rs ~140-160 and ~239
    - apps/sucrose/src/render.rs ~1090-1100 (the Assignment in a test)
    - docs/DECISIONS.md D-038 and .planning/phases/04-m3-engines-and-the-wallpaper-api/04-37-SUMMARY.md line 133 (the D9 finding)
  </read_first>
  <behavior>
    - profile: acquire with wanted Ephemeral while nobody holds the lock returns Ephemeral at <ephemeral_root>/<engine>/<slot>, and a following acquire with wanted Persistent returns Persistent (the Ephemeral request never took the lock)
    - profile: acquire with wanted Persistent while a lease is held in this process returns Ephemeral after at least the given wait (200 ms) and well before 2 s
    - profile: acquire with wanted Persistent while a lease is held and dropped by another thread after 100 ms, with a 2 s wait, returns Persistent
    - profile: the existing tests keep their meaning with wanted Persistent (first is Persistent, a concurrent one Ephemeral, the cross-process holder makes a second request Ephemeral until it dies, engines do not share a lock, the engine name cannot leave the web folder)
    - topology: web_profile_owner returns the monitor flagged primary in any list order, the lowest id when none is flagged, None for an empty list
    - service: with two monitors and the primary listed second, the primary's slot gets web_profile Persistent and the other slot Ephemeral; a snapshot that moves the primary flag flips the stored assignments; a Use restart (adopt_job) keeps each slot's web_profile
    - protocol: Assign keeps wire index 1, PROTOCOL_VERSION is 10, an Assignment with web_profile Persistent survives a postcard round trip
  </behavior>
  <action>
Implements D9 (maintainer decision 2026-11-27; amends 04-CONTEXT D-33 and the profile rule of D-038; already settled, so no decision checkpoint). Write the tests of the behavior block first, see them fail, then implement.

1. crates/sucrose-engine-web/src/profile.rs: derive Serialize and Deserialize on ProfileKind (as args::Switch does). Add a public constant START_WAIT of 1 second, documented as how long an instance waits at its start for a profile folder another instance is still letting go of. Change acquire to take two more parameters after slot: wanted (a ProfileKind) and wait (a Duration). The engine-name check stays first and unchanged. Wanted Persistent: create the folder and open the lock file as today, then try_lock; on WouldBlock try again every 50 ms until wait has elapsed, then take the ephemeral path; any other lock error is returned as today. Wanted Ephemeral: go straight to the ephemeral path without creating or opening the persistent folder or its lock file. Rewrite the module docs: the service chooses which instance asks for the persistent profile (the primary monitor's renderer, D-33 as amended by 04-37 D9) and the lock guards that no two instances share it; drop the "first running instance" wording.
2. crates/sucrose-ipc/src/protocol.rs: append a field web_profile of type sucrose_engine_web::profile::ProfileKind as the last field of Assignment, documented as the profile the renderer's web engine asks for (the service sets Persistent for the primary monitor's renderer only; engines that are not web engines ignore it). Bump PROTOCOL_VERSION from 9 to 10 (a struct field changes the postcard layout, which appending variants cannot express; precedent 353649b4). Update every Assignment literal and the version assertion in the tests, and add the round-trip assertion of the behavior block.
3. apps/sucrose/src/topology.rs: add a pure public function web_profile_owner over a monitor slice returning the monitor that owns the persistent web profile: the one flagged primary, otherwise the one with the lowest id (the fallback of choose_master), None for an empty slice. Unit tests next to it.
4. apps/sucrose/src/service.rs: Job::assignment takes the web_profile kind and puts it into the Assignment. Fleet::spawn computes it from web_profile_owner over the snapshot being applied (Persistent when the owner's id is the spawned monitor's id, otherwise Ephemeral): give the snapshot to apply/spawn (or set it on the fleet before applying) on both paths, the startup block and apply_snapshot, because both apply before the fleet's monitor list is updated. After apply_snapshot stored a snapshot whose monitors changed, refresh web_profile of every stored assignment from web_profile_owner of the new list through slots.monitor_of(slot), so the next start of a slot follows the current primary; a running renderer is not restarted for it. adopt_job and apply_live leave web_profile alone (it belongs to the monitor, not to the job). Add the owner's id as a field of the existing "renderers started" INFO line. Unit tests with the existing fleet helpers.
5. crates/sucrose-engine-cef/src/engine.rs and crates/sucrose-engine-webview2/src/engine.rs: acquire_profile takes the slot, the monitor name and the wanted kind, and passes assignment.web_profile and profile::START_WAIT to acquire. The "web profile" INFO line gains the fields wanted and monitor next to kind, engine, path and slot. When wanted was Persistent but the lease is Ephemeral, also log at INFO that the persistent web profile is still held by another renderer, so this page keeps nothing across runs. Leave the existing WARN fallback for a failed acquire untouched here (task 2 addresses why it fired). Update the doc comments that say "the first running ... renderer" to the primary-monitor rule.
6. apps/sucrose/src/render.rs: the test Assignment gets web_profile Ephemeral. crates/sucrose-engine-web/tests/profile.rs: existing calls pass wanted Persistent with Duration::ZERO; the cross-process tests that expect Ephemeral while the child holds the lock pass a short wait (100 ms) so they stay fast.

Keep each file's line endings (CRLF list in the context; cargo fmt writes LF, so compare git ls-files --eol before and after and convert back with perl -pi -e 's/\r?\n/\r\n/' only files that were w/crlf). No new crate, no unsafe in these files, thiserror/anyhow rules unchanged.
  </action>
  <verify>
    <automated>cargo test -p sucrose-engine-web --test profile && cargo test -p sucrose-ipc && cargo test -p sucrose --bin sucrose topology && cargo test -p sucrose --bin sucrose service::tests && test "$(grep -cE 'pub const PROTOCOL_VERSION: u16 =.10;' crates/sucrose-ipc/src/protocol.rs)" -ge 1 && test "$(grep -c 'pub fn web_profile_owner' apps/sucrose/src/topology.rs)" -ge 1 && cargo build -p sucrose && cargo xtask synth-library --out target/uos-store --count 2 --web-fixtures --select storage --engine Web=WebViewLive --force && rm -rf target/uos-store/Log && for run in 1 2; do (SUCROSE_DATA_DIR="$(wslpath -w target/uos-store)" SUCROSE_LOG="info,sucrose_engine_webview2=debug" WSLENV=SUCROSE_DATA_DIR:SUCROSE_LOG target/debug/sucrose.exe --no-portal &) ; sleep 15; taskkill.exe /IM sucrose.exe /F; sleep 3; done; test "$(grep -h 'web profile' target/uos-store/Log/Renderer-*.log | grep 'wanted=Persistent' | grep -c 'kind=Persistent')" -ge 2 # [HEAD=0 EXPECT>=2 @d1d76bd7] (protocol and topology greps: [HEAD=0 EXPECT>=1 @d1d76bd7])</automated>
  </verify>
  <done>The allocator, the protocol, the service and both web engines carry the wanted kind end to end: unit tests of the behavior block pass, PROTOCOL_VERSION is 10, and a one-monitor WebView2 run logs wanted=Persistent with kind=Persistent on slot 0 in both runs. No Rust Sucrose process is left running.</done>
  <reversibility rating="costly">Which monitor keeps the web data is user-visible (04-CONTEXT D-33 rating); the maintainer settled the primary-monitor rule on 2026-11-27, so no new checkpoint.</reversibility>
</task>

<task type="auto" tdd="true">
  <name>Task 2: D10: a WebView2 renderer lets go of its profile only after its browser process exited, an ephemeral folder still in use is emptied with retries, and the docs record D9 and D10</name>
  <files>crates/sucrose-engine-webview2/src/host.rs, crates/sucrose-engine-webview2/src/engine.rs, crates/sucrose-engine-web/src/profile.rs, crates/sucrose-engine-web/tests/profile.rs, docs/DECISIONS.md, docs/FEATURES.md, .planning/phases/04-m3-engines-and-the-wallpaper-api/04-CONTEXT.md</files>
  <read_first>
    - crates/sucrose-engine-webview2/src/host.rs ~290-405 (wait_for, try_environment, create_environment) and ~670-1090 (Host struct, open, close, close_inner, Drop)
    - crates/sucrose-engine-webview2/src/engine.rs ~620-640 (WebView2Pump::close)
    - webview2-com 0.39.1 callback.rs around BrowserProcessExitedEventHandler and webview2-com-sys bindings.rs around impl ICoreWebView2Environment5 (in the Windows cargo registry: /mnt/c/Users/Taiizor/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/)
    - apps/sucrose/src/render.rs ~325-345 (close order) and the surface window procedure in sucrose-platform that a message dispatched during close would reach
    - .planning/phases/04-m3-engines-and-the-wallpaper-api/deferred-items.md (the D10 evidence) and apps/sucrose/tests/portal.rs ~834-879 (the switching test)
    - docs/DECISIONS.md D-038, D-039 and the tail before "Open questions"; docs/FEATURES.md rows "Web (local HTML)" and "**New:** web data persists ..."; 04-CONTEXT.md D-33 (~187)
  </read_first>
  <behavior>
    - (Windows only, cfg(windows) test) a file inside <ephemeral_root>/<engine>/<slot> held open without delete sharing (std::os::windows::fs::OpenOptionsExt share_mode 0) by a thread that closes it after 200 ms: acquire with wanted Ephemeral and a 2 s wait returns Ephemeral and the folder is empty
    - (Windows only) the same file held for 1 s with a 200 ms wait: acquire returns an error after at least 200 ms (the engines keep their logged fallback)
    - the existing test that an earlier run's ephemeral folder is emptied still passes
  </behavior>
  <action>
Implements the D10 fix next to D9; the WebView2 part is the root cause, the allocator retry covers a killed renderer and CEF's subprocesses.

1. crates/sucrose-engine-web/src/profile.rs, ephemeral path: when removing the slot folder fails with an error other than NotFound, try again every 50 ms until the wait given to acquire has elapsed, then return the last error. The retry stays OS-neutral (no error-code test in this crate): a sharing violation (os error 32: a process of the ended engine still has a file open) is the case it is for. The path never changes: still <ephemeral_root>/<engine>/<slot> from the validated engine name and the slot; the persistent folder is never emptied. Windows-only tests of the behavior block in crates/sucrose-engine-web/tests/profile.rs.
2. crates/sucrose-engine-webview2/src/host.rs (Windows module): in close_inner, when a controller was opened, cast the environment to ICoreWebView2Environment5 and register a BrowserProcessExitedEventHandler that sends on an mpsc channel, before the controller's Close. After the controller, controller3 and webview are released, pump this thread's messages until that event arrives or a new constant BROWSER_EXIT_WAIT of 1 s has passed, using the same MsgWaitForMultipleObjects / PeekMessage loop as wait_for (give wait_for a timeout parameter, CREATE_TIMEOUT stays the value for creation, and keep its WM_QUIT re-post). Then remove the handler, release the environment and balance COM as today. Log at INFO "webview2 browser process exited" with the elapsed ms; on timeout log a WARN that the WebView2 browser process did not exit in time and its profile folder may still be in use, with the ms; when the runtime has no ICoreWebView2Environment5 or WM_QUIT was seen, log at debug and do not wait. Every new unsafe block gets a // SAFETY: comment. The bound stays below the service's SHUTDOWN_GRACE (2 s) so a Quit never escalates to the job kill; record the measured exit times in the SUMMARY, and if they regularly exceed about 700 ms raise BROWSER_EXIT_WAIT to at most 1.5 s and say why. Before relying on the pumping, confirm in sucrose-platform that a message dispatched to the surface after the renderer's loop returned is harmless (wait_for already dispatches during start); report instead of working around it if it is not. Never kill msedgewebview2.exe in any check.
3. crates/sucrose-engine-webview2/src/engine.rs: WebView2Pump::close keeps host.close() before drop(profile); update its comment to say the lease ends only after WebView2's browser process released the folder (D10).
4. docs/DECISIONS.md (LF, append-only): add a new entry after D-039 and before the "---" that precedes "Open questions", numbered with the next free id (D-040 unless another entry was appended first: check the last "## D-0" heading). Title "The primary monitor keeps the persistent web profile". Status: Accepted (2026-11-27), the maintainer's decision on 04-37 D9; amends the profile rule of D-038 (04-CONTEXT D-33). Decision: the service picks the owner from the monitor list (the primary monitor, else the lowest id) and sends each renderer the kind to ask for in Assignment.web_profile; only the owner tries the lock, waiting at most 1 s for an instance that is still ending, then runs ephemeral; every other instance is ephemeral without touching the lock; a primary switch while playing moves the profile at the next start of each renderer. Why: the user looks at the primary monitor, and the lock race gave its web data to whichever renderer started first (04-37 A5). Consequences: replaces the D-038 consequence that the owner "follows which renderer locked first"; a WebView2 renderer releases its profile only after its browser process exited (bounded 1 s, D10), and an ephemeral folder still in use is emptied with retries for up to 1 s; PROTOCOL_VERSION 10. Do not edit D-038 itself.
5. docs/FEATURES.md (CRLF, no final newline; edit inside the lines): retitle the row "**New:** web data persists for the first instance per engine (...)" to "**New:** web data persists for the primary monitor's instance per engine (...)" and rewrite its notes for the D9 rule and the D10 close wait (cite the new decision id); in the "Web (local HTML)" row, change the sentence about the first running CEF wallpaper keeping its localStorage and the WebView2 "same lock rule" sentence to the primary-monitor rule.
6. .planning/phases/04-m3-engines-and-the-wallpaper-api/04-CONTEXT.md (LF): under D-33 add a sub-bullet: amended 2026-11-27 (04-37 D9, maintainer): with several monitors the persistent profile goes to the primary monitor's instance, see docs/DECISIONS.md and the new entry id.
7. Gate (DISPLAY9 detached): cargo fmt --all, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace (equivalently cargo xtask ci). If DISPLAY9 happens to be attached, the two known portal.rs one-monitor tests fail (item 261127-uou): report, do not fix them here. Restore CRLF on every file that was w/crlf before (git ls-files --eol), check that DECISIONS.md has only added lines (git diff --numstat shows 0 deleted lines) and that FEATURES.md still has no final newline. Stop every Rust Sucrose process you started (taskkill.exe /IM sucrose.exe /F, /IM sucrose-portal.exe /F).
  </action>
  <verify>
    <automated>cargo test -p sucrose-engine-web --test profile && cargo test -p sucrose --test portal switching_web_themes_on_webview2_closes_each_renderer_cleanly && test "$(grep -h 'webview2 browser process exited' target/tmp/portal/webview2-switch/Log/Renderer-*.log | wc -l)" -ge 2 && test "$(grep -c 'add_BrowserProcessExited' crates/sucrose-engine-webview2/src/host.rs)" -ge 1 && test "$(grep -c 'The primary monitor keeps the persistent web profile' docs/DECISIONS.md)" -ge 1 && test "$(grep -c 'persists for the primary monitor.s instance per engine' docs/FEATURES.md)" -ge 1 && test "$(grep -c '04-37 D9' .planning/phases/04-m3-engines-and-the-wallpaper-api/04-CONTEXT.md)" -ge 1 && eol="$(git ls-files --eol crates/sucrose-ipc/src/protocol.rs apps/sucrose/src/service.rs apps/sucrose/src/topology.rs apps/sucrose/src/render.rs crates/sucrose-engine-cef/src/engine.rs crates/sucrose-engine-web/src/profile.rs crates/sucrose-engine-web/tests/profile.rs docs/FEATURES.md)" && test "$(grep -c 'w/crlf' <<< "$eol")" -ge 8 && cargo xtask ci # [HEAD=0 EXPECT>=2 @d1d76bd7] browser-exit lines; add_BrowserProcessExited [HEAD=0 EXPECT>=1]; DECISIONS title [HEAD=0 EXPECT>=1]; FEATURES title [HEAD=0 EXPECT>=1]; 04-CONTEXT [HEAD=0 EXPECT>=1]; w/crlf [HEAD=8 EXPECT>=8 GUARD MUTATION: cargo fmt through the wrapper rewrites CRLF files to LF]</automated>
  </verify>
  <done>On one monitor, the WebView2 switching test passes and each closed WebView2 renderer logs that its browser process exited before it ended; the Windows-only allocator tests prove a folder held for a moment is emptied after retries; DECISIONS, FEATURES and 04-CONTEXT record the primary-monitor rule and the close wait; cargo xtask ci passes; line endings are as before; no Rust Sucrose process is left running.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 3: Two-monitor check with DISPLAY9 attached: the primary monitor keeps the storage count on both engines, quick WebView2 switches keep every profile folder</name>
  <what-built>D9 (the service gives the persistent web profile to the primary monitor's renderer on CEF and WebView2) and D10 (a WebView2 renderer waits for its browser process before it releases its profile; a folder still in use is emptied with retries). Tasks 1 and 2 are committed and machine-checked on one monitor.</what-built>
  <how-to-verify>
Before returning this checkpoint: no Rust Sucrose process runs (tasklist.exe shows no sucrose.exe or sucrose-portal.exe), the debug build is current (cargo build -p sucrose). The orchestrator presents these steps to the maintainer in Turkish.

1. Maintainer: attach the virtual monitor DISPLAY9 (VDD) next to DISPLAY1, keep DISPLAY1 the primary monitor, and reply that it is attached.
2. After that reply, machine evidence (no mouse or keyboard input is synthesized; only scratch folders under target/ are written):
   a. cargo test -p sucrose --test portal switching_web_themes_on_webview2_closes_each_renderer_cleanly passes with two monitors (it failed on 2026-11-27 at 12:37:41Z with the slot-1 profile WARN, os error 32). In target/tmp/portal/webview2-switch/Log/Renderer-*.log: grep -c "no web profile folder" gives 0, every "web profile" line with kind=Persistent names DISPLAY1 in its monitor field, and every closed renderer logged "webview2 browser process exited".
   b. D9 on WebView2: cargo xtask synth-library --out target/uos-2mon-wv2 --count 2 --web-fixtures --select storage --engine Web=WebViewLive --force, then three runs of target/debug/sucrose.exe --no-portal with SUCROSE_DATA_DIR set to that folder (wslpath -w), SUCROSE_LOG="info,sucrose_engine_webview2=debug", WSLENV=SUCROSE_DATA_DIR:SUCROSE_LOG, 15 s each, stopped with taskkill.exe /IM sucrose.exe /F and 3 s apart. The renderer log of the DISPLAY1 slot shows kind=Persistent and the requests __storage/1, __storage/2 and __storage/3 over the runs; the DISPLAY9 renderer shows kind=Ephemeral and only __storage/1 in each run. Map slot to monitor through the monitor field of the "web profile" line and the primary field of the service's "renderers started" line.
   c. D9 on CEF: the same with --engine Web=CefSharpLive into target/uos-2mon-cef and SUCROSE_LOG="info,sucrose_engine_cef=debug"; same expectations.
   d. Stop everything started (taskkill.exe /IM sucrose.exe /F; never msedgewebview2.exe by name), copy the renderer and service logs to target/uos-logs/, and report the counts to the maintainer. Do not run cargo test --workspace with DISPLAY9 attached: an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen fails on its one-monitor count until item 261127-uou lands (known, not this item).
3. Maintainer (optional eye check during 2b): the storage page on DISPLAY1 shows 1, 2, 3 over the three runs and the one on DISPLAY9 shows 1 each time. Then detach DISPLAY9 or keep it attached (his choice) and give the verdict.
  </how-to-verify>
  <resume-signal>"approved" with whether DISPLAY9 stays attached, or the step numbers that failed and what was seen</resume-signal>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| service → renderer (local pipe) | The Assignment, now with web_profile, crosses from the service to a renderer it spawned; the pipe is owner-SID restricted (02-05) |
| renderer → profile folders on disk | A renderer locks, empties and hands folders under the user's data folder to a browser engine |
| renderer ↔ WebView2 runtime processes | msedgewebview2.exe processes started for a renderer hold files in its profile folder and outlive the controller's close |

## STRIDE Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation Plan |
|-----------|----------|-----------|----------|-------------|-----------------|
| T-uos-01 | Tampering | profile::acquire, Assignment.web_profile | medium | mitigate | The OS lock stays the only way to the persistent folder: a renderer told Persistent must still win the lock (bounded wait, then ephemeral) and one told Ephemeral never opens it, so a wrong or stale web_profile can never put two engines on one profile; tests in crates/sucrose-engine-web/tests/profile.rs (task 1) |
| T-uos-02 | Denial of Service | renderer start (lock wait, emptying retry) and WebView2 close (browser-exit wait) | medium | mitigate | START_WAIT and BROWSER_EXIT_WAIT are 1 s hard bounds, below SHUTDOWN_GRACE (2 s); a timed-out wait logs and continues; tests measure the bound (tasks 1 and 2) |
| T-uos-03 | Tampering | ephemeral emptying retry | medium | mitigate | The removed path stays <ephemeral_root>/<engine>/<slot> from the validated single-folder engine name and a u8 slot; the retry never changes the path and the persistent folder is never emptied; the engine-name test stays (task 2) |
| T-uos-04 | Spoofing | Assignment over the local pipe | low | accept | Only renderers the service spawned are accepted and the pipe is owner-SID restricted (02-05, D-14); a forged web_profile could only make a renderer try the lock, which still guards the folder |
| T-uos-05 | Denial of Service | msedgewebview2.exe of other applications | low | mitigate | The renderer waits for its own browser process through BrowserProcessExited; no check kills msedgewebview2.exe by name; only sucrose.exe and sucrose-portal.exe are stopped |
| T-uos-06 | Information Disclosure | new log fields | low | accept | The new fields are the monitor device name, slot, wanted and held kind and elapsed ms; the profile path was already logged; no page data is logged |
| T-uos-SC | Tampering | npm/pip/cargo installs | high | accept | No package install: webview2-com 0.39.1 already provides BrowserProcessExitedEventHandler and ICoreWebView2Environment5; no new crate (T-04-SC) |
</threat_model>

<verification>
- Task 1 and task 2 automated commands pass on one monitor; cargo xtask ci passes with DISPLAY9 detached.
- Task 3 evidence with DISPLAY9 attached: the WebView2 switching test passes with no profile WARN, and the storage count rises only on DISPLAY1 on both engines.
- git diff of docs/DECISIONS.md adds lines only; D-038 is unchanged.
- No Rust Sucrose process is left running after any step.
</verification>

<success_criteria>
- The primary monitor's renderer holds kind=Persistent at every start on CEF and WebView2 with two monitors; the other monitor runs ephemeral and never touches the lock (D9).
- Quick WebView2 theme switches with two monitors log no failed profile folder, and each closed WebView2 renderer logs its browser process exit before it ends (D10).
- docs/DECISIONS.md, docs/FEATURES.md and 04-CONTEXT D-33 describe the rule the code follows; line endings are unchanged.
</success_criteria>

<output>
Create `.planning/quick/261127-uos-04-37-d9-d10-give-the-persistent-web-profile-to-the-primary/261127-uos-SUMMARY.md` when done, with the measured browser-exit times, the two-monitor counts, the decision id used and the line-ending check.
</output>
