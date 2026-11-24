---
phase: 03-m2-service-ipc-and-portal-connected
reviewed: 2026-11-24T12:57:00Z
depth: standard
files_reviewed: 87
files_reviewed_list:
  - Cargo.toml
  - apps/sucrose-portal/Cargo.toml
  - apps/sucrose-portal/locales/legacy.en.yml
  - apps/sucrose-portal/locales/sucrose.en.yml
  - apps/sucrose-portal/src/app.rs
  - apps/sucrose-portal/src/assets.rs
  - apps/sucrose-portal/src/background.rs
  - apps/sucrose-portal/src/banner.rs
  - apps/sucrose-portal/src/dialogs/about.rs
  - apps/sucrose-portal/src/dialogs/help.rs
  - apps/sucrose-portal/src/dialogs/mod.rs
  - apps/sucrose-portal/src/dialogs/move_library.rs
  - apps/sucrose-portal/src/dialogs/tray_hint.rs
  - apps/sucrose-portal/src/header.rs
  - apps/sucrose-portal/src/i18n.rs
  - apps/sucrose-portal/src/ipc.rs
  - apps/sucrose-portal/src/library/card.rs
  - apps/sucrose-portal/src/library/grid.rs
  - apps/sucrose-portal/src/library/layout.rs
  - apps/sucrose-portal/src/library/mod.rs
  - apps/sucrose-portal/src/library/pagination.rs
  - apps/sucrose-portal/src/library/preview.rs
  - apps/sucrose-portal/src/library/thumbs.rs
  - apps/sucrose-portal/src/main.rs
  - apps/sucrose-portal/src/settings/general.rs
  - apps/sucrose-portal/src/settings/mod.rs
  - apps/sucrose-portal/src/settings/personal.rs
  - apps/sucrose-portal/src/settings/row.rs
  - apps/sucrose-portal/src/status_bar.rs
  - apps/sucrose-portal/src/theme.rs
  - apps/sucrose-portal/src/toast.rs
  - apps/sucrose-portal/themes/sucrose.json
  - apps/sucrose/Cargo.toml
  - apps/sucrose/src/fullscreen.rs
  - apps/sucrose/src/library_ops.rs
  - apps/sucrose/src/lifecycle.rs
  - apps/sucrose/src/main.rs
  - apps/sucrose/src/modules.rs
  - apps/sucrose/src/portal_link.rs
  - apps/sucrose/src/render.rs
  - apps/sucrose/src/service.rs
  - apps/sucrose/src/settings_host.rs
  - apps/sucrose/src/startup.rs
  - apps/sucrose/tests/common/mod.rs
  - apps/sucrose/tests/portal.rs
  - crates/sucrose-config/Cargo.toml
  - crates/sucrose-config/src/legacy.rs
  - crates/sucrose-config/src/lib.rs
  - crates/sucrose-config/src/migrate.rs
  - crates/sucrose-config/src/model.rs
  - crates/sucrose-config/src/store.rs
  - crates/sucrose-config/tests/migrate.rs
  - crates/sucrose-config/tests/store.rs
  - crates/sucrose-core/src/lib.rs
  - crates/sucrose-core/src/library.rs
  - crates/sucrose-core/src/paths.rs
  - crates/sucrose-core/src/settings.rs
  - crates/sucrose-core/src/theme.rs
  - crates/sucrose-core/tests/library.rs
  - crates/sucrose-engine-mpv/src/player.rs
  - crates/sucrose-ipc/Cargo.toml
  - crates/sucrose-ipc/src/lib.rs
  - crates/sucrose-ipc/src/portal.rs
  - crates/sucrose-ipc/src/protocol.rs
  - crates/sucrose-platform/Cargo.toml
  - crates/sucrose-platform/src/autostart.rs
  - crates/sucrose-platform/src/lib.rs
  - crates/sucrose-platform/src/os_linux.rs
  - crates/sucrose-platform/src/os_macos.rs
  - crates/sucrose-platform/src/os_windows.rs
  - crates/sucrose-platform/src/os_windows/autostart.rs
  - crates/sucrose-platform/src/os_windows/foreground.rs
  - crates/sucrose-platform/src/os_windows/tray.rs
  - crates/sucrose-platform/src/trash.rs
  - crates/sucrose-platform/src/unsupported.rs
  - crates/sucrose-platform/tests/autostart.rs
  - deny.toml
  - docs/ARCHITECTURE.md
  - docs/DECISIONS.md
  - docs/FEATURES.md
  - docs/GPUI.md
  - docs/MEASUREMENTS.md
  - xtask/Cargo.toml
  - xtask/src/localize.rs
  - xtask/src/main.rs
  - xtask/src/measure.rs
  - xtask/src/synth_library.rs
findings:
  critical: 2
  warning: 8
  info: 12
  total: 22
  necessity: 1
status: issues_found
---

# Phase 3: Code Review Report

**Reviewed:** 2026-11-24T12:57:00Z
**Depth:** standard
**Files Reviewed:** 87
**Status:** issues_found

## Summary

Reviewed the M2 service, IPC and Portal work: the service main loop and its library move, maintenance and Use paths (`apps/sucrose/src/service.rs`, `library_ops.rs`, `portal_link.rs`, `settings_host.rs`, `startup.rs`), the Portal client and pages, the settings store, the IPC protocol crates, the platform autostart code and the xtask tools. Doc and test files were read for contradictions with the code; the `.planning` files and lock files were excluded. No fallow structural findings were supplied and no external reviewer evidence was given. The necessity reference was read.

The IPC trust model is sound (the pipe has a single-SID DACL, `interprocess` rejects remote clients by default, renderer Hellos are checked against the pids the service spawned, `check_selected` and `contained_file` confine theme paths, the Portal executable is only ended after its path is verified). The defects are mostly at the edges of the new state machines.

Two issues should block the phase: the untrusted-image limits do not stop a progressive-JPEG memory bomb, and the settings store can overwrite a good settings file after a transient read error. The warnings are about the library move not being exclusive with other service work, a Portal registration that is dropped without closing its connection, and a few consistency gaps.

## Critical Issues

### CR-01: Progressive JPEG bypasses the decode limits (memory bomb from a theme thumbnail or background image)

**File:** `apps/sucrose-portal/src/library/thumbs.rs:250-283` (also `apps/sucrose-portal/src/background.rs:165-215`)
**Issue:** Theme images are documented as untrusted (T-03-25) and are guarded by `MAX_SIDE = 16384` per side plus `Limits::max_alloc` on the generic decoder. The JPEG fast path (`decode_jpeg_scaled`) uses `jpeg_decoder` directly and none of those limits apply to it. In `jpeg-decoder 0.3.2` a progressive frame allocates `vec![0i16; block_count * 64]` for every component at the first scan, at the full image size, before any scaling (`decoder.rs:401-412`). `scale()` only shrinks the output. `set_max_decoding_buffer_size` defaults to `usize::MAX` and is checked only against the output size after all scans were read (`decoder.rs:631-642`). A 16384 x 16384 progressive JPEG whose file is a few KB (well under the 64 MB file cap) passes the header check (`into_dimensions`, then `width > MAX_SIDE`) and then allocates about 3 x 16384^2 x 2 bytes = 1.6 GB. `DECODE_SLOTS = 3` lets three of them decode at once, and the library decodes every thumbnail of the shown page without user action. One crafted theme in the library therefore exhausts memory and kills the Portal each time the library is shown. The background-image path has the same hole.
**Fix:** Bound the pixel count, not only each side, before the JPEG decoder runs, and do it for both call sites (one shared helper):
```rust
/// Largest decoded image area the Portal accepts, in pixels (a progressive JPEG needs about 6 bytes per pixel).
pub const MAX_PIXELS: u64 = 32 * 1024 * 1024;

fn check_dimensions(width: u32, height: u32) -> anyhow::Result<()> {
    if width > MAX_SIDE || height > MAX_SIDE || u64::from(width) * u64::from(height) > MAX_PIXELS {
        bail!("the image is {width} x {height} pixels, over the limit");
    }
    Ok(())
}
```
Call it from `decode_bytes` (thumbs.rs) and `decode_bytes` (background.rs) right after `into_dimensions`, and add a test that a 16384 x 16384 header is refused.

### CR-02: A transient read error makes the settings store restore an older backup, or migrate and overwrite, a good `Sucrose.json`

**File:** `crates/sucrose-config/src/store.rs:85-131, 181-196`
**Issue:** `read_file` maps every I/O error except `NotFound` to `Read::Corrupt` (lines 191-194). `Store::open` treats `Corrupt` exactly like a file that does not exist: it loads `Sucrose.json.bak` and writes its bytes over `Sucrose.json` with `write_atomic` (lines 98-103), or, when there is no backup, runs `migrate` and calls `persist()` (lines 117-130), which replaces the file with the migrated legacy values or the defaults. A sharing violation, an antivirus scan, a network-drive hiccup or an access-denied while the file is held by another process is not corruption, yet it makes the service silently roll back the user's settings (or reset them completely on a first start with a legacy folder) and write that over the only good copy. The legacy reader in the same crate already knows that such errors are transient (`legacy.rs:250-259`, three attempts), the store does not.
**Fix:** Distinguish "unreadable" from "corrupt". Retry the read (as `legacy::read` does) and, if it still fails with an I/O error, return `Err(StoreError::Io)` (the host already falls back to an in-memory, no-save mode, which is safe) instead of restoring or migrating over the file:
```rust
enum Read { Missing, Corrupt, Unreadable(io::Error), Good(Box<Settings>, Vec<u8>) }
// in open(): if let Read::Unreadable(error) = read_file(&path) { return Err(error.into()); }
```
Only `Missing` and `Corrupt` (valid read, invalid content) may reach the backup and migration branches.

## Warnings

### WR-01: MaintainLibrary is accepted while a library move runs

**File:** `apps/sucrose/src/service.rs:1296-1310` (compare `service.rs:305-308`)
**Issue:** `MoveEnv::start` refuses a move while a maintenance run is going (`maintenance_running`), but the `MaintainLibrary` arm only checks `library_job` and never `fleet.library_move`. A Portal that asks for maintenance during the Closing, Copying or Finishing stage starts `library_ops::maintain` on the old location while `move_library` renames and copies the same folders on the other worker. Concretely: `copy_showcase` can create a showcase folder in the source after `entries_of` listed it, so it is neither moved nor deleted and then `record_names` (`service.rs:1353-1358`) records it as copied, so it is never copied again into the new library; `trash_corrupt` can send a theme to the bin while it is being renamed or verified, which makes `copy_verified` fail and rolls the move back for no user-visible reason. The Portal normally sends MaintainLibrary once per connection after the first scan, so a reconnect during a move reaches this path.
**Fix:** Make the two exclusive in both directions:
```rust
PortalRequest::MaintainLibrary => {
    if library_job || fleet.library_move.is_some() {
        // the answer comes with the run that is going, or the Portal asks again after the move
        tracing::debug!("library maintenance skipped: busy");
        portals.send(PortalEvent::LibraryMaintained(NOTHING_MAINTAINED));
    } else if let Some(root) = legacy_root.as_deref() { /* as before */ }
}
```

### WR-02: Portal requests are still handled once the service is quitting, so a move can be cut off by process exit

**File:** `apps/sucrose/src/service.rs:1218-1346` (request handling), `1185-1188`, `1425-1432`, `1503-1506`
**Issue:** `quitting` is checked for tray Open, Launch and the monitor timers, but not for `Event::Portal`. After Close Applications is chosen, the Portal gets `Close` but may still send `MoveLibrary`, `Use` or `Change`. `MoveEnv::start` does not look at `quitting`, so a move can begin after the renderers were told to close; `Event::ShutdownDone` / `Closing::done()` then return `Flow::Exit` while the `library-ops` thread is mid-copy. The tray Exit path explicitly protects against exactly this ("Leaving in the middle of a move could leave the library split between two folders", line 1429), but Close Applications and the shutdown worker do not. A Use during `quitting` also writes `Library.Selected` for a wallpaper that will never start (`start_shutdown` sets the supervisor's `closing` flag on its own thread, so a spawn can slip into the gap).
**Fix:** Ignore (or refuse with a result) every Portal request that changes state while `quitting`, and hold the exit while a move runs:
```rust
if quitting && !matches!(request, PortalRequest::RefreshStartup) { /* answer Refused / ignore */ continue; }
// and before returning Flow::Exit on ShutdownDone / closing.done(): wait while fleet.library_move.is_some()
```

### WR-03: The library move does not fully quiesce the fleet (monitor changes are dropped, a queued respawn can reopen the old folder)

**File:** `apps/sucrose/src/service.rs:1642-1660` and `1136-1147`
**Issue:** (1) While a move runs, a settled monitor snapshot (`step.apply`) is taken from the debounce and thrown away (line 1642). `Fleet::monitors` is not updated and nothing re-applies the snapshot when the move ends, so a monitor plugged in during a move gets no renderer (and an unplugged one keeps its slot) until the next unrelated display event. (2) `Event::Exited` calls `respawn_closed` for a slot the fullscreen rule had closed (`note_exit`, line 1143) without checking `library_move`; a slot already marked `back` before the move starts is respawned immediately and opens a file of the folder that is being renamed or copied (a held file makes `rename` fail, falls back to copy, and `finish_move` cannot delete it).
**Fix:** Keep the pending snapshot and re-run `apply_snapshot` from `MoveEnv::end` (or re-arm the debounce there), and guard `respawn_closed` (return early while `library_move.is_some()`); `resume_wallpaper` already calls it afterwards.

### WR-04: A Portal dropped for a full queue keeps its pipe open, the Portal is never told, and the service may never exit

**File:** `apps/sucrose/src/portal_link.rs:176-189`; `apps/sucrose/src/service.rs:2005-2050, 1392-1399`
**Issue:** `PortalRegistry::send` sets `self.registered = None` when `offer` fails and logs "its connection is dropped". Dropping the `PortalLink` only ends the writer thread; the reader thread in `serve_portal` still owns the other handle of the same pipe, so the connection stays open and the Portal sees nothing: it keeps showing "connected" with no further events, and its requests are ignored as "not registered". D-033 says such a Portal "reconnects"; it cannot, because it never learns it was dropped (and a second Portal may now register next to it). When that Portal later exits, `portals.gone(connection)` returns `false` (it is no longer registered) so `check_exit` is not set (lines 1392-1399) and a service that was kept alive only by that Portal (tray hidden, no wallpaper) stays running.
**Fix:** Make the drop close the connection: keep a second handle (or a shutdown flag) that the reader checks, or have `send` return the dropped link's connection id so the reader thread can `break`/drop its handle; and set `check_exit` when any Portal connection ends, registered or not.

### WR-05: Turning startup off can fail half-way and leave the setting and the registry disagreeing

**File:** `crates/sucrose-platform/src/os_windows/autostart.rs:224-232`; `apps/sucrose/src/startup.rs:83-96`
**Issue:** `disable_autostart` deletes the Run value first and then returns an error if deleting the StartupApproved flag fails (`delete_value(&approved, ..)?`). `set_startup` treats any error as "nothing changed" (`StartupChange::Failed`, no `run_startup = 0`), so the Run value is already gone while `General.RunStartup` still says 1. `enable_autostart` handles the same flag as best-effort (a warning, line 211-219); the two are inconsistent. The Portal then shows Normal with "no entry" until the user acts.
**Fix:** Treat the approved-flag removal as best-effort in `disable_autostart` too (log and return `Ok` once the Run value is gone), or roll the Run value back when the flag cannot be removed.

### WR-06: `synth-library` guard against the real data folder is lexical and bypassed by a verbatim path

**File:** `xtask/src/synth_library.rs:328-347, 428-436, 462-464`
**Issue:** The tool deletes `<out>/Library`, `<out>/Setting` and `<out>/.pool` recursively. The only protection for the real Sucrose folder is `is_inside`, which compares lowercased path components of `std::path::absolute(out)` and `legacy_data_dir()`. `--out "\\?\C:\Users\<me>\AppData\Roaming\Sucrose"` has a different `Prefix` component (`\\?\C:` against `C:`), as do an 8.3 short name or a junction/symlink to the folder, so the guard passes and `remove_dir_all` runs on the maintainer's real Library and Setting folders. CLAUDE.md treats those as read-only real data. A missing `.synth` marker is not an obstacle either: an empty or newly named folder passes, and `--force` skips the emptiness check.
**Fix:** Canonicalize both sides (the `out` ancestor that exists and the real folder) and strip the verbatim prefix before comparing, and also refuse when `SUCROSE_DATA_DIR` / `paths::data_dir()` is inside `out`:
```rust
let real = paths::legacy_data_dir().and_then(|p| fs::canonicalize(p).ok());
let target = nearest_existing_ancestor(&out).and_then(|p| fs::canonicalize(p).ok());
```

### WR-07: Theme folder names bypass the control-character filter on the tray and log path

**File:** `apps/sucrose/src/service.rs:511-541` (lines 536-538), `563-571`
**Issue:** IN-08 requires that text from a theme or settings reaches the log and the tooltip only through `printable()`. `plan_for` does that (`service.rs:3289`), but `resume_wallpaper` builds `Status::NoWallpaper { reason: rejection.to_string() }` from `selected_job` without it. `ThemeRejection::FolderMissing/InfoUnreadable/NotVideo/SourceInvalid` interpolate `library.selected` (any string in `Sucrose.json`), so a settings value with a newline or an escape sequence forges a log line (`tracing::warn!(status = %status.text(), ...)`, `settle_move`) and garbles the tray tooltip after a library move.
**Fix:** `reason: printable(&rejection.to_string())` here, or better make `Status::NoWallpaper`'s constructor sanitize once so no caller can forget.

### WR-08: A queued restart of the old file can be taken for the first frame of the new source

**File:** `crates/sucrose-engine-mpv/src/player.rs:338-358, 646-657`
**Issue:** `load_source` sets `loading = true` and then issues `loadfile`. The event thread treats the next `PLAYBACK_RESTART` while `loading` is set as the new file's first frame (`loading.swap(false)`), sends `PlayerEvent::Started` and re-anchors on that position. A restart event of the old file that is already queued in mpv (a loop wrap or a seek that finished just before the switch, frequent for a short looping wallpaper) is consumed by that branch: `has_started()` turns true with the old frame, the clock thread announces `Playing` for the new theme before it plays, and the follower anchors on the old file's position. The window is small but real for looping videos.
**Fix:** Tie the flag to the file: compare `MPV_EVENT_START_FILE` / the `file-loaded` event generation (or count `loadfile` calls and ignore a restart that precedes the matching `FILE_LOADED`) instead of using one boolean.

## Info

### IN-01: No limit on connections and no read timeout on the first message

**File:** `apps/sucrose/src/service.rs:1799-1855`
**Issue:** `accept_loop` starts one thread per connection and `read_connection` blocks forever on the first message. A same-user process that connects and stays silent (or a crashed launcher) leaves a thread per connection for the life of the service, and the unbounded event channel has no back-pressure for a client that floods `PortalRequest`s. The pipe DACL limits this to the same user, so it is robustness, not privilege.
**Fix:** Cap concurrent unregistered connections and give the first read a deadline (read on a helper with `recv_timeout`, as `serve_launcher` does for its reply).

### IN-02: Endpoint hardening is Windows-only while Linux gets an abstract socket

**File:** `crates/sucrose-ipc/src/lib.rs:135-145, 181-196`
**Issue:** The module doc says a Unix socket elsewhere, but `GenericNamespaced::is_supported()` is true on Linux, so `socket_name` uses an abstract socket there, which has no filesystem permissions; `restrict_to_owner` exists only for Windows and the endpoint name falls back to the `USER` variable. Another local user could connect to or pre-bind `Sucrose.Service.<user>` once the Linux service exists (today `EventLoop::new()` fails there, so it is latent).
**Fix:** Before enabling Linux, use a path socket in a 0700 runtime directory (`XDG_RUNTIME_DIR`) and check peer credentials; fix the doc comment now.

### IN-03: Duplicated code

**File:** `apps/sucrose-portal/src/library/thumbs.rs:269-283` and `apps/sucrose-portal/src/background.rs:196-215`; `apps/sucrose-portal/src/library/mod.rs:65-72` and `apps/sucrose-portal/src/settings/mod.rs:61-64`
**Issue:** `decode_jpeg_scaled` exists twice (with different targets), which is how CR-01 has to be fixed in two places; `bin_name` exists twice (one by `cfg!(windows)`, one by `std::env::consts::OS`).
**Fix:** One `decode_jpeg_scaled(bytes, (w, h))` in `thumbs.rs` and one `bin_name`.

### IN-04: Stale `#[allow(dead_code)]` with outdated reasons

**File:** `apps/sucrose-portal/src/ipc.rs:148-152`, `apps/sucrose-portal/src/app.rs:27-30`, `apps/sucrose-portal/src/main.rs:21-22`
**Issue:** The reasons say "the following plans" will call these, but `ServiceLink::send`, the `PortalApp` fields and the `i18n` helpers are used now. The blanket allows hide the next real dead item from clippy.
**Fix:** Remove them and fix whatever clippy still reports.

### IN-05: `StartActions.start_wallpaper` is read only by tests

**File:** `apps/sucrose/src/lifecycle.rs:10, 20`
**Issue:** The field is set from `selected_exists` but nothing in non-test code reads it (`service.rs` uses `start.open_portal` only).
**Fix:** Drop the field, or use it where the wallpaper start is decided.

### IN-06: Generality without instantiation, the `Restart` use transition

**File:** `apps/sucrose/src/service.rs:2814-2828, 2993-2996, 3128-3144`
**Category:** necessity
**Issue:** `UseTransition::Restart`, `USE_TRANSITION` and `Fleet::restart_with` exist for a fallback "if the maintainer sees a black frame". The constant is fixed to `LoadSource` (`service.rs:2828`), the `Restart` arm is unreachable (`#[allow(dead_code)]`) and `restart_with` has no other caller. It is a deliberate documented fallback, so this rests on judgment.
**Fix:** If the fallback is not needed after the D-26 verification, delete the enum, the const and `restart_with` and keep the decision in DECISIONS.md.

### IN-07: Paths in info-level log lines, against the "values only" rule

**File:** `apps/sucrose/src/service.rs:876`, `apps/sucrose-portal/src/ipc.rs:408`, `crates/sucrose-config/src/legacy.rs:287`
**Issue:** `started the Portal`, `started the service` and `legacy setting file read as defaults` log full paths (user profile folder) at info/warn, while the migration and settings logs deliberately avoid paths (D-08, T-02-40). A missing legacy file is also logged at warn although it is the normal case without the C# app.
**Fix:** Log the file name only, and log `NotFound` at debug.

### IN-08: The Portal re-derives the default library folder instead of using the service's answer

**File:** `apps/sucrose-portal/src/library/mod.rs:79-100`
**Issue:** `PortalSettings.library_location` is the configured value, empty for the default, and the Portal rebuilds the default from `paths::legacy_data_dir()` while the service uses `Settings::library_location`. The two agree today, but a change in one silently makes the Portal scan a different folder than the one the service maintains and moves.
**Fix:** Have the service send the resolved folder (clipped to `MAX_PATH_TEXT`) in `Welcome`.

### IN-09: A refresh answer can cause a false "could not be saved" toast

**File:** `apps/sucrose-portal/src/settings/general.rs:603-612`
**Issue:** `on_startup` compares any `Startup` event with `pending_startup`. The service answers `RefreshStartup` from a worker thread (`start_legacy_check`) but a `SetStartup` on the main thread, so a refresh answer built from the settings before the `SetStartup` was applied can arrive while a request is pending and show an error toast and revert the select until the real answer arrives.
**Fix:** Only clear and compare `pending_startup` for the answer that follows a `SetStartup` (tag it, or ignore refresh answers while a request is pending).

### IN-10: An invalid `Version` is read as current and then overwritten

**File:** `crates/sucrose-config/src/model.rs:744-753`
**Issue:** A `Version` that does not fit `u32` (or is not a number) is replaced by `SETTINGS_VERSION`, so the "file from a newer build is kept unchanged" protection (`store.rs:169-179`) does not apply and the next write replaces the file.
**Fix:** Treat a present but unreadable `Version` as newer (or corrupt) and keep the file.

### IN-11: Background image path check accepts NUL and UNC paths

**File:** `apps/sucrose/src/settings_host.rs:121-142`
**Issue:** `check_image` requires an absolute path and an image extension but, unlike `parse_destination` (`service.rs:262-273`), does not reject NUL, and on Windows a UNC path (`\\host\share\x.png`) is absolute, so the Portal's `decode_file` would open a remote share when such a value arrives over IPC or from the settings file (NTLM authentication to that host).
**Fix:** Share one path validator with `parse_destination` and refuse UNC and device prefixes for the background image.

### IN-12: Timing assertion in the memory integration test is tight

**File:** `apps/sucrose/tests/portal.rs:47-81`
**Issue:** The test requires two memory reports within 3 s of the Welcome while the module reports once per second after the Portal registers (first report at about 1 s, second at about 2 s); a loaded CI machine or antivirus delay makes it flaky.
**Fix:** Assert on arrival of two reports with a generous deadline (for example 10 s) and keep the period check in a unit test of `MemoryModule`.

---

_Reviewed: 2026-11-24T12:57:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
