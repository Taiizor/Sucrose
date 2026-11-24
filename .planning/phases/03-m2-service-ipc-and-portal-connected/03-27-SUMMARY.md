---
phase: 03-m2-service-ipc-and-portal-connected
plan: 27
subsystem: ui
tags: [gpui, sprite-atlas, memory, paging, ui-07, gap-closure]

requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: library thumbnail cache, hover preview and background image of plans 03-24 and 03-25
provides:
  - "atlas::release_image / atlas::release_images: deferred release of RenderImages from every window's sprite atlas"
  - "GPUI tests that pin why a direct App::drop_image inside a window update misses that window, and that the helper fixes it"
  - "\"page shown\" debug line with thumbs, thumb_bytes and preview_playing for the D-25 reruns"
  - "docs/GPUI.md release rule, evidence and texture-pinning arithmetic"
affects: [03-28, 03-37, 03-38, 03-39, 03-40]

actuals:
  tokens: 5600
  tasks: 2
  commits: 2
plan_head_before: 5f37d54e281b35b795413d13b1c35b714929d497
plan_head_after: 05ac3117905115a69080928c47549ce7dd411279

tech-stack:
  added: ["gpui-pre test-support (dev-dependency of sucrose-portal only)"]
  patterns: ["release every GPUI image through crate::atlas, never a direct cx.drop_image"]

key-files:
  created: [apps/sucrose-portal/src/atlas.rs]
  modified:
    - Cargo.toml
    - Cargo.lock
    - apps/sucrose-portal/Cargo.toml
    - apps/sucrose-portal/src/main.rs
    - apps/sucrose-portal/src/library/mod.rs
    - apps/sucrose-portal/src/background.rs
    - docs/GPUI.md

key-decisions:
  - "Images are released with App::defer so the drop runs after the outermost update, when the updated window is back in App.windows"
  - "GPUI test support comes from a gpui-pre dev-dependency (workspace key gpui), not from gpui-kit's test-support feature, which cannot resolve against the locked wgpu"
  - "The thumbnail and preview caps stay 64 images / 48 MiB and 120 frames / 24 MiB"

patterns-established:
  - "atlas::release_image / release_images are the only way the Portal frees an image from GPUI"

requirements-completed: [UI-07]

coverage:
  - id: D1
    description: "A hover preview released by the pointer-leave handler (inside the window update) leaves the window's atlas"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/atlas.rs#release_image_during_a_window_update_empties_the_atlas_entry"
        status: pass
    human_judgment: false
  - id: D2
    description: "The cause is pinned: App::drop_image with no window inside that window's update leaves the entry in place"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/atlas.rs#gpui_drop_image_during_a_window_update_misses_that_window"
        status: pass
    human_judgment: false
  - id: D3
    description: "Releases are idempotent and work from an async task, from a window update and outside any update"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/atlas.rs#releasing_an_image_twice_is_harmless"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/atlas.rs#release_from_an_async_task_and_from_a_window_update_both_empty_the_atlas"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/atlas.rs#release_image_outside_a_window_update_empties_the_atlas_entry"
        status: pass
    human_judgment: false
  - id: D4
    description: "Evicted thumbnails and replaced background images (shown and still) release through the helper; no direct drop_image remains in library/mod.rs or background.rs"
    requirement: UI-07
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/background.rs#background_release_inside_a_window_update_empties_the_atlas"
        status: pass
    human_judgment: false
  - id: D5
    description: "The share of the 287 MB peak and 269 MB final of the 500-theme paging run that this cause explains"
    requirement: UI-07
    verification: []
    human_judgment: true
    rationale: "The tests prove the mechanism, not its share of the growth; the D-25 reruns (03-37, 03-38) with the attribution of 03-40 measure it and 03-39 judges it"

duration: 14 min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 27: Images the Portal stops showing really leave GPUI Summary

**Every image release of the Portal (hover preview, evicted thumbnail, replaced background image) now goes through `atlas::release_image(s)`, which defers GPUI's `drop_image` past the window update that the old direct call silently missed; GPUI tests pin the cause and the fix.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-11-24T14:40:00Z
- **Completed:** 2026-11-24T14:54:00Z
- **Tasks:** 2
- **Files modified:** 8 (one created)

## Accomplishments

- `apps/sucrose-portal/src/atlas.rs` with `release_image` and `release_images` (`App::defer` then `App::drop_image(image, None)`), module doc citing the GPUI doc comment, and five GPUI tests.
- The hover preview path (`after_preview_change`), thumbnail eviction (`on_decoded`) and `BackgroundImage::drop_decoded` (`shown` and `still`) use the helper; `cat library/mod.rs background.rs | grep -v '^\s*//' | grep -c 'drop_image('` prints 0.
- The "page shown" debug line keeps its message and `page` field and adds `thumbs`, `thumb_bytes` and `preview_playing`.
- `docs/GPUI.md` (CRLF kept) states the release rule, the evidence and the texture arithmetic; the paging-memory bullet names this the leading cause and defers confirmation to the D-25 reruns with 03-40's attribution. The numbers of `docs/MEASUREMENTS.md` are untouched.

## Task Commits

1. **Task 1 (tracer): a hover preview released by the pointer-leave handler leaves the atlas** - `f2420003` (fix)
2. **Task 2: thumbnail evictions and background images release through the helper; page log; GPUI.md** - `05ac3117` (fix)

**Plan metadata:** the docs commit that carries this SUMMARY, STATE.md and ROADMAP.md.

## Tracer feedback gate

The tracer's `<verify>` is automated only and `human_verify_mode` is the default `end-of-phase`: re-ran `cargo test -p sucrose-portal atlas::` (5 passed) after the commit, then expanded.

## Evidence

Tests (`cargo test -p sucrose-portal atlas::` and `background::`), all ok:

- `gpui_drop_image_during_a_window_update_misses_that_window` (control): inside `VisualTestContext::update`, `cx.drop_image(image, None)` leaves `has_image_atlas_entry` true, during and after the update. This is the recorded cause; it fails loudly if a future GPUI changes the rule.
- `release_image_during_a_window_update_empties_the_atlas_entry`, `release_image_outside_a_window_update_empties_the_atlas_entry`, `releasing_an_image_twice_is_harmless` (also through `release_images`), `release_from_an_async_task_and_from_a_window_update_both_empty_the_atlas`, and `background_release_inside_a_window_update_empties_the_atlas`.

Call paths that released inside a window update before this plan (they all missed the Portal window's atlas):

1. Hover leave: the card's `on_hover(false)` listener runs in the window's event dispatch, then `hover_card` and `after_preview_change`.
2. Page change (`set_page`, pagination click) calls `stop_preview`.
3. Search (`set_query`, header input event) calls `stop_preview`.
4. Settings change (`set_settings`, `update_settings`) and `suspend` / `reload`.
5. Background change: `PortalApp::apply_background` runs inside the window update and `BackgroundImage::set` released `shown` and `still`.

Ran from async callbacks, where no window is being updated (so they did reach the windows): thumbnail eviction in `on_decoded`, and the stale-result branch of the background `finish`. They use the helper too now, for one rule.

Arithmetic: a preview frame is 360 x 203 BGRA, about 0.3 MB. A leaked tile keeps its whole 1024 x 1024 BGRA atlas texture (4 MiB) alive, because `gpui-pre-windows` `directx_atlas.rs` frees a texture only when its last tile is removed (confirmed in the registry source, `remove` decrements the ref count and frees at zero). About 100 hovers across 17 pages can pin tens of textures while thumbnails churn through the atlas. The tests prove the mechanism, not its share of the 287 MB peak and 269 MB final: the reruns of 03-37 and 03-38 with the attribution of 03-40 measure that.

## Package legitimacy (T-03-62)

`gsd-tools query package-legitimacy check --ecosystem crates` over every package `Cargo.lock` gained: all 40 verdicts OK (none other):
alloca, anes, atomic, bit-set, bit-vec, borsh, borsh-derive, c-enum, cast, ciborium, ciborium-io, ciborium-ll, clap, clap_builder, clap_lex, convert_case, criterion, criterion-plot, crossbeam-queue, fixedbitset, futures-concurrency, gpui-pre-bench-metrics, oorandom, page_size, perf-event-data, perf-event-open-sys2, perf-event2, plotters, plotters-backend, plotters-svg, pollster, postage, proptest, proptest-macro, quick-error, rand_xorshift, rusty-fork, tinytemplate, unarray, wait-timeout. `Cargo.lock` also rewrites a few dependency lines of existing entries (bit-set, bit-vec, convert_case, pollster, quick-error) to versioned names because a second version of each is now in the lock; no version of an existing package changed. `cargo deny` is not installed here; the CI deny job covers it. The packages are reached only from the `[dev-dependencies]` of `sucrose-portal`.

## Files Created/Modified

- `apps/sucrose-portal/src/atlas.rs` - deferred release helper and its GPUI tests
- `apps/sucrose-portal/src/library/mod.rs` - preview and eviction releases through the helper; richer "page shown" line
- `apps/sucrose-portal/src/background.rs` - `drop_decoded` through the helper; background GPUI test
- `apps/sucrose-portal/src/main.rs` - `mod atlas;`
- `Cargo.toml`, `apps/sucrose-portal/Cargo.toml`, `Cargo.lock` - `gpui-pre` test-support dev-dependency
- `docs/GPUI.md` - release rule, evidence, paging-memory bullet

## Decisions Made

- Deferred release with `App::defer` (runs after the outermost update, every window back in `App.windows`) rather than threading `Some(window)` through each caller: the async and window-update callers share one helper.
- Caps unchanged (64 images / 48 MiB, 120 frames / 24 MiB); the prohibition on moving them to pass a measurement holds.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] gpui-kit `test-support` cannot resolve; used a gpui-pre dev-dependency**
- **Found during:** Task 1
- **Issue:** `gpui-kit = { workspace = true, features = ["test-support"] }` under `[dev-dependencies]` made Cargo resolve gpui-kit's optional `gpui-fast` tree (weak feature references), whose `gpui-fast-wgpu` pins `wgpu =30.0.1` against the locked 30.0.2; the lockfile could not be built, and fixing it would have meant downgrading wgpu and adding the whole unvetted gpui-fast tree.
- **Fix:** `gpui = { package = "gpui-pre", version = "=0.3.12", features = ["test-support"] }` in the root `[workspace.dependencies]` (pinned, the same crate and version gpui-kit already builds on) and `gpui = { workspace = true }` under `[dev-dependencies]` of `sucrose-portal`. Tests use `#[gpui_kit::test]`, `TestAppContext` and `Window::has_image_atlas_entry` as planned. Normal builds are unchanged.
- **Files modified:** Cargo.toml, apps/sucrose-portal/Cargo.toml, Cargo.lock
- **Verification:** `cargo test -p sucrose-portal` (189 passed), `cargo xtask ci` exit 0; all 40 new lock packages checked OK.
- **Commit:** f2420003
- **Note for review:** the plan's acceptance grep for `test-support` in `apps/sucrose-portal/Cargo.toml` is met by the root `Cargo.toml` entry that the Portal inherits; the lockfile gained 40 packages instead of the 3 families the planner listed because gpui-pre's test-support also enables its bench-metrics (criterion, plotters, clap and friends), all dev-only.

**2. [TDD gate] No separate RED commit**
- The plan is one tracer commit per task; the control test (`gpui_drop_image_during_a_window_update_misses_that_window`) is the recorded RED evidence of the cause and passes only because GPUI behaves as pinned. The helper tests were written before the helper and compiled against it in the same commit.

---

**Total deviations:** 2 (1 Rule 3 blocking, 1 TDD-shape note)
**Impact on plan:** None on behavior or scope; the test infrastructure comes from the same GPUI crate by another route.

## Issues Encountered

None beyond deviation 1.

## Known Stubs

None.

## Threat Flags

None.

## Next Phase Readiness

- 03-28 (animated hover previews) can rely on `atlas::release_image`; 03-37 and 03-38 can split rounds on "page shown" and read `thumbs`, `thumb_bytes`, `preview_playing`; 03-40 adds the growth attribution.
- Not verified here: the real 500-theme paging run (no app was started; the share of the growth this explains is for the reruns).

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*

## Self-Check: PASSED
