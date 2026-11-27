---
phase: quick-261127-uou
plan: 01
subsystem: testing
tags: [portal-tests, multi-monitor, renderer-logs]
requires: [261127-uos]
provides:
  - per-slot renderer log helpers and per-slot engine-start assertions in apps/sucrose/tests/portal.rs
affects: [apps/sucrose/tests/portal.rs, deferred-items.md]
key-files:
  modified:
    - apps/sucrose/tests/portal.rs
    - .planning/phases/04-m3-engines-and-the-wallpaper-api/deferred-items.md
decisions:
  - "The renderer slot is the unit of every engine-start count; the joined renderer_logs stays for presence checks only"
metrics:
  tasks: 3
  completed: 2026-11-27
status: complete
actuals:
  tokens: 9000
  tasks: 3
  commits: 4
plan_head_before: 93e3f099b1bbdc7c1d7eb826d264a7cb143b313f
plan_head_after: 7b0f4abdc84b055131e04914e1c4bd1f2f897a9e
---

# Phase quick-261127-uou Plan 01: per-slot engine-start counting in portal.rs Summary

The real-desktop tests of apps/sucrose/tests/portal.rs now wait for and assert engine starts on the log of each monitor's renderer slot, so the suite passes with DISPLAY9 attached exactly as with one monitor; no product code changed.

## Commits

- 72e4b592 test: failing tests for per-slot renderer logs (RED, does not compile)
- c3a0fb82 test: per-slot helpers (slot_of, logs_by_slot, renderer_logs_by_slot, starts, every_slot_started, monitor_slots, wait_for_engine_starts_on_every_slot) and the engine-change test on them
- 0c9d897e test: every other engine-start count per slot; joined-count wait helper and renderer_log_files removed
- 7b0f4abd docs: both portal.rs entries of deferred-items.md resolved

## What was done

- Task 1 (tracer): helpers plus pure tests renderer_logs_are_grouped_by_the_slot_in_their_file_name and every_monitor_slot_has_to_reach_the_count (the 04-37 shape). The engine-change test checks each slot (Cef first, WebView2 after).
- Task 2: converted a_gif_engine_change..., switching_web_themes_on_webview2..., restart_now_restarts_only_the_affected_wallpapers, saved_webview2_arguments_reach_the_restarted_renderer, and the new a_webview2_renderer_closed_right_after_its_start_on_a_new_profile_exits_at_once (e747c7be). The no-WARN check of the switching test stays over every slot's whole log, unfiltered. Greps: no joined engine-start count, no old wait helper, 11 callers of the new wait, one " WARN " assertion kept.
- Task 3: DISPLAY9 was already attached by the maintainer (Screen.AllScreens: \\.\DISPLAY1 primary, \\.\DISPLAY9), so no checkpoint was needed.

## Verification (two monitors, Screen.AllScreens.Count = 2)

- target/uou-2mon-1.log: test result: ok. 34 passed; 0 failed; 8 ignored (second attempt, see flake below)
- target/uou-2mon-2.log: test result: ok. 34 passed; 0 failed; 8 ignored
- target/uou-2mon-ignored.log: test result: ok. 3 passed (restart_now..., saved_webview2_arguments..., a_gif_engine_change...)
- target/uou-2mon-workspace.log: cargo test --workspace exit 0 (portal binary 34 passed; first binary 402 passed)
- No ": skipped," line in any of the logs; Renderer-0 and Renderer-1 logs exist in engine-restart (Cef then WebView2, two starts each) and webview2-switch (three WebView2 starts each, 0 WARN). cargo clippy --workspace --all-targets -- -D warnings and cargo fmt --check are clean; Cargo manifests untouched; portal.rs and deferred-items.md stay LF.
- Baseline before the Task 2 conversion (two monitors, target/uou-t2-baseline.log): all three ignored tests failed on the joined counts ("a saved setting never restarts the wallpaper by itself" left 2 right 1; the GIF built-in count).
- tasklist.exe shows no sucrose processes.

## Deviations from Plan

- The single-monitor proofs of Tasks 1 and 2 were run on two monitors (DISPLAY9 was attached before the item started and the maintainer asked not to detach); the per-slot logic is the same for one slot and the pure tests cover the one-slot shape. A one-monitor run was not repeated.
- [Rule 1 - Bug] The new early-close test (e747c7be) waited for the first start on the joined log and read the renderer file count; it is converted to a per-slot wait with a 10 ms poll (wait_for_engine_starts_polling) so both renderers are still closed right after their own start, and its exit/close assertions are per slot. renderer_log_files was removed with it.

## Observations (not fixed, out of scope)

- Flake: a_webview2_renderer_closed_right_after_its_start_on_a_new_profile_exits_at_once failed once in a full-suite two-monitor run ("the browser process of slot 0 exited within a second of the close: [1034]"); alone it takes 190-300 ms, passed in five isolated runs and in the next three full-suite runs (the exit time under suite load varied 428-703 ms, close to the 1000 ms bound). The bound is tight under load; counting is not involved. A 1 s wait at the product (BROWSER_EXIT_WAIT) plus the 1000 ms test bound leaves no margin.
- engine-restart: a CEF WARN "the page failed to load code=ERR_ABORTED url=https://sucrose.wallpaper.engine/index.html" appears intermittently in a slot's log when CEF is replaced during its page load; that test does not assert the absence of WARN and the switching test (WebView2 only) had none.

## Known Stubs

None.

## Self-Check: PASSED

- apps/sucrose/tests/portal.rs contains renderer_logs_by_slot and wait_for_engine_starts_on_every_slot; commits 72e4b592, c3a0fb82, 0c9d897e, 7b0f4abd are ancestors of HEAD.
