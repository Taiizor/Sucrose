---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 21
subsystem: measurement
tags: [cef, cefsharp, measure, xtask, memory, cpu, maintainer-checkpoint]
requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-20: the release build and the scratch Round A run folder (target/round-a) with the maintainer's Web themes"
provides:
  - "cargo xtask measure counts a CEF wallpaper's whole process tree on both sides (roles cef, cefsharp, cefsharp-subprocess)"
  - "The recorded M3 Web wallpaper on CEF row (Rust only) and its reading paragraph in docs/MEASUREMENTS.md, with the C# CefSharp numbers of the same theme"
affects: [verify-work, debug, round-b, gap-closure]
plan_head_before: ca757d7eba08e819a37169d8c5148a9756fd9b95
plan_head_after: 2187bcd0ab931590b8de8b6af58ee354099b1ee3
actuals:
  tokens: 3400
  tasks: 3
  commits: 2
tech-stack:
  added: []
  patterns:
    - "Rust role cef for sucrose.exe with a --type= argument; C# roles cefsharp and cefsharp-subprocess; the --row adapter check accepts the GPU-using CEF processes of a CEF run"
key-files:
  created: []
  modified:
    - xtask/src/measure.rs
    - docs/MEASUREMENTS.md
key-decisions:
  - "The M3 CEF row holds the Rust numbers alone because --row refused the pair (no Rust process has a GPU adapter, G-02-6); the C# CefSharp numbers are in the paragraph, as the final M1 row did"
  - "Outcome rule applied: Rust RAM lower, Rust CPU higher is recorded as found and flagged for /gsd-verify-work; nothing was tuned"
requirements-completed: []
coverage:
  - id: D1
    description: "cargo xtask measure counts the CEF process trees (Rust cef role, C# cefsharp and cefsharp-subprocess roles) and prints per-role totals in the row"
    requirement: ENG-03
    verification:
      - kind: unit
        ref: "cargo test -p xtask measure (46 passed; row_prints_the_cef_totals_per_role_on_both_sides, subtotal_label_names_the_engine_processes_present)"
        status: pass
    human_judgment: false
  - id: D2
    description: "The M3 Web wallpaper on CEF row and reading paragraph are in docs/MEASUREMENTS.md with the C# CefSharp numbers and the outcome rule"
    requirement: ENG-03
    verification: []
    human_judgment: true
    rationale: "A single run by hand on the maintainer's machine; whether the Rust CPU result is acceptable is a judgment for /gsd-verify-work"
duration: "about 30 min of agent time in this continuation, plus the maintainer's runs (sampling 22:28-22:59 +03)"
completed: 2026-11-26
status: complete
---

# Phase 4 Plan 21: CEF memory and CPU measurement Summary

**The measure tool now sees a CEF wallpaper's whole process tree, and the Farm Weather Cycles web theme was measured on Rust CEF (661 MB working set, 337 MB private, 8.80 % CPU) against C# CefSharp (1578 MB, 1251 MB, 1.13 % CPU): RAM is lower on Rust, CPU is higher, recorded as found and not tuned.**

## Performance

- **Duration:** about 30 min of agent time in this continuation, plus the maintainer's runs
- **Completed:** 2026-11-26
- **Tasks:** 3 (Task 1 committed, Task 2 the ready gate, Task 3 the measured runs and the row)
- **Files modified:** 2 (xtask/src/measure.rs, docs/MEASUREMENTS.md)

## Accomplishments

- Task 1: the measure tool classifies sucrose.exe with a --type= argument as role cef, Sucrose.Live.CefSharp as cefsharp and CefSharp.BrowserSubprocess as cefsharp-subprocess; the C# subtotal sums the engine processes present plus Backgroundog; the row prints per-role totals for CEF runs; the --row adapter check accepts the GPU-using CEF processes of a CEF run. Proven by unit tests and a short Rust smoke pass (commit 794b6c1d).
- Task 3: one Web theme (Farm Weather Cycles, folder ynGju7Rw1cDhNQdT2Y6lfcE5B) sampled on both engines, 60 s warm-up and 300 s sample each, on one active display; the row and the reading paragraph are in docs/MEASUREMENTS.md.

## The recorded row (docs/MEASUREMENTS.md)

`--row` refused the pair: "no Rust renderer/cef process has a GPU adapter, so its GPU adapter is unknown; the comparison is not valid, measure again" (G-02-6). As the final M1 row did, the row holds the Rust numbers alone and the scenario wording was adjusted so it stays true (prefix kept):

| Date | Scenario | C# app | Rust |
|---|---|---|---|
| 2026-11-26 | M3 Web wallpaper on CEF, 1 monitor, Rust only (the same-GPU pairing was refused) | – | service + renderer + 5 CEF processes: RAM 661 MB (peak 766), private 337 MB (peak 451), CPU 8.80% (peak 33.5%), GPU 0% (no process has a GPU adapter); the service alone 19.5 MB RAM, 6.3 MB private |

The notes cell names the theme, the release commit ca757d7e, xtask 794b6c1d, Windows 10.0.26300.9974, the hybrid GPU laptop, one active display, warm-up and sample, T0 and the refusal. The C# CefSharp numbers of the same theme are in the paragraph "**M3 Web wallpaper on CEF, reading the numbers.**":

| Side | Working set | Private | CPU | GPU |
|---|---|---|---|---|
| C# CefSharp (Launcher, Backgroundog, Sucrose.Live.CefSharp, 5 CefSharp.BrowserSubprocess; Reportdog ran and is not counted) | 1578 MB (peak 1612) | 1251 MB | 1.13 % (peak 3.91 %) | 59.9 % on one CefSharp subprocess, RTX 3050, 394 MB dedicated / 294 MB shared |
| Rust (service, renderer, 5 CEF processes) | 661 MB (peak 766) | 337 MB (peak 451) | 8.80 % (peak 33.5 %) | none attributed (no adapter) |

Rust service alone: 19.5 MB working set, 6.3 MB private (no libcef, D-32/D-036); renderer 152 MB; largest CEF process 314 MB working set with 8.49 of the 8.80 % CPU. dwm.exe: 1.7 % (Rust run), 4.8 % (C# run), both on the Intel UHD.

Outcome rule: Rust RAM lower and Rust CPU higher is recorded as found and flagged for /gsd-verify-work, not tuned. The likely cause (not proven): the known CEF GPU process crash (CefRenderer-0.log, three times at the start, exit_code=-2147483645), after which Chromium draws in software, while CefSharp draws on the NVIDIA GPU.

## T0 and runs (local time +03, 2026-11-26)

| Run | Side | T0 | Status |
|---|---|---|---|
| 1 | Rust | 22:28:31 | DISCARDED: the C# app played the same theme at the same time, and the Rust theme had no API key (empty weatherApiKey, the page logged "City and API Key are required" and never fetched the weather). Numbers (855 MB working set, 26 % CPU) not used. |
| 2 | C# | 22:39:50 | used |
| 3 | Rust | 22:53:03 | used (file target/measure/cef-rust-2.json; target/measure/cef-rust.json is run 1) |

Before run 3 every C# and Rust Sucrose process was closed (tasklist.exe). The maintainer's OpenWeather API key and the city New York were copied from his C# property cache byte-identically over the scratch theme's SucroseProperties.json (scratch copy only; the real library H:\Sucrose was not touched) and the scratch file was restored afterwards (cmp-verified). The key is not recorded in any repository file.

## The maintainer's replies (verbatim, translations in brackets)

- Ready gate (Task 2): "start"
- Step 2 (C# ready): "ready" and "not: zaten rust hali oynarken de açıktı" [note: it was already open while the Rust version was playing]
- Asked what was open during the Rust run: "rust ölçümünde aynı tema oynuyordu işte. ben açtım ve sana start dedim." [the same theme was playing in the C# app during the Rust measurement; I opened it and then told you start] and "kendi temam derken? video seçtim işte. şuanda tüm sucrose uygulamaları kapalı" [by "my own theme"? I selected a video theme, you know. All Sucrose apps are closed right now]
- Choice for a clean Rust rerun: "rust again fakat C# halinde api key de giriliydi. rust halinde de girili olmalı eşit şartlarda ölçüm yapılacaksa eğer: <API key> - New York" [rust again, but the API key was entered in the C# version; it must be entered in the Rust version too if the measurement is to be under equal conditions: <API key> - New York]
- DISPLAY9: "evet kapalıydılar. 1 mönitördü" [yes, they were off; it was one monitor] (applies to all three runs)

## Task Commits

1. **Task 1: the measure tool sees a CEF wallpaper's whole tree** - `794b6c1d` (feat)
2. **Task 2: ready gate** - checkpoint, no commit
3. **Task 3: the M3 CEF row and paragraph** - `2187bcd0` (docs)

**Plan metadata:** the commit after this SUMMARY (docs: complete plan)

## Files Created/Modified

- `xtask/src/measure.rs` - CEF process-tree roles for both sides, per-role totals in the row, the CEF-aware adapter check, tests
- `docs/MEASUREMENTS.md` - the M3 Web wallpaper on CEF row and its reading paragraph (CRLF kept: `w/crlf` before and after, 31 of 31 lines with CR)

## Decisions Made

- The row holds Rust alone because the same-adapter evidence is missing on the Rust side; the scenario text no longer says "same GPU" so it stays true. C# numbers sit in the paragraph.
- No tuning and no code change after the run (outcome rule).

## Deviations from Plan

1. **[Task 1 TDD order]** The implementation was written before the tests and there was no failing-first run; the tests were added after and pass (46 in `cargo test -p xtask measure`).
2. **[same_adapter change for CEF runs]** The --row adapter check was changed so a CEF run is judged on its GPU-using processes only (utility and renderer children use no GPU). Without it a CEF tree would always be refused; the change is in 794b6c1d.
3. **[Run 1 discarded and rerun]** Run 1 was invalid (C# playing the same theme at the same time; Rust theme without the API key). It was rerun as run 3 with the C# property values; the plan's cef-rust.json name holds run 1 and was not used for the row.
4. **[Reportdog not counted]** Sucrose.Reportdog.exe ran during the C# sample and the tool does not count it; the C# totals are therefore slightly low.
5. **[Per-user property cache gap]** The Rust web engine reads only the theme folder's SucroseProperties.json, not the per-user Cache\Properties working copy the C# engines use. The run worked around it by copying the C# cache over the scratch theme file. Not changed here (no code or FEATURES change in this plan).
6. **[Scenario wording]** "same GPU" was dropped from the scenario because the pairing was refused (plan step 4 allows this).

**Total deviations:** 6 (none changed product code).

## Issues Encountered

None beyond the deviations above.

## Follow-ups (for /gsd-verify-work, /gsd-debug and a later round)

- CEF GPU process crash: three crashes at the start, then software drawing; add the hybrid-GPU hint for the CEF subprocesses and re-measure. This is the likely cause of the Rust CPU being 7.8 x the C# value.
- "SucrosePropertyListener is not defined" on Critter Crossing, seen in the Task 1 smoke run.
- The property-cache gap (item 5 above): the Rust web engine does not read the per-user Cache\Properties copy.
- A Rust GPU adapter for CEF processes (G-02-6), so a future --row pair can be proven same-GPU.
- The CPU result needs a re-measure after the GPU fix; a single run per side is all this row rests on.

## Known Stubs

None.

## User Setup Required

None.

## Next Phase Readiness

Round A's measurement is recorded. The Rust CEF CPU above C# is flagged, not resolved.

## Self-Check: PASSED

- FOUND: xtask/src/measure.rs, docs/MEASUREMENTS.md
- FOUND commits: 794b6c1d, 2187bcd0
- `grep -c 'M3 Web wallpaper on CEF' docs/MEASUREMENTS.md` = 2; `git ls-files --eol` shows `w/crlf`
- No API key in any repository file
