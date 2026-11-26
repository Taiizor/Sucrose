---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 03
subsystem: formats
tags: [serde_json, json, theme-formats, SucroseProperties, SucroseCompatible, fixtures]

requires:
  - phase: 02-m1-library-and-settings
    provides: "theme.rs read_info/write_info patterns (ThemeError, CRLF two-space writing)"
provides:
  - "sucrose_core::properties: SucroseProperties, Control, ControlKind (ten types + Unknown), read/write/parse, Control::to_json_value"
  - "sucrose_core::compatible: SucroseCompatible, CompatibleKey (16, table order), DataGroup, templates(), data_groups(), read/write/parse"
  - "7 real, 13 showcase and hand-made fixtures for both formats"
affects: [04-09 property calls and compatible API, 04-10 data hub group list]

actuals:
  tokens: 165000   # chars/4 over the realized diff; about 15500 of it is code and tests, the rest are the copied real and showcase fixtures
  tasks: 2
  commits: 2
plan_head_before: d9620024faba861df18c0e7208009c1e27884131
plan_head_after: f58be6cc2fabf3830b6f56053fa7cd112c54facc

tech-stack:
  added: []
  patterns:
    - "Ordered JSON objects through a map visitor into Vec<(String, T)> (serde_json has no preserve_order here)"
    - "Newtonsoft-style leniency: any key letter case, numbers as integer or double, missing fields take their zero value"
    - "Error reasons name the kind of problem and where, never quote the file"

key-files:
  created:
    - crates/sucrose-core/src/properties.rs
    - crates/sucrose-core/src/compatible.rs
    - crates/sucrose-core/tests/properties.rs
    - crates/sucrose-core/tests/compatible.rs
    - crates/sucrose-core/tests/fixtures/properties/
    - crates/sucrose-core/tests/fixtures/compatible/
  modified:
    - crates/sucrose-core/src/lib.rs
    - docs/FEATURES.md

key-decisions:
  - "Control.text and help are Option<String> so a label without text stays without text on write; Control::to_json_value gives the legacy page form (text empty, help null)"
  - "Dropdown valuetext is computed from items and value on read, write and page calls, as the legacy model does; the file's own valuetext is ignored"
  - "An unknown control type keeps the whole original object (ControlKind::Unknown.raw) and is written back verbatim; unknown fields of known types live in Control.extra"
  - "A control without a type, or one that is not an object, is an error value with a reason (legacy threw as well); other missing fields take their zero value"
  - "SucroseCompatible stores entries in file order; known keys match in any letter case and are written under the legacy spelling; a missing key stays missing on write"
  - "compatible reuses PropertiesError (alias CompatibleError) instead of a second error enum"

patterns-established:
  - "Format models keep unknown content (extra maps / raw objects) so an old Sucrose never drops what a newer theme added"

requirements-completed: [FMT-03]

duration: 11 min
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 03: Theme formats of the JavaScript API Summary

**SucroseProperties.json (ten control types, ordered list and localization) and SucroseCompatible.json (16 template keys with data-group mapping) read and written compatibly in sucrose-core, proven with the maintainer's 7+1 real files and the 13+10 legacy showcase files.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-11-26T01:12:19Z
- **Completed:** 2026-11-26T01:23:14Z
- **Tasks:** 2
- **Files modified:** 43 (mostly fixtures)

## Accomplishments

- `properties` module: typed model for all ten legacy control types plus Unknown, property list and localization in file order, unknown top-level keys and control fields kept, CRLF two-space writing like `write_info`, `Control::to_json_value()` for the page calls of plan 09.
- `compatible` module: the 16 keys in the legacy table order, `templates()` (non-empty, table order), `data_groups()` (Bios..Motherboard, what the data hub of 04-10 samples), unknown keys kept in file order.
- Fixtures: 7 real property files and 1 real compatible file from the maintainer's library (copied read-only; checked for user names, absolute paths, e-mail addresses and URLs; none found, no sanitising needed), 13 + 10 showcase files, and hand-made all-types, unknown-type, empty-list, not-json, unknown-key and empty cases.
- Reading never writes: tests check that a not-JSON file keeps its bytes. No theme file is deleted or moved (the legacy app deleted unparsable copies).
- FEATURES formats row for SucroseInfo/Properties/Compatible is now done.

## Task Commits

1. **Task 1: SucroseProperties.json round-trips** - `33fc5028` (feat)
2. **Task 2: SucroseCompatible.json round-trips and names its data groups** - `f58be6cc` (feat)

**Plan metadata:** the docs(04-03) commit that adds this summary.

## Files Created/Modified

- `crates/sucrose-core/src/properties.rs` - properties model, ordered map visitor, control parsing, read/write
- `crates/sucrose-core/src/compatible.rs` - compatible model, key table, data groups, read/write
- `crates/sucrose-core/tests/properties.rs` (11 tests), `tests/compatible.rs` (9 tests) - fixture round trips and edge cases
- `crates/sucrose-core/tests/fixtures/{properties,compatible}/` - real, showcase and hand-made fixtures
- `crates/sucrose-core/src/lib.rs` - `pub mod properties; pub mod compatible;`
- `docs/FEATURES.md` - formats row text and status

## Decisions Made

See key-decisions above. None of them changes a legacy behavior that a page can observe: the page form of a control (`to_json_value`) follows the legacy serialization (empty text, null help, computed valuetext, no unknown fields).

## Deviations from Plan

### TDD ordering

The plan marks both tasks `tdd="true"`, but `workflow.tdd_mode` is off and the executor wrote the module and its tests together; each task is one `feat` commit with tests included, there is no separate failing-test commit and no RED evidence record. The tests were not run against a missing implementation first, so a passing first run proves less than a RED/GREEN pair would. The fixture round trips were checked as real behavior (not-JSON, unknown type, key order in a non-alphabetical fixture).

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy manual_range_contains in integer_field**
- **Found during:** Task 1 verify
- **Fix:** `(-9.0e15..=9.0e15).contains(&number)`
- **Files modified:** crates/sucrose-core/src/properties.rs
- **Committed in:** 33fc5028

---

**Total deviations:** 1 auto-fixed (lint), 1 process note (TDD ordering). **Impact:** none on behavior.

## Issues Encountered

None. The real library has exactly the counts the plan expected (7 property files, 1 compatible file); the showcase has 13 and 10.

## Threat Surface

T-04-08: parsing returns errors, no panics (tests with malformed controls, empty input, wrong top-level type); the ordered visitor uses a hash index so a huge object is not quadratic. T-04-09: reading only opens files; tests prove a rejected file is unchanged. T-04-10: real fixtures checked, nothing personal found. T-04-SC: no crate added.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 09 can build the page calls on `SucroseProperties`, `Control::to_json_value` and `SucroseCompatible::templates`; plan 10 can read `data_groups()`. `cargo xtask ci` passes. FMT-03 is marked complete.

## Self-Check: PASSED

- Files exist: properties.rs, compatible.rs, both test files, both fixture folders (7 real + 13 showcase property fixtures, 1 real + 10 showcase compatible fixtures).
- Commits 33fc5028 and f58be6cc are on the branch; `cargo test -p sucrose-core` (properties 11, compatible 9), clippy -D warnings and `cargo xtask ci` pass; docs/FEATURES.md is still CRLF with no newline after its last line.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-26*
