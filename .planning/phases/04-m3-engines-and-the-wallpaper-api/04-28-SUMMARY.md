---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 28
subsystem: sysinfo
tags: [audio, wasapi, loopback, realfft, visualizer, data-hub, web-api]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-10/04-11 data hub with the Release hook, 04-24 SucroseCompatible templates on both web engines"
provides:
  - "sucrose_sysinfo::visualizer: the legacy AudioVisualizer algorithm as a pure module (Visualizer, Bands, BAND_COUNT)"
  - "sucrose_sysinfo::audio: WASAPI loopback capture of the default render device (AudioCapture) and the GetAudioInfo object (audio_unavailable)"
  - "Audio arm of the hub sampler: capture opened by the first read, dropped by the Release hook, logs audio capture started / stopped"
  - "Fixture web theme audio (SystemAudio template, 128-band canvas, reports each call)"
affects: [04-29, 04-37, WAPI-05]

actuals:
  tokens: 18000
  tasks: 2
  commits: 2

tech-stack:
  added: ["realfft 3.5.1 (workspace dependency, audited OK in 04-RESEARCH)", "tracing in sucrose-sysinfo (existing workspace dependency)"]
  patterns:
    - "Pure DSP module with audio-time throttling (one result per 16 ms of pushed audio), so the algorithm is tested without a device"
    - "Capture worker thread that owns COM and the device, stopped and joined on Drop; the hub holds it only while the group is wanted"

key-files:
  created:
    - crates/sucrose-sysinfo/src/visualizer.rs
    - crates/sucrose-sysinfo/src/audio.rs
    - crates/sucrose-sysinfo/tests/visualizer.rs
    - apps/sucrose/tests/fixtures/web-themes/audio/SucroseInfo.json
    - apps/sucrose/tests/fixtures/web-themes/audio/SucroseCompatible.json
    - apps/sucrose/tests/fixtures/web-themes/audio/index.html
  modified:
    - Cargo.toml
    - Cargo.lock
    - crates/sucrose-sysinfo/Cargo.toml
    - crates/sucrose-sysinfo/src/lib.rs
    - apps/sucrose/src/data_hub.rs
    - docs/FEATURES.md

key-decisions:
  - "A change of the default render device is noticed by comparing the device id once a second, not through IMMNotificationClient: the #[implement] macro needs a direct windows-core dependency, which is only transitive here and has no package audit (T-04-SC)"
  - "A loopback of a device that plays nothing delivers no packets, so the capture feeds the visualizer silence for the time that passes; the bars fall to 0 by the release envelope instead of freezing, and an envelope below 0.0001 on silence is set to 0 (recorded in FEATURES.md as a behavior change)"
  - "A running but silent capture publishes 128 zeros from the start; the legacy default of empty arrays stays for a missing device"

requirements-completed: []

plan_head_before: 2bf5b6f1e571019e4c399f5bbf713e80da6ef21e
plan_head_after: 9a277d2f57f8ea636e69aec99087c7a48143f0a1

coverage:
  - id: D1
    description: "The visualizer reproduces the legacy algorithm: 1 kHz tone in the right band, silence gives zeros, left-only leaves right at 0 and mono at half the peak, one result per 16 ms, neighbour average at the edges, release factor 0.06 per result down to 0"
    requirement: WAPI-05
    verification:
      - kind: unit
        ref: "crates/sucrose-sysinfo/src/visualizer.rs#tests (9 tests)"
        status: pass
      - kind: unit
        ref: "crates/sucrose-sysinfo/tests/visualizer.rs#silence_after_a_loud_signal_lowers_every_band_by_the_release_factor_per_result_until_it_reaches_zero"
        status: pass
      - kind: unit
        ref: "crates/sucrose-sysinfo/tests/visualizer.rs#a_chirp_moves_the_peak_across_the_bands"
        status: pass
    human_judgment: false
  - id: D2
    description: "The Audio object has the GetAudioInfo keys in order, Data/DataLeft/DataRight as 128 numbers in [0, 1], and the legacy default (State false, empty arrays) without a device"
    requirement: WAPI-05
    verification:
      - kind: unit
        ref: "crates/sucrose-sysinfo/src/audio.rs#the_object_has_the_get_audio_info_keys_in_that_order"
        status: pass
      - kind: unit
        ref: "crates/sucrose-sysinfo/src/audio.rs#with_no_device_the_group_is_the_legacy_default_with_state_false_and_empty_arrays"
        status: pass
    human_judgment: false
  - id: D3
    description: "The loopback capture exists only while a running page wants Audio: started by the first read, released with the last page, a failed start is forgotten by the release"
    requirement: WAPI-05
    verification:
      - kind: unit
        ref: "apps/sucrose/src/data_hub.rs#audio_is_released_when_its_last_page_stops_wanting_it_and_the_other_groups_go_on"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/data_hub.rs#a_page_that_wants_audio_again_gets_it_read_afresh_after_the_release"
        status: pass
      - kind: integration
        ref: "audio fixture on CEF: 1 data wanted line with Audio, 1 audio capture started line (48000 Hz, 2 channels, float32), 177 SystemAudio reports in about 16 s"
        status: pass
    human_judgment: false
  - id: D4
    description: "The bars of the audio fixture follow real music on CEF and WebView2"
    requirement: WAPI-05
    verification: []
    human_judgment: true
    rationale: "No audio was playing during the automated run and WebView2 was not run in this plan; the maintainer's music check is the final checkpoint 04-37"

duration: 22min
completed: 2026-11-27
status: complete
---

# Phase 4 Plan 28: The audio spectrum for web wallpapers Summary

**Legacy AudioVisualizer ported as a pure realfft module and fed by a WASAPI loopback capture of the default render device that the data hub holds only while a running page defines SystemAudio.**

## Performance

- **Duration:** 22 min
- **Started:** 2026-11-27T00:24:42Z
- **Completed:** 2026-11-27T00:46:30Z
- **Tasks:** 2
- **Files modified:** 12 (6 created, 6 modified)

## Accomplishments

- `sucrose_sysinfo::visualizer`: 4096-sample ring per channel, Hann window, realfft without scaling, 128 log-spaced bands from 20 Hz to min(16 kHz, Nyquist) with the peak per band, +2 dB/octave tilt, x2.5 over the full-scale tone magnitude minus 0.01, clamp, attack 0.35 / release 0.06, +/-1 neighbour average, mono as |(L+R)/2|, at most one result per 16 ms of audio. Every legacy constant is a named constant.
- `sucrose_sysinfo::audio`: loopback capture of the default render device (shared mode, float32 / PCM16 / PCM32) on its own COM thread, joined on drop; the Audio object in GetAudioInfo key order with Data = mono, DataLeft, DataRight; every media field at its legacy default until 04-29.
- Data hub: the Audio arm opens the capture on the first read (`audio capture started`, with the device format) and the Release hook drops it (`audio capture stopped`); a failed start gives the legacy default and is retried after 60 s.
- Real run on CEF with the audio fixture: Audio wanted once, capture started once (48000 Hz, 2 channels, float32), 177 SystemAudio reports in about 16 s (the 10 a second cadence).

## Task Commits

1. **Task 1: audio fixture bars follow the system sound (tracer)** - `ec546c73` (feat)
2. **Task 2: payload and release pinned by tests** - `9a277d2f` (test)

Plan metadata: the docs commit that follows this file.

_Note: the visualizer, payload and hub tests were written together with the code in Task 1 (the hub tests sit in data_hub.rs, so they landed in the Task 1 commit); Task 2 added the integration tests of the envelope and the FEATURES row. There is no separate RED commit._

## Files Created/Modified

- `crates/sucrose-sysinfo/src/visualizer.rs` - the pure algorithm and its unit tests
- `crates/sucrose-sysinfo/src/audio.rs` - sample conversion, the Audio object, WASAPI loopback worker (unsafe with SAFETY comments)
- `crates/sucrose-sysinfo/tests/visualizer.rs` - envelope decay, attack, chirp, rising tones
- `crates/sucrose-sysinfo/src/lib.rs`, `Cargo.toml` (crate), root `Cargo.toml`, `Cargo.lock` - module wiring, realfft, tracing, windows features Win32_Media_Audio / Win32_System_Com / Win32_System_Com_StructuredStorage
- `apps/sucrose/src/data_hub.rs` - Audio arm, release, tests
- `apps/sucrose/tests/fixtures/web-themes/audio/*` - the fixture theme
- `docs/FEATURES.md` - Audio spectrum row now in progress with the recorded behavior changes

## Decisions Made

See `key-decisions`. All three are recorded in the FEATURES row; none is a DECISIONS.md entry (no Accepted decision was reopened).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Device change noticed by polling the device id, not IMMNotificationClient**
- **Found during:** Task 1 (first build)
- **Issue:** `#[implement(IMMNotificationClient)]` expands to `windows_core::...` paths and needs a direct `windows-core` dependency; that crate is only transitive in this workspace and T-04-SC requires a fresh legitimacy check and a blocking human checkpoint for such a dependency.
- **Fix:** the capture worker compares the default render device id once a second and re-creates the capture when it differs (also when a capture call fails); the plan's truth "a default-device change re-creates the capture" holds.
- **Files modified:** crates/sucrose-sysinfo/src/audio.rs
- **Verification:** builds, clippy clean; the re-creation path is not exercised by an automated test (needs a second audio device).
- **Committed in:** ec546c73

**2. [Rule 1 - Bug] Bars froze instead of falling to 0 when the device delivers nothing**
- **Found during:** Task 1 (design check against the plan's truth "with nothing playing the bands fall to 0 by the release envelope")
- **Issue:** a loopback of a device that plays nothing delivers no packets, and a visualizer fed only real packets (as the legacy one) keeps its last values.
- **Fix:** the worker feeds silence for the elapsed time when no packet arrived for 40 ms and the envelope is not yet 0; an envelope under 0.0001 on silence is set to 0 (so the capture stops feeding); a running capture starts with 128 zeros instead of empty arrays.
- **Files modified:** crates/sucrose-sysinfo/src/audio.rs, src/visualizer.rs
- **Verification:** decay test (release factor 0.06 per result down to 0); recorded as behavior changes in docs/FEATURES.md.
- **Committed in:** ec546c73

**3. [Rule 1 - Bug] Existing hub test expected Audio to have no provider**
- **Found during:** Task 1
- **Issue:** `the_system_sampler_gives_the_date_and_nothing_for_groups_without_a_provider` used Audio as its example of a group with no provider.
- **Fix:** it now uses Battery, which still has none.
- **Files modified:** apps/sucrose/src/data_hub.rs
- **Committed in:** ec546c73

**4. [Rule 3 - Blocking] The plan's verify command counted 0 SystemAudio reports on CEF**
- **Found during:** Task 1 (run)
- **Issue:** the CEF "origin request rejected" lines come from `sucrose_engine_web::serve` at debug level, which `SUCROSE_LOG="info,sucrose_engine_cef=debug"` does not enable (the same finding as 04-24).
- **Fix:** the run used `SUCROSE_LOG="info,sucrose_engine_cef=debug,sucrose_engine_web=debug"`: 177 reports.
- **Files modified:** none

---

**Total deviations:** 4 (2 Rule 1, 2 Rule 3). **Impact:** deviations 1 and 2 change how, not whether, the plan's truths hold; both are recorded in FEATURES.md. No scope creep.

## Issues Encountered

- CEF logged GPU process exits (`exit_code=-2147483645`) during the run; the page loaded (status 200) and reported every call, so it did not affect the check. Not investigated (outside this plan).
- Line endings: `cargo fmt` rewrote Cargo.toml, data_hub.rs, the sysinfo Cargo.toml, lib.rs and FEATURES.md (all w/crlf before) to LF working copies; CRLF was restored on exactly those five before each commit (`git ls-files --eol` shows w/crlf). New files are LF, Cargo.lock stays LF. crates/sucrose-engine-web/src/args.rs and tests/args.rs still show as modified (the 04-25 line-ending artifact); not touched.

## Known Stubs

None that block the goal. The media-session fields of the Audio object (Title, Artist, State, ...) are intentionally at their legacy defaults; 04-29 fills them.

## Threat Flags

None. T-04-62 (samples stay in the capture thread's ring; only 128-band magnitudes are published, nothing is logged or written) and T-04-63 (capture exists only while wanted, joins on drop, 60 s retry) are implemented as planned. The device format is logged, never audio content.

## User Setup Required

None.

## Next Phase Readiness

- 04-29 can extend `payload()` in audio.rs with the now-playing fields; the capture and hub plumbing is in place.
- WAPI-05 stays Pending until 04-37: the WebView2 run and a check with real music are not done here (no audio was playing, WebView2 not run).
- Not checked: bars with real sound, WebView2, a default-device switch while running.

## Self-Check: PASSED

- Created files exist (visualizer.rs, audio.rs, tests/visualizer.rs, three fixture files).
- Commits `ec546c73` and `9a277d2f` are on the branch; `cargo xtask ci` exit 0; `cargo test -p sucrose-sysinfo` 29 + 8 + 4 passed, `cargo test -p sucrose data_hub` 22 passed; clippy -D warnings clean.
- Acceptance criteria: BAND_COUNT, realfft and audio capture started greps are at least 1; Cargo.toml, data_hub.rs and FEATURES.md are w/crlf.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-27*
