# Roadmap

Milestones for the Rust rewrite. Each milestone is a usable vertical slice with explicit "done when" criteria. Check items off as they land and keep the status line current.

Status legend: ⬜ not started · 🟨 in progress · ✅ done

---

## M0: Foundation 🟨
- [x] Cargo workspace at the repo root (`crates/`, `apps/`, `xtask/`), `[workspace.dependencies]`, `[workspace.lints]` (D-011); `src/` and `exp/` are excluded
- [x] `rust-toolchain.toml` (1.99.0), `rustfmt.toml`, `clippy.toml`, `deny.toml` (all 818 dependencies pass the license list)
- [x] CI workflow `.github/workflows/rust.yml`: fmt; clippy `-D warnings` and tests on Windows, Ubuntu and macOS; cargo-deny
- [x] `sucrose-log`: daily rolling log files under `Log/`; `SUCROSE_DATA_DIR` overrides the data folder, `SUCROSE_LOG` the level
- [x] `sucrose-core` (paths), `sucrose-platform` (trait + stubs for all three OSes)
- [x] `apps/sucrose` (service stub) and `apps/sucrose-portal` (empty GPUI window); the prototype stays in `exp/` as an experiment
- [x] `cargo xtask ci` runs the CI checks locally
- [ ] CI green on all three OSes (needs a push; so far verified on Windows only)

**Done when** CI is green on all three OSes with stub platform implementations for macOS and Linux.

## M1: Video wallpaper on the desktop (Windows) ⬜
- [ ] `sucrose-core`: `SucroseInfo.json` model + fixtures from real themes
- [ ] `sucrose-config`: read `Library.json` and `General.json` (read-only at first)
- [ ] `sucrose-platform/windows`: desktop surface (Progman/WorkerW incl. Windows 11 24H2), monitors + DPI, fullscreen detection, monitor-change and explorer-restart events
- [ ] `sucrose-engine-mpv`: runtime-loaded libmpv, embedding into the surface, hardware decoding, loop, pause, volume
- [ ] `apps/sucrose`: plays the selected video theme on every monitor, pauses on fullscreen apps, tray with Quit

**Done when**
1. The selected video plays behind the desktop icons on every monitor.
2. It survives an explorer.exe restart and monitors being plugged/unplugged.
3. It pauses while a fullscreen game runs and resumes afterwards.
4. CPU and RAM are measured against the C# MpvPlayer engine + Backgroundog and recorded below.

## M2: Service, IPC and Portal connected ⬜
- [ ] `sucrose-ipc` (D-009) + single instance
- [ ] `sucrose-config`: new versioned settings format with atomic writes + automatic migration from legacy `Setting/*.json` (D-007)
- [ ] Modular service: optional modules start only when enabled (D-008)
- [ ] Portal uses the service: "Use" really applies the wallpaper; settings persist
- [ ] Autostart (`auto-launch`; Task Scheduler option later)

## M3: Engines and the wallpaper API ⬜
- [ ] WebView2 engine + JavaScript API compatible with the C# app
- [ ] Built-in GIF/APNG/WebP renderer
- [ ] CEF engine, libVLC engine, built-in OS video engine (D-006)
- [ ] URL, YouTube and Application wallpaper types
- [ ] System data (CPU/RAM/GPU/network/battery), audio spectrum, media session, input forwarding
- [ ] Display modes (per screen, span, duplicate), **a different wallpaper per monitor** (new), stretch modes, property editing
- [ ] Engine arguments editor (D-021)
- [ ] All Backgroundog performance rules

## M4: Product features ⬜
- [ ] Store (Soferity; protocol per D-022), downloads, cache
- [ ] Theme create/edit/import/export/share/report/review
- [ ] Wallpaper cycling, lock screen, display preferences
- [ ] Localization: 22 languages converted by `xtask`
- [ ] Discord Rich Presence
- [ ] Crash reporting and telemetry with the existing opt-outs and per-OS identity (D-018)
- [ ] Own installer, updater and uninstaller (D-019), signed with signtool via `xtask` (D-025), verified update packages, migration from the C# install

## M5: macOS ⬜
## M6: Linux (X11, then Wayland compositors) ⬜

## M7: Parity sign-off ⬜
- [ ] Every row in `FEATURES.md` is ported or has an accepted decision
- [ ] Remove the C# code, rewrite `CLAUDE.md` and these docs for the finished project

---

## Measurements

| Date | Scenario | C# app | Rust | Notes |
|---|---|---|---|---|
| 2026-11-21 | Portal prototype, settings page | – | ~90 MB working set | GPUI prototype, Windows, release build |
| 2026-11-21 | Portal prototype, 30 thumbnails | 173–201 MB (README screenshots, not same machine) | ~142 MB | thumbnails decoded at 480x270 |
| 2026-11-21 | Portal prototype, window appears | – | ~0.3 s | |