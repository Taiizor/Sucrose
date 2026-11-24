---
phase: 03-m2-service-ipc-and-portal-connected
verified: 2026-11-24T13:05:00Z
status: gaps_found
score: 7/13 must-haves verified
covered_files:
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-01-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-01-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-02-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-02-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-03-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-03-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-04-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-04-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-05-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-05-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-06-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-06-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-07-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-07-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-08-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-08-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-09-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-09-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-10-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-10-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-11-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-11-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-12-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-12-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-13-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-13-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-14-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-14-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-15-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-15-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-16-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-16-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-17-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-17-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-18-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-18-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-19-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-19-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-20-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-20-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-21-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-21-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-22-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-22-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-23-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-23-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-24-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-24-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-25-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-25-SUMMARY.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-26-PLAN.md
  - .planning/phases/03-m2-service-ipc-and-portal-connected/03-26-SUMMARY.md
  - apps/sucrose-portal/src/background.rs
  - apps/sucrose-portal/src/library/card.rs
  - apps/sucrose-portal/src/library/mod.rs
  - apps/sucrose-portal/src/library/preview.rs
  - apps/sucrose-portal/src/library/thumbs.rs
  - apps/sucrose/src/lifecycle.rs
  - apps/sucrose/src/modules.rs
  - apps/sucrose/src/service.rs
  - apps/sucrose/src/startup.rs
  - crates/sucrose-config/src/migrate.rs
  - crates/sucrose-config/src/store.rs
  - crates/sucrose-platform/src/autostart.rs
  - docs/MEASUREMENTS.md
covered_digest: "v4:sha256:8f1d58d2a592c95872d056be5fb8b46bf35bb4bc8abe561ff9b70d31a22ff5a3"
behavior_unverified: 2
overrides_applied: 0
gaps:
  - truth: "SC1: the user can see hover previews of themes in the library"
    status: failed
    reason: "Hover previews show only the first frame in the release Portal (maintainer-observed in both D-25 runs). The decode path is tested (17 preview tests pass) but the card paints the preview with an img() that has no element id; gpui-pre 0.3.12 advances and repaints a multi-frame image only when the element has a global id (img.rs: frame_index lives in with_optional_element_state(global_id), and request_animation_frame needs global_id.is_some()). No test covers the paint path."
    artifacts:
      - path: "apps/sucrose-portal/src/library/card.rs"
        issue: "line 282-283: card.child(img(image).absolute().inset_0()...) has no .id(...), so the GIF never leaves frame 0"
      - path: "apps/sucrose-portal/src/library/mod.rs"
        issue: "hover_card (about 541-562) returns silently on several paths, so a second cause cannot be excluded without a run"
    missing:
      - "Give the preview img() a stable element id (for example per card index) so GPUI animates it, then confirm by a human hover run"
      - "A test or capture that proves a second frame is painted (decode tests alone cannot see it)"
      - "Repeat both D-25 scenarios after the fix (plan 24 and 25 both say so)"
  - truth: "SC5: scrolling through a large library keeps the Portal's memory bounded (release build, recorded in docs/MEASUREMENTS.md)"
    status: failed
    reason: "docs/MEASUREMENTS.md records the 500-theme synthetic library as Rule: FAIL: working set first page 131 MB, peak 287, final 269 against a limit of 181 MB (private 124 / 279 / 262). The 90-theme real library passes (final 206 against 223). Both runs had non-animating previews and the scenario 2 round windows were skewed, so the cause (atlas pages, decode buffers, allocator retention) is not established. REQUIREMENTS.md keeps UI-07 Pending."
    artifacts:
      - path: "docs/MEASUREMENTS.md"
        issue: "rows 14-15 and the paging paragraph: scenario 2 FAIL, preview cost never measured"
      - path: "apps/sucrose-portal/src/library/thumbs.rs"
        issue: "something outside the 64 image / 48 MiB thumbnail cache grows with the pages visited"
    missing:
      - "Find and fix the growth source on the 500-theme library (GPUI atlas release, decode buffers, allocator retention)"
      - "Repeat D-25 scenarios 1 and 2 with working previews and a tighter round cue; both must pass the rule"
  - truth: "Untrusted theme images (thumbnail, background image) cannot exhaust memory (threat T-03-25 / T-03-40; review CR-01)"
    status: failed
    reason: "decode_bytes in thumbs.rs checks only each side against MAX_SIDE (16384) and then calls decode_jpeg_scaled (jpeg_decoder directly) with no pixel-count or buffer limit; grep finds no MAX_PIXELS and no set_max_decoding_buffer_size. The review reports a progressive JPEG allocates about 6 bytes per pixel before scaling (about 1.6 GB at 16384 x 16384, three decodes in parallel) and the library decodes every thumbnail of the shown page without user action. background.rs reuses the same path. Disposition is open."
    artifacts:
      - path: "apps/sucrose-portal/src/library/thumbs.rs"
        issue: "lines 250-283: per-side check only, JPEG fast path outside Limits::max_alloc"
      - path: "apps/sucrose-portal/src/background.rs"
        issue: "same decode path"
    missing:
      - "A shared pixel-area limit checked right after into_dimensions in both decode_bytes functions, with a test that a 16384 x 16384 header is refused"
  - truth: "The settings store never replaces a good Sucrose.json after a transient read error (ARCH-04 / ROADMAP SC2 'settings persist'; review CR-02)"
    status: failed
    reason: "crates/sucrose-config/src/store.rs read_file maps every I/O error except NotFound to Read::Corrupt (lines 191-194); Store::open then restores Sucrose.json.bak over the file (lines 98-103) or runs migrate and persist (lines 117-130). A sharing violation or antivirus lock therefore silently rolls the user's settings back or resets them. legacy.rs already retries such reads; the store does not. Disposition is open."
    artifacts:
      - path: "crates/sucrose-config/src/store.rs"
        issue: "Read::Corrupt used for unreadable files; no retry and no Unreadable state"
    missing:
      - "Separate Unreadable from Corrupt, retry the read, and return Err(StoreError::Io) (in-memory no-save mode) instead of restoring or migrating"
      - "A store test with an unreadable (locked or access-denied) Sucrose.json that proves the file is not overwritten"
behavior_unverified_items:
  - truth: "SC3: Exit / Close Applications closes every Sucrose process (TRAY-05)"
    test: "Release build playing a wallpaper with the Portal open: right-click the tray icon, check Open Sucrose is greyed, choose Close Applications, then list processes (plan 03-16 human check)"
    expected: "Wallpaper stops, Portal closes, tray icon gone within about 3 s, no sucrose.exe or sucrose-portal.exe left"
    why_human: "The Closing state machine and the Portal-ending rule are unit tested, but the end-to-end kill of renderer, Portal and service needs a tray click (no synthesized input). Review WR-02 also notes the service still handles Portal requests while quitting."
  - truth: "SC4: with start with the OS on, Sucrose starts after sign-in and resumes the wallpaper (TRAY-03)"
    test: "Release build with the maintainer's settings: General row Normal, sign out and in, then None (plan 03-23 human check)"
    expected: "After sign-in Sucrose starts and the selected wallpaper plays; the C# entry is unchanged; after None the Sucrose.Service Run value is gone"
    why_human: "Run value text, LaunchAgent and XDG entry text are tested (18 platform tests pass; the three registry tests are #[ignore] because they write the real HKCU key and were not run here); a real logon cannot be automated. macOS and Linux are unexercised. Review WR-05 notes turning startup off can leave setting and registry disagreeing."
human_verification:
  - test: "Hover previews after the fix: hover several cards slowly, sweep fast across a row, watch the status bar memory (03-19)"
    expected: "A slowly hovered card animates and hides its title strip; a fast sweep starts nothing; memory returns near its previous value"
    why_human: "Real pointer movement; no synthesized input"
  - test: "Card click and keyboard Use, Web card tooltip (03-12); search typing, Escape, Ctrl+F, sort (03-14)"
    expected: "Clicked Video theme plays, Selected pill moves; Web card shows not-supported tooltip; search filters after about 0.5 s; Escape restores; sort reorders at once"
    why_human: "Typing and clicking need a person"
  - test: "Second launch focuses the open Portal (03-15)"
    expected: "Existing Portal comes to the front, no second window or tray icon"
    why_human: "Foreground rules need real user input"
  - test: "Light/dark toggle and OS following; Acrylic with JPEG and GIF background; Help, About, Premium links; tray hint dialog (03-17, 03-18)"
    expected: "As in the plans' expected text"
    why_human: "Visual and browser launch"
  - test: "Settings rows show migrated values and survive restart; 250 clamps to 100 (03-20)"
    expected: "Dark, Acrylic, Visible, AppExit checked, library path, pagination 30; changes persist"
    why_human: "Driving the controls needs real input"
  - test: "Corrupt themes to the Recycle Bin and restore (03-21); library move with picker, confirmation, progress and non-empty refusal (03-22)"
    expected: "Toast and Recycle Bin contents; library reloads from the new folder, old folder gone, non-empty refused"
    why_human: "Desktop and picker checks"
---

# Phase 3: M2 - Service, IPC and Portal connected Verification Report

**Phase Goal:** The service and the Portal work as one app: the user picks a theme in the Portal's library, "Use" applies it through the service, settings migrate from the C# install and persist, and Sucrose can start with the OS.
**Verified:** 2026-11-24T13:05:00Z
**Status:** gaps_found
**Re-verification:** No, initial verification

## Goal Achievement

The core connected loop works: the Portal connects to the service on one endpoint, a card click sends `PortalRequest::Use`, the service decides, stores `Library.Selected` and sends `LoadSource` to the running renderers, and the maintainer watched it (03-11 D-26 clean on run 2, 03-12 tracer approved). Settings migration, atomic persistence, single instance, bounded restarts and optional-module gating are backed by tests I re-ran. The goal is not fully achieved because two ROADMAP success-criteria items fail outright (hover previews, bounded memory on a large library) and two critical review findings (CR-01, CR-02) are still open and confirmed in the code.

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SC1a: first run shows the library with showcase copied, corrupt and incompatible themes handled as legacy, sortable, searchable, paged, movable | VERIFIED | `crates/sucrose-core` library tests 20 pass (1 ignored); `apps/sucrose/tests/portal.rs` 26 pass, including showcase copy once, damaged theme refused as corrupt, newer-Sucrose theme refused, `a_library_moves_to_an_empty_folder_and_the_location_follows`, `a_destination_that_is_not_empty_is_refused`; header search, pagination and move dialog wired (`header.rs`, `pagination.rs`, `settings/general.rs:508`). Visual and input parts are in Human Verification. |
| 2 | SC1b: the user sees hover previews | FAILED | Maintainer reports only frame 0 shows (03-24-SUMMARY). `card.rs:282` paints `img(image)` with no `.id()`; in gpui-pre 0.3.12 `img.rs` the frame index and `request_animation_frame` exist only when `global_id` is Some (lines 285-296, 378-383). Preview unit tests (17) pass but do not reach painting. BLOCKER. |
| 3 | SC1c: "Use" on a video theme plays on the desktop through the service | VERIFIED | `card.rs:164` sends `PortalRequest::Use`; `service.rs:1265` handles it, `decide_use` (2872), `USE_TRANSITION = LoadSource` (2828), `load_source` (3113) sends `ToRenderer::LoadSource`; portal tests for Use refusals pass; maintainer observed Use between two videos (03-11, 03-12). `use_switches_between_video_themes` is #[ignore] (drives the real service), covered by the maintainer run. |
| 4 | SC2: legacy settings migrate (nine files key by key, Donate and documented exclusions out), data folder layout kept, a changed setting survives restart, a kill during write never leaves a corrupt file, legacy files never modified | VERIFIED | `cargo test -p sucrose-config`: 29 migrate/legacy tests and 11 store tests pass (`kill_during_write_never_leaves_a_corrupt_file`, `migration_runs_once_and_never_writes_a_legacy_file`, `the_nine_real_files_serialize_to_stable_text_without_excluded_keys`); `apps/sucrose/tests/portal.rs#a_change_survives_a_restart` passes. Exclusions are listed in `migrate.rs:24-38` and the FEATURES.md formats row; 14 legacy files = 9 migrated + Donate + Objectionable + Aurora, Kernel, System (all excluded per formats.md §1.3). The CR-02 hole in this area is its own truth (13). |
| 5 | SC3a: starting the service or Portal a second time reuses the running instance | VERIFIED | portal.rs tests pass: `a_second_portal_is_told_the_first_is_open_and_the_first_is_focused`, `a_launch_with_a_portal_registered_gets_portal_running`, `a_launcher_that_passes_on_the_foreground_gets_the_open_portal_focused`, `a_portal_that_left_makes_room_for_the_next`. |
| 6 | SC3b: Exit closes every Sucrose process | PRESENT_BEHAVIOR_UNVERIFIED | `Closing` state machine (`lifecycle.rs:31`), `end_portal` with sibling-path check (`modules.rs:83`), Close Applications path (`service.rs:1432-1582`) are present and wired; no test spawns the real processes, plan 03-16 left a human tray check. |
| 7 | SC3c: a renderer that keeps crashing is restarted a bounded number of times, never in a loop | VERIFIED | `supervisor.rs` MAX_CRASHES 3 / MAX_SURFACE_LOSSES 5 in a sliding window with doubling backoff; `cargo test -p sucrose supervisor` 17 pass including `three_crashes_in_a_minute_are_restarted_with_growing_pauses_and_the_fourth_is_refused`. |
| 8 | SC4a: an optional module the user turns off starts no thread and does no polling | VERIFIED | `modules.rs` `start_module` / `module_off`; `cargo test -p sucrose modules` 9 pass (tray off with AppVisible false, memory module only while a Portal is connected); portal test `the_tray_follows_the_notification_area_switch_live` passes; 03-13 service run logged `module off`. |
| 9 | SC4b: with start with the OS on, Sucrose starts after sign-in and resumes the wallpaper | PRESENT_BEHAVIOR_UNVERIFIED | Run value / LaunchAgent / XDG entry code and text tests pass (18 platform tests); `start_actions` is used at `service.rs:1003` so a selected theme starts at launch; sign-in itself was never exercised (human check 03-23, registry tests ignored). |
| 10 | SC5a: the Portal follows light/dark/OS theme with Mica/Acrylic backdrop and optional background image, has header actions, search, status bar, Help/About | VERIFIED | `theme.rs`, `background.rs`, `header.rs`, `status_bar.rs`, `dialogs/{help,about,tray_hint}.rs` exist, are substantive and wired through `app.rs`; executor captures and unit tests per 03-17, 03-18; visual confirmation remains in Human Verification. |
| 11 | SC5b: scrolling a large library keeps memory bounded, recorded in MEASUREMENTS.md | FAILED | MEASUREMENTS.md rows 14-15: real 90 themes pass; 500 themes FAIL (131 -> 269 MB against 181 MB limit). Runs had no animating previews. UI-07 correctly still Pending. BLOCKER. |
| 12 | Derived: untrusted theme/background images cannot exhaust memory (T-03-25, T-03-40) | FAILED | See gap 3 (CR-01, open): no pixel-area cap before `decode_jpeg_scaled`. BLOCKER. |
| 13 | Derived: the settings store never overwrites a good Sucrose.json after a transient read error | FAILED | See gap 4 (CR-02, open): `store.rs:191-194` read error becomes Corrupt, then backup restore or migrate-and-persist. BLOCKER. |

**Score:** 7/13 truths verified (2 present, behavior-unverified; 4 failed)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/sucrose-config/src/{store,migrate,legacy,model}.rs` | versioned settings, atomic write, migration | VERIFIED (CR-02 flaw) | substantive, tests pass; read-error handling defect |
| `apps/sucrose/src/{service,portal_link,settings_host,library_ops,lifecycle,modules,startup,supervisor}.rs` | service host, Portal protocol, Use, move, lifecycle | VERIFIED | wired, tests pass; WR-01..WR-04 open warnings |
| `apps/sucrose-portal/src/library/{card,mod,grid,layout,pagination,thumbs}.rs` | library UI | VERIFIED | wired; thumbs has CR-01 |
| `apps/sucrose-portal/src/library/preview.rs` | capped GIF preview and state machine | HOLLOW | decodes and tests pass, but the painted result never animates |
| `crates/sucrose-platform/src/{autostart,trash}.rs`, `os_windows/autostart.rs` | OS start entry, recycle bin | VERIFIED (Windows) | macOS/Linux text-tested only |
| `docs/MEASUREMENTS.md` | D-25 rows | VERIFIED as record | records a failing result honestly |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| card click | service Use | `PortalRequest::Use` card.rs:164 -> service.rs:1265 | WIRED | |
| service Use | renderer | `ToRenderer::LoadSource` service.rs:3113 | WIRED | maintainer-observed |
| Portal settings rows | Sucrose.json | `PortalRequest::Change` -> settings_host | WIRED | `a_change_survives_a_restart` |
| autostart row | Run value | `startup::set_startup` -> platform `enable_autostart` | WIRED | |
| hover event | animated preview | `on_hover` -> `hover_card` -> PreviewState -> `img(image)` | PARTIAL | paint step lacks element id, frame 0 only |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| card preview layer | `data.preview` | `decode_preview` multi-frame RenderImage | Yes (frames decoded) | HOLLOW at paint: only frame 0 shown |
| library grid | `themes` | `sucrose-core` library scan | Yes | FLOWING |
| status bar memory | `MemoryReport` | service sampler over IPC | Yes | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Settings migration and store | `cargo test -p sucrose-config` | 29 + 11 pass | PASS |
| Portal protocol, Use, move, single instance | `cargo test -p sucrose --test portal` | 26 pass, 1 ignored | PASS |
| Renderer restart budget | `cargo test -p sucrose --bin sucrose supervisor` | 17 pass | PASS |
| Module gating | `cargo test -p sucrose --bin sucrose modules` | 9 pass | PASS |
| Preview decode | `cargo test -p sucrose-portal preview` | 17 pass | PASS (does not cover paint) |
| Autostart text and state | `cargo test -p sucrose-platform --test autostart` | 18 pass, 3 ignored (real HKCU) | PASS |
| Library scan | `cargo test -p sucrose-core --test library` | 20 pass, 1 ignored | PASS |

The orchestrator also reported the workspace at 858 passed, 0 failed; I did not re-run the full suite. Green tests do not detect the hover, memory, CR-01 and CR-02 defects.

### Probe Execution

Step 7c: SKIPPED (no probe scripts declared by the phase plans).

### Requirements Coverage

All 21 phase IDs appear in at least one PLAN `requirements:` field and in REQUIREMENTS.md; no orphaned IDs.

| Requirement | Source Plan(s) | Status | Evidence |
|-------------|----------------|--------|----------|
| ARCH-01 | 03-09 | SATISFIED | supervisor bounded restart, per-monitor renderers, tests |
| ARCH-02 | 03-06, 03-08, 03-15 | SATISFIED | one endpoint, AlreadyOpen, PortalRunning tests |
| ARCH-03 | 03-13 | SATISFIED | modules tests, `module off` log |
| ARCH-04 | 03-01, 03-07 | PARTIAL | versioned atomic store and kill test pass; CR-02 can overwrite a good file after a transient read error |
| ARCH-05 | 03-07, 03-09, 03-11, 03-12, 03-20 | SATISFIED | Use through service, change survives restart; maintainer-observed |
| FMT-01 | 03-01, 03-04, 03-26 | SATISFIED | nine real-file fixtures, exclusions proven |
| FMT-06 | 03-01, 03-04 | SATISFIED | legacy layout kept, 25-char names (library/paths tests) |
| TRAY-03 | 03-23 | NEEDS HUMAN | Windows Run value implemented and text-tested; sign-in unverified; macOS/Linux unexercised |
| TRAY-05 | 03-16 | NEEDS HUMAN | state machine tested, real process end not exercised |
| LIB-01 | 03-04, 03-21 | SATISFIED | showcase copy once and recorded (portal test) |
| LIB-02 | 03-02, 03-21 | SATISFIED | scan rules, recycle bin (legacy deleted; moved to the bin) |
| LIB-03 | 03-02, 03-14 | SATISFIED | sort and search logic tests; typing is a human check |
| LIB-04 | 03-10, 03-14, 03-20 | SATISFIED | layout and pagination tests |
| LIB-06 | 03-19 | BLOCKED | previews do not animate; REQUIREMENTS.md marks it Complete, which is wrong |
| LIB-08 | 03-02, 03-09, 03-12 | SATISFIED | newer-version refusal and badge, tests |
| LIB-09 | 03-22 | SATISFIED | move with rollback, nested refusal, portal test; picker is a human check |
| UI-01 | 03-17, 03-20 | SATISFIED (visual human) | backdrop by capability, background image; CR-01 touches the image decode |
| UI-02 | 03-17, 03-20 | SATISFIED (visual human) | theme mode and OS following |
| UI-03 | 03-03, 03-08, 03-14, 03-18 | SATISFIED | header, search, status bar, arguments |
| UI-05 | 03-03, 03-18 | SATISFIED (visual human) | Help, About, tray hint dialogs exist |
| UI-07 | 03-05, 03-10, 03-19, 03-24, 03-25 | BLOCKED | 500-theme paging fails the rule; REQUIREMENTS.md correctly Pending |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (apps, crates, xtask) | - | TBD/FIXME/XXX/TODO/todo!/unimplemented! | none | grep returned nothing; one `allow(dead_code)` remains (IN-04) |
| `apps/sucrose-portal/src/library/card.rs` | 282 | `img(image)` without element id | BLOCKER | multi-frame preview stays on frame 0 |
| `crates/sucrose-config/src/store.rs` | 191-194 | read error mapped to Corrupt | BLOCKER | CR-02 |
| `apps/sucrose-portal/src/library/thumbs.rs` | 250-283 | no pixel-area cap before JPEG fast path | BLOCKER | CR-01 |

The review's eight warnings (WR-01 to WR-08) and twelve info items are all open in 03-REVIEW-DISPOSITION.md; none was individually re-confirmed here and they are not counted as gaps, but WR-02 and WR-05 touch truths 6 and 9.

### Human Verification Required

See frontmatter `human_verification` and `behavior_unverified_items` (the end-of-phase checks harvested from plans 03-08 to 03-23 and the two unverified truths). Plan 03-25's Verification scope matches this list.

### Gaps Summary

Four gaps block the goal, grouped by root cause:

1. **Hover previews do not animate** (LIB-06, SC1). Highest-confidence cause: the preview `img()` has no element id, so GPUI never advances or repaints it. Fixing it is a one-line change but must be confirmed by hand; the decode tests cannot see it.
2. **Paging memory fails on the 500-theme library** (UI-07, SC5). The measurement was never repeated with working previews, so the result mixes the unknown preview cost with an unexplained growth that sits outside the 64 image / 48 MiB cache. A gap-closure plan must find the source, fix it and rerun D-25 scenarios 1 and 2.
3. **CR-01 progressive JPEG decode bomb** in thumbnail and background decoding.
4. **CR-02 settings store overwrites a good file after a transient read error**, which undermines the "settings persist" promise.

Everything else in the five success criteria is present, wired and test-backed, apart from the end-of-phase human checks and the two present-but-unexercised behaviors (Close Applications end to end, start with the OS after sign-in). Remaining open warnings (WR-01 to WR-08) should be triaged in the same gap-closure round; REQUIREMENTS.md should have LIB-06 reopened. None of these gaps is addressed by a later phase (Phase 5 reuses the cache but does not own the bound), so nothing is deferred.

---

_Verified: 2026-11-24T13:05:00Z_
_Verifier: Claude (gsd-verifier)_
