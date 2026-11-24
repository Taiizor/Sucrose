---
phase: 03-m2-service-ipc-and-portal-connected
plan: 03
subsystem: ui
tags: [rust, rust-i18n, xtask, localization, portal, yaml, xaml]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: Portal crate skeleton (apps/sucrose-portal) and the xtask structure
provides:
  - cargo xtask localize --lang EN [--check] (legacy XAML string resources to rust-i18n YAML with legacy key names)
  - apps/sucrose-portal/locales/legacy.en.yml (generated, 361 keys, 86 quotings) and sucrose.en.yml (41 hand-written Phase 3 keys)
  - Portal rust-i18n setup (i18n! at the crate root) and window title from Portal.MainWindow.Title
  - i18n::{tr, tr_with, tr_count, tr_count_with, REQUIRED_KEYS} with key-coverage tests
  - deny.toml allows bzip2-1.0.6 so the CI deny job can judge this phase's crates
affects: [03-04, 03-09, 03-10, 03-14, 03-21, 03-22, UI-06]
actuals:
  tokens: 14000
  tasks: 2
  commits: 4
plan_head_before: 9952e7231869d118b6f5c9e67f4e30bf03e1b87f
plan_head_after: e214446fc6385ac977b2cd2e11b585db0d9307ef
tech-stack:
  added: [rust-i18n 4.2.4 (workspace dependency, already in the lock file through gpui-component)]
  patterns: [flat dotted YAML keys in two files merged into one locale, generator plus --check for committed generated files, REQUIRED_KEYS coverage list that later plans append to, no key defined in two locale files]
key-files:
  created:
    - xtask/src/localize.rs
    - apps/sucrose-portal/src/i18n.rs
    - apps/sucrose-portal/locales/legacy.en.yml
    - apps/sucrose-portal/locales/sucrose.en.yml
  modified:
    - Cargo.toml
    - Cargo.lock
    - xtask/src/main.rs
    - apps/sucrose-portal/Cargo.toml
    - apps/sucrose-portal/src/main.rs
    - deny.toml
    - docs/GPUI.md
key-decisions:
  - "The two legacy keys Phase 3 rewords for the bin (Portal.PersonalSettingPage.Delete.Description and .DeleteHint) are left out of the generated file (REPLACED_KEYS) and defined in sucrose.en.yml, so no key is defined in two files"
  - "tr_count(key, n) fills only %{count}; tr_count_with adds further placeholders, because the bin name is platform-specific and cfg(target_os) may not live in the Portal"
  - "Counted messages use <key> for the plural and <key>One for exactly one"
requirements-completed: [UI-03, UI-05]
coverage:
  - id: D1
    description: "cargo xtask localize --lang EN regenerates legacy.en.yml from the legacy EN and neutral XAML with the legacy key names; --check fails when the file is stale"
    requirement: UI-03
    verification:
      - kind: unit
        ref: "xtask/src/localize.rs#tests (7 tests: entry order, entities, skipped elements, YAML escaping, key filter, replaced keys)"
        status: pass
      - kind: integration
        ref: "cargo xtask localize --lang EN --check (361 keys, current)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The Portal resolves legacy keys to the legacy text (Portal.Quoting0, all 86 quotings) and takes its window title from Portal.MainWindow.Title"
    requirement: UI-03
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/main.rs#tests::legacy_keys_resolve_to_their_text"
        status: pass
      - kind: unit
        ref: "apps/sucrose-portal/src/i18n.rs#tests::every_quoting_resolves_to_text"
        status: pass
    human_judgment: false
  - id: D3
    description: "Every key of the UI-SPEC String Inventory (legacy and new) resolves in English; placeholders and singular/plural work"
    requirement: UI-05
    verification:
      - kind: unit
        ref: "apps/sucrose-portal/src/i18n.rs#tests (every_required_key_resolves_to_text, every_new_key_is_required, the_two_locale_files_share_no_key, tr_count_picks_the_singular_text_for_one, tr_with_fills_a_named_placeholder)"
        status: pass
    human_judgment: false
  - id: D4
    description: "deny.toml allows bzip2-1.0.6 so the CI deny job can pass for this phase's crates"
    verification: []
    human_judgment: true
    rationale: "cargo-deny is not installed on the maintainer machine; the license entry is only confirmed by the CI deny job"
duration: 11min
completed: 2026-11-24
status: complete
---

# Phase 3 Plan 03: Portal Localization Foundation Summary

**Legacy XAML strings converted to rust-i18n YAML with the legacy key names by `cargo xtask localize`, Phase 3 keys added next to them with coverage tests, and the Portal window title already read through rust-i18n.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-11-24T04:20:48Z
- **Completed:** 2026-11-24T04:31:16Z
- **Tasks:** 2 (tracer + auto, both with a RED then GREEN commit)
- **Files modified:** 11

## Accomplishments

- `cargo xtask localize --lang EN [--check]`: a purpose-built XAML reader (no XML crate) that decodes entities, skips comments, self-closing and other elements, filters keys to the UI-SPEC String Inventory prefixes and writes sorted, double-quoted flat dotted YAML. `--check` exits 1 with "stale: run cargo xtask localize --lang EN" and tolerates a CRLF checkout.
- Assumption A7 is proven: flat dotted keys resolve in rust-i18n 4.2.4, also through a runtime `&str` key.
- `sucrose.en.yml` carries the 41 new keys (library load error, connection banner, corrupt-themes toast with singular and plural, use failed, status bar, theme mode, library move confirmation, startup notes, backdrop tooltips, tray countdown, save failed).
- `i18n.rs` helpers plus `REQUIRED_KEYS` (legacy keys shown in Phase 3 and all new keys); tests fail on a key without text, on a key missing from the list, and on a key defined in both locale files.
- `deny.toml` allows `bzip2-1.0.6` (RESEARCH open question 9).

## Task Commits

1. **Task 1 (tracer): legacy XAML strings reach the Portal window**
   - RED `f618dc53` (test): converter skeleton with stubs and 6 failing tests
   - GREEN `0e986e5c` (feat): converter, generated `legacy.en.yml`, Portal `i18n!` setup and title
2. **Task 2: new keys, coverage test, deny job**
   - RED `fcd179e4` (test): `REQUIRED_KEYS`, helper stubs and 3 failing tests
   - GREEN `e214446f` (feat): helpers, `sucrose.en.yml`, `REPLACED_KEYS`, `deny.toml`, `docs/GPUI.md` note

**Plan metadata:** added by the SUMMARY and state commits that follow this file.

_Tracer gate: the tracer's automated verify (`localize --check`, `cargo test -p xtask localize`, `cargo test -p sucrose-portal`, Quoting85 count) was re-run end to end and passed before Task 2 started._

## Files Created/Modified

- `xtask/src/localize.rs` - XAML to rust-i18n YAML converter with unit tests
- `xtask/src/main.rs` - `localize` task and usage line
- `apps/sucrose-portal/locales/legacy.en.yml` - generated English strings with legacy keys
- `apps/sucrose-portal/locales/sucrose.en.yml` - hand-written Phase 3 English strings
- `apps/sucrose-portal/src/i18n.rs` - `tr`, `tr_with`, `tr_count`, `tr_count_with`, `REQUIRED_KEYS` and tests
- `apps/sucrose-portal/src/main.rs` - `i18n!`, localized window title, key test
- `Cargo.toml`, `Cargo.lock`, `apps/sucrose-portal/Cargo.toml` - `rust-i18n` workspace dependency
- `deny.toml` - `bzip2-1.0.6` license
- `docs/GPUI.md` - rust-i18n gotchas

## Decisions Made

- Reworded legacy keys are removed from the generated file by `REPLACED_KEYS` rather than relying on rust-i18n's file merge order.
- `tr_count_with` exists next to the planned `tr_count` so callers can fill `%{bin}` and other placeholders.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Duplicate keys between the two locale files**
- **Found during:** Task 2
- **Issue:** `Portal.PersonalSettingPage.Delete.Description` and `.DeleteHint` already exist in the legacy resources, so the planned "new keys" file would define them a second time. rust-i18n does not promise which file wins for two files of the same locale.
- **Fix:** `REPLACED_KEYS` in the converter drops those two keys from `legacy.en.yml` (regenerated, 363 to 361 keys); a test asserts that the two files share no key.
- **Files modified:** `xtask/src/localize.rs`, `apps/sucrose-portal/locales/legacy.en.yml`
- **Verification:** `the_two_locale_files_share_no_key`, `keep_key_drops_the_keys_phase_3_rewords`, `localize --check`
- **Committed in:** `e214446f`

**2. [Rule 3 - Blocking] A comment-only locale file loads no strings**
- **Found during:** Task 2 RED
- **Issue:** a `sucrose.en.yml` with only comments made every lookup (including legacy keys) return the key, so the RED run failed for the wrong reason.
- **Fix:** the RED commit carries one real key; the behavior is recorded in `docs/GPUI.md` (project rule: new lessons go there).
- **Files modified:** `apps/sucrose-portal/locales/sucrose.en.yml`, `docs/GPUI.md`
- **Committed in:** `fcd179e4`, `e214446f`

**3. [Rule 2 - Missing critical] `tr_count_with`**
- **Found during:** Task 2
- **Issue:** the corrupt-themes toast needs `%{count}` and `%{bin}`; `tr_count(key, n)` alone leaves `%{bin}` unfilled.
- **Fix:** added `tr_count_with(key, count, args)`; `tr_count` calls it with no extra arguments.
- **Committed in:** `e214446f`

---

**Total deviations:** 3 auto-fixed (1 bug, 1 blocking, 1 missing critical)
**Impact on plan:** No scope creep; `docs/GPUI.md` and `xtask/src/localize.rs` were touched in Task 2 although the plan's file list names neither.

## Issues Encountered

- `cargo-deny` is not installed here, so the `bzip2-1.0.6` entry is checked only by the CI deny job.
- Working-tree line endings: the edited files are CRLF and were restored to CRLF after `cargo fmt` and the edit helper; the two new YAML files are LF (generated file, per plan). Pre-existing uncommitted changes in `apps/sucrose/src/service.rs`, `crates/sucrose-config/src/model.rs` and `crates/sucrose-core/src/lib.rs` were left untouched and uncommitted.

## Known Stubs

None. The helpers are unused by pages until later plans (`#[allow(dead_code)]` with a reason on `mod i18n`); that is intentional, not a stub.

## Threat Flags

None. The converter reads tracked repository files only and writes one generated file (T-03-07); only the single license id was added to `deny.toml` (T-03-08); `rust-i18n` is the audited crate from RESEARCH, pinned once in the workspace (T-03-SC).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Later Portal plans only reference keys; a new key goes into `sucrose.en.yml` and `REQUIRED_KEYS`.
- UI-06 (Phase 5) generates the other 24 languages with the same task (`--lang` currently accepts EN only).
- The native tray menu labels stay English constants in the service until UI-06.

---
*Phase: 03-m2-service-ipc-and-portal-connected*
*Completed: 2026-11-24*

## Self-Check: PASSED

Created files exist (`xtask/src/localize.rs`, `apps/sucrose-portal/src/i18n.rs`, both locale files); commits `f618dc53`, `0e986e5c`, `fcd179e4`, `e214446f` are ancestors of HEAD; `cargo xtask ci` passed.
