---
phase: 04-m3-engines-and-the-wallpaper-api
plan: 29
subsystem: sysinfo
tags: [gsmtc, media-session, now-playing, thumbnail, ipc, blob, data-hub, web-api]

requires:
  - phase: 04-m3-engines-and-the-wallpaper-api
    provides: "04-28 AudioCapture, the Audio arm of the hub sampler and the audio fixture; 04-10/04-11 data hub with the Release hook"
provides:
  - "sucrose_sysinfo::media: MediaSession (GSMTC worker), the legacy GetAudioInfo media keys, the legacy Thumbnail.cs crop, media_unavailable"
  - "sucrose_ipc::blob: chunks, Reassembler, BLOB_CHUNK, BLOB_MAX and ToRenderer::Blob (wire index 12, appended)"
  - "Data hub art delivery (Thumbnails: take + deliver) and the renderer merge of the newest art into the Audio group"
  - "Audio fixture shows title, artist, state and art and reports /__audio/state/<PlaybackState>"
affects: [04-37, WAPI-06]

actuals:
  tokens: 27500
  tasks: 2
  commits: 2

tech-stack:
  added: ["image (existing workspace entry) in sucrose-sysinfo", "windows features Foundation, Media, Media_Control, Storage_Streams, Wdk_System_SystemServices in sucrose-sysinfo"]
  patterns:
    - "Media worker thread that owns WinRT in an MTA, signalled by change events and a 2 s timeout, stopped and joined on Drop; held by the hub only while Audio is wanted"
    - "Chunked IPC payload: appended ToRenderer variant, one-blob-at-a-time bounded reassembler that reports a drop once through its return value"
    - "Hub-side delivery state per pass: the art goes to each Audio slot once, late subscribers get it, a full queue retries under a new id"

key-files:
  created:
    - crates/sucrose-sysinfo/src/media.rs
    - crates/sucrose-ipc/src/blob.rs
  modified:
    - Cargo.lock
    - crates/sucrose-sysinfo/Cargo.toml
    - crates/sucrose-sysinfo/src/lib.rs
    - crates/sucrose-sysinfo/src/audio.rs
    - crates/sucrose-ipc/src/lib.rs
    - crates/sucrose-ipc/src/protocol.rs
    - apps/sucrose/src/data_hub.rs
    - apps/sucrose/src/service.rs
    - apps/sucrose/src/render.rs
    - apps/sucrose/tests/fixtures/web-themes/audio/index.html
    - docs/FEATURES.md

key-decisions:
  - "GSMTC change handlers do not read the session themselves: they signal the worker, which re-reads and updates the shared record (WinRT reads block and belong on the worker's MTA thread, not on a thread-pool callback)"
  - "The media worker reports its start only after the first reading, so the first snapshot a page gets already shows what plays (without it the first Data batch carried PlaybackState Unknown)"
  - "Reassembler reports a dropped blob as a return value (Outcome::Dropped, once per blob, later chunks of it are Ignored) and the renderer logs it: sucrose-ipc gets no tracing dependency"
  - "A queue that took only part of a chunk set makes the hub offer the art again under a new blob id; the renderer drops the partial blob (strict seq rule kept)"
  - "Art is read by the worker when the track identity (title, artist, album, source app) changes, asked again for 10 s while a track has no art, and an empty text clears the previous art"
  - "PositionSetFileTime is filled from the timeline's LastUpdatedTime (UTC text), the nearest GSMTC field; PropsValid and LastPlayingFileTime keep their legacy defaults"

requirements-completed: []

plan_head_before: 83ca35b57234253b9f299b286c411d8f1a2e71e4
plan_head_after: 780c176e54576880aad36c5c0a1c0b4e2dc73df1

coverage:
  - id: D1
    description: "The media session fills the legacy Audio media keys (names, types, order, times in milliseconds) and with no session they are the legacy defaults with State false"
    requirement: WAPI-06
    verification:
      - kind: unit
        ref: "crates/sucrose-sysinfo/src/media.rs#tests (the_media_keys_are_those_of_get_audio_info_in_that_order, a_session_fills_the_legacy_keys_with_the_legacy_types_and_the_times_in_milliseconds, with_no_session_the_media_keys_are_the_legacy_defaults_and_state_is_false)"
        status: pass
      - kind: unit
        ref: "crates/sucrose-sysinfo/src/audio.rs#media_keys_replace_their_defaults_in_place_and_never_touch_the_spectrum_or_the_thumbnail"
        status: pass
      - kind: integration
        ref: "audio fixture on CEF with the maintainer's real session playing: one /__audio/state/Playing report, hub logs media session followed"
        status: pass
    human_judgment: false
  - id: D2
    description: "The thumbnail is the legacy ThumbnailString: base64 of a PNG cropped to 233 x 233 at (34, 1) when pixel (0, 0) is transparent and the build is below 22000, whole otherwise"
    requirement: WAPI-06
    verification:
      - kind: unit
        ref: "crates/sucrose-sysinfo/src/media.rs#art_with_a_transparent_corner_is_cropped_to_233_by_233_from_34_1_unless_the_system_is_cobalt"
        status: pass
      - kind: unit
        ref: "crates/sucrose-sysinfo/src/media.rs#art_is_left_whole_on_cobalt_and_when_the_corner_is_opaque"
        status: pass
    human_judgment: false
  - id: D3
    description: "Art bigger than one frame reaches the renderer as bounded chunks, is reassembled with the 2 MiB cap and 5 s limit and lands in the Audio group before the page gets it"
    requirement: WAPI-06
    verification:
      - kind: unit
        ref: "crates/sucrose-ipc/src/blob.rs#tests (13 tests)"
        status: pass
      - kind: unit
        ref: "crates/sucrose-ipc/src/protocol.rs#blob_keeps_its_wire_index_and_survives_the_wire"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/data_hub.rs#tests (a_changed_thumbnail_goes_to_each_audio_renderer_once_before_the_data_batch and 5 more)"
        status: pass
      - kind: unit
        ref: "apps/sucrose/src/render.rs#tests (the_newest_art_goes_into_the_audio_group_in_place_and_nothing_else_changes and 4 more)"
        status: pass
      - kind: integration
        ref: "audio fixture on CEF: real art of 42076 and 1333536 base64 bytes (28 chunks) received by the renderer (album art received lines)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Now-playing data reaches only pages that define SystemAudio and is never logged beyond counts and sizes; the session is released with the last such page"
    requirement: WAPI-06
    verification:
      - kind: unit
        ref: "apps/sucrose/src/data_hub.rs#nothing_is_held_or_delivered_while_no_page_wants_audio"
        status: pass
    human_judgment: true
    rationale: "What the user listens to is a privacy property judged by reading the code and the logs; the plan marks the prohibition as judgment"
  - id: D5
    description: "The title and the art show on a theme while real music plays, on CEF and WebView2"
    requirement: WAPI-06
    verification: []
    human_judgment: true
    rationale: "The maintainer's music check is the final checkpoint 04-37; WebView2 was not run in this plan and the picture was not looked at"

duration: 26min
completed: 2026-11-27
status: complete
---

# Phase 4 Plan 29: Now playing for web wallpapers Summary

**The current Windows media session (GSMTC) fills the legacy media keys of the Audio group, and its album art, cropped by the legacy rule, reaches the page through a new chunked ToRenderer::Blob path (48 KiB chunks, 2 MiB cap, 5 s limit) without raising the 64 KiB frame cap.**

## Performance

- **Duration:** 26 min
- **Started:** 2026-11-27T00:55:00Z
- **Completed:** 2026-11-27T01:21:00Z
- **Tasks:** 2
- **Files modified:** 13 (2 created)

## Accomplements

- `sucrose_sysinfo::media`: a worker thread follows `GlobalSystemMediaTransportControlsSessionManager` (current-session, media-properties, playback-info and timeline events, plus the legacy 2 s re-check) and `fields()` maps the record to the `GetAudioInfo` keys in order (times in ms, enums by name); with no session they are the legacy defaults and `State` is false.
- The thumbnail follows the legacy `Thumbnail.cs` (base64, crop to 233 x 233 at (34, 1) for a transparent corner below build 22000), read only when the track changes.
- `sucrose_ipc::blob` and the appended `ToRenderer::Blob` (wire index 12); the hub delivers art to every renderer that wants Audio before the next Data batch; the renderer merges the newest art into the Audio group.
- The audio fixture shows title, artist, state and art and reports `/__audio/state/<PlaybackState>`.

## Ignored test output (now_playing_reads_the_current_session)

The machine had a real session (Edge, a video playing). The title and artist are replaced by placeholders here because they are the maintainer's listening data; every other value is as printed:

```
now playing session found:
{ "State": true, "Title": "<redacted>", "Artist": "<redacted>", "Subtitle": "", "AlbumTitle": "", "AlbumArtist": "",
  "SourceAppId": "MSEdge", "TrackNumber": 0, "MediaType": "Music", "PlaybackRate": 1.0, "PropsValid": "Capabilities",
  "RepeatMode": "Unknown", "ShuffleEnabled": false, "PlaybackMode": "Music", "AlbumTrackCount": 0,
  "PlaybackState": "Playing", "EndTime": 6819021.0, "Position": 2212006.674, "StartTime": 0.0,
  "LastPlayingFileTime": "0001-01-01T00:00:00", "PositionSetFileTime": "2026-11-27T00:52:28.5814495Z",
  "MaxSeekTime": 6819021.0, "MinSeekTime": 0.0 }
thumbnail text length: Some(42076)
```

Legacy keys that keep their default because GSMTC has no equivalent: `PropsValid` (`Capabilities`) and `LastPlayingFileTime` (`0001-01-01T00:00:00`). `TrackNumber`/`AlbumTrackCount` are 0 when a player gives none (this one gave none), and `RepeatMode` is `Unknown` when the player gives no repeat mode. The enum member names are the GSMTC names; the Playhead enum members could not be compared (no package source on the machine). All of this is in the FEATURES row.

## Task Commits

1. **Task 1: the current media session reaches the audio fixture** - `32f8bb7e` (feat)
2. **Task 2: album art bigger than one frame arrives in chunks** - `780c176e` (feat)

**Plan metadata:** the docs commit that carries this SUMMARY (docs: complete plan)

Tests were written with their code inside each task commit, not as a separate RED commit (the plan is `type: execute`, not a TDD plan).

## Decisions Made

See `key-decisions` above.

## Verification

- `cargo test -p sucrose-sysinfo`: 41 + 8 + 4 passed, 1 ignored (the machine-reading test, run on its own as above).
- Task 1 end-to-end (audio fixture on CEF, scratch data folder, maintainer's real Edge session): `grep -c '__audio/state/'` gave 1 (`PlaybackState` Playing) after the start-after-first-reading change; the first try without it gave 2 (Unknown, then Playing).
- Task 2: `cargo test -p sucrose-ipc blob` 13, `cargo test -p sucrose-ipc protocol`, `cargo test -p sucrose data_hub` 28, `cargo test -p sucrose render` 66 passed; `cargo clippy -p sucrose-ipc -p sucrose -p sucrose-sysinfo --all-targets -- -D warnings` clean; `cargo xtask ci` exit 0.
- Real art end to end: the renderer log showed `album art received` with 42076 and 1333536 base64 bytes (the latter 28 chunks), so the chunk path ran on real data.

## Files Created/Modified

- `crates/sucrose-sysinfo/src/media.rs` - the session worker, the mapper, the crop and base64 (new)
- `crates/sucrose-ipc/src/blob.rs` - chunks, Reassembler, caps (new)
- `crates/sucrose-sysinfo/src/audio.rs` - the Audio object takes the media keys (`snapshot_with`, `audio_unavailable_with`)
- `crates/sucrose-ipc/src/protocol.rs`, `lib.rs` - `ToRenderer::Blob` and the module
- `apps/sucrose/src/data_hub.rs` - `MediaSession` beside the capture, the art delivery state (`Thumbnails`, `PassState`)
- `apps/sucrose/src/service.rs` - `deliver_blob` and the wiring
- `apps/sucrose/src/render.rs` - `Art` (reassembly and merge)
- `apps/sucrose/tests/fixtures/web-themes/audio/index.html` - title, artist, state, art
- `docs/FEATURES.md` - the Now playing row (🟨)
- `crates/sucrose-sysinfo/Cargo.toml`, `src/lib.rs`, `Cargo.lock` - dependency and exports

Line endings: `crates/sucrose-sysinfo/Cargo.toml`, `crates/sucrose-sysinfo/src/lib.rs`, `apps/sucrose/src/data_hub.rs`, `render.rs`, `service.rs`, `crates/sucrose-ipc/src/lib.rs`, `protocol.rs` and `docs/FEATURES.md` were w/crlf and are w/crlf again (restored with `sed -i 's/\r\?$/\r/'` after the edits that go through the REPL write; `audio.rs` and the fixture page were w/lf and stay so; `media.rs` and `blob.rs` are new, LF). `crates/sucrose-engine-web/src/args.rs` and `tests/args.rs` are still shown as modified (the 04-25 line-ending artifact); they were not touched or staged.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] windows 0.62 has no `IAsyncOperation::get`**
- **Found during:** Task 1
- **Issue:** the blocking call on a WinRT async operation is `join()` in windows-future.
- **Fix:** `join()` on the worker's MTA thread.
- **Files modified:** crates/sucrose-sysinfo/src/media.rs
- **Committed in:** 32f8bb7e

**2. [Rule 1 - Bug] the first Data batch carried `PlaybackState: Unknown`**
- **Found during:** Task 1 verification
- **Issue:** `MediaSession::start` returned before the worker's first reading, so the first snapshot was the empty default.
- **Fix:** the worker reports its start after the first reading.
- **Files modified:** crates/sucrose-sysinfo/src/media.rs
- **Committed in:** 32f8bb7e

**3. [Rule 1 - Bug] a chunk set that did not fit the queue could never be resent under the same id**
- **Found during:** Task 2 design (strict seq rule drops the blob and ignores the rest of its id)
- **Fix:** the hub offers the art again under a new blob id after a refusal, with a test.
- **Files modified:** apps/sucrose/src/data_hub.rs
- **Committed in:** 780c176e

**4. [Rule 3 - Blocking] clippy `print_stdout` on the ignored test; `system_sampling` unused outside tests**
- **Fix:** `#[allow(clippy::print_stdout)]` on that test (as other tests in the workspace do); `system_sampling` is `#[cfg(test)]` because the service uses `system_sampling_with_thumbnails`.
- **Committed in:** 32f8bb7e, 780c176e

### Plan wording that was adapted

- The verify command's log filter `info,sucrose_engine_cef=debug` does not show the `origin request rejected` lines (they are debug lines of `sucrose_engine_web::serve`, the same finding as 04-24 and 04-28); the run used `info,sucrose_engine_web=debug,sucrose_engine_cef=debug`.
- "Each handler only updates the shared record": handlers signal the worker, which updates the record (see key-decisions).
- `chunks` returns `Vec<ToRenderer>` (empty above `BLOB_MAX`), and the hub replaces art over the cap by an empty text before it calls it.

---

**Total deviations:** 4 auto-fixed (2 Rule 1, 2 Rule 3), 3 plan-wording adaptations. **Impact:** no scope change.

## Issues Encountered

- **Art over the 2 MiB cap is dropped.** The maintainer's Edge session offered a first track whose art was 3,145,928 base64 bytes (about 2.3 MB of image); the hub warns and the page gets an empty `ThumbnailString` for that track, while a 1.3 MB base64 art passed through 28 chunks. The cap is the plan's (T-04-65); if real players routinely exceed it, 04-37 should raise `BLOB_MAX` or the art should be re-encoded smaller before sending. Not decided here.
- Not checked: WebView2 (the fixture was run on CEF only), the picture itself (no screenshot of the art on the page; the renderer log proves reception, the page's `img` is not observed), Windows 10 (the crop only runs below build 22000, covered by unit tests only), a session that appears or ends while a page runs.

## Known Stubs

None.

## Threat Flags

None. T-04-64: the media session is opened with the first page that wants Audio and dropped with the last; the hub holds no art without such a page; logs carry sizes and counts only (the `album art` lines print a size, not text). T-04-65: 48 KiB chunks, 2 MiB cap, one open blob, 5 s limit, drop on any mismatch, art re-read only on a track change (and for 10 s while a track has none).

## User Setup Required

None.

## Next Phase Readiness

- 04-30 and later can use `ToRenderer::Blob`/`blob::chunks` for any payload bigger than a frame.
- WAPI-06 stays Pending until 04-37: the maintainer's check with real music on a theme, WebView2 and the oversized-art question are open.

## Self-Check: PASSED

- Created files exist (media.rs, blob.rs); commits `32f8bb7e` and `780c176e` are on the branch; `cargo xtask ci` exit 0.
- Acceptance criteria: `GlobalSystemMediaTransportControlsSessionManager` in media.rs and `MediaSession` in data_hub.rs at least 1; `BLOB_CHUNK` in blob.rs (12), `Blob {` in protocol.rs (2), `Reassembler` in render.rs (2); protocol.rs, render.rs and FEATURES.md are w/crlf.

---
*Phase: 04-m3-engines-and-the-wallpaper-api*
*Completed: 2026-11-27*
