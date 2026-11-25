---
phase: 03-m2-service-ipc-and-portal-connected
plan: 47
subsystem: ui
tags: [gpui, backdrop, mica, acrylic, dwm, scrollbar, uat, windows]
requires:
  - phase: 03-m2-service-ipc-and-portal-connected
    provides: "03-45 Opaque-first backdrop reset and DWM type; 03-46 library scrollbar sibling mount and settled requests"
provides:
  - "Maintainer's eye confirmation of Mica, Tabbed, Acrylic and None in Dark and Light, both directions, on the active window"
  - "Maintainer's confirmation of the grid scrollbar in both OS scrollbar modes and of WR-16 on the desktop"
  - "frame_extension / backdrop_next / backdrop_confirmed verdict keys read by 03-48, 03-49 and the phase verification"
  - "docs/GPUI.md and docs/FEATURES.md corrected to what was confirmed"
affects: [03-48, 03-49, 03-50, phase-3-verification]
gap_closure: true
gap_ids: [G-03-5, G-03-6, WR-14, WR-15, WR-16]
frame_extension: not-needed
backdrop_next: closed
backdrop_confirmed: yes
actuals:
  tokens: 1200
  tasks: 3
  commits: 1
plan_head_before: cfdc014f34f4a5f0c82ce82166765ace5cdd23a8
plan_head_after: 7050949de69dc44d1d367e4c9c9c40c889cf01a5
tech-stack:
  added: []
  patterns: ["fixed verdict rule: eye answers plus dwm_result log lines decide frame_extension, backdrop_next and backdrop_confirmed"]
key-files:
  created: []
  modified: [docs/GPUI.md, docs/FEATURES.md]
key-decisions:
  - "frame_extension: not-needed - A1-A7 all yes and every Mica/Tabbed switch logged dwm_result=0x00000000, so 03-48 and 03-49 change nothing"
  - "Hover previews not played in this check is the migrated LibraryPreview=false scratch setting (log: preview switched off), not a gap; Library Preview on was confirmed in 03-43"
requirements-completed: [UI-01, UI-02, LIB-04]
coverage:
  - id: D1
    description: "Mica, Tabbed, Acrylic and None each change the active Portal window in Dark and Light, in both directions (UI-01, UI-02)"
    requirement: "UI-01"
    verification:
      - kind: manual_procedural
        ref: "maintainer reply A1-A7 (verbatim below) with 42 'backdrop applied' lines, all supported=true and dwm_result=0x00000000"
        status: pass
    human_judgment: true
    rationale: "A backdrop shows only on the active window and PrintWindow returns no content under it; only the maintainer's eye can judge it"
  - id: D2
    description: "The library grid scrollbar shows on the right edge, follows the cards' scroll and works in both OS scrollbar modes; cards in view keep thumbnails after a resize at the page end (LIB-04, WR-15, WR-16)"
    requirement: "LIB-04"
    verification:
      - kind: manual_procedural
        ref: "maintainer replies B1 a-e and B2 a-d (verbatim below) with the 'thumbnails requested' lines around B1 e"
        status: pass
    human_judgment: true
    rationale: "Scrollbar visibility in two OS modes and the look of thumbnails after a resize need the maintainer's eye on the desktop"
  - id: D3
    description: "docs/GPUI.md and docs/FEATURES.md state only what the maintainer confirmed"
    verification:
      - kind: other
        ref: "grep -c '(03-47)' docs/GPUI.md docs/FEATURES.md (2 each); git ls-files --eol shows w/crlf for both"
        status: pass
    human_judgment: false
duration: 25min
completed: 2026-11-25
status: complete
---

# Phase 3 Plan 47: Backdrop and grid scrollbar eye check Summary

**The maintainer confirmed by eye that Mica, Tabbed, Acrylic and None each change the active Portal window in Dark and Light and in both directions (no DWM frame extension needed), that the library grid scrollbar sits on the right edge in both OS scrollbar modes, and that thumbnails stay after a resize at the page end (WR-16); the docs now say so.**

## Performance

- **Duration:** about 25 min of executor time across three executors (the maintainer's waits excluded where they could not be measured)
- **Started:** 2026-11-25T17:10:57Z (plan commit ledger written)
- **Completed:** 2026-11-25T17:35Z
- **Tasks:** 3 (Task 1 tracer, Task 2 and Task 3 maintainer checkpoints)
- **Files modified:** 2 (docs/GPUI.md, docs/FEATURES.md); no code changed

## Verdicts

- `frame_extension: not-needed`: A4 and A6 say Mica and Tabbed each look different from None; A1 and A5 say leaving Acrylic changes the window; the reply and the log agree (every switch below has `supported=true`, `dwm_result=0x00000000` and steps ending in `MicaBackdrop` or `MicaAltBackdrop` for Mica and Tabbed).
- `backdrop_next: closed`: the backdrop half of SC5 is confirmed by this check; 03-48 and 03-49 change nothing, 03-50 runs, and the phase verification after 03-50 scores the backdrop truth on this SUMMARY.
- `backdrop_confirmed: yes`: final value; evidence is the A1-A7 reply and the 'backdrop applied' lines of this SUMMARY. UI-01 and UI-02 are listed in `requirements-completed` here (the closed route); the marking itself happens through the shared-ID gate once 03-49's SUMMARY exists.
- LIB-04: scrollbar seen on the right edge in both modes (B1 b, B2 a-d) and B1 e yes (clarified reading), so it is listed in `requirements-completed`.
- WR-16 (B1 e): yes, see below.

## Task 1: unattended preparation (tracer)

Release build up to date; `cargo xtask fetch-engines` libmpv up to date. Scratch migration with `sucrose.exe --no-portal` logged `settings loaded outcome=Migrated` (that service started the migrated theme's renderers, so the taskkill stopped three `sucrose.exe` processes). Values changed in the scratch `Setting/Sucrose.json` from the migrated ones: `LibraryPagination` 30 to 100, `HintTrayIcon` true to false. Already as required after migration: `DeleteCorrupt: false`, `ThemeMode: Dark`, `Backdrop: Acrylic`, Portal `BackgroundImage: ""`. `LibraryPreview` stayed false as migrated.

Smoke lines (`target/uat-gaps-smoke.txt`):

```
backdrop applied backdrop=Acrylic supported=true appearance=Blurred steps=[Opaque, Blurred] root_transparent=true dwm_result=0x00000000
library loaded themes=90 corrupt=16 ignored=2
page shown page=1 pages=1 thumbs=0 thumb_bytes=0 preview_playing=false heap_allocated_mb=38 heap_committed_mb=40 gpu_local_mb=30 gpu_non_local_mb=0
thumbnails requested first=0 last=11 wanted=12 queued=12
```

`tasklist.exe` showed no Rust Sucrose process afterwards. Task 1 verify exited 0.

## Registry readings (read-only, `reg.exe query`, `target/uat-gaps-scrollbars.txt`)

| # | When | `DynamicScrollbars` |
|---|------|--------------------|
| 1 | before the check | `REG_DWORD 0x1` (auto-hide) |
| 2 | after the maintainer's switch | `REG_DWORD 0x0` (always show) |
| 3 | after the check (17:30:27Z, taken by this executor) | `REG_DWORD 0x1` (auto-hide again; the maintainer restored it, he did not write the word "restored") |

## Maintainer replies (verbatim, Turkish)

Task 2 reply (run 1, OS setting auto-hide, Portal started with focus):

```
a1 yes
a2 yes
a3 yes
a4 yes
a5 yes
a6 yes
a7 yes

b1a gif oynamadı
b1b scroll gorunur oldu
b1c yes
b1d yes
b1e gifler zaten oynamıyordu b1a da da belirttim

her zaman kaydırma çubuklarını göster şuan açık
```

Task 3 reply (run 2, OS setting "always show", Portal restarted with focus on the same scratch data, it connected to the running service):

```
b2a yes
b2b yes
b2c yes
b2d yes

pencereyi genişlettikce duvar kağıtlarını resimleri tekrardan boyutlanıyor. C# halindeki gibi yani. thumbnailler gösteriliyor fakat fare ile üzerlerinde durunca preview gifleri hiç görünmüyor ve dolayısıyla oynatılmıyor
```

Readings: A1-A7 all yes, B2 a-d all yes. B1 b: the bar became visible. B1 c, d yes. B1 e was answered about GIFs and clarified in the Task 3 reply: "as I widen the window the wallpaper images are resized again, like the C# version; the thumbnails are shown", so WR-16 is yes.

### Hover previews (B1 a and the Task 3 note): not a defect of 03-45 or 03-46

"gif oynamadı" and "preview gifleri hiç görünmüyor" are the scratch setting `LibraryPreview: false` (migrated as it was), not an overlay regression. The final Portal log has 256 `hover preview not started reason="preview switched off"` lines (179 in run 1, 77 in run 2) and no other hover reason, so hover reaches the cards through the 03-46 overlay and the previews are switched off by setting. 03-43 had the setting on and the maintainer confirmed previews played then; hover previews with Library Preview on were not rechecked here. No gap opened for it.

## Run evidence (`target/uat-gaps-run.log`, copy of the Portal log of both runs)

'library loaded' lines: `17:12:00.136Z library loaded themes=90 corrupt=16 ignored=2` (run 1) and `17:25:16.243Z library loaded themes=90 corrupt=16 ignored=2` (run 2).

All 42 'backdrop applied' lines in order (41 in run 1, 17:12:00Z to 17:19:05Z; 1 in run 2 at 17:25:16Z, which started with `Backdrop: None` because the maintainer's last switch in run 1 left None in the scratch settings). Every line has `supported=true` and `dwm_result=0x00000000`; steps are `[Opaque, Blurred]` for Acrylic, `[Opaque, MicaBackdrop]` for Mica, `[Opaque, MicaAltBackdrop]` for Tabbed and `[Opaque]` for None.

| Time | backdrop | supported | steps | dwm_result |
|------|----------|-----------|-------|------------|
| 17:12:00.092Z | Acrylic | true | [Opaque, Blurred] | 0x00000000 |
| 17:15:53.848Z | None | true | [Opaque] | 0x00000000 |
| 17:15:55.909Z | Acrylic | true | [Opaque, Blurred] | 0x00000000 |
| 17:15:57.574Z | Mica | true | [Opaque, MicaBackdrop] | 0x00000000 |
| 17:15:59.104Z | None | true | [Opaque] | 0x00000000 |
| 17:16:00.701Z | Mica | true | [Opaque, MicaBackdrop] | 0x00000000 |
| 17:16:02.006Z | None | true | [Opaque] | 0x00000000 |
| 17:16:04.502Z | Tabbed | true | [Opaque, MicaAltBackdrop] | 0x00000000 |
| 17:16:05.871Z | Acrylic | true | [Opaque, Blurred] | 0x00000000 |
| 17:16:07.173Z | Tabbed | true | [Opaque, MicaAltBackdrop] | 0x00000000 |
| 17:16:08.557Z | None | true | [Opaque] | 0x00000000 |
| 17:16:09.718Z | Acrylic | true | [Opaque, Blurred] | 0x00000000 |
| 17:16:10.865Z | None | true | [Opaque] | 0x00000000 |
| 17:16:17.953Z | Mica | true | [Opaque, MicaBackdrop] | 0x00000000 |
| 17:16:19.445Z | Tabbed | true | [Opaque, MicaAltBackdrop] | 0x00000000 |
| 17:16:21.335Z | Mica | true | [Opaque, MicaBackdrop] | 0x00000000 |
| 17:17:53.986Z | None | true | [Opaque] | 0x00000000 |
| 17:17:55.299Z | Mica | true | [Opaque, MicaBackdrop] | 0x00000000 |
| 17:17:57.108Z | Acrylic | true | [Opaque, Blurred] | 0x00000000 |
| 17:17:58.869Z | Mica | true | [Opaque, MicaBackdrop] | 0x00000000 |
| 17:18:00.765Z | Tabbed | true | [Opaque, MicaAltBackdrop] | 0x00000000 |
| 17:18:05.527Z | Acrylic | true | [Opaque, Blurred] | 0x00000000 |
| 17:18:16.401Z | None | true | [Opaque] | 0x00000000 |
| 17:18:18.018Z | Tabbed | true | [Opaque, MicaAltBackdrop] | 0x00000000 |
| 17:18:19.258Z | None | true | [Opaque] | 0x00000000 |
| 17:18:20.544Z | Tabbed | true | [Opaque, MicaAltBackdrop] | 0x00000000 |
| 17:18:21.724Z | None | true | [Opaque] | 0x00000000 |
| 17:18:24.752Z | Acrylic | true | [Opaque, Blurred] | 0x00000000 |
| 17:18:49.037Z | None | true | [Opaque] | 0x00000000 |
| 17:18:50.708Z | Tabbed | true | [Opaque, MicaAltBackdrop] | 0x00000000 |
| 17:18:52.000Z | None | true | [Opaque] | 0x00000000 |
| 17:18:53.482Z | Acrylic | true | [Opaque, Blurred] | 0x00000000 |
| 17:18:54.780Z | None | true | [Opaque] | 0x00000000 |
| 17:18:56.183Z | Mica | true | [Opaque, MicaBackdrop] | 0x00000000 |
| 17:18:57.309Z | None | true | [Opaque] | 0x00000000 |
| 17:18:58.641Z | Mica | true | [Opaque, MicaBackdrop] | 0x00000000 |
| 17:18:59.615Z | None | true | [Opaque] | 0x00000000 |
| 17:19:00.733Z | Tabbed | true | [Opaque, MicaAltBackdrop] | 0x00000000 |
| 17:19:01.820Z | None | true | [Opaque] | 0x00000000 |
| 17:19:03.464Z | Tabbed | true | [Opaque, MicaAltBackdrop] | 0x00000000 |
| 17:19:05.096Z | None | true | [Opaque] | 0x00000000 |
| 17:25:16.198Z | None | true | [Opaque] | 0x00000000 |

### WR-16 (B1 e) and the 'thumbnails requested' lines around it

`thumbnails requested` lines: 276 in total (177 in run 1, 99 in run 2). Around B1 d and B1 e in run 1:

```
17:21:21.788Z thumbnails requested first=68 last=89 wanted=22 queued=21   (end of the 100-card page, B1 d)
17:21:21.884Z thumbnail cache images=64 bytes=18974880
(no 'thumbnails requested' line from 17:21:21.9Z to 17:22:48.4Z)
17:22:48.474Z thumbnails requested first=51 last=68 wanted=18 queued=0   (after the resize, B1 e)
17:22:50.188Z thumbnails requested first=51 last=65 wanted=15 queued=0   (wheel scroll back up follows)
```

After the resize the offset was clamped to rows 51-68, all already cached (`queued=0`); the maintainer saw the thumbnails shown with no extra wheel step. WR-16: yes.

## Task Commits

1. **Task 1: Prepare the check and prove it unattended (tracer)** - no commit (only untracked `target/` data)
2. **Task 2: Maintainer check A1-A7 and B1 a-e, then OS setting switched** - no commit (checkpoint, no files)
3. **Task 3: Verdicts, frame-extension rule and docs** - `7050949d` (docs)

**Plan metadata:** the docs(03-47) completion commit that follows this SUMMARY.

## Files Created/Modified

- `docs/GPUI.md` - "Windows backdrops" and "Library grid scrolling" bullets state the confirmed result (03-47); CRLF kept
- `docs/FEATURES.md` - "Pagination, adaptive layout" and "Window backdrop" rows state the confirmed result (03-47), statuses stay 🟨; CRLF and the missing final newline kept
- Untracked data: `target/uat-gaps/`, `target/uat-gaps-smoke.txt`, `target/uat-gaps-scrollbars.txt`, `target/uat-gaps-run1.log` (run 1 Portal log), `target/uat-gaps-run.log`

## Decisions Made

- Docs wording for auto-hide says the bar "becomes visible at the right edge" (B1 b); whether it is hidden at rest over the cards was not separately answered, so the docs do not claim "on hover only" as seen by the maintainer.
- The Portal-hover preview observation is recorded as the `LibraryPreview` setting, no gap.

## Deviations from Plan

None - plan executed exactly as written. Two executors ended at the checkpoints and a stale background task of the earlier executor may have reported the first Portal's exit; ignored.

## Issues Encountered

None. B1 e was first answered about GIFs and was clarified through the Task 3 reply.

## Verification

- `cargo xtask ci` (fmt, clippy, tests): exit 0.
- `git diff --stat HEAD~1 -- apps crates`: no code file changed; `git ls-files --eol docs/GPUI.md docs/FEATURES.md`: `w/crlf` for both; `grep -c '(03-47)'` is 2 in each doc.
- `tasklist.exe` after the check: no Rust Sucrose process (taskkill stopped `sucrose-portal.exe` 29324 and `sucrose.exe` 31400, 17088, 10700).

## Known Stubs

None.

## Threat Flags

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `backdrop_next: closed`: 03-48 and 03-49 change nothing on the backdrop (their gates read `frame_extension: not-needed`); 03-50 (the D-25 memory rerun) is next.
- Hover previews were not rechecked with Library Preview on; that was confirmed in 03-43.

## Self-Check: PASSED

Found: docs/GPUI.md, docs/FEATURES.md, target/uat-gaps-run.log, target/uat-gaps-run1.log, target/uat-gaps-smoke.txt, target/uat-gaps-scrollbars.txt; commit 7050949d is on HEAD; the three verdict keys are present.
