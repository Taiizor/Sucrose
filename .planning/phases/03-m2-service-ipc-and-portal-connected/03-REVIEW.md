---
phase: 03-m2-service-ipc-and-portal-connected
reviewed: 2026-11-24T20:00:00Z
depth: standard
files_reviewed: 31
files_reviewed_list:
  - Cargo.lock
  - Cargo.toml
  - apps/sucrose-portal/Cargo.toml
  - apps/sucrose-portal/src/atlas.rs
  - apps/sucrose-portal/src/background.rs
  - apps/sucrose-portal/src/library/card.rs
  - apps/sucrose-portal/src/library/mod.rs
  - apps/sucrose-portal/src/library/preview.rs
  - apps/sucrose-portal/src/library/thumbs.rs
  - apps/sucrose-portal/src/main.rs
  - apps/sucrose/src/portal_link.rs
  - apps/sucrose/src/service.rs
  - apps/sucrose/src/settings_host.rs
  - apps/sucrose/src/startup.rs
  - crates/sucrose-config/src/store.rs
  - crates/sucrose-config/tests/store.rs
  - crates/sucrose-engine-mpv/src/player.rs
  - crates/sucrose-engine-mpv/tests/options.rs
  - crates/sucrose-ipc/src/lib.rs
  - crates/sucrose-ipc/tests/socket.rs
  - crates/sucrose-platform/Cargo.toml
  - crates/sucrose-platform/src/lib.rs
  - crates/sucrose-platform/src/os_windows.rs
  - crates/sucrose-platform/src/os_windows/autostart.rs
  - crates/sucrose-platform/src/os_windows/memory.rs
  - docs/FEATURES.md
  - docs/GPUI.md
  - docs/MEASUREMENTS.md
  - xtask/src/main.rs
  - xtask/src/measure.rs
  - xtask/src/synth_library.rs
findings:
  critical: 0
  warning: 4
  info: 10
  total: 14
status: issues_found
---

# Phase 3: Code Review Report

**Reviewed:** 2026-11-24T20:00:00Z
**Depth:** standard
**Files Reviewed:** 31
**Status:** issues_found

## Summary

Incremental re-review after the fixes for CR-01..CR-02 and WR-01..WR-08. The diff since 1571dc40 was read in full for service.rs, player.rs, portal_link.rs, the Portal image code, measure.rs and synth_library.rs; store.rs, ipc/lib.rs, startup.rs, settings_host.rs, autostart.rs, memory.rs and os_windows.rs were read in full. The previous critical fixes hold: the pixel-area limit refuses the 16384-square header before any decoder allocates, and a transient read error no longer replaces a good `Sucrose.json`. No new critical issue was found.

The remaining concerns are narrower. The WR-02/WR-01 fix for quitting during a library move introduces a path on which the service can never exit. The WR-08 fix replaced an atomic `swap` with a plain `store` that can erase a newer request's flag. The settings store still overwrites a corrupt `Sucrose.json` without keeping a copy when there is no `.bak`. The CR-01 limit is a cap per image, not a bound on concurrent decodes.

The necessity reference (`necessity-review.md`) was not read for this pass, so no `necessity:` count is given.

IDs: new findings number from WR-09 and IN-13. IN-02, IN-04, IN-10 and IN-11 are deferred ledger rows that are still present in the code; they keep their ledger ids and carry an Origin line.

## Warnings

### WR-09: A corrupt settings file with no backup is overwritten and its content is lost

**File:** `crates/sucrose-config/src/store.rs:105-143` (and `:167-178`)
**Issue:** `Store::open` treats `Read::Corrupt` like `Read::Missing`. With a corrupt `Sucrose.json` and no `.bak` (the first run after the `.bak` feature, a deleted backup, or any file whose previous version never was "good"), it falls through to migration or defaults and calls `persist()`. `persist` only backs up the file it replaces when that file reads as `Good`, so the corrupt file is replaced by `write_atomic` and no copy of it survives. A hand-edited file with a typo, a truncated file, or a file from a newer build that an older build cannot parse (an enum variant it does not know reads as corrupt, not as a newer `Version`) is silently replaced with defaults or the legacy migration. The user's settings are gone with only a `warn` log line. This is the same class of loss CR-02 closed for unreadable files; corrupt files were left out.
**Fix:** Before writing over a `Corrupt` main file, move it aside under a name that does not end in `.json` (for example `Sucrose.json.corrupt`, replacing an older one), and log that it was kept:
```rust
Read::Corrupt => {
    let aside = setting_dir.join("Sucrose.json.corrupt");
    if let Err(error) = fs::rename(&path, &aside) {
        tracing::warn!(%error, "the corrupt settings file cannot be kept aside");
    }
}
```
Do this only on the path that will persist (after the backup restore failed), and add a test that a corrupt file with no `.bak` is still present, byte for byte, after `open`.

### WR-10: A quit that arrives during a library move can never finish (service never exits)

**File:** `apps/sucrose/src/service.rs:1224-1226` (with `:1242`, `:430`, `:1636-1637`, `:1669-1678`)
**Issue:** The WR-02 fix keeps the service alive on the Quit path until a running library move ends (`quit_done` plus `settle_exit`). But the renderer-exit handler returns early with `if quitting { continue; }` before `fleet.rule_closed.note_exit(exited.slot)`. A move in its `Closing` stage has marked every slot in `rule_closed` (`close_for_move`) and `MoveEnv::drive` only advances when `!rule_closed.any_awaiting()`. Once `quitting` is set, no exit event is ever recorded as ended, so `any_awaiting()` stays true, the stage stays `Closing`, `fleet.library_move` stays `Some`, and `settle_exit` returns false forever. `MOVE_TIMER` only calls `kill_survivors` once and is not re-armed. Before the fix the Quit path returned `Flow::Exit` at once, so this is a regression. It is reachable only when `LoopEvent::Quit` is handled after a move started: the only sender is `tray_proxy.quit()` at line 1651, which requires no registered Portal and no move, so the window is a Portal that connects and starts a move between the post and the handling of the quit message. It is narrow, but the code was written to handle exactly this case and the new tests only cover the pure predicates (`may_exit`, `resume_after_move`), not the sequence.
**Fix:** Record the exit for the move before the quitting early-return, for example call `fleet.rule_closed.note_exit(exited.slot)` (and, if `library_move` is some, keep the `fleet.stopped`/`shell` bookkeeping skipped) ahead of `if quitting { continue; }`. Add a test that drives `Closing` plus quit plus renderer exits to `ended`. Alternatively give `settle_exit` a deadline after which a move is abandoned (rolled back or left to the next start's recovery) so a stuck move cannot hold the process.

### WR-11: The event thread clears `loading` with a plain store and can erase a newer `load_source`

**File:** `crates/sucrose-engine-mpv/src/player.rs:726-728` (also `:746-748`), against `:412-414`
**Issue:** WR-08 replaced `shared.loading.swap(false)` with `switch.restart()` followed by `shared.loading.store(false)`. `load_source` runs on another thread and does `loading.store(true)` then `load_generation.fetch_add(1)`. If a second `load_source` runs between the event thread's `switch.restart()` returning `NewStart` (for request N) and its `loading.store(false)`, the store wipes the flag the newer request set. The newer request is still tracked by `SourceSwitch` (it reads the generation on the next event), so a `Started` event will follow, but until then `has_started()` returns true and `position`/first-frame readings are taken as belonging to the new file, which is what `loading` exists to prevent. The old `swap` was a single atomic step per restart; the new code splits decision and clear. The window is a few microseconds between two back-to-back loads, but the comment in `event_loop` already acknowledges back-to-back loads as the scenario.
**Fix:** Clear `loading` only when no newer request exists: compare `shared.load_generation.load()` with `switch.requested` first, and skip the clear when they differ, or replace the bool with the generation (`loading_for: AtomicU64`, `has_started` compares it with `started_generation`) so there is nothing to wipe.

### WR-12: The pixel-area cap bounds one JPEG decode, not memory; three decodes plus the background image can hold about 800 MB

**File:** `apps/sucrose-portal/src/library/thumbs.rs:29`, `:270`, `:281-300`; `apps/sucrose-portal/src/background.rs:171`
**Issue:** CR-01 was fixed by refusing images over `MAX_PIXELS` (32 Mi pixels). The comment itself puts a progressive JPEG at about 6 bytes per pixel before scaling, so one accepted file costs about 200 MB. `DECODE_SLOTS` is 3 and the background image decodes on its own task, so four hostile theme thumbnails (or one background and three thumbnails) can hold about 800 MB at once, while the generic decoder is capped at `MAX_DECODE_ALLOC` (256 MB per image) and the Portal's stated budget is a 48 MiB cache. jpeg-decoder 0.3.2 has `Decoder::set_max_decoding_buffer_size` (decoder.rs:163, enforced at :636 for the coefficient buffer) and it is not used. Theme folders are untrusted input (T-03-25).
**Fix:** Set the limit on the decoder in `decode_jpeg_scaled` so the decoder refuses before allocating, with a value aligned to `MAX_DECODE_ALLOC`:
```rust
let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(bytes));
decoder.set_max_decoding_buffer_size(MAX_DECODE_ALLOC as usize);
```
and consider lowering `MAX_PIXELS` to what a thumbnail source realistically needs (for example 16 Mi pixels), or sharing one decode budget between thumbnails and the background image.

## Info

### IN-02: Endpoint hardening is Windows-only while Linux gets an abstract socket

**Origin:** deferred ledger row IN-02
**File:** `crates/sucrose-ipc/src/lib.rs:140-148`, `:162-168`
**Issue:** Still present. `GenericNamespaced::is_supported()` is true on Linux, so the endpoint is an abstract Unix socket that any local user can connect to, and `owner_sid` is discarded (`let _ = owner_sid`). Latent until the Linux service exists, as the ledger says.
**Fix:** As deferred: before Linux support is enabled, use a filesystem socket in a mode 0700 per-user folder, or check peer credentials on accept.

### IN-04: Stale `#[allow(dead_code)]` with an outdated reason

**Origin:** deferred ledger row IN-04
**File:** `apps/sucrose-portal/src/main.rs:22`
**Issue:** Still present: `#[allow(dead_code, reason = "the Portal pages of the following plans call these helpers")]` on `mod i18n`. The reason refers to plans that have since landed, so it hides genuinely unused helpers.
**Fix:** Remove the allow and delete or use whatever the compiler then reports.

### IN-10: An invalid `Version` is read as current and then overwritten

**Origin:** deferred ledger row IN-10
**File:** `crates/sucrose-config/src/model.rs:744-750`
**Issue:** Still present: an invalid `Version` logs "using the current version" and the next update writes the file. Relates to WR-09, because a damaged or newer-format version field is the way a newer file can be lost.
**Fix:** As deferred; with WR-09 fixed, also treat an unparsable `Version` as "keep a copy aside before the first write".

### IN-11: Background image path check accepts NUL and UNC paths

**Origin:** deferred ledger row IN-11
**File:** `apps/sucrose/src/settings_host.rs:121-142`
**Issue:** Still present: `check_image` checks emptiness, length, absoluteness and extension only. A path with an embedded NUL passes and is stored; a UNC path (`\\\\server\\share\\a.png`) makes the Portal open a network location on the user's behalf. Same-user IPC only, as the ledger says.
**Fix:** Reject `\0` in the text and, unless UNC backgrounds are wanted, reject paths that start with `\\\\`.

### IN-13: The doc comment of `monotonic_ns` is attached to `disconnect_pipe`

**File:** `crates/sucrose-platform/src/os_windows.rs:105-118`
**Issue:** The two doc lines of `monotonic_ns` ("QueryPerformanceCounter scaled to nanoseconds ... RESEARCH A4") sit directly above the doc of `disconnect_pipe`, so rustdoc merges them into the documentation of `disconnect_pipe` and `monotonic_ns` has none.
**Fix:** Move the two lines down to `pub(crate) fn monotonic_ns`.

### IN-14: A Portal dropped by `send` during Close Applications is not counted as gone

**File:** `apps/sucrose/src/portal_link.rs:218-229`, `apps/sucrose/src/service.rs:1514-1531`
**Issue:** `PortalRegistry::send` clears `registered` when the queue is full. During Close Applications `closing` waits for exactly that Portal, but the later `PortalGone` is classed as "not the registered one" (`was_registered` is false), so `closing.portal_gone()` is not called. The service then waits for `CLOSE_TIMER` (2 s) and ends the process, instead of leaving at once. `portal_gone_effect` is also a function that returns `check_exit: true` in both cases.
**Fix:** Remember the pid or connection of the Portal the closing sequence waits for (`closing_portal`) and call `closing.portal_gone()` when a connection of that Portal ends; inline `check_exit = true`.

### IN-15: GPUI.md contradicts itself about whether the preview cost is measured

**File:** `docs/GPUI.md:46-47`
**Issue:** The bullet on hover previews ends "The preview cost is still not in the UI-07 numbers until the D-25 runs are repeated", and the next bullet and `docs/MEASUREMENTS.md` say the runs were repeated with animating previews and pass. The project rule is that docs stay true to each other in the same change.
**Fix:** Replace the sentence with a pointer to the repeated runs ("the preview cost is inside the 2026-11-24 repeat rows of MEASUREMENTS.md").

### IN-16: `request_page` reads heap and DXGI memory on the GPUI thread

**File:** `apps/sucrose-portal/src/library/mod.rs:525-531`, `crates/sucrose-platform/src/os_windows/memory.rs:33-97`
**Issue:** With debug logging on, every page change calls `memory_detail()` on the UI thread: `HeapSummary` takes the process heap lock and walks every block, and `CreateDXGIFactory1` plus adapter enumeration is a COM round trip. The UI rule (D-12, `.claude/rules/ui.md`) is that the GPUI thread does no blocking work. It is gated by `tracing::enabled!(DEBUG)` and is a measurement aid, so the cost is only in diagnostic runs, but it also perturbs the measurements it supports and the heap lock stalls decode threads.
**Fix:** Take the reading on `cx.background_executor()` and log it from there, or sample it from the measurement tool only.

### IN-17: `measure --side portal` reports success when the Portal crashed

**File:** `xtask/src/measure.rs:2152-2163` (and the `ended_by` assignment above it)
**Issue:** `ended_by` is `portal-exit` for any early end, so a crash and the maintainer closing the Portal after the idle window look the same, and the command now exits 0 in both cases (it used to fail). The page-log summary accepts such a series; rounds and 20 s idle checks catch most crashed runs but not a crash during the final idle window, which then produces a "final" value from a shortened tail.
**Fix:** Record the Portal's exit code or ask the maintainer to close it only after the planned time, and keep the non-zero exit when the process ended with a failure code.

### IN-18: The scratch endpoint name hashes the unnormalized data-folder text

**File:** `crates/sucrose-ipc/src/lib.rs:115-126`
**Issue:** `endpoint_name_for` hashes `SUCROSE_DATA_DIR` as written. `E:\\scratch`, `e:\\scratch\\` and a junction to it give three endpoint names, so two services can run on one data folder, which breaks the rule that the service is the only writer of `Sucrose.json`. It also lossy-converts non-UTF-8 paths, so two different folders can collide. Only scratch runs set the variable.
**Fix:** Hash the result of the same resolution that `synth_library::resolve` does (absolute, canonical where it exists, verbatim prefix stripped, lowercase on Windows), or document that the value must be spelled the same way for every process.

---

_Reviewed: 2026-11-24T20:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
