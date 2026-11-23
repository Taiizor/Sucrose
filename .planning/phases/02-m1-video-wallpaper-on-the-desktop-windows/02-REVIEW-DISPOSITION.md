---
phase: 02
review: 02-REVIEW.md
titles: json
findings:
  - id: WR-01
    severity: warning
    disposition: open
    title: "A Playing report that is processed after its renderer's Exit leaves a stale \"playing\" slot"
  - id: WR-02
    severity: warning
    disposition: open
    title: "The follower controller records a speed as sent before `set_speed` succeeded"
  - id: WR-03
    severity: warning
    disposition: open
    title: "The video's frame period is cached once, including the fallback default"
  - id: WR-04
    severity: warning
    disposition: open
    title: "The order guard removes itself permanently after ten restores in a minute"
  - id: IN-01
    severity: info
    disposition: open
    title: "Playing is now only reported after the first frame, with no path for a file that never starts"
  - id: IN-02
    severity: info
    disposition: open
    title: "Stale comments after the 02-19/02-20/UAT changes"
  - id: IN-03
    severity: info
    disposition: open
    title: "The GPU preference overwrites a choice the user made in Windows Settings, on every start"
  - id: IN-04
    severity: info
    disposition: open
    title: "The renderer creates a Direct3D device just to log the adapter"
  - id: IN-05
    severity: info
    disposition: open
    title: "Smaller inconsistencies in the new code"
  - id: WR-05
    severity: warning
    disposition: open
    title: "Hello trusts the peer's claimed pid and slot"
  - id: WR-06
    severity: warning
    disposition: open
    title: "mpv hardening does not cover scripts, IPC server and friends when a user config is loaded"
  - id: WR-07
    severity: warning
    disposition: open
    title: "libmpv is also loaded from `PATH` folders in release builds"
  - id: WR-08
    severity: warning
    disposition: open
    title: "`DesktopSurface::attach` sleeps up to 10 s on a window-owning thread that is not pumping messages"
  - id: WR-09
    severity: warning
    disposition: open
    title: "A failing `GetMessageW` ends the loop like a normal quit and the renderer exits with code 0"
  - id: IN-06
    severity: info
    disposition: open
    title: "Unused public timer methods on `EventLoop`"
  - id: IN-07
    severity: info
    disposition: open
    title: "Generality without instantiation: affected-monitor scope and `PauseKind` plumbing"
  - id: IN-08
    severity: info
    disposition: open
    title: "Untrusted theme text reaches the log and the tooltip unsanitised"
  - id: IN-09
    severity: info
    disposition: open
    title: "Backslash-escaped backticks in a crate doc comment"
  - id: IN-10
    severity: info
    disposition: open
    title: "The recorded GPU regression has an obvious first suspect in the generated options"
open: 19
total: 19
recorded: 2026-11-23T03:03:03.052Z
---

# Phase 02: Code Review Disposition

| Finding | Severity | Disposition | Source |
|---------|----------|-------------|--------|
| WR-01 | warning | open | - |
| WR-02 | warning | open | - |
| WR-03 | warning | open | - |
| WR-04 | warning | open | - |
| IN-01 | info | open | - |
| IN-02 | info | open | - |
| IN-03 | info | open | - |
| IN-04 | info | open | - |
| IN-05 | info | open | - |
| WR-05 | warning | open | - (not in the current review) |
| WR-06 | warning | open | - (not in the current review) |
| WR-07 | warning | open | - (not in the current review) |
| WR-08 | warning | open | - (not in the current review) |
| WR-09 | warning | open | - (not in the current review) |
| IN-06 | info | open | - (not in the current review) |
| IN-07 | info | open | - (not in the current review) |
| IN-08 | info | open | - (not in the current review) |
| IN-09 | info | open | - (not in the current review) |
| IN-10 | info | open | - (not in the current review) |

Dispositions: `open` (recorded, not yet triaged), `fixed`, `skipped`, `deferred`.
Set `deferred` by hand and put the reason in the Source cell; both are preserved. A `|` in the reason is kept as prose and escaped on the next run.
Re-running the gate keeps every row it can. A row the current review no longer reports is kept and its Source cell flagged, so a finding does not leave this record silently. ONE exception: when a finding id is REUSED by a different finding, the earlier decision cannot keep a row — the id is taken — and it is dropped. A RECORDED decision (anything but `open`) is named on the console when that happens; a row still at `open` is replaced silently, because `open` records no decision to lose.
