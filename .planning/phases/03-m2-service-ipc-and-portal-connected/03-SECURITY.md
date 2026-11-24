---
phase: "03"
slug: "m2-service-ipc-and-portal-connected"
status: verified
threats_open: 0
register_authored_at_plan_time: true
asvs_level: 1
block_on: high
created: "2026-11-24"
---

# Phase 03 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Legacy Setting folder to migration | Files written by the C# app, read once, never written; Objectionable.json (plaintext token) is never read | Legacy settings |
| Sucrose.json on disk | Settings written atomically by the service; same-user programs may edit | Settings values |
| Portal to service (IPC pipe) | Owner-only named pipe; one registered Portal; every request validated by the service | Typed requests and events, memory figures |
| Service to renderer | Job object, pids spawned by the service, hardened libmpv | Absolute source path |
| Theme folders to Portal decoders | Untrusted images and GIFs; contained paths only, dimension and byte caps | Thumbnails, previews, background |
| Service to OS | HKCU Run entry, recycle bin, library move, process end | Own startup entry, library folders |
| crates.io to the build | Pinned, legitimacy-checked, cargo deny | Dependencies |
| Verification runs to maintainer data | Scratch SUCROSE_DATA_DIR, real data read-only | Settings, library |

---

## Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation (evidence) | Status |
|-----------|----------|-----------|----------|-------------|-----------------------|--------|
| T-03-01 | Denial of Service | crates/sucrose-config/src/store.rs | high | mitigate | crates/sucrose-config/src/store.rs (temp + sync_all + rename, BACKUP_FILE Sucrose.json.bak line 23, load order line 79); test kill_during_write_never_leaves_a_corrupt_file (tests/store.rs:412); 03-01-SUMMARY | closed |
| T-03-02 | Tampering | legacy Setting/*.json | high | mitigate | crates/sucrose-config/src/migrate.rs:4 (never writes, renames or deletes a legacy file); legacy.rs reader read-only (tests/legacy.rs); 03-01-SUMMARY: SHA-256 of 14 real legacy files matched before and after | closed |
| T-03-03 | Tampering | Sucrose.json written by a newer build or edited by hand | medium | mitigate | crates/sucrose-config/src/model.rs:79 #[serde(flatten)] extra map; store.rs:49 StoreError::NewerVersion kept unchanged; typed getters in legacy.rs (get_clamped/get_enum) | closed |
| T-03-SC | Tampering | crates.io dependencies | high | mitigate | Package legitimacy recorded in 03-27-SUMMARY:149 (40 OK); crates pinned in root Cargo.toml [workspace.dependencies] (rust-i18n=4.2.4, trash=5.4.0, gpui =0.3.12); cargo deny job .github/workflows/rust.yml:95-99; no crate added in 03-01/03-04 summaries | closed |
| T-03-04 | Information Disclosure | crates/sucrose-core/src/library.rs | high | mitigate | crates/sucrose-core/src/theme.rs:344 (".." rejected), :370 "Source resolves outside the theme folder"; tests/theme.rs | closed |
| T-03-05 | Denial of Service | crates/sucrose-core/src/library.rs | medium | mitigate | crates/sucrose-core/src/library.rs:69,152 Unreadable distinct from Corrupt; theme.rs:185 ThemeError::Unreadable | closed |
| T-03-06 | Tampering | SucroseInfo.json parsing | low | accept | accept: documented in 03-02 plan; theme.rs:224 serde_json typed parse, malformed file only makes theme corrupt | closed |
| T-03-07 | Tampering | xtask/src/localize.rs | low | accept | accept: documented in 03-03 plan; xtask/src/localize.rs has no network call; 03-03-SUMMARY threat flags | closed |
| T-03-08 | Tampering | deny.toml license allow-list | medium | mitigate | deny.toml:16-17 only "bzip2-1.0.6" added with comment naming the crate; 03-03-SUMMARY | closed |
| T-03-09 | Information Disclosure | crates/sucrose-config/src/legacy.rs | high | mitigate | crates/sucrose-config/src/legacy.rs has no Objectionable variant (0 matches in LegacyFile); migrate.rs:26 documents it; 03-04-SUMMARY (grep count 0) | closed |
| T-03-10 | Tampering | crates/sucrose-config/src/migrate.rs | medium | mitigate | crates/sucrose-config/src/legacy.rs:135-160 get_clamped/get_enum typed getters; model.rs:1120 engine(allowed); 03-04-SUMMARY | closed |
| T-03-11 | Tampering | xtask/src/synth_library.rs | medium | mitigate | xtask/src/synth_library.rs:37 MARKER ".synth", --force handling at :65; 03-05-SUMMARY | closed |
| T-03-12 | Denial of Service | generated library size | low | accept | accept: documented in 03-05 plan; synth_library.rs:41 MAX_COUNT = 1000 | closed |
| T-03-13 | Spoofing | apps/sucrose/src/portal_link.rs | medium | mitigate | apps/sucrose/src/portal_link.rs:29-32 pid recorded per connection (one registered Portal); renderer pid check (accept_hello) unchanged; every request validated by service (settings_host, check_selected) | closed |
| T-03-14 | Denial of Service | apps/sucrose/src/service.rs read_connection | high | mitigate | crates/sucrose-ipc frame::MAX_FRAME (lib.rs:34); apps/sucrose/src/portal_link.rs:56 crossbeam bounded queue, :122 try_send ends connection on overflow | closed |
| T-03-15 | Information Disclosure | crates/sucrose-ipc/src/portal.rs | low | accept | accept: documented in 03-06 plan; no Objectionable data in crates/sucrose-ipc (0 matches) | closed |
| T-03-16 | Tampering | endpoint name | low | mitigate | crates/sucrose-ipc/src/lib.rs:112-121 FNV-1a 64 hash of folder, not path text, in the endpoint name | closed |
| T-03-17 | Tampering | apps/sucrose/src/settings_host.rs | high | mitigate | apps/sucrose/src/settings_host.rs:15,82,103 validate/clamp to legacy ranges, MAX_PATH_TEXT, one-writer; 03-07-SUMMARY threat flags | closed |
| T-03-18 | Repudiation | settings changes | low | accept | accept: documented in 03-07 plan; settings_host logs variant only (tracing, no paths) | closed |
| T-03-19 | Elevation of Privilege | apps/sucrose-portal/src/ipc.rs service start | high | mitigate | apps/sucrose-portal/src/ipc.rs:395-405 current_exe() sibling joined with fixed name, Command::new with argument vector; owner-only endpoint | closed |
| T-03-20 | Denial of Service | Portal UI thread | medium | mitigate | apps/sucrose-portal/src/ipc.rs:4 reader and writer threads, no I/O on GPUI thread (D-012) | closed |
| T-03-21 | Tampering | command-line arguments | low | mitigate | apps/sucrose-portal/src/main.rs:78,106 EXIT_USAGE = 64 on unknown arguments | closed |
| T-03-22 | Tampering | apps/sucrose/src/service.rs Use | high | mitigate | apps/sucrose/src/service.rs:23,657,3119 check_selected on a folder; decide_use at :3045; 03-09-SUMMARY | closed |
| T-03-23 | Denial of Service | renderer restarts after a Use | medium | mitigate | apps/sucrose/src/supervisor.rs:140 RestartBudget; service.rs:805 budget spent behavior; 03-09-SUMMARY | closed |
| T-03-24 | Elevation of Privilege | crates/sucrose-engine-mpv/src/player.rs | medium | mitigate | crates/sucrose-engine-mpv/src/config.rs:31-35 load-scripts/ytdl/scripts/config options disabled; 03-09-SUMMARY (LoadSource hardening kept) | closed |
| T-03-25 | Denial of Service | apps/sucrose-portal/src/library/thumbs.rs | high | mitigate | apps/sucrose-portal/src/library/thumbs.rs:4,22,29 check_dimensions, CACHE_BYTES 48 MiB, MAX_PIXELS; 03-29 adds the header check | closed |
| T-03-26 | Information Disclosure | thumbnail paths | medium | mitigate | apps/sucrose-portal/src/library/thumbs.rs decodes only the scan Thumbnail path (contained by theme.rs:344,370) | closed |
| T-03-27 | Tampering | malformed image | low | accept | accept: documented in 03-10 plan; assets.rs:24, card.rs:202 ImageOff on decode error | closed |
| T-03-28 | Tampering | maintainer data during the run | medium | mitigate | process rule: 03-11 plan and run records scratch SUCROSE_DATA_DIR (target/scratch-data); later runs 03-24/03-37/03-38 SUMMARY repeat it | closed |
| T-03-29 | Tampering | apps/sucrose-portal/src/library/card.rs | low | accept | accept: documented in 03-12 plan; authoritative decide_use at apps/sucrose/src/service.rs:3045 | closed |
| T-03-30 | Denial of Service | repeated Use clicks | low | mitigate | apps/sucrose-portal/src/library/mod.rs:209,265,288 pending_use (one pending, last wins); 03-12-SUMMARY threat flags | closed |
| T-03-31 | Information Disclosure | apps/sucrose/src/modules.rs | low | mitigate | apps/sucrose/src/modules.rs:122 MemorySampler samples only spawned or registered pids (sysinfo); memory figures only (MemoryReport) | closed |
| T-03-32 | Denial of Service | memory timer | low | mitigate | apps/sucrose/src/modules.rs:1,12,18 timer only when module enabled; Memory events droppable | closed |
| T-03-33 | Information Disclosure | apps/sucrose-portal/src/header.rs | low | mitigate | apps/sucrose-portal/src/header.rs:42,127 query handled in process (normalize_query, no network); no log of query text | closed |
| T-03-34 | Elevation of Privilege | apps/sucrose/src/service.rs open_portal | high | mitigate | apps/sucrose/src/service.rs:968-975 fixed sibling sucrose-portal{EXE_SUFFIX}, Command::new with enum-chosen arguments; 03-15-SUMMARY | closed |
| T-03-35 | Denial of Service | Launch flood | low | mitigate | apps/sucrose/src/service.rs:52,1543,1599 spawn_pending guard; one Portal at a time | closed |
| T-03-36 | Spoofing | allow_set_foreground | low | accept | accept: documented in 03-15 plan; allow_set_foreground only for registered pid (portal_link.rs:304) | closed |
| T-03-37 | Elevation of Privilege | apps/sucrose/src/service.rs Close Applications | high | mitigate | apps/sucrose/src/modules.rs:83 end_portal(pid, sibling); lifecycle.rs:215 test only_the_sibling_portal_executable_may_be_ended; 03-16-SUMMARY | closed |
| T-03-38 | Denial of Service | close sequence | medium | mitigate | 03-16-SUMMARY threat flags: bounded 2 s wait while loop keeps pumping | closed |
| T-03-39 | Spoofing | a same-user process registered as the Portal | low | accept | accept: documented in 03-16 plan; end_portal checks executable path (modules.rs:83) | closed |
| T-03-40 | Denial of Service | apps/sucrose-portal/src/background.rs | high | mitigate | apps/sucrose-portal/src/background.rs:5,24,27 check_dimensions before decode, MAX_LONG_EDGE 2560, off-thread; atlas release at :369 | closed |
| T-03-41 | Elevation of Privilege | RtlGetVersion call | low | mitigate | crates/sucrose-platform/src/os_windows.rs:26 RtlGetVersion confined to sucrose-platform; no unsafe in Portal (0 matches in apps/sucrose-portal/src) | closed |
| T-03-42 | Spoofing | apps/sucrose-portal/src/dialogs/help.rs, about.rs, header.rs | low | mitigate | apps/sucrose-portal/src/header.rs:30,310 and dialogs/about.rs:99 https constants; 03-18-SUMMARY (unit test checks scheme) | closed |
| T-03-43 | Information Disclosure | clipboard | low | accept | accept: documented in 03-18 plan; 03-18-SUMMARY | closed |
| T-03-44 | Denial of Service | apps/sucrose-portal/src/library/preview.rs | high | mitigate | apps/sucrose-portal/src/library/preview.rs:2,5 HOVER_DWELL, MAX_SIDE/MAX_PIXELS GIF screen refusal, frame/byte caps; 03-29 | closed |
| T-03-45 | Information Disclosure | preview path | low | mitigate | apps/sucrose-portal/src/library/preview.rs opens only the contained Preview path; 03-19-SUMMARY | closed |
| T-03-46 | Tampering | apps/sucrose-portal/src/settings/*.rs | low | mitigate | apps/sucrose-portal/src/settings/*.rs clamp; service validate authoritative (settings_host.rs:82); 03-20-SUMMARY | closed |
| T-03-47 | Denial of Service | background image picker | low | mitigate | apps/sucrose-portal/src/settings/general.rs:490,526 cx.spawn background tasks; 03-20-SUMMARY | closed |
| T-03-48 | Tampering | apps/sucrose/src/library_ops.rs corrupt step | high | mitigate | apps/sucrose/src/library_ops.rs:14,29 scan-classified Corrupt only, DeleteCorrupt flag, trash crate (Cargo.toml:49); 03-21-SUMMARY | closed |
| T-03-49 | Tampering | copy_tree | medium | mitigate | apps/sucrose/src/library_ops.rs:160,168 symlink_metadata, symlinks skipped; 03-21-SUMMARY | closed |
| T-03-50 | Denial of Service | service main thread | medium | mitigate | apps/sucrose/src/library_ops.rs:1 library-ops worker thread; one job at a time | closed |
| T-03-51 | Tampering | apps/sucrose/src/library_ops.rs move | high | mitigate | apps/sucrose/src/library_ops.rs:63,114,228 verified copy, rollback_move, finish_move, MoveRefusal::Nested; 03-22-SUMMARY | closed |
| T-03-52 | Denial of Service | a move that recurses into itself | high | mitigate | apps/sucrose/src/library_ops.rs:228 MoveRefusal::Nested before anything runs; symlinks skipped (:63) | closed |
| T-03-53 | Elevation of Privilege | destination path text | medium | mitigate | apps/sucrose/src/service.rs:306 text.len() <= MAX_PATH_TEXT check; 03-22-SUMMARY | closed |
| T-03-54 | Elevation of Privilege | crates/sucrose-platform/src/os_windows/autostart.rs | high | mitigate | crates/sucrose-platform/src/autostart.rs:80-83 quoted exe, quote refused; :93,108 xml_escape; os_windows/autostart.rs:30 HKCU Run only | closed |
| T-03-55 | Tampering | the C# startup entries | high | mitigate | crates/sucrose-platform/src/os_windows/autostart.rs:2,33 C# Run value only read; 03-23-SUMMARY test autostart_registry_enable_disable_leaves_the_csharp_entry_alone and threat flags (schtasks /Query only) | closed |
| T-03-56 | Repudiation | startup changes | low | accept | accept: documented in 03-23 plan; apps/sucrose/src/startup.rs:89,109 info logs | closed |
| T-03-57 | Tampering | maintainer data during the runs | medium | mitigate | process rule: 03-24 SUMMARY records scratch SUCROSE_DATA_DIR runs; maintainer data untouched | closed |
| T-03-58 | Information Disclosure | docs/MEASUREMENTS.md | low | mitigate | docs/MEASUREMENTS.md rows contain counts and memory figures only (no path/user matches); 03-25 plan | closed |
| T-03-59 | Information Disclosure | crates/sucrose-config/src/migrate.rs, tests/migrate.rs | high | mitigate | crates/sucrose-config/tests/migrate.rs:377 sentinel Objectionable.json test, :682 MIGRATED_FILES names check; migrate.rs:26 | closed |
| T-03-60 | Tampering | crates/sucrose-config/src/migrate.rs | medium | mitigate | crates/sucrose-config/src/model.rs:1119-1120 engine(allowed) typed; runtime keys not read (migrate.rs, tests/migrate.rs) | closed |
| T-03-61 | Denial of service | Portal sprite atlas (library/mod.rs, background.rs) | medium | mitigate | apps/sucrose-portal/src/atlas.rs:16,21 release_image(s), used at background.rs:369; 03-27-SUMMARY GPUI tests; D-25 reruns 03-37/03-38 | closed |
| T-03-62 | Tampering | dev-only crates from gpui-kit test-support | high | mitigate | 03-27-SUMMARY:149 package legitimacy check (all 40 verdicts OK); Cargo.toml:31 gpui test-support; apps/sucrose-portal/Cargo.toml [dev-dependencies] only; .github/workflows/rust.yml:95-99 cargo deny | closed |
| T-03-63 | Information disclosure | hover diagnostics in library/mod.rs | low | mitigate | apps/sucrose-portal/src/library/mod.rs:328,340 debug! messages without folder/title/path | closed |
| T-03-64 | Denial of service | animating preview frames in the atlas | medium | mitigate | preview.rs caps (03-19) plus atlas::release_image (atlas.rs:16); D-25 measurement in 03-37/03-38 | closed |
| T-03-65 | Tampering | maintainer data during the hover run | medium | mitigate | process rule: 03-28 SUMMARY threat flags record scratch SUCROSE_DATA_DIR, real library read-only | closed |
| T-03-66 | Denial of service | thumbs::decode_bytes (JPEG fast path, CR-01) | high | mitigate | apps/sucrose-portal/src/library/thumbs.rs:4,29 check_dimensions / MAX_PIXELS; 03-29 test with 16384 x 16384 header | closed |
| T-03-67 | Denial of service | background::decode_bytes and preview GIF screen | high | mitigate | apps/sucrose-portal/src/background.rs:5,24 and preview.rs:5 check_dimensions; 03-29-SUMMARY (decode_generic keeps Limits::max_alloc) | closed |
| T-03-68 | Tampering | Store::open restore and migration (CR-02) | high | mitigate | crates/sucrose-config/src/store.rs:73,104,127 Read::Unreadable returns StoreError::Io writes nothing; 03-30 locked-file tests | closed |
| T-03-69 | Information disclosure | store warnings | low | mitigate | crates/sucrose-config/src/store.rs:200,218-220 error kind only, message omitted | closed |
| T-03-70 | Tampering | library maintenance during a move (WR-01) | medium | mitigate | apps/sucrose/src/service.rs:130,148 NOTHING_MAINTAINED, MoveEnv::start Busy refusal | closed |
| T-03-71 | Tampering | renderer respawn during a move (WR-03) | medium | mitigate | apps/sucrose/src/service.rs:1249,2723 respawn_closed waits during move; 03-31-SUMMARY | closed |
| T-03-72 | Spoofing | log and tray text (WR-07) | low | mitigate | apps/sucrose/src/service.rs:810-816 Status::text() through printable() | closed |
| T-03-73 | Tampering | Portal requests while quitting (WR-02) | medium | mitigate | apps/sucrose/src/service.rs:319,1328 acts_while_quitting gate | closed |
| T-03-74 | Tampering | process exit during a library move | high | mitigate | apps/sucrose/src/service.rs:334,338 may_exit holds exit while library_move is Some | closed |
| T-03-75 | Denial of service | dropped Portal keeps its pipe and the service alive (WR-04) | medium | mitigate | crates/sucrose-platform/src/lib.rs:235 disconnect_pipe used by PortalRegistry::send give-up (portal_link.rs); 03-33-SUMMARY | closed |
| T-03-76 | Elevation of privilege | new unsafe DisconnectNamedPipe call | medium | mitigate | crates/sucrose-platform/src/os_windows.rs:110 SAFETY comment on DisconnectNamedPipe; 03-33-SUMMARY (real pipe pair test) | closed |
| T-03-77 | Tampering | player start detection (WR-08) | low | mitigate | crates/sucrose-engine-mpv/src/player.rs:197,203 SourceSwitch; 03-34 sequence tests | closed |
| T-03-78 | Tampering | startup entry vs RunStartup (WR-05) | low | mitigate | apps/sucrose/src/startup.rs:152,163 FakeOs StartupOs tests; 03-35 plan | closed |
| T-03-79 | Tampering | the maintainer's HKCU Run key | medium | mitigate | crates/sucrose-platform/tests/autostart.rs:273,316 HKCU tests #[ignore]; startup.rs FakeOs; 03-35-SUMMARY | closed |
| T-03-80 | Tampering | synth-library guard (WR-06) | high | mitigate | xtask/src/synth_library.rs:8,65 canonical compare, --force never bypasses; 03-36-SUMMARY | closed |
| T-03-81 | Repudiation | D-25 round windows | low | mitigate | xtask/src/measure.rs:56 reads Portal timestamped page log; 03-36 plan | closed |
| T-03-82 | Tampering | maintainer data during the run | medium | mitigate | 03-37-SUMMARY threat flags: scratch SUCROSE_DATA_DIR target/measure-real-2, nothing in real data written, apps checked with tasklist | closed |
| T-03-83 | Tampering | maintainer data during the run | medium | mitigate | 03-38-SUMMARY threat flags: scratch SUCROSE_DATA_DIR target/synth-500, nothing real written, apps checked with tasklist | closed |
| T-03-84 | Repudiation | MEASUREMENTS rows and requirement status | medium | mitigate | 03-39-SUMMARY: Task 3 (3860b872) marked LIB-06 and UI-07 only on evidence; 03-39 threat flags none | closed |
| T-03-85 | Information disclosure | "page shown" memory fields | low | mitigate | apps/sucrose-portal/src/library/mod.rs:525-539 MiB figures only, gated on the debug filter; 03-40-SUMMARY | closed |
| T-03-86 | Tampering | os_windows/memory.rs FFI (memory safety) | medium | mitigate | crates/sucrose-platform/src/os_windows/memory.rs:34,43 SAFETY comments, cb set before the call; results checked; 03-40-SUMMARY | closed |
| T-03-87 | Denial of service | memory_detail on the GPUI thread | low | accept | accept: documented in 03-40 plan; library/mod.rs:525-527 memory_detail only when the debug filter is on | closed |

*Status: open · closed · open — below high threshold (non-blocking)*
*Severity: critical > high > medium > low — only open threats at or above workflow.security_block_on count toward threats_open*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*
*T-03-SC is declared by all 40 plans (03-01..03-40); strictest severity high.*
*Evidence depth: L1 (grep and SUMMARY records); no threat was run dynamically in this audit.*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-03-01 | T-03-06 | SucroseInfo.json parsing: serde_json with typed fields and defaults; an oversized or malformed file only makes that theme corrupt | plan 03-02 (phase plan author) | 2026-11-24 |
| AR-03-02 | T-03-07 | xtask/src/localize.rs: Reads tracked repository files only and writes one generated file whose diff is reviewed in the commit; no network | plan 03-03 (phase plan author) | 2026-11-24 |
| AR-03-03 | T-03-12 | generated library size: `--count` is capped at 1000 and output goes to `target/` (gitignored); the size is printed | plan 03-05 (phase plan author) | 2026-11-24 |
| AR-03-04 | T-03-15 | crates/sucrose-ipc/src/portal.rs: The view carries settings values the same user can read in the file; Objectionable.json data never exists in the model (plan 04) | plan 03-06 (phase plan author) | 2026-11-24 |
| AR-03-05 | T-03-18 | settings changes: Each accepted change is logged with its variant and value (no paths); enough for support, no audit need on a single-user desktop app | plan 03-07 (phase plan author) | 2026-11-24 |
| AR-03-06 | T-03-27 | malformed image: The image and jpeg-decoder crates return errors on malformed input; the card shows ImageOff and the error is logged | plan 03-10 (phase plan author) | 2026-11-24 |
| AR-03-07 | T-03-29 | apps/sucrose-portal/src/library/card.rs: The Portal's own disabled state is a convenience; the service's decide_use (plan 09) is authoritative and refuses the same cases | plan 03-12 (phase plan author) | 2026-11-24 |
| AR-03-08 | T-03-36 | allow_set_foreground: The grant goes to the pid the service registered for its own Portal; granting a pid only lets that process take the foreground once | plan 03-15 (phase plan author) | 2026-11-24 |
| AR-03-09 | T-03-39 | a same-user process registered as the Portal: It would only receive Close and be ended if it is the sucrose-portal executable; anything else is left running | plan 03-16 (phase plan author) | 2026-11-24 |
| AR-03-10 | T-03-43 | clipboard: Only the quote text the user clicked is copied | plan 03-18 (phase plan author) | 2026-11-24 |
| AR-03-11 | T-03-56 | startup changes: Each write or removal is logged with the value name; enough on a single-user desktop | plan 03-23 (phase plan author) | 2026-11-24 |
| AR-03-12 | T-03-87 | memory_detail on the GPUI thread: One heap summary and one DXGI factory per page change, only with the debug filter on; a normal run makes no call | plan 03-40 (phase plan author) | 2026-11-24 |

*Accepted risks do not resurface in future audit runs.*

---

## Threat Flags (SUMMARY review)

All 27 SUMMARY files with a "Threat Flags" section state "None" or restate planned mitigations; no new threat surfaced.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-11-24 | 88 | 88 | 0 | gsd-secure-phase (L1 grep audit) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-11-24
