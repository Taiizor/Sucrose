---
phase: "03"
slug: m2-service-ipc-and-portal-connected
status: validated
nyquist_compliant: true
wave_0_complete: true
created: "2026-11-24"
---

# Phase 03 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via cargo (unit tests next to the code, integration tests under each crate's `tests/`) |
| **Config file** | none — workspace `Cargo.toml`, `rust-toolchain.toml`; `cargo xtask ci` runs fmt, clippy -D warnings and tests |
| **Quick run command** | `cargo test -p <crate>` (sucrose, sucrose-portal, sucrose-config, sucrose-core, sucrose-ipc, sucrose-platform, sucrose-engine-mpv, xtask) |
| **Full suite command** | `cargo test --workspace` (and `cargo xtask ci`) |
| **Estimated runtime** | ~30 s warm (measured 32 s, 2026-11-24); ~2–3 min after a clean build |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p <crate touched>`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** `cargo xtask ci` must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

Plans 03-11 (T1, T2), 03-24 (T2, T3), 03-28 (T3), 03-37 (T2, T3), 03-38 (T2, T3), 03-43 (T2) and 03-44 (T2, T3) are maintainer-run tasks without an automated command; they are listed under Manual-Only. Plan 03-25 T1 and 03-39 T1 run `cargo xtask measure --portal-summary` as a judgment step. Threat Ref lists the plan's threats with disposition mitigate.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-01-1 | 01 | 1 | ARCH-04, FMT-01, FMT-06 | T-03-01, T-03-02 | The service plays the selected theme from a migrated Setting/Sucrose.j | unit | `cargo test -p sucrose-config` | ✅ | ✅ green |
| 03-01-2 | 01 | 1 | ARCH-04, FMT-01, FMT-06 | T-03-01, T-03-02 | Store hardening — .bak restore, newer-version guard, unknown keys kept | unit | `cargo test -p sucrose-config --test store` | ✅ | ✅ green |
| 03-02-1 | 02 | 1 | LIB-02, LIB-03, LIB-08 | T-03-04, T-03-05 | Scan the real library read-only with the legacy validity rules (librar | unit | `cargo test -p sucrose-core --test library && cargo test -p sucrose-core --test library -- --ignored real_library --nocapture` | ✅ | ✅ green |
| 03-02-2 | 02 | 1 | LIB-02, LIB-03, LIB-08 | T-03-04, T-03-05 | Sort, search index, legacy URL check, SortMode and SortKind, showcase  | unit | `cargo test -p sucrose-core` | ✅ | ✅ green |
| 03-03-1 | 03 | 1 | UI-03, UI-05 | T-03-08 | Legacy XAML strings reach the Portal window (XAML → cargo xtask locali | unit | `cargo xtask localize --lang EN --check && cargo test -p xtask localize` | ✅ | ✅ green |
| 03-03-2 | 03 | 1 | UI-03, UI-05 | T-03-08 | The new Phase 3 keys, a key-coverage test, and a deny job that can jud | unit | `cargo test -p sucrose-portal i18n` | ✅ | ✅ green |
| 03-04-1 | 04 | 2 | FMT-01, FMT-06, LIB-01 | T-03-09, T-03-10 | The maintainer's Portal look and library state migrate (General, Porta | unit | `cargo test -p sucrose-config --test migrate && cargo test -p sucrose-config --test migrate -- --ignored real_install_migrates --nocapture` | ✅ | ✅ green |
| 03-04-2 | 04 | 2 | FMT-01, FMT-06, LIB-01 | T-03-09, T-03-10 | LIB-01 and the FEATURES.md Showcase and Window backdrop rows describe  | doc/grep check | `grep -c 'tracked in the new settings' .planning/REQUIREMENTS.md && grep -c -F 'S` | ✅ | ✅ green |
| 03-05-1 | 05 | 2 | UI-07 | T-03-11 | A synthetic library the service loads as is (xtask → Library + prepare | doc/grep check | `cargo xtask synth-library --out target/synth-small --count 40 --force && test -s` | ✅ | ✅ green |
| 03-05-2 | 05 | 2 | UI-07 | T-03-11 | Real-sized thumbnails and previews, determinism and the 500-theme run | unit | `cargo test -p xtask synth` | ✅ | ✅ green |
| 03-06-1 | 06 | 3 | ARCH-02 | T-03-13, T-03-16 | A Portal client gets Welcome from the real service (endpoint per data  | integration | `cargo test -p sucrose-ipc && cargo test -p sucrose --test portal` | ✅ | ✅ green |
| 03-06-2 | 06 | 3 | ARCH-02 | T-03-13, T-03-16 | The full message set, single-instance Portal, ordered writer, size and | integration | `cargo test -p sucrose-ipc portal && cargo test -p sucrose --test portal` | ✅ | ✅ green |
| 03-07-1 | 07 | 4 | ARCH-05, ARCH-04 | T-03-17 | A Portal change survives a restart (Portal client Change → settings ho | integration | `cargo test -p sucrose --test portal a_change_survives_a_restart && cargo test -p sucrose --test portal` | ✅ | ✅ green |
| 03-07-2 | 07 | 4 | ARCH-05, ARCH-04 | T-03-17 | Validation, clamping, Rejected on a failed write, arrival order, fulls | integration | `cargo test -p sucrose settings_host && cargo test -p sucrose --test portal` | ✅ | ✅ green |
| 03-08-1 | 08 | 5 | ARCH-02, UI-03 | T-03-20, T-03-21 | The Portal starts the service and connects (sucrose-portal → spawn suc | doc/grep check | `cargo xtask synth-library --out target/synth-small --count 40 && cargo build -p ` | ✅ | ✅ green |
| 03-08-2 | 08 | 5 | ARCH-02, UI-03 | T-03-20, T-03-21 | Connection banners, reconnect, single-instance exit, strict page argum | unit | `cargo test -p sucrose-portal` | ✅ | ✅ green |
| 03-09-1 | 09 | 6 | ARCH-05, ARCH-01, LIB-08 | T-03-22, T-03-23 | Use loads a new video into the running renderers (Portal client Use →  | integration | `cargo test -p sucrose-ipc && cargo test -p sucrose-engine-mpv` | ✅ | ✅ green |
| 03-09-2 | 09 | 6 | ARCH-05, ARCH-01, LIB-08 | T-03-22, T-03-23 | Refusals, idempotent Use, budget reset, rule state and runtime publica | integration | `cargo test -p sucrose decide_use && cargo test -p sucrose load_source` | ✅ | ✅ green |
| 03-10-1 | 10 | 6 | LIB-04, UI-07 | T-03-25, T-03-26 | The Portal shows the first page of the library with card-size thumbnai | doc/grep check | `cargo xtask synth-library --out target/synth-small --count 40 && cargo build -p ` | ✅ | ✅ green |
| 03-10-2 | 10 | 6 | LIB-04, UI-07 | T-03-25, T-03-26 | Bounded thumbnail cache with atlas release, size buckets, layout math, | unit | `cargo test -p sucrose-portal thumbs && cargo test -p sucrose-portal layout` | ✅ | ✅ green |
| 03-11-3 | 11 | 7 | ARCH-05 | T-03-28 | Apply and record the decision | unit | `cargo test -p sucrose decide_use` | ✅ | ✅ green |
| 03-12-1 | 12 | 8 | ARCH-05, LIB-08 | T-03-30 | A card click applies the theme through the service (card → Use request | doc/grep check | `cargo build -p sucrose -p sucrose-portal && rm -rf target/scratch-data && (SUCRO` | ✅ | ✅ green |
| 03-12-2 | 12 | 8 | ARCH-05, LIB-08 | T-03-30 | Disabled reasons in priority, keyboard Use, toasts, FEATURES notes for | unit | `cargo test -p sucrose-portal` | ✅ | ✅ green |
| 03-13-1 | 13 | 8 | ARCH-03 | T-03-31, T-03-32 | The Portal receives the memory of all Sucrose processes, only while co | integration | `cargo test -p sucrose modules && cargo test -p sucrose --test portal memory_reports_arrive_while_connected` | ✅ | ✅ green |
| 03-13-2 | 13 | 8 | ARCH-03 | T-03-31, T-03-32 | Live tray toggle, the fullscreen gate through the helper, the process- | unit | `cargo test -p sucrose` | ✅ | ✅ green |
| 03-14-1 | 14 | 9 | LIB-03, LIB-04, UI-03 | T-03-33 | The user pages through the library (settings view → page count → pagin | unit | `cargo test -p sucrose-portal pagination` | ✅ | ✅ green |
| 03-14-2 | 14 | 9 | LIB-03, LIB-04, UI-03 | T-03-33 | Header with Library tab and search; empty, search-empty and load-error | unit | `cargo test -p sucrose-portal` | ✅ | ✅ green |
| 03-15-1 | 15 | 10 | ARCH-02 | T-03-35 | A second sucrose opens the Portal through the running service (launche | integration | `cargo test -p sucrose --test portal` | ✅ | ✅ green |
| 03-15-2 | 15 | 10 | ARCH-02 | T-03-35 | The legacy Launcher start rule (D-12) and the service exit rule (D-13) | integration | `cargo test -p sucrose lifecycle && cargo test -p sucrose --test portal` | ✅ | ✅ green |
| 03-15-3 | 15 | 10 | ARCH-02 | T-03-35 | Foreground handoff for a second sucrose and a second Portal | integration | `cargo test -p sucrose-platform && cargo test -p sucrose --test portal` | ✅ | ✅ green |
| 03-16-1 | 16 | 11 | TRAY-05 | T-03-38 | The tray shows Open Sucrose and the AppExit-labelled exit item and dri | unit | `cargo test -p sucrose-platform && cargo test -p sucrose` | ✅ | ✅ green |
| 03-16-2 | 16 | 11 | TRAY-05 | T-03-38 | Close Applications and Exit, the live exit label, the tray back after  | unit | `cargo test -p sucrose lifecycle && cargo test -p sucrose` | ✅ | ✅ green |
| 03-17-1 | 17 | 12 | UI-01, UI-02 | T-03-40, T-03-41 | The Portal follows the stored theme mode and the header toggle changes | unit | `cargo test -p sucrose-portal theme` | ✅ | ✅ green |
| 03-17-2 | 17 | 12 | UI-01, UI-02 | T-03-40, T-03-41 | Window backdrop through a platform capability (D-22) | unit | `cargo test -p sucrose-platform && cargo test -p sucrose-portal theme` | ✅ | ✅ green |
| 03-17-3 | 17 | 12 | UI-01, UI-02 | T-03-40, T-03-41 | Background image layer with stretch, opacity and release; FEATURES and | unit | `cargo test -p sucrose-portal background` | ✅ | ✅ green |
| 03-18-1 | 18 | 13 | UI-03, UI-05 | — | The status bar shows the total memory of all Sucrose processes from th | unit | `cargo test -p sucrose-portal status_bar` | ✅ | ✅ green |
| 03-18-2 | 18 | 13 | UI-03, UI-05 | — | Premium, the ⋮ menu, and the Help and About dialogs | unit | `cargo test -p sucrose-portal` | ✅ | ✅ green |
| 03-18-3 | 18 | 13 | UI-03, UI-05 | — | The tray hint dialog on the first close (UI-05) | unit | `cargo test -p sucrose-portal tray_hint` | ✅ | ✅ green |
| 03-19-1 | 19 | 14 | LIB-06, UI-07 | T-03-45 | A hovered card plays its preview (hover → dwell → background decode →  | unit | `cargo test -p sucrose-portal preview` | ✅ | ✅ green |
| 03-19-2 | 19 | 14 | LIB-06, UI-07 | T-03-45 | Previews follow the settings live and stay out of the thumbnail cache | unit | `cargo test -p sucrose-portal preview && cargo test -p sucrose-portal thumbs` | ✅ | ✅ green |
| 03-20-1 | 20 | 15 | ARCH-05, LIB-04, UI-01, UI-02 | T-03-46, T-03-47 | A setting changed on a page reaches the service and the library (gear  | unit | `cargo test -p sucrose-portal settings` | ✅ | ✅ green |
| 03-20-2 | 20 | 15 | ARCH-05, LIB-04, UI-01, UI-02 | T-03-46, T-03-47 | The remaining Personal rows | unit | `cargo test -p sucrose-portal` | ✅ | ✅ green |
| 03-20-3 | 20 | 15 | ARCH-05, LIB-04, UI-01, UI-02 | T-03-46, T-03-47 | The General rows: theme mode, tray icon and AppExit, backdrop with bac | unit | `cargo test -p sucrose-portal settings` | ✅ | ✅ green |
| 03-21-1 | 21 | 16 | LIB-01, LIB-02 | T-03-48, T-03-49, T-03-50 | Showcase themes are copied into the library once (Portal load → Mainta | unit | `cargo test -p sucrose library_ops` | ✅ | ✅ green |
| 03-21-2 | 21 | 16 | LIB-01, LIB-02 | T-03-48, T-03-49, T-03-50 | Corrupt themes to the recycle bin through a platform wrapper; toast; F | unit | `cargo test -p sucrose library_ops && cargo test -p sucrose-platform` | ✅ | ✅ green |
| 03-22-1 | 22 | 17 | LIB-09 | T-03-51, T-03-52, T-03-53 | A library moves to an empty folder through the service (MoveLibrary →  | integration | `cargo test -p sucrose library_ops && cargo test -p sucrose --test portal` | ✅ | ✅ green |
| 03-22-2 | 22 | 17 | LIB-09 | T-03-51, T-03-52, T-03-53 | Folder picker, confirmation dialog and progress state in the Portal; F | unit | `cargo test -p sucrose-portal` | ✅ | ✅ green |
| 03-23-1 | 23 | 18 | TRAY-03 | T-03-55 | Normal startup writes Sucrose's own entry through the service (Portal  | integration | `cargo test -p sucrose-platform autostart && cargo test -p sucrose-platform --test autostart -- --ignored` | ✅ | ✅ green |
| 03-23-2 | 23 | 18 | TRAY-03 | T-03-55 | Legacy entry detection, the Startup row with its notes, D-034 and FEAT | unit | `cargo test -p sucrose-portal settings && cargo test -p sucrose-platform` | ✅ | ✅ green |
| 03-24-1 | 24 | 19 | UI-07 | T-03-57 | The Portal's memory is sampled and summarized (sucrose-portal → measur | unit | `cargo test -p xtask measure` | ✅ | ✅ green |
| 03-25-1 | 25 | 20 | UI-07 | T-03-58 | Judge and record the UI-07 numbers (stop scenario 2 → summaries → pass | doc/grep check | `cargo xtask measure --portal-summary target/measure/portal-real.json --warmup 20` | ✅ | ✅ green |
| 03-25-2 | 25 | 20 | UI-07 | T-03-58 | ARCHITECTURE, FEATURES and GPUI notes for M2; the verification scope | doc/grep check | `grep -c 'Sucrose.json' docs/ARCHITECTURE.md && grep -c 'no-portal' docs/ARCHITEC` | ✅ | ✅ green |
| 03-26-1 | 26 | 3 | FMT-01 | T-03-60 | The maintainer's engine, fullscreen, cycling and update settings migra | unit | `cargo test -p sucrose-config && cargo test -p sucrose-config --test migrate -- --ignored real_install_migrates --nocapture` | ✅ | ✅ green |
| 03-26-2 | 26 | 3 | FMT-01 | T-03-60 | ROADMAP criterion 2, FMT-01 and the FEATURES.md Formats row say what i | doc/grep check | `grep -c 'documented exclusions' .planning/ROADMAP.md && grep -c 'documented excl` | ✅ | ✅ green |
| 03-27-1 | 27 | 1 | UI-07 | T-03-62 | A hover preview released by the pointer-leave handler leaves the atlas | unit | `cargo test -p sucrose-portal atlas::` | ✅ | ✅ green |
| 03-27-2 | 27 | 1 | UI-07 | T-03-62 | Thumbnail evictions and background images release through the helper;  | unit | `cargo test -p sucrose-portal` | ✅ | ✅ green |
| 03-28-1 | 28 | 2 | LIB-06 | T-03-63, T-03-64, T-03-65 | A hovered card paints its preview's second frame (card element id → GP | unit | `cargo test -p sucrose-portal card_preview_` | ✅ | ✅ green |
| 03-28-2 | 28 | 2 | LIB-06 | T-03-63, T-03-64, T-03-65 | The hover path says why a preview did not start; GPUI.md and FEATURES. | unit | `cargo test -p sucrose-portal` | ✅ | ✅ green |
| 03-29-1 | 29 | 2 | UI-07, UI-01 | — | A 16384 x 16384 thumbnail header is refused before any decoder runs (l | unit | `cargo test -p sucrose-portal pixel_area` | ✅ | ✅ green |
| 03-29-2 | 29 | 2 | UI-07, UI-01 | — | The background image and the preview GIF use the same limit; one JPEG  | unit | `cargo test -p sucrose-portal pixel_area && cargo test -p sucrose-portal` | ✅ | ✅ green |
| 03-30-1 | 30 | 1 | ARCH-04 | T-03-68, T-03-69 | A locked Sucrose.json is never overwritten (Unreadable → retry → Err(S | unit | `cargo test -p sucrose-config --test store unreadable` | ✅ | ✅ green |
| 03-30-2 | 30 | 1 | ARCH-04 | T-03-68, T-03-69 | An unreadable backup never leads to a migration over the file; the ret | integration | `cargo test -p sucrose-config && cargo test -p sucrose --bin sucrose settings_host` | ✅ | ✅ green |
| 03-31-1 | 31 | 1 | LIB-09 | T-03-70, T-03-71, T-03-72 | Maintenance asked for during a library move is answered at once and st | unit | `cargo test -p sucrose --bin sucrose library_move` | ✅ | ✅ green |
| 03-31-2 | 31 | 1 | LIB-09 | T-03-70, T-03-71, T-03-72 | The fleet stays quiet during a move and catches up when it ends (pendi | integration | `cargo test -p sucrose --bin sucrose library_move && cargo test -p sucrose --test portal library` | ✅ | ✅ green |
| 03-31-3 | 31 | 1 | LIB-09 | T-03-70, T-03-71, T-03-72 | Status text is always printable (WR-07) | unit | `cargo test -p sucrose --bin sucrose printable && cargo test -p sucrose` | ✅ | ✅ green |
| 03-32-1 | 32 | 2 | TRAY-05, ARCH-05 | T-03-73, T-03-74 | A Portal request that changes state is not acted on once the service i | unit | `cargo test -p sucrose --bin sucrose quitting` | ✅ | ✅ green |
| 03-32-2 | 32 | 2 | TRAY-05, ARCH-05 | T-03-73, T-03-74 | The service waits for a running library move before it exits | unit | `cargo test -p sucrose --bin sucrose quitting && cargo test -p sucrose` | ✅ | ✅ green |
| 03-33-1 | 33 | 3 | ARCH-02 | — | Disconnecting a connection wakes its blocked reader and tells the clie | unit | `cargo test -p sucrose-ipc disconnect && cargo test -p sucrose-platform` | ✅ | ✅ green |
| 03-33-2 | 33 | 3 | ARCH-02 | — | A Portal given up for a full queue is disconnected, and any Portal tha | integration | `cargo test -p sucrose --bin sucrose portal && cargo test -p sucrose --test portal` | ✅ | ✅ green |
| 03-34-1 | 34 | 1 | ARCH-01, ARCH-05 | T-03-77 | The new source starts only after its own file loaded (SourceSwitch → e | unit | `cargo test -p sucrose-engine-mpv source_switch` | ✅ | ✅ green |
| 03-34-2 | 34 | 1 | ARCH-01, ARCH-05 | T-03-77 | The real-libmpv tests and the service Use tests still pass | unit | `cargo test -p sucrose-engine-mpv && cargo test -p sucrose-engine-mpv -- --ignored` | ✅ | ✅ green |
| 03-35-1 | 35 | 1 | TRAY-03 | T-03-78, T-03-79 | Turning startup off succeeds once the Run value is gone (two-step remo | unit | `cargo test -p sucrose-platform --lib autostart::tests` | ✅ | ✅ green |
| 03-35-2 | 35 | 1 | TRAY-03 | T-03-78, T-03-79 | The startup setting follows what the OS reports after a failed request | unit | `cargo test -p sucrose --bin sucrose startup && cargo test -p sucrose-platform` | ✅ | ✅ green |
| 03-36-1 | 36 | 2 | UI-07 | T-03-80, T-03-81 | Rounds come from the Portal's page log (series start in ms → log parse | unit | `cargo test -p xtask portal_` | ✅ | ✅ green |
| 03-36-2 | 36 | 2 | UI-07 | T-03-80, T-03-81 | synth-library cannot be pointed at the real data folder by any spellin | unit | `cargo test -p xtask synth` | ✅ | ✅ green |
| 03-37-1 | 37 | 5 | UI-07 | T-03-82 | Prepare scenario 1 and prove the pipeline on a short unattended pass ( | doc/grep check | `cargo build --release -p sucrose -p sucrose-portal && test -f target/measure-rea` | ✅ | ✅ green |
| 03-38-1 | 38 | 6 | UI-07 | T-03-83 | Prepare scenario 2 and prove the pipeline on a short unattended pass ( | doc/grep check | `cargo xtask synth-library --out target/synth-500 --count 500 && grep -c '"Librar` | ✅ | ✅ green |
| 03-39-1 | 39 | 7 | LIB-06, UI-07 | T-03-84 | The reruns are judged and recorded (summaries → rule → MEASUREMENTS ro | doc/grep check | `cargo xtask measure --portal-summary target/measure/portal-real-2.json --portal-` | ✅ | ✅ green |
| 03-39-2 | 39 | 7 | LIB-06, UI-07 | T-03-84 | Every review finding has its final disposition | doc/grep check | `f=.planning/phases/03-m2-service-ipc-and-portal-connected/03-REVIEW-DISPOSITION.` | ✅ | ✅ green |
| 03-39-3 | 39 | 7 | LIB-06, UI-07 | T-03-84 | LIB-06 and UI-07 are marked Complete on evidence | doc/grep check | `grep -c '^- \[x\] \*\*LIB-06\*\*' .planning/REQUIREMENTS.md && grep -c '^- \[x\]` | ✅ | ✅ green |
| 03-40-1 | 40 | 4 | UI-07 | T-03-85, T-03-86 | The page log carries the Portal's heap and video memory (platform read | unit | `cargo test -p sucrose-platform memory_detail` | ✅ | ✅ green |
| 03-40-2 | 40 | 4 | UI-07 | T-03-85, T-03-86 | The log-split summary attributes the growth to GPU memory, held heap o | unit | `cargo test -p xtask portal_` | ✅ | ✅ green |
| 03-41-1 | 41 | 1 | UI-01, UI-02 | T-03-88, T-03-89 | Acrylic shows through: the Base Root fill is cleared while a backdrop  | unit | `cargo test -p sucrose-portal theme_ && grep -c 'flat=true' target/backdrop-check` | ✅ | ✅ green |
| 03-41-2 | 41 | 1 | UI-01, UI-02 | T-03-88, T-03-89 | The Root fill is pinned by GPUI tests for every backdrop change | unit | `cargo test -p sucrose-portal theme_ && cargo clippy -p sucrose-portal --all-targ` | ✅ | ✅ green |
| 03-42-1 | 42 | 2 | LIB-04, LIB-06, UI-07 | T-03-91, T-03-92, T-03-93, T-03-94 | Cards past the 64th get thumbnails (rows in view, wanted keys, cache t | unit | `cargo test -p sucrose-portal layout_ && cargo test -p sucrose-portal thumbs_ && ` | ✅ | ✅ green |
| 03-42-2 | 42 | 2 | LIB-04, LIB-06, UI-07 | T-03-91, T-03-92, T-03-93, T-03-94 | The grid shows its scrollbar and cards that are not loading stop anima | unit | `cargo test -p sucrose-portal && cargo clippy -p sucrose-portal --all-targets -- ` | ✅ | ✅ green |
| 03-43-1 | 43 | 3 | UI-01, UI-02, LIB-04, LIB-06 | T-03-95 | Prepare the maintainer check and prove it unattended (scratch settings | doc/grep check | `test -f target/uat-gaps/Setting/Sucrose.json && grep -c '"DeleteCorrupt": false'` | ✅ | ✅ green |
| 03-44-1 | 44 | 4 | UI-07 | T-03-96, T-03-97 | Prepare scenario 2 after 03-42 and prove the pipeline on a short unatt | doc/grep check | `cargo xtask synth-library --out target/synth-500 --count 500 && grep -c '"Librar` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Requirement Coverage

| Requirement | Class | Key automated evidence |
|-------------|-------|------------------------|
| ARCH-01 | COVERED | service.rs load_source_* tests; mpv player source_switch_* and tests/options.rs#load_source_reports_started_for_the_new_file; player.rs finishing_a_load_never_wipes_a_newer_request, a_failed_load_stops_loading_unless_a_newer_one_follows (WR-11) |
| ARCH-02 | COVERED | apps/sucrose/tests/portal.rs (welcome, single portal, garbage frame, launch handoff); sucrose-ipc portal/endpoint tests; tests/socket.rs disconnect tests; portal_link.rs tests |
| ARCH-03 | COVERED | modules.rs tests; portal.rs the_tray_follows_the_notification_area_switch_live, memory_reports_arrive_while_connected |
| ARCH-04 | COVERED | sucrose-config tests/store.rs (atomic write, kill during write, .bak restore, newer version, unreadable file, corrupt file kept aside as Sucrose.json.corrupt — WR-09); settings_host.rs tests |
| ARCH-05 | COVERED | portal.rs a_change_survives_a_restart, use_switches_between_video_themes; settings_host.rs; service.rs decide_use, quitting tests |
| FMT-01 | COVERED | sucrose-config tests/migrate.rs (real and fixture files, nine migrated files, exclusions) |
| FMT-06 | COVERED | tests/store.rs#nothing_at_all_gives_the_defaults_in_a_new_setting_folder; migrate.rs empty, bom, retry, clamp tests |
| TRAY-03 | COVERED | startup.rs and os_windows/autostart.rs tests; tests/autostart.rs plist and desktop-entry tests; registry tests are ignored (manual) |
| TRAY-05 | COVERED | lifecycle.rs closing_* and only_the_sibling_portal_executable_may_be_ended; service.rs quitting_* tests; real Exit-closes-all is manual |
| LIB-01 | COVERED | library_ops.rs copies_only_the_showcase_folders_that_are_not_listed; portal.rs showcase_themes_are_copied_once_and_recorded_in_the_settings; migrate.rs library_and_warehouse_keep_their_values |
| LIB-02 | COVERED | sucrose-core tests/library.rs (six fixture themes, corrupt reasons, path escape); library_ops.rs corrupt-to-bin tests |
| LIB-03 | COVERED | tests/library.rs sort and search tests; header.rs search tests; grid.rs state tests |
| LIB-04 | COVERED | layout.rs layout_* (10), pagination.rs and library/mod.rs page tests; visual capture is manual; layout.rs rows_in_view and cards_to_request tests (03-42); scrollbar visibility by hand is manual |
| LIB-06 | COVERED | library/preview.rs preview_* (15 tests), card.rs card_preview_paints_its_second_frame, mod.rs preview_turned_off_*; hover by eye is manual |
| LIB-08 | COVERED | service.rs decide_use; portal.rs refusal tests; card.rs use_state and tooltip tests |
| LIB-09 | COVERED | library_ops.rs move, rollback and nesting tests; portal.rs library move tests; service.rs library_move tests and a_renderer_exit_while_quitting_* (WR-10) |
| UI-01 | COVERED | os_windows.rs#backdrop_needs_windows_11_22h2; theme.rs backdrop test; background.rs tests; general.rs backdrop options test; theme.rs theme_* root fill tests (03-41: root_fill, apply_root_fill clears and restores the Base Root); Mica and Tabbed by eye are manual |
| UI-02 | COVERED | theme.rs#theme_system_follows_the_os_appearance; header.rs theme toggle test; settings/mod.rs gear test |
| UI-03 | COVERED | xtask localize tests, i18n.rs key tests, main.rs argument tests, status_bar.rs, header.rs, ipc.rs state tests; captures are manual |
| UI-05 | COVERED | i18n.rs coverage tests; dialogs help, about, tray_hint, link tests |
| UI-07 | COVERED (automated parts) | thumbs.rs cache, queue, decode, pixel-area and decode-budget tests (WR-12; WR-13 the_generic_fallback_decodes_under_a_permit_priced_from_the_header); atlas.rs release tests; xtask measure and synth tests; memory.rs tests. The memory bound itself is judged by D-25 measurements (manual); 03-42 thumbs.rs peek, touch, insert_keeping and trim tests (wanted thumbnails are never evicted) |

Counts: 21 requirements, 21 COVERED, 0 PARTIAL, 0 MISSING. A spot check of 220 test-function references from the SUMMARY coverage entries found every function present (one reference, `the_runtime_state_is_published_only_when_it_differs_from_the_last_one_sent` in 03-09, is the shortened name `..._from_the_last_one` in apps/sucrose/src/service.rs).

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Exit / Close Applications ends the service, renderers and Portal on a real desktop (SC3) | TRAY-05, ARCH-02 | Needs real processes, tray and desktop; unit tests cover the sequencing rules (lifecycle closing_*, quitting_*) only | Start sucrose.exe on a scratch SUCROSE_DATA_DIR, open the Portal, choose tray Close Applications, confirm no sucrose* process remains |
| Autostart across a real logon (SC4) | TRAY-03 | Needs a Windows sign-out/sign-in; registry round trip is covered by ignored tests run by hand | Enable Startup in General, sign out and in, confirm the service starts; disable and repeat |
| Hover preview animates by eye | LIB-06 | Perceived animation needs a human on the release Portal; frame painting is covered by card_preview_paints_its_second_frame | Hover a GIF card for over 200 ms in the release Portal and watch the preview play (03-28 T3; re-run with Library Preview on past the 64th card in 03-43 T2 C: yes, 2026-11-25) |
| D-25 memory and CPU measurements: real library and 500-theme synthetic library, paging and hovering by hand | UI-07 | Needs the maintainer driving a release Portal for minutes; judged by `cargo xtask measure --portal-summary` (03-24, 03-37, 03-38, 03-39; 03-25 D2 failed, then passed after the atlas fix) | Follow the ready gate in 03-37/03-38; run measure with --portal-summary and compare with docs/MEASUREMENTS.md |
| Release-build Video theme switching and the Use transition choice (D-26) | ARCH-05 | Maintainer watches and chooses (03-11 T1, T2); decide_use is automated | Switch Video themes ten times in the release build and judge the transition |
| Visual captures: window, dark theme, backdrop, banners, header, status bar, pagination | UI-01, UI-02, UI-03, LIB-04 | PrintWindow captures read by a person on a real desktop; logic behind them is unit tested | Run sucrose-portal.exe --no-focus on a synth-library scratch folder and capture with capture.ps1 |
| Real Recycle Bin move for corrupt themes | LIB-02 | Touches the real bin; trash test is #[ignore] and run once by hand | cargo test -p sucrose-platform -- --ignored a_temporary_folder_moves_to_the_real_bin |
| Real HKCU Run key tests | TRAY-03 | Write the maintainer registry; ignored by default | cargo test -p sucrose-platform --test autostart -- --ignored |
| Real-library migration and scan | FMT-01, LIB-02 | Read the maintainer real data; ignored tests (real_install_migrates, real_library) | cargo test -p sucrose-config --test migrate -- --ignored real_install_migrates; cargo test -p sucrose-core --test library -- --ignored real_library |
| Real libmpv Video theme playback | ARCH-01 | Needs engine binaries from cargo xtask fetch-engines; ignored tests | cargo xtask fetch-engines && cargo test -p sucrose-engine-mpv -- --ignored |
| Mica, Tabbed and None look different from Acrylic on the active window, in Light and Dark | UI-01, UI-02 | DWM draws Mica and Tabbed only on an active window, and PrintWindow returns no content with a backdrop; the Root fill logic is unit tested (theme_*) | 03-43 T2 A on the release Portal. 2026-11-25: Acrylic and None as expected; Mica and Tabbed not distinct (stay transparent after Acrylic, opaque after None): open gap |
| Grid scrollbar visible, and every card past the 64th shows its thumbnail while scrolling by hand at 100 per page | LIB-04, UI-07 | Needs real wheel and scrollbar input, which is never synthesized; rows_in_view and the keep-wanted cache are unit tested | 03-43 T2 B. 2026-11-25: thumbnails yes, scrollbar not visible: open gap |
| D-25 scenario 2 after the in-view thumbnail requests (03-42): 500-theme synthetic library, paged, scrolled and hovered by hand | UI-07 | Needs the maintainer driving a release Portal for about 12 minutes; judged by `cargo xtask measure --portal-summary` | 03-44 T2 and T3. 2026-11-25: skipped by the maintainer, no measurement; UAT test 10 stays open |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or are maintainer-run manual tasks listed above
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (none)
- [x] No watch-mode flags
- [x] Feedback latency < 180s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-11-24; re-audited after gap plans 03-41..03-44 on 2026-11-25 (no automated gaps)

## Validation Audit 2026-11-24

| Metric | Count |
|---|---|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |
