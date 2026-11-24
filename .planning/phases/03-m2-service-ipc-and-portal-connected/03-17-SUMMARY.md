---
phase: 03-m2-service-ipc-and-portal-connected
plan: 17
subsystem: ui
tags: [gpui, theme, backdrop, mica, acrylic, background-image, rtlgetversion, gif]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "PortalSettings and SettingChange (plans 04/06), the Header entity (plan 14), the service link and Welcome/Changed events"
provides:
  - "Sucrose ThemeConfig (light and dark) embedded in the Portal and applied with Light, Dark or System mode"
  - "Header theme toggle that changes the mode through the service; live OS following with no polling"
  - "Capabilities.window_backdrop and the window backdrop mapping (None/Mica/Acrylic/Tabbed) with opaque fallback"
  - "Background image layer: stretch, opacity, off-thread decode, GIF caps, atlas release, error classification"
affects: [03-18, 03-20, settings pages, status bar]

actuals:
  tokens: 15700
  tasks: 3
  commits: 5
plan_head_before: ec08a2a43c15dbd027e39496306146b17cbc7b90
plan_head_after: e7297fa6efc7f474a8b06afb0050b5962d8411ed

tech-stack:
  added: []
  patterns:
    - "ThemeConfig registered with ThemeRegistry::load_themes_from_str, switched with Theme::change (never Theme::global_mut colors)"
    - "Window background effects gated by a platform capability, never faked"
    - "Image layer = absolute overflow-hidden div + explicit placement rectangle + ObjectFit::Fill"

key-files:
  created:
    - apps/sucrose-portal/themes/sucrose.json
    - apps/sucrose-portal/src/theme.rs
    - apps/sucrose-portal/src/background.rs
  modified:
    - apps/sucrose-portal/src/app.rs
    - apps/sucrose-portal/src/header.rs
    - apps/sucrose-portal/src/main.rs
    - apps/sucrose-portal/src/assets.rs
    - apps/sucrose-portal/src/library/thumbs.rs
    - apps/sucrose-portal/Cargo.toml
    - crates/sucrose-platform/Cargo.toml
    - crates/sucrose-platform/src/lib.rs
    - crates/sucrose-platform/src/os_windows.rs
    - docs/FEATURES.md
    - docs/GPUI.md

key-decisions:
  - "The decode size is the smallest size that still covers the window (never enlarged, long edge capped at 2560): a non-matching aspect ratio may exceed the window on one edge so UniformToFill is never upscaled"
  - "A stored backdrop the OS cannot show is applied as Opaque with no change sent; the window background is read once from Capabilities.window_backdrop"
  - "The theme toggle in System mode sets the opposite of the effective theme explicitly (Light or Dark), as UI-SPEC says"

patterns-established:
  - "theme::follow_os re-applies the theme on the window appearance event only while the stored mode is System"
  - "BackgroundImage entity: generation counter drops stale decodes, release() frees replaced images from the GPUI atlas"

requirements-completed: [UI-01, UI-02]

duration: 19 min
completed: 2026-11-24
status: complete

coverage:
  - id: D1
    description: "The Portal follows the stored theme mode (Light, Dark, System) and the header toggle sets Light or Dark through the service"
    requirement: UI-02
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/theme.rs#theme_system_follows_the_os_appearance"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/header.rs#theme_toggle_asks_for_the_opposite_of_what_is_shown"
        status: pass
      - kind: e2e
        ref: "debug Portal on a Dark scratch library: 'theme applied mode=Dark' logged, capture target/synth-dark/portal-dark.png is #202020 with the Sun toggle"
        status: pass
    human_judgment: true
    rationale: "Colors and the live OS theme switch are visual and need real input; the maintainer's check is the plan 20 row"
  - id: D2
    description: "The window backdrop follows the stored setting through Capabilities.window_backdrop and falls back to opaque where the OS cannot show it"
    requirement: UI-01
    verification:
      - kind: unit
        ref: "crates/sucrose-platform/src/os_windows.rs#backdrop_needs_windows_11_22h2"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/theme.rs#theme_backdrop_maps_to_the_window_background"
        status: pass
      - kind: e2e
        ref: "debug Portal with Acrylic: 'backdrop applied backdrop=Acrylic supported=true appearance=Blurred' logged, capture target/synth-bg/portal-bg.png"
        status: pass
    human_judgment: true
    rationale: "A dark Acrylic window on the maintainer's migrated settings is a visual judgment (plan 20 row)"
  - id: D3
    description: "The background image layer draws with the legacy stretch and opacity inside the UI-07 memory rules and exposes its error"
    requirement: UI-01
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/background.rs#background_fit_uniform_to_fill_covers_the_area"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/background.rs#background_a_gif_over_the_frame_cap_shows_only_its_first_frame"
        status: pass
    human_judgment: true
    rationale: "Image fit, GIF animation and the memory returning after Remove are visual and need the file picker of plan 20"
---

# Phase 3 Plan 17: Theme mode, backdrop and background image Summary

**Sucrose light/dark ThemeConfig with a header toggle and live OS following, a window backdrop gated by a new `Capabilities.window_backdrop` (Windows build 22621+), and an off-thread background image layer with legacy stretch/opacity and a GIF cap**

## Performance

- **Duration:** 19 min
- **Started:** 2026-11-24T09:06:50Z
- **Completed:** 2026-11-24T09:25:30Z
- **Tasks:** 3 (Task 1 tracer, Tasks 2 and 3 TDD)
- **Files modified:** 15

## Accomplishments

- `themes/sucrose.json` carries the UI-SPEC tokens (#F3F3F3/#202020 pages, #005FB8/#60CDFF accent, destructive, muted, radius 8/12, 14 px base) and is embedded with `include_str!`; there is no folder watcher (D-008).
- `theme.rs`: `apply_mode`, `apply_initial`, `effective_dark`, `follow_os` (re-applies on the window appearance event only in System mode), `appearance`, `backdrop_active`, `page_alpha`. The app applies the mode on Welcome and on `Changed(ThemeMode)` and logs `theme applied`.
- Header toggle (Sun while dark, Moon while light, legacy tooltips) sends `Change(ThemeMode(Light|Dark))` through the service.
- `Capabilities.window_backdrop` (true from build 22621 via `RtlGetVersion`, false elsewhere); the app sets `window_background` at open and on Welcome/`Changed(Backdrop)`; an unsupported stored value is applied as Opaque and never rewritten.
- `background.rs`: header-checked, off-thread decode at no more than the window need (long edge 2560), JPEG DCT scaling, GIF frames kept within 120 frames / 24 MiB otherwise first frame only, still frame while the window is hidden, previous image released with `drop_image`, `BackgroundError` classification with the legacy text keys.
- Layering from the bottom: window backdrop, image layer, tinted page (alpha 0.78 with a backdrop or an image, else 1.0), content.

## Task Commits

1. **Task 1: theme mode and header toggle (tracer)** - `eae00c45` (feat)
2. **Task 2: window backdrop capability (TDD)** - RED `5b663c73` (test), GREEN `fcec79df` (feat)
3. **Task 3: background image layer and docs (TDD)** - RED `607e1f7b` (test), GREEN `e7297fa6` (feat)

**Plan metadata:** the docs commit that follows this summary.

TDD RED evidence (libtest has no `check tdd-red-evidence` adapter, the documented Rust gap, so the evidence is the cargo test output read by hand). Task 2 RED: `backdrop_needs_windows_11_22h2` and `this_windows_reports_a_build` failed on the assertions for build 22621 and a real build number, and `theme_backdrop_maps_to_the_window_background` and `theme_page_is_opaque_only_without_a_backdrop_and_an_image` failed on the planned mappings (the stubs returned Opaque, 1.0 and false). Task 3 RED: 11 of 18 background tests failed on the planned assertions (fit placement, decode size, opacity map, error classes, and the PNG/JPEG decode sizes that depend on `decode_size`); the 7 that passed covered GIF caps and extension/error-key behavior that the decode pipeline already had. Semantic assessment: every failure was an assertion on the planned value, not a build, load or fixture fault.

## Tracer gate

The tracer's `<human-check>` names the executor as the one who opens `target/synth-dark/portal-dark.png` and defers the click and OS-switch checks to the end of the phase (plan 20), so as in earlier plans of this phase the executor read the capture and continued with no checkpoint. The capture is dark (#202020 page, light text, Sun toggle, accent-blue active tab and page button), and the log carries `theme applied mode=Dark`.

## Files Created/Modified

- `apps/sucrose-portal/themes/sucrose.json` - Sucrose Light and Sucrose Dark ThemeConfig
- `apps/sucrose-portal/src/theme.rs` - theme mode, OS following, backdrop mapping, page alpha
- `apps/sucrose-portal/src/background.rs` - background image decode, placement and entity
- `apps/sucrose-portal/src/app.rs` - applies theme, backdrop and background; layers the window
- `apps/sucrose-portal/src/header.rs` - theme toggle button
- `apps/sucrose-portal/src/main.rs` - installs the themes, reads the capability once, opens with the stored backdrop
- `apps/sucrose-portal/src/assets.rs` - Sun and Moon icons
- `apps/sucrose-portal/src/library/thumbs.rs` - `to_bgra` extracted and `decode_generic` made public for the background image
- `crates/sucrose-platform/src/{lib.rs,os_windows.rs,Cargo.toml}` - `window_backdrop`, `backdrop_supported`, `windows_build`
- `docs/FEATURES.md`, `docs/GPUI.md` - backdrop row (build 22621) and GPUI notes

## Decisions Made

- Decode size covers the window (cover scale, never enlarged, long edge 2560) instead of fitting inside it, so a UniformToFill image with a different aspect ratio is not upscaled; both edges stay bounded by 2560 and the 16384 header check.
- The image is decoded for the window size at selection time; enlarging the window later scales it (no re-decode while dragging; see Next Phase Readiness).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Extra `windows` feature `Win32_System_SystemInformation`**
- **Found during:** Task 2
- **Issue:** `RtlGetVersion` and `OSVERSIONINFOW` are gated behind `Win32_System_SystemInformation` besides `Wdk_System_SystemServices`.
- **Fix:** added the feature in `crates/sucrose-platform/Cargo.toml` (no new crate).
- **Commit:** `fcec79df`

**2. [Rule 3 - Blocking] `serde_json` as a dev-dependency of the Portal**
- **Found during:** Task 1
- **Issue:** the theme-file test parses the embedded JSON; the Portal had no JSON crate.
- **Fix:** `serde_json = { workspace = true }` under `[dev-dependencies]` (already in the workspace and lock file; threat T-03-SC adds no new crate).
- **Commit:** `eae00c45`

**3. [Rule 3 - Blocking] `thumbs.rs` touched although not in the plan's file list**
- **Found during:** Task 3
- **Issue:** the background image needs the same BGRA premultiply and the generic limited decoder as the thumbnails.
- **Fix:** extracted `to_bgra` (used by `to_render_image`) and made `decode_generic` `pub`; no behavior change, all thumbnail tests pass.
- **Commit:** `607e1f7b`

**4. [Rule 1 - Adjustment] `theme::follow_os` carries the `observe_window_appearance` call**
- The acceptance criterion greps `observe_window_appearance` in `theme.rs`, so the subscription helper lives there and the app passes its closures.

---

**Total deviations:** 4 (3 blocking fixes, 1 placement adjustment). **Impact:** none on scope; no new crate.

## Issues Encountered

- A first visual run wrote Windows backslashes into the scratch `Sucrose.json`, which made the settings file invalid JSON, so the service migrated from the real legacy `Setting` folder (read-only, D-03) and started the maintainer's selected theme on two renderers for about ten seconds. Nothing was written to the real data; the run was stopped with `taskkill` and the capture rerun on a `synth-library` folder with forward-slash paths.

## Known Stubs

None. `BackgroundError::text_key` and `BackgroundImage::error` are real code with `#[allow(dead_code)]` until the settings page of plan 20 reads them.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 20 can read `BackgroundImage::error()` and `BackgroundError::text_key()` for the Background image row, `theme::appearance`/`Capabilities.window_backdrop` for the disabled backdrop options, and send `Change(Backdrop|BackgroundImage|BackgroundStretch|BackgroundOpacity)`.
- Open for the end-of-phase review (plan 20 rows): the human checks of Task 1 (toggle twice, OS theme switch in System mode) and Task 3 (JPEG then GIF, Uniform at 30 %, Remove, memory returns); a later window enlargement scales the decoded image rather than decoding again; macOS and Linux rendering is unverified by hand (UI-SPEC E9, flagged assumption).

## Self-Check: PASSED

- Created files exist: `apps/sucrose-portal/themes/sucrose.json`, `theme.rs`, `background.rs`.
- Commits exist: `eae00c45`, `5b663c73`, `fcec79df`, `607e1f7b`, `e7297fa6`.
- `cargo xtask ci` exits 0; `cargo test -p sucrose-portal` (103 tests) and `-p sucrose-platform` pass; clippy `-D warnings` is clean.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*
