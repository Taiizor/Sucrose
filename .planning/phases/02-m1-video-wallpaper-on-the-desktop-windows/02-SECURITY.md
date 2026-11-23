---
phase: "02"
slug: "m1-video-wallpaper-on-the-desktop-windows"
status: verified
# threats_open = count of OPEN threats at or above workflow.security_block_on severity (the blocking gate)
threats_open: 0
asvs_level: 1
created: "2026-11-22"
---

# Phase 02 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| GitHub release → developer machine | Archive bytes from the network are untrusted until the SHA256 matches the pin | see the 02-01 threat model |
| PATH / current folder → renderer process | DLLs found on PATH may be planted by another program | see the 02-01 threat model |
| Mirror or upstream release host → developer machine | Archive bytes are untrusted until the pinned SHA256 matches | see the 02-02 threat model |
| C# app's setting files → Rust service | Files written by another program, possibly truncated or hand-edited | see the 02-03 threat model |
| Theme folder → service | SucroseInfo.json is untrusted text; this plan only parses and logs it (plan 06 gates playback on it) | see the 02-03 threat model |
| Maintainer's real files → committed fixtures | Copies may carry personal paths or names | see the 02-04 threat model |
| Other local processes/users → service named pipe | Anything on the machine can try to open a named pipe | see the 02-05 threat model |
| Renderer process → service | Messages from a separate process are untrusted bytes | see the 02-05 threat model |
| Theme folder (possibly from the store) → service | SucroseInfo.json Source is untrusted text | see the 02-06 threat model |
| Renderer → explorer.exe window tree | Messages to Progman and window reparenting into another process's hierarchy | see the 02-06 threat model |
| Video file → libmpv in the renderer | Media and demuxer input is untrusted | see the 02-07 threat model |
| Theme folder → mpv options | A theme must not be able to make mpv load scripts, configs or URLs | see the 02-07 threat model |
| Theme folder (store download or user copy) → service | SucroseInfo.json fields are untrusted | see the 02-08 threat model |
| Renderer → service (Failed reason text) | Text from another process ends up in the tooltip and log | see the 02-09 threat model |
| Service ↔ child renderer processes | Children can crash or hang; the service must stay in control | see the 02-10 threat model |
| Legacy Engine.json values → mpv options | User-editable values become engine options | see the 02-11 threat model |
| User mpv config file → libmpv | A file in the user's profile can enable any mpv feature | see the 02-11 threat model |
| Window messages from the shell and other processes → service window | Broadcasts (TaskbarCreated, WM_DISPLAYCHANGE) can come from any process | see the 02-12 threat model |
| Other applications' windows → service | Foreground window data belongs to other programs | see the 02-13 threat model |
| Renderer connections → service relay | Clock messages from any renderer connection | see the 02-14 threat model |
| xtask → other processes' statistics | Reads data about the maintainer's running processes | see the 02-15 threat model |
| Other processes → service and renderer windows | Any process of the session can send window messages to the hidden loop window | see the 02-16 threat model |
| explorer.exe → renderer (WinEvent callbacks) | Window events of the shell process run code on the renderer thread | see the 02-16 threat model |
| Sucrose → the user's registry (HKCU) | The platform writes one value of a per-user key that Windows' graphics settings own | see the 02-17 and 02-18 threat models |
| Legacy Engine.json → service | A user-editable file decides which GPU preference is written | see the 02-18 threat model |
| Renderer connections → service (master election) | Playing messages decide which renderer becomes the master clock | see the 02-19 threat model |
| Service relay → follower | Clock ticks from the master drive the follower's speed and seeks | see the 02-20 threat model |
| xtask → other processes (GPU adapters and counters) | The measure task reads process statistics and OS performance counters of the C# app, dwm.exe and Sucrose | see the 02-21 threat model |
| Generated mpv options → libmpv in the renderer | Option changes alter what the engine does with untrusted theme video files | see the 02-22 threat model |

---

## Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation | Status |
|-----------|----------|-----------|----------|-------------|------------|--------|
| T-02-01 | Tampering | xtask/src/fetch_engines.rs | high | mitigate | SHA256 of the whole archive is compared with the pin in xtask/engines.toml before sevenz-rust2 opens it; mismatch deletes the file and fails; no unpinned fallback — **Evidence:** `xtask/src/fetch_engines.rs` hashes the whole archive and compares with the `engines.toml` pin before extraction; mismatch deletes the file and fails | closed |
| T-02-02 | Elevation of Privilege | crates/sucrose-engine-mpv/src/loader.rs | high | mitigate | Load by absolute path only, exe folder first, PATH entries filtered to absolute ones, current folder never searched, LOAD_WITH_ALTERED_SEARCH_PATH; the loaded path is logged — **Evidence:** `crates/sucrose-engine-mpv/src/loader.rs`: absolute-path load, `LOAD_WITH_ALTERED_SEARCH_PATH`, relative PATH entries skipped; loaded path logged; since WR-07 the current folder and network (UNC) `PATH` entries are skipped too (`folders_with`, `is_network_folder`, test `a_network_path_entry_is_skipped`) | closed |
| T-02-03 | Tampering | xtask/src/fetch_engines.rs extraction | medium | mitigate | Only two named entries are extracted, to fixed destination paths; archive-supplied paths are never used as destinations — **Evidence:** `xtask/src/fetch_engines.rs` extracts only `libmpv-2.dll` and `client.h` to fixed destinations | closed |
| T-02-04 | Tampering | xtask/engines.toml url change | high | mitigate | The URL may change, the sha256 and size may not (acceptance criterion diff check); a re-fetch after deleting .verified proves the new host serves byte-identical archives — **Evidence:** `git diff bf25ff2d HEAD -G "^(sha256\|size)" -- xtask/engines.toml` is empty (upstream-now kept, pins unchanged) | closed |
| T-02-05 | Repudiation | GPL distribution of the bundled DLL | medium | mitigate | D-028 records the maintainer's sign-off, the license and the source-offer coordinates (recipe tag 20261111, commit bcde6b93cc) — **Evidence:** `docs/DECISIONS.md` D-028 records the GPL-2.0-or-later sign-off and source-offer coordinates (tag 20261111, commit bcde6b93cc) | closed |
| T-02-06 | Tampering | crates/sucrose-config/src/legacy.rs | high | mitigate | Files are opened with std::fs::read only; no write/create API exists in the crate (acceptance criterion); tests use fixtures — **Evidence:** no `fs::write`/`File::create`/`OpenOptions`/`create_dir` in `crates/sucrose-config/src` | closed |
| T-02-07 | Denial of Service | legacy reader on malformed or huge files | low | accept | Setting files are user-owned and small; parse errors fall back to defaults and never panic; size cap not needed at ASVS L1 — **Evidence:** accepted (AR-01) | closed |
| T-02-08 | Information Disclosure | committed fixtures | medium | mitigate | Fixtures are sanitized copies; the fixtures_contain_no_profile_paths test fails on any user profile path — **Evidence:** `fixtures_contain_no_profile_paths` test over the committed fixtures | closed |
| T-02-09 | Spoofing | crates/sucrose-ipc/src/lib.rs Listener::bind | high | mitigate | First-pipe-instance creation (squatting a name fails the bind and is logged), per-user name Sucrose.Service.<SID> — **Evidence:** `crates/sucrose-ipc/src/lib.rs` `Listener::bind` via interprocess 2.4.9, which sets `FILE_FLAG_FIRST_PIPE_INSTANCE`; taken name maps to `AlreadyRunning`; test `a_second_listener_on_the_same_name_is_already_running` | closed |
| T-02-10 | Elevation of Privilege | named pipe DACL | high | mitigate | SDDL D:P(A;;GA;;;<current SID>) via SecurityDescriptor::deserialize; other users cannot open the pipe — **Evidence:** `D:P(A;;GA;;;<SID>)` security descriptor applied in `restrict_to_owner` (`crates/sucrose-ipc/src/lib.rs`); second-account denial not exercised (manual-only) | closed |
| T-02-11 | Denial of Service | crates/sucrose-ipc/src/frame.rs | medium | mitigate | MAX_FRAME 64 KiB checked before allocation; decode errors drop only that connection — **Evidence:** `MAX_FRAME` (64 KiB) checked before allocation in `crates/sucrose-ipc/src/frame.rs`; frame-limit and garbage tests | closed |
| T-02-12 | Spoofing | apps/sucrose/src/service.rs accept_hello | low | mitigate | Hello must carry the current PROTOCOL_VERSION and a pid the service spawned; same-user processes are inside the trust boundary (ASVS L1) — **Evidence:** `accept_hello` in `apps/sucrose/src/service.rs` checks `PROTOCOL_VERSION` and a spawned pid; `accept_hello_*` tests | closed |
| T-02-13 | Tampering | apps/sucrose/src/service.rs Source check | high | mitigate | Only a relative path without .., ://, drive or UNC prefix that exists as a file in the theme folder is played; plan 08 adds canonicalized containment — **Evidence:** superseded by `check_selected` in `crates/sucrose-core/src/theme.rs` (plan 08) — see T-02-16 | closed |
| T-02-34 | Denial of Service | crates/sucrose-platform/src/os_windows/surface.rs messages to explorer | medium | mitigate | Progman is only messaged with SendMessageTimeoutW (SMTO_NORMAL, 1000 ms), 0x052C is sent once and only when no wallpaper WorkerW exists, and attach retries stop after 10 s, so a hung explorer never blocks the renderer forever — **Evidence:** `crates/sucrose-platform/src/os_windows/surface.rs` uses `SendMessageTimeoutW` with `SMTO_NORMAL` 1000 ms; 0x052C sent once only when no wallpaper WorkerW exists; retries stop after 10 s | closed |
| T-02-14 | Elevation of Privilege | crates/sucrose-engine-mpv/src/player.rs options | high | mitigate | load-scripts=no, ytdl=no, config=no, input-default-bindings=no, input-vo-keyboard=no, osc=no set before mpv_initialize; option errors are fatal — **Evidence:** `crates/sucrose-engine-mpv/src/config.rs` hardening options; `embedding_options_harden_the_instance`; user-config tests prove a user config cannot re-enable them | closed |
| T-02-15 | Denial of Service | renderer crash on a malformed file | medium | accept | Crash is contained in the renderer process (D-008); plan 10 adds the bounded restart budget — **Evidence:** accepted (AR-02) | closed |
| T-02-16 | Tampering | crates/sucrose-core/src/theme.rs check_selected | high | mitigate | Source must be a relative regular file whose canonical path stays inside the canonical theme folder; URL, absolute, UNC/device and .. are rejected before canonicalization (path traversal is canon; covered also by /gsd-secure-phase) — **Evidence:** `check_selected` rejects URL, absolute, UNC/device and `..` before canonicalization and requires the canonical path inside the canonical theme folder; `tests/theme.rs` gate cases | closed |
| T-02-17 | Tampering | crates/sucrose-core/src/theme.rs write_info | medium | mitigate | Writer only writes the explicit path it gets; no caller in M1 passes a library path; tests write under temp folders — **Evidence:** `write_info` has no non-test caller outside `crates/sucrose-core/src/theme.rs`; tests write under temp folders | closed |
| T-02-18 | Information Disclosure | ThemeRejection Display text | low | accept | Reasons name the theme folder and the problem only, never file contents; they stay in local logs and the tooltip — **Evidence:** accepted (AR-03) | closed |
| T-02-19 | Tampering | apps/sucrose/src/service.rs tooltip | low | mitigate | Reason strings are length-limited (127 UTF-16 units) and come only from verified renderer connections (plan 05 pid check) — **Evidence:** tooltip text limited to 127 UTF-16 units; received failure text capped at 300 characters; reasons only from verified renderer connections | closed |
| T-02-20 | Denial of Service | Quit with a hung renderer | medium | mitigate | Close, then a 2 s wait, then kill; the service never blocks its message loop on a child — **Evidence:** Quit closes, waits 2 s on a worker thread, then ends survivors through `RendererJob::terminate_all`; message loop never blocks | closed |
| T-02-21 | Elevation of Privilege | Quit touching other processes | high | mitigate | Shutdown only addresses Child handles the service spawned; no process enumeration or name-based kill exists in the app (D-14) — **Evidence:** no process enumeration or name-based kill in `apps/` or `crates/` (only a doc comment mentions `taskkill`); shutdown addresses spawned children/job only | closed |
| T-02-22 | Denial of Service | apps/sucrose/src/supervisor.rs restart loop | medium | mitigate | RestartBudget 3 crashes per 60 s per monitor with 0.5/1/2 s backoff; SurfaceLost capped at 5 per 60 s — **Evidence:** `RestartBudget` in `apps/sucrose/src/supervisor.rs` (3 per 60 s, 0.5/1/2 s backoff); SurfaceLost capped at 5 per 60 s; supervisor tests | closed |
| T-02-23 | Denial of Service | orphan renderers after a service crash | medium | mitigate | Job object with JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE; verified by killing the service in Task 1 — **Evidence:** `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` in `crates/sucrose-platform/src/os_windows/job.rs`; verified live by killing the service (02-10 SUMMARY) | closed |
| T-02-24 | Tampering | crates/sucrose-engine-mpv/src/config.rs | medium | mitigate | Values come only from typed enums, a clamped u8 and bools; option names come from a constant allow-list; no user text is concatenated into option strings — **Evidence:** option names from constants in `crates/sucrose-engine-mpv/src/config.rs`; values from typed enums, clamped u8 and bools | closed |
| T-02-25 | Elevation of Privilege | uMpvPlayer.config (D-10) | low | accept | Same trust boundary as the user's own profile (the C# app loads it the same way); its use is logged; embedding options are set before it — **Evidence:** accepted (AR-04); implementation is stronger than planned: hardening options are applied after the user config | closed |
| T-02-26 | Denial of Service | spoofed TaskbarCreated / WM_DISPLAYCHANGE floods | low | mitigate | Display events are debounced (2000 + 500 ms); shell restarts respawn at most once per monitor per event and SurfaceLost is capped at 5 per 60 s — **Evidence:** display events debounced (2000 + 500 ms); shell restarts at most 5 per 60 s; `debounce`/`shell_restart` tests | closed |
| T-02-27 | Denial of Service | duplicate renderers per monitor | medium | mitigate | One BTreeMap keyed by device name owns slots; diff never spawns for an existing id — **Evidence:** one `BTreeMap` keyed by device name owns the slots; `Supervisor::spawn` refuses a running slot; `topology` tests | closed |
| T-02-28 | Information Disclosure | crates/sucrose-platform/src/os_windows/foreground.rs | medium | mitigate | Only rect, class name, pid and state flags are read; window titles are never read or logged (acceptance criterion) — **Evidence:** `crates/sucrose-platform/src/os_windows/foreground.rs` reads rect, class, pid and state only; no `GetWindowText` anywhere | closed |
| T-02-29 | Denial of Service | a window that flips fullscreen rapidly | low | mitigate | 5/3 debounce limits flips to at most one per 3–5 s — **Evidence:** 5/3 debounce in `apps/sucrose/src/fullscreen.rs`; `fullscreen` tests | closed |
| T-02-30 | Spoofing | apps/sucrose/src/service.rs Clock relay | low | mitigate | Only Clock from the current master's verified connection is relayed; others are ignored and logged at debug — **Evidence:** `relay_clock` in `apps/sucrose/src/service.rs` ignores (debug-logs) a Clock from a non-master slot | closed |
| T-02-31 | Denial of Service | seek storms on followers | low | mitigate | Seeks only above 0.5 s drift; speed correction below; ticks about 1 Hz — **Evidence:** `sync.rs` seeks only above 0.5 s drift, speed correction below, exact seeks at most every 2 s; ticks ~1 Hz; `sync` tests | closed |
| T-02-32 | Information Disclosure | xtask/src/measure.rs result files | low | accept | Only image names, pids and resource numbers are written, under target/ (ignored by git); no command lines beyond the --render check are stored — **Evidence:** accepted (AR-05) | closed |
| T-02-33 | Tampering | C# processes during measurement | medium | mitigate | The tool only reads statistics; the maintainer starts the C# app (checkpoint); no kill or signal code exists in measure.rs — **Evidence:** `xtask/src/measure.rs` only reads statistics (sysinfo, typeperf); no kill/terminate/signal code; the maintainer started the C# app | closed |
| T-02-35 | Denial of Service | crates/sucrose-platform/src/os_windows/event_loop.rs drain post | low | mitigate | WM_DRAIN is posted at most once per pending batch (DRAIN_POSTED), so a flood of sent display messages produces a bounded number of posts; the handler work per DisplayChanged is only a debounce timer restart — **Evidence:** `crates/sucrose-platform/src/os_windows/event_loop.rs`: the thread-local `DRAIN_POSTED` is checked before `PostMessageW(WM_DRAIN)` and cleared when `WM_DRAIN` is dispatched or the post fails | closed |
| T-02-36 | Denial of Service | crates/sucrose-platform/src/os_windows/surface.rs order guard | low | mitigate | Hook limited to the Progman owner process and events 0x8000..0x8004, O(1) filtering per callback; at most 10 restores per 60 s, then the guard unhooks with a warning — **Evidence:** `install_order_guard` calls `SetWinEventHook(EVENT_OBJECT_CREATE, EVENT_OBJECT_REORDER, …)` with Progman's process id and `WINEVENT_OUTOFCONTEXT` + `WINEVENT_SKIPOWNPROCESS`; `order_guard_proc` filters on event, object id, parent and class; `restore_order` allows `GUARD_MAX_RESTORES` = 10 per `GUARD_WINDOW` = 60 s, then `remove_guard` calls `UnhookWinEvent` and logs a warning | closed |
| T-02-37 | Tampering | crates/sucrose-platform/src/os_windows/surface.rs order guard and verify | low | mitigate | The guard only reorders the renderer's own surface (SetWindowPos on its own window); it never moves, resizes, restyles or destroys a shell window, so a fake WorkerW from another process can at most trigger a bounded number of own-window reorders — **Evidence:** every `SetWindowPos`, `SetParent`, `SetWindowLongPtrW` and `DestroyWindow` call in `surface.rs` targets the renderer's own surface (`surface` / `self.hwnd`); shell windows are only queried (`GetAncestor`, class name, child order) or messaged with the bounded 0x052C (T-02-34); restores bounded by T-02-36 | closed |
| T-02-38 | Tampering | crates/sucrose-platform/src/os_windows/gpu.rs registry write | medium | mitigate | Only the value named by Sucrose's own exe path is read and written, under HKCU only (no HKLM, no elevation); other pairs in that value are preserved; nothing is ever deleted; an unchanged value is not rewritten; tests use a scratch subkey that is deleted afterwards and never the real DirectX key — **Evidence:** `write_preference` opens `HKEY_CURRENT_USER\Software\Microsoft\DirectX\UserGpuPreferences` with `KEY_QUERY_VALUE` + `KEY_SET_VALUE` only, the value name comes from `current_exe()` (`apply_gpu_preference` → `value_name`); `merged_value` keeps the other pairs and returns `None` for an unchanged value (`GpuPreferenceChange::Unchanged`, no write); no HKLM in `crates/` or `apps/`; `RegDeleteTreeW` appears only in `#[cfg(test)]` on `Software\SucroseTest\…`; tests `gpu_other_pairs_are_kept`, `gpu_registry_write_keeps_the_pairs_it_did_not_own`, `gpu_registry_round_trip_on_a_scratch_key` | closed |
| T-02-39 | Tampering | crates/sucrose-config/src/legacy.rs Engine.json GraphicPreference | low | mitigate | Only the three enum names or ordinals 0..2 are accepted; anything else falls back to the legacy default HighPerformance, so a corrupt file cannot write an arbitrary number — **Evidence:** `properties.get_enum("GraphicPreference", GpuPreference::default(), GpuPreference::from_legacy_name, GpuPreference::from_legacy_index)`; `from_legacy_index` maps only the indexes of `GpuPreference::ALL`, anything else warns and returns the default; `crates/sucrose-config/tests` cover `"Turbo"` and `"3"` → HighPerformance | closed |
| T-02-40 | Information Disclosure | service and renderer logs (GPU preference and adapter) | low | mitigate | Logs carry the preference, the outcome and the adapter name/LUID only, never the exe path or other registry content — **Evidence:** `apply_gpu_preference` in `apps/sucrose/src/service.rs` logs `?preference` and the outcome or error only; the `gpu.rs` errors carry an API name and a status code (`PlatformError::Os`) or a fixed text; `log_video_adapter` in `apps/sucrose/src/render.rs` logs name, LUID and dedicated MB | closed |
| T-02-41 | Spoofing | apps/sucrose/src/service.rs Event::Renderer slot | low | mitigate | The slot of a Playing or Aligned message is taken from the connection the service verified at Hello (pid it started, protocol version), never from message content, so a renderer cannot mark another slot as playing or aligned — **Evidence:** `read_connection` takes `slot` from `accept_hello(&supervisor.pids(), &connected, &first)` (since WR-05 the pid must have been started for the claimed slot and a connected slot is never taken over; `accept_hello_*` tests) and tags every later message as `Event::Renderer { slot, message }`; `ToService::Playing` is a unit variant (`crates/sucrose-ipc/src/protocol.rs`); no Aligned message was built (sticky election, D-030) | closed |
| T-02-42 | Denial of Service | apps/sucrose/src/service.rs election churn | low | mitigate | Elections run only on Playing, exit and applied topology changes (all bounded by the restart budget and the 2.5 s debounce); sticky or aligned-only handover prevents role ping-pong between two renderers — **Evidence:** `fleet.elect` runs only at start, on a slot's first `Playing` (`fleet.playing.insert(slot)`), on the exit of a playing renderer and after `apply_snapshot`; `choose_master` keeps the current master while it is still playing; test `a_sticky_master_survives_a_new_primary_through_the_fleet` | closed |
| T-02-43 | Denial of Service | crates/sucrose-engine-mpv/src/sync.rs follower controller | low | mitigate | Speed bounded to ±5 % of the master's, changes below 0.1 % suppressed, seeks only beyond 0.5 s and at most one per 2 s, integral clamped (anti-windup), so a bad or noisy clock cannot make a follower oscillate or seek-storm — **Evidence:** `MAX_SPEED_STEP` = 0.05 (correction clamped to ±`MAX_SPEED_STEP`), `MIN_SPEED_CHANGE` = 0.001, `SEEK_THRESHOLD_S` = 0.5, `MIN_SEEK_GAP_NS` = 2 s, integration step capped by `MAX_INTEGRATION_S` and the integral grows only inside `INTEGRAL_ZONE_S` below saturation; `sync` tests | closed |
| T-02-44 | Tampering | xtask/src/measure.rs | low | mitigate | The tool only reads process statistics and counters; it never starts, stops or changes a C# process, dwm.exe or a setting (D-07, D-14); the maintainer starts and exits the C# app — **Evidence:** `xtask/src/measure.rs` reads sysinfo statistics and `typeperf` counters (its only `Command::new`); no kill, terminate or signal code; the maintainer started and exited the C# app (02-21 and 02-22 SUMMARY) | closed |
| T-02-45 | Information Disclosure | measurement JSON and rows (adapter names, LUIDs) | low | accept | Adapter names and LUIDs describe the maintainer's hardware only; LUIDs change every boot and carry no personal data; results stay under target/ and the committed row names only the GPU model — **Evidence:** accepted (AR-07) | closed |
| T-02-46 | Tampering | crates/sucrose-engine-mpv/src/config.rs (conditional change) | low | mitigate | Only deband and the profile line may change; the hardening options and their order after the user's config stay untouched and are re-checked by tests/options.rs — **Evidence:** not triggered: `git log 4b768f29..HEAD -- crates/sucrose-engine-mpv/src/config.rs` is empty (02-22 kept `deband=yes` and `profile=low-latency`, R = 1.02); the hardening stays covered by T-02-14 and `crates/sucrose-engine-mpv/tests/options.rs` | closed |
| T-02-47 | Repudiation | docs/MEASUREMENTS.md | low | mitigate | The replaced rows are named in a note ("compared two GPUs") instead of being dropped silently, so the history of the claim stays traceable — **Evidence:** the M1 reading note in `docs/MEASUREMENTS.md` says the first M1 rows (2026-11-22, plan 02-15) compared two GPUs and are replaced by the same-GPU rows | closed |
| T-02-48 | Denial of Service | crates/sucrose-platform/src/os_windows/surface.rs location watch | low | mitigate | Hook limited to EVENT_OBJECT_LOCATIONCHANGE from the process and thread that own the surface's parent; O(1) handle and object-id comparisons; re-place only when the parent's client rectangle differs from the recorded one; only the renderer's own surface moves; own moves skipped — **Evidence:** `SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_LOCATIONCHANGE, None, Some(location_watch_proc), process, thread, WINEVENT_OUTOFCONTEXT \| WINEVENT_SKIPOWNPROCESS)` with `process`/`thread` from `GetWindowThreadProcessId(root)`; `location_reaction` returns true only for `OBJID_WINDOW` on the parent or its root and only when `recorded != Some(now)`; the hook is removed with `UnhookWinEvent` in `Drop`; covered by `cargo test -p sucrose-platform watch` (12 tests) | closed |
| T-02-49 | Tampering | crates/sucrose-platform/src/os_windows/surface.rs location watch | low | accept | A process that moves Progman can make a renderer re-place its own surface at the target the service gave it; the renderer never moves, resizes, restyles or destroys a shell window — **Evidence:** accepted risk AR-08 | closed |
| T-02-50 | Information Disclosure | renderer placement log lines (apps/sucrose/src/render.rs) | low | accept | New log fields are window and screen rectangles and a verdict, no window titles or process names (T-02-28 rule); logs stay in the user's own data folder — **Evidence:** accepted risk AR-09 | closed |
| T-02-51 | Denial of Service | apps/sucrose/src/service.rs apply_snapshot | low | mitigate | One SetRect per connected renderer per applied display change, bounded by the existing 2000 ms + 500 ms debounce with two equal looks and the bounded per-renderer queue — **Evidence:** `DEBOUNCE_MS = 2000` and `SETTLE_MS = 500` in `apps/sucrose/src/topology.rs`; `topology::replace_all` emits one SetRect per surviving monitor (7 pure tests) and `Fleet::apply_snapshot` runs it once per applied snapshot (2 Fleet tests; `cargo test -p sucrose replace_all`, 9 tests) | closed |
| T-02-52 | Tampering | maintainer run (02-26 Task 1, Task 2) | low | mitigate | Scratch `SUCROSE_DATA_DIR`, real settings read only; the executor synthesizes no input, changes no display setting and stops only `sucrose.exe` — **Evidence:** the 02-26 start command set `SUCROSE_DATA_DIR="$(wslpath -w target/scratch-data)"`; every display change was made by the maintainer (02-26 SUMMARY); the run was stopped with `taskkill.exe /IM sucrose.exe /F` and `tasklist.exe` showed no `sucrose.exe` afterwards; no C# process was running or touched | closed |
| T-02-53 | Information Disclosure | surface.rs watch log lines | low | accept | The watch logs window and screen rectangles and a verdict, no window titles or process names (T-02-28 rule); logs stay in the user's own data folder — **Evidence:** accepted risk AR-10 | closed |
| T-02-54 | Spoofing | apps/sucrose/src/service.rs handle_report | low | mitigate | A report counts only while its renderer's slot belongs to a connected monitor and is not closed by the fullscreen rule, on top of the existing pid checks — **Evidence:** `struct SlotState` (service.rs:1069) carries `assigned`, which the service loop fills from `Fleet::slot_assigned` (service.rs:1386, `SlotMap::monitor_of(slot)` plus the rule-closed set); `a_late_failed_of_a_removed_monitor_is_dropped` and `a_report_of_a_renderer_whose_slot_is_not_assigned_is_dropped_even_while_connected` pass (`cargo test -p sucrose`, 137 tests) | closed |
| T-02-55 | Denial of Service | apps/sucrose/src/service.rs handle_report (tray status) | low | mitigate | The new drop is limited to unassigned slots, so a real failure of an assigned monitor still reaches the tooltip — **Evidence:** `a_late_failed_of_an_assigned_slot_still_counts` (service.rs:1960) and the WR-18 tests run with `assigned: true` pass | closed |
| T-02-56 | Repudiation | docs/DECISIONS.md | low | mitigate | D-031 is append-only and carries the maintainer's verbatim reply and its checkpoint — **Evidence:** `git diff -U0 d3670ec4 HEAD -- docs/DECISIONS.md` removes 0 lines; one `## D-031: ` heading before `## Open questions` | closed |
| T-02-57 | Repudiation | docs/FEATURES.md, 02-UAT.md | low | mitigate | The row says "Accepted limitation (G-02-7, D-031)" and "not bounded by the app" and drops the unconfirmed cause; G-02-7 cannot be closed silently — **Evidence:** both phrases found once in docs/FEATURES.md; `git diff --quiet d3670ec4 HEAD -- 02-UAT.md` exits 0; the row keeps 🟨 | closed |
| T-02-58 | Tampering | maintainer run (02-29 Task 2) | low | mitigate | Scratch `SUCROSE_DATA_DIR`, legacy settings read only, no synthesized input, only the started `sucrose.exe` stopped, precondition refuses to start beside a C# Sucrose process — **Evidence:** 02-29 SUMMARY: started with `SUCROSE_DATA_DIR` = target/scratch-data, stopped with `taskkill.exe /IM sucrose.exe /F`; `tasklist.exe` shows no `sucrose.exe` afterwards; the C# app was exited by the maintainer before the run | closed |
| T-02-59 | Repudiation | docs/MEASUREMENTS.md | low | mitigate | `--row` refuses cross-adapter pairs; a refused pair becomes a Rust-only row with the reason in its notes; result files are never edited — **Evidence:** the 2026-11-23 "final M1 build (Rust)" row has an empty C# column and names the adapter and the LUID change after the reboot as the reason; `target/measure/rust-final-build.json` is untracked and unedited | closed |
| T-02-SC | Tampering | crates.io dependencies (plans 02-16 … 02-26) | high | mitigate | No crate is added: only features of the already pinned `windows` 0.62.2 crate and workspace crates; `cargo deny check` runs in CI — **Evidence:** `git diff cc92d08e HEAD -- Cargo.lock` adds only the workspace crates `sucrose-core` (to sucrose-platform) and `sucrose-platform` (to xtask), no new package; the new `windows` features (`Win32_UI_Accessibility`, `Win32_System_Registry`, `Win32_Graphics_Dxgi`, `Win32_Graphics_Direct3D`, `Win32_Graphics_Direct3D11`) use the root pin `windows = "0.62.2"`; the `deny` job in `.github/workflows/rust.yml` runs `EmbarkStudios/cargo-deny-action@v2`; the code review fix round (WR-05 … IN-17) adds no package either: `git diff 0dadcaa1 HEAD -- Cargo.lock` only adds `sucrose-engine-mpv` as its own dev-dependency for the test-only `test-support` feature (IN-14); the G-02-7 gap closure (02-23 … 02-26) adds no package and no `windows` feature: `git diff dd51e9c2 HEAD -- Cargo.lock Cargo.toml` is empty | closed |

*Status: open · closed · open — below high threshold (non-blocking)*
*Severity: critical > high > medium > low — only open threats at or above workflow.security_block_on count toward threats_open*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

### Summary Threat Flags

| Flag | Plan | File | Disposition |
|------|------|------|-------------|
| local-message-surface | 02-09 | `crates/sucrose-platform/src/os_windows/event_loop.rs` | accepted (AR-06): the hidden service window accepts WM_APP / WM_APP+1 from any same-user process, which can stop the service; a same-user process can already terminate it, so no new capability |

All other SUMMARY Threat Flags sections report no new surface beyond the register (for the gap closure plans, 02-17, 02-20, 02-21 and 02-22 say so explicitly; 02-16, 02-18 and 02-19 have no Threat Flags section). The G-02-7 gap closure plans 02-23, 02-24 and 02-25 report no new surface beyond the register (T-02-48, T-02-50, T-02-51); 02-26 has no Threat Flags section and adds no code. The gap closure plans 02-27 … 02-29 add no surface beyond the register: 02-27 reports T-02-54 and T-02-55 mitigated, 02-28 reports none, and 02-29 adds no code (T-02-58, T-02-59).

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-01 | T-02-07 | Setting files are user-owned and small; parse errors fall back to defaults and never panic; a size cap is not needed at ASVS L1 | 02-04 plan threat model | 2026-11-22 |
| AR-02 | T-02-15 | A crash on a malformed file is contained in the renderer process (D-008); the bounded restart budget (T-02-22) limits repeats | 02-07 plan threat model | 2026-11-22 |
| AR-03 | T-02-18 | Rejection reasons name the theme folder and the problem only, never file contents; they stay in local logs and the tooltip | 02-08 plan threat model | 2026-11-22 |
| AR-04 | T-02-25 | `uMpvPlayer.config` is inside the user's own trust boundary (the C# app loads it the same way); its use is logged; the hardening options are applied after it, so it cannot re-enable scripts, ytdl, config loading or input bindings (tested) | 02-11 plan threat model | 2026-11-22 |
| AR-05 | T-02-32 | Measurement files hold only image names, pids and resource numbers, under the git-ignored `target/` | 02-15 plan threat model | 2026-11-22 |
| AR-06 | 02-09 flag local-message-surface | Same-user processes are inside the trust boundary (ASVS L1) and can already terminate the service | 02-09 SUMMARY threat flag | 2026-11-22 |
| AR-07 | T-02-45 | Adapter names and LUIDs describe the maintainer's hardware only; LUIDs change every boot and carry no personal data; results stay under the git-ignored `target/`; the committed row names only the GPU model (the reading note also quotes one boot-scoped LUID, which this rationale covers) | 02-21 plan threat model | 2026-11-23 |
| AR-08 | T-02-49 | A process that can move Progman can already change what the desktop shows; the location watch only re-places the renderer's own surface at the target the service gave it and never touches a shell window | 02-24 plan threat model | 2026-11-23 |
| AR-09 | T-02-50 | Placement log lines hold rectangles and a verdict only (T-02-28 rule) and stay in the user's own data folder | 02-23 plan threat model | 2026-11-23 |
| AR-10 | T-02-53 | Watch log lines hold rectangles and a verdict only (T-02-28 rule) and stay in the user's own data folder | 02-24 plan threat model | 2026-11-23 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-11-22 | 34 | 34 | 0 | /gsd-secure-phase 02 (L1 grep-depth verification by the orchestrator; register authored at plan time, short-circuit per ASVS L1) |
| 2026-11-23 | 48 | 48 | 0 | /gsd-secure-phase 02 after gap closure 02-16 … 02-22 (14 new threats: T-02-35 … T-02-47 and T-02-SC; L1 verification by the orchestrator; register authored at plan time, short-circuit per ASVS L1) |
| 2026-11-23 | 48 | 48 | 0 | /gsd-secure-phase 02 after the code review fixes WR-05 … WR-14 and IN-01 … IN-17 (L1 re-check of the mitigation anchors in the 17 changed source files; no new threat; evidence of T-02-02, T-02-41 and T-02-SC updated; short-circuit per ASVS L1) |
| 2026-11-23 | 48 | 48 | 0 | /gsd-secure-phase 02 after the fixes WR-15 … WR-17 (L1 re-check of the anchors in `apps/sucrose/src/service.rs` and `render.rs`; no new threat, no new dependency; short-circuit per ASVS L1) |
| 2026-11-23 | 48 | 48 | 0 | /gsd-secure-phase 02 after the WR-18 fix (L1 re-check of the anchors in `apps/sucrose/src/service.rs`; no new threat, no new dependency; short-circuit per ASVS L1) |
| 2026-11-23 | 54 | 54 | 0 | /gsd-secure-phase 02 after the G-02-7 gap closure 02-23 … 02-26 (6 new threats: T-02-48 … T-02-53, T-02-SC evidence extended; L1 verification by the orchestrator; register authored at plan time, short-circuit per ASVS L1) |
| 2026-11-23 | 60 | 60 | 0 | /gsd-secure-phase 02 after the gap closure 02-27 … 02-29 (6 new threats: T-02-54 … T-02-59; L1 verification by the orchestrator; register authored at plan time, short-circuit per ASVS L1) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-11-22

**Approval (gap closure):** verified 2026-11-23 (`/gsd-secure-phase 02` for plans 02-16 … 02-22: 14 threats added, 14 closed, 0 open; AR-07 added)

**Approval (gap closure G-02-7):** verified 2026-11-23 (`/gsd-secure-phase 02` for plans 02-23 … 02-26: 6 threats added, 6 closed, 0 open; AR-08 … AR-10 added)

## Security Audit 2026-11-23

| Metric | Count |
|---|---|
| Threats found | 48 |
| Closed | 48 |
| Open | 0 |

## Security Audit 2026-11-23

| Metric | Count |
|---|---|
| Threats found | 54 |
| Closed | 54 |
| Open | 0 |

## Security Audit 2026-11-23

| Metric | Count |
|---|---|
| Threats found | 60 |
| Closed | 60 |
| Open | 0 |
