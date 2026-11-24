---
phase: 03
review: 03-REVIEW.md
titles: json
findings:
  - id: WR-13
    severity: warning
    disposition: open
    title: "The JPEG decode budget is bypassed by the generic fallback, so the WR-12 bound does not hold"
  - id: IN-02
    severity: info
    disposition: deferred
    title: "Endpoint hardening is Windows-only while Linux gets an abstract socket"
  - id: IN-04
    severity: info
    disposition: deferred
    title: "Stale `#[allow(dead_code)]` with an outdated reason"
  - id: IN-10
    severity: info
    disposition: deferred
    title: "An invalid `Version` is read as current and then overwritten"
  - id: IN-11
    severity: info
    disposition: deferred
    title: "Background image path check accepts NUL and UNC paths"
  - id: IN-13
    severity: info
    disposition: open
    title: "The doc comment of `monotonic_ns` is attached to `disconnect_pipe`"
  - id: IN-14
    severity: info
    disposition: open
    title: "A Portal dropped by `send` during Close Applications is not counted as gone"
  - id: IN-15
    severity: info
    disposition: open
    title: "GPUI.md contradicts itself about whether the preview cost is measured"
  - id: IN-16
    severity: info
    disposition: open
    title: "`request_page` reads heap and DXGI memory on the GPUI thread"
  - id: IN-17
    severity: info
    disposition: open
    title: "`measure --side portal` reports success when the Portal crashed"
  - id: IN-18
    severity: info
    disposition: open
    title: "The scratch endpoint name hashes the unnormalized data-folder text"
  - id: IN-19
    severity: info
    disposition: open
    title: "A synchronous `loadfile` failure marks every earlier in-flight request as finished"
  - id: IN-20
    severity: info
    disposition: open
    title: "A corrupt file is kept in one slot only, and a failed move still lets the write replace it"
  - id: WR-09
    severity: warning
    disposition: fixed
    title: "A corrupt settings file with no backup is overwritten and its content is lost"
  - id: WR-10
    severity: warning
    disposition: fixed
    title: "A quit that arrives during a library move can never finish (service never exits)"
  - id: WR-11
    severity: warning
    disposition: fixed
    title: "The event thread clears `loading` with a plain store and can erase a newer `load_source`"
  - id: WR-12
    severity: warning
    disposition: fixed
    title: "The pixel-area cap bounds one JPEG decode, not memory; three decodes plus the background image can hold about 800 MB"
  - id: CR-01
    severity: critical
    disposition: fixed
    title: "Progressive JPEG bypasses the decode limits (memory bomb from a theme thumbnail or background image)"
  - id: CR-02
    severity: critical
    disposition: fixed
    title: "A transient read error makes the settings store restore an older backup, or migrate and overwrite, a good `Sucrose.json`"
  - id: WR-01
    severity: warning
    disposition: fixed
    title: "MaintainLibrary is accepted while a library move runs"
  - id: WR-02
    severity: warning
    disposition: fixed
    title: "Portal requests are still handled once the service is quitting, so a move can be cut off by process exit"
  - id: WR-03
    severity: warning
    disposition: fixed
    title: "The library move does not fully quiesce the fleet (monitor changes are dropped, a queued respawn can reopen the old folder)"
  - id: WR-04
    severity: warning
    disposition: fixed
    title: "A Portal dropped for a full queue keeps its pipe open, the Portal is never told, and the service may never exit"
  - id: WR-05
    severity: warning
    disposition: fixed
    title: "Turning startup off can fail half-way and leave the setting and the registry disagreeing"
  - id: WR-06
    severity: warning
    disposition: fixed
    title: "`synth-library` guard against the real data folder is lexical and bypassed by a verbatim path"
  - id: WR-07
    severity: warning
    disposition: fixed
    title: "Theme folder names bypass the control-character filter on the tray and log path"
  - id: WR-08
    severity: warning
    disposition: fixed
    title: "A queued restart of the old file can be taken for the first frame of the new source"
  - id: IN-01
    severity: info
    disposition: deferred
    title: "No limit on connections and no read timeout on the first message"
  - id: IN-03
    severity: info
    disposition: deferred
    title: "Duplicated code"
  - id: IN-05
    severity: info
    disposition: deferred
    title: "`StartActions.start_wallpaper` is read only by tests"
  - id: IN-06
    severity: info
    disposition: deferred
    title: "Generality without instantiation, the `Restart` use transition"
  - id: IN-07
    severity: info
    disposition: deferred
    title: "Paths in info-level log lines, against the \"values only\" rule"
  - id: IN-08
    severity: info
    disposition: deferred
    title: "The Portal re-derives the default library folder instead of using the service's answer"
  - id: IN-09
    severity: info
    disposition: deferred
    title: "A refresh answer can cause a false \"could not be saved\" toast"
  - id: IN-12
    severity: info
    disposition: deferred
    title: "Timing assertion in the memory integration test is tight"
open: 9
total: 35
recorded: 2026-11-24T21:02:46.020Z
---

# Phase 03: Code Review Disposition

| Finding | Severity | Disposition | Source |
|---------|----------|-------------|--------|
| WR-13 | warning | open | - |
| IN-02 | info | deferred | Latent until the Linux service exists (EventLoop::new fails there today); fix before Linux support is enabled |
| IN-04 | info | deferred | Lint hygiene only, no behavior impact; next Portal cleanup (same finding as the earlier IN-04, re-reported with Origin and narrowed to apps/sucrose-portal/src/main.rs:22; the decision was restored after the ledger tool dropped it on the changed title) |
| IN-10 | info | deferred | Needs a hand-edited or damaged Version; fix with the next settings format change |
| IN-11 | info | deferred | Same-user settings and IPC only; share one path validator when the background image picker grows (UI-04) |
| IN-13 | info | open | - |
| IN-14 | info | open | - |
| IN-15 | info | open | - |
| IN-16 | info | open | - |
| IN-17 | info | open | - |
| IN-18 | info | open | - |
| IN-19 | info | open | - |
| IN-20 | info | open | - |
| WR-09 | warning | fixed | 03-REVIEW-FIX.md (not in the current review) |
| WR-10 | warning | fixed | 03-REVIEW-FIX.md (not in the current review) |
| WR-11 | warning | fixed | 03-REVIEW-FIX.md (not in the current review) |
| WR-12 | warning | fixed | 03-REVIEW-FIX.md (not in the current review) |
| CR-01 | critical | fixed | 03-29 (not in the current review) |
| CR-02 | critical | fixed | 03-30 (not in the current review) |
| WR-01 | warning | fixed | 03-31 (not in the current review) |
| WR-02 | warning | fixed | 03-32 (not in the current review) |
| WR-03 | warning | fixed | 03-31 (not in the current review) |
| WR-04 | warning | fixed | 03-33 (not in the current review) |
| WR-05 | warning | fixed | 03-35 (not in the current review) |
| WR-06 | warning | fixed | 03-36 (not in the current review) |
| WR-07 | warning | fixed | 03-31 (not in the current review) |
| WR-08 | warning | fixed | 03-34 (not in the current review) |
| IN-01 | info | deferred | Same-user only (single-SID pipe DACL); connection cap and first-read deadline belong with the next IPC hardening pass (not in the current review) |
| IN-03 | info | deferred | Fixed in part by 03-29 (one decode_jpeg_scaled for thumbnails and the background image, with CR-01); the bin_name duplicate remains: no behavior impact, next Portal cleanup (not in the current review) |
| IN-05 | info | deferred | Test-only field, no behavior impact; next lifecycle change (not in the current review) |
| IN-06 | info | deferred | Deliberate documented D-26 fallback; remove after the LoadSource path has run longer (not in the current review) |
| IN-07 | info | deferred | Log hygiene (paths in info lines), no functional impact; with the logging and backup work (not in the current review) |
| IN-08 | info | deferred | Both derivations agree today; move to a service-sent resolved folder with the full settings pages (UI-04) (not in the current review) |
| IN-09 | info | deferred | Rare transient toast in a refresh/SetStartup race; fix with the full settings pages (UI-04) (not in the current review) |
| IN-12 | info | deferred | Test robustness; passes today; widen the deadline if it flakes on CI (not in the current review) |

Dispositions: `open` (recorded, not yet triaged), `fixed`, `skipped`, `deferred`.
Set `deferred` by hand and put the reason in the Source cell; both are preserved. A `|` in the reason is kept as prose and escaped on the next run.
Re-running the gate keeps every row it can. A row the current review no longer reports is kept and its Source cell flagged, so a finding does not leave this record silently. ONE exception: when a finding id is REUSED by a different finding, the earlier decision cannot keep a row — the id is taken — and it is dropped. A RECORDED decision (anything but `open`) is named on the console when that happens; a row still at `open` is replaced silently, because `open` records no decision to lose.
