---
phase: quick-261127-uot
plan: 01
subsystem: sucrose-sysinfo, sucrose (data hub)
tags: [media, album-art, ipc-blob, image, parity]
requires: []
provides:
  - "Album art over the 2 MiB IPC blob cap is re-encoded to fit (JPEG q90 for opaque art, PNG with alpha for transparent art, own size first, then scaled down) instead of being replaced by an empty ThumbnailString"
  - "THUMBNAIL_TEXT_MAX in sucrose-sysinfo, tied to sucrose_ipc::blob::BLOB_MAX by a compile-time assert in the data hub"
affects: [now-playing path of WAPI-06, docs/FEATURES.md Now playing row]
tech-stack:
  added: []
  patterns: ["bounded image decode (8192 px a side, 256 MiB) for all untrusted art", "fit ladder: own size first, then scale x clamp(sqrt(limit / length) * 0.9, 0.25, 0.75), max 8 encodes, min 64 px"]
key-files:
  created: []
  modified:
    - crates/sucrose-sysinfo/src/media.rs
    - crates/sucrose-sysinfo/src/lib.rs
    - apps/sucrose/src/data_hub.rs
    - docs/FEATURES.md
key-decisions:
  - "The cap stays (maintainer decision 2026-11-27, STATE.md Pending Todos (b)); the art is made to fit, BLOB_MAX and BLOB_CHUNK are unchanged"
  - "The fit runs in the media-session worker once per track change, before the Shared lock; the data hub keeps its over-cap guard"
  - "No new crate, feature or dev-dependency (T-04-SC); test noise comes from a fixed-seed LCG in the test module"
status: complete
commits: 3
plan_head_before: c1ac7f05de48a4f62ade2a56a4d775ed5e221778
plan_head_after: 7b0f4abdc84b055131e04914e1c4bd1f2f897a9e
actuals:
  tasks: 3
  commits: 3
---

# Phase quick-261127-uot Plan 01: Album art over the blob cap is re-encoded to fit Summary

Album art whose base64 text is over the 2 MiB IPC blob cap is now re-encoded in the media-session worker (own size first, scaled down only when needed; JPEG quality 90 for opaque art, PNG with alpha for transparent art) so the page gets a fitting, non-empty `ThumbnailString` instead of an empty one; the cap and the data hub guard stay.

## Tasks

| Task | Name | Commit | Files |
| ---- | ---- | ------ | ----- |
| 1 | Opaque art over the cap is fitted to one IPC blob (tracer) | 5de6cf69 | crates/sucrose-sysinfo/src/media.rs, crates/sucrose-sysinfo/src/lib.rs, apps/sucrose/src/data_hub.rs |
| 2a | Transparent art stays a PNG when it is fitted | deb5dca9 | crates/sucrose-sysinfo/src/media.rs |
| 2b | FEATURES.md Now playing row (fit behavior and the behavior change recorded) | e54c0523 | docs/FEATURES.md |
| 3 | Maintainer check with the real over-cap track | no commit (scratch run under target/art-fit) | none |

Commits of this item: 3 (5de6cf69, deb5dca9, e54c0523). `plan_head_before` is c1ac7f05; `plan_head_after` is the HEAD at the time of writing (7b0f4abd). The range c1ac7f05..7b0f4abd also holds 10 commits of the sibling items 261127-uos, 261127-uov, 261127-uou and e747c7be, which ran in the same batch; they are not part of this item. No manifest (Cargo.toml, Cargo.lock, member manifests) changed.

## Checks

- `cargo test -p sucrose-sysinfo media`: 19 passed, 1 ignored (18 passed after Task 1, 19 after Task 2).
- `cargo test -p sucrose data_hub`: 29 passed.
- `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo test --workspace --no-fail-fast`: 2 portal.rs desktop tests failed while DISPLAY9 was attached (logs show `slot=1 monitor=\\.\DISPLAY9`). This is the known two-monitor case (fixed by 261127-uou); portal.rs was not touched here. `an_engine_change_restarts_the_running_wallpaper_on_the_engine_chosen` and `switching_web_themes_on_webview2_closes_each_renderer_cleanly` were seen failing, but not both in one run, so which two fail together is not confirmed. A third run showed 1701 passed and 2 failed in total.
- Line endings after the commits (`git ls-files --eol`):

```
i/lf    w/crlf  attr/text=auto        	apps/sucrose/src/data_hub.rs
i/lf    w/crlf  attr/text=auto        	crates/sucrose-sysinfo/src/lib.rs
i/lf    w/lf    attr/text=auto        	crates/sucrose-sysinfo/src/media.rs
i/lf    w/crlf  attr/text=auto        	docs/FEATURES.md
```

  docs/FEATURES.md keeps its missing final newline; its diff is 1 line added, 1 deleted. Diff over the three commits: 4 files, 304 insertions, 13 deletions.

## Maintainer check (Task 3)

Run by the coordinator: release build at HEAD 7b0f4abd (the item's code plus later commits of 261127-uos, 261127-uov, 261127-uou and e747c7be), scratch data in `target/art-fit` (audio fixture, `Web=WebViewLive`), `sucrose.exe --no-portal` with `SUCROSE_LOG=debug` started at 2026-11-28 00:05:59 +03, two monitors attached (DISPLAY1 primary, DISPLAY9). The maintainer played the local FLAC track with the oversized art. Logs: `target/art-fit/Log-maintainer-check`. The process was stopped with taskkill afterwards and `tasklist.exe` shows no Sucrose process.

Machine evidence (sizes only, no track name):

- `sucrose_sysinfo::media` DEBUG "the album art was made smaller to fit the renderer": `text=3145928 fitted=167404 width=768 height=768 attempts=1 elapsed_ms=15`. The art was an opaque 768 x 768 picture, so the first encode (JPEG q90 at its own size) already fit; no scaling was needed.
- `sucrose::data_hub` DEBUG "album art taken": `size=167404` (between 1 and 2097152).
- `sucrose::render` DEBUG "album art received": `id=1 size=167404` in both renderers' logs (Renderer-0 and Renderer-1).
- No "the album art is too large for the renderer" WARN and no "cannot be made small enough" WARN in the service or renderer logs (0 matches each).

Maintainer verdict, verbatim:

> kapak görünüyor, bozuk değil gibi

Translation: "the cover shows, it does not look broken."

The must-have truth with `verification: backstop` (the re-encoded cover of the maintainer's own track shows on the desktop page and looks like the original) is confirmed by this verdict.

## Deviations from Plan

### Process deviation

**1. [Process] No RED run for the new tests**
- **Found during:** Task 1 (tdd tracer)
- **Issue:** The plan asks for the behavior-block tests to be written and seen failing before the implementation. The implementation was written before the tests, so there was no RED run for the new tests.
- **Fix:** None possible after the fact. The new tests were written against the behavior block and all pass (19 media tests, 29 data_hub tests); they were not shown to fail first.
- **Commit:** 5de6cf69

### Auto-fixed Issues

None.

### Environment note

With DISPLAY9 attached, two portal.rs desktop tests fail in `cargo test --workspace` (known; handled by 261127-uou). They are unrelated to this item.

## Known Stubs

None.

## Threat Flags

None. The new surface (decoding untrusted album art) is covered by T-uot-01 in the plan: the decode is bounded (8192 px a side, 256 MiB), at most 8 encodes, once per track change on the media-session worker. Only lengths, sizes, attempts and elapsed time are logged (T-uot-02).

## Note for the orchestrator

STATE.md Pending Todos (b) (04-29 album art over the 2 MiB blob cap) can be marked done. WAPI-06 is not marked here; it stays open for its 04-37 B5 re-run.

## Self-Check: PASSED

- 5de6cf69, deb5dca9 and e54c0523 are ancestors of HEAD.
- media.rs, lib.rs, data_hub.rs and FEATURES.md exist and carry the item's changes.
- No Sucrose process is running.
