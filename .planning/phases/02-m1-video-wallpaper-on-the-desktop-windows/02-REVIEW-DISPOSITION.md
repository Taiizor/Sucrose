---
phase: 02
review: 02-REVIEW.md
titles: json
findings:
  - id: WR-15
    severity: warning
    disposition: fixed
    title: "A late `Playing` or `Failed` of an already ended renderer still rewrites the status (reopens IN-16 and WR-01)"
  - id: WR-16
    severity: warning
    disposition: fixed
    title: "A refused Hello makes the renderer exit with code 0, and the monitor stays without a wallpaper (reopens WR-05)"
  - id: WR-17
    severity: warning
    disposition: fixed
    title: "A failed start loses its reason from the tooltip right after the exit (reopens IN-01, WR-11)"
  - id: IN-07
    severity: info
    disposition: skipped
    title: "Generality without instantiation: affected-monitor scope and `PauseKind` plumbing"
  - id: IN-19
    severity: info
    disposition: open
    title: "The order guard no longer limits a failing restore (reopens WR-14)"
  - id: IN-20
    severity: info
    disposition: open
    title: "A Quit queued while the surface attaches is not seen until attach ends (reopens WR-08)"
  - id: IN-21
    severity: info
    disposition: open
    title: "A stuck or broken renderer link is only logged, and warns once per clock reading"
  - id: WR-05
    severity: warning
    disposition: fixed
    title: "Hello trusts the peer's claimed slot, and a second Hello replaces the registered writer"
  - id: WR-06
    severity: warning
    disposition: fixed
    title: "mpv hardening does not neutralise scripts, IPC server, log file or includes when a user config is loaded"
  - id: WR-07
    severity: warning
    disposition: fixed
    title: "libmpv is also loaded from user-writable `PATH` folders"
  - id: WR-08
    severity: warning
    disposition: fixed
    title: "`DesktopSurface::attach` sleeps up to 10 s on a thread that owns two windows and pumps nothing"
  - id: WR-09
    severity: warning
    disposition: fixed
    title: "A failing `GetMessageW` ends the loop like a normal quit and the renderer exits with code 0"
  - id: WR-10
    severity: warning
    disposition: fixed
    title: "Loop wrap is never detected, `loop_gen` stays 0 for an ordinary loop"
  - id: WR-11
    severity: warning
    disposition: fixed
    title: "Tray/log status stays \"no wallpaper\" after the renderer recovers"
  - id: WR-12
    severity: warning
    disposition: fixed
    title: "Unsynchronised writes on the reader's pipe handle can interleave frames"
  - id: WR-13
    severity: warning
    disposition: fixed
    title: "Blocking pipe writes while holding the global `Renderers` mutex can freeze the service loop"
  - id: WR-14
    severity: warning
    disposition: fixed
    title: "The order guard that holds back never looks again, so a surface left under the wallpaper stays hidden"
  - id: IN-01
    severity: info
    disposition: fixed
    title: "Playing is only reported after the first frame, with no path for a file that never starts"
  - id: IN-02
    severity: info
    disposition: fixed
    title: "Stale comments after the 02-19/02-20/UAT changes and after the WR-03 fix"
  - id: IN-03
    severity: info
    disposition: fixed
    title: "The GPU preference overwrites a choice made in Windows Settings, on every start"
  - id: IN-04
    severity: info
    disposition: fixed
    title: "The renderer creates a Direct3D device just to log the adapter"
  - id: IN-05
    severity: info
    disposition: fixed
    title: "Smaller inconsistencies in error codes and measurement notes"
  - id: IN-06
    severity: info
    disposition: fixed
    title: "Unused public timer methods on `EventLoop`"
  - id: IN-08
    severity: info
    disposition: fixed
    title: "Untrusted theme text reaches the log and the tooltip unsanitised"
  - id: IN-09
    severity: info
    disposition: fixed
    title: "Backslash-escaped backticks in a crate doc comment"
  - id: IN-11
    severity: info
    disposition: fixed
    title: "`reparent` sets the child style after `SetParent`"
  - id: IN-12
    severity: info
    disposition: fixed
    title: "Log message and behaviour disagree for a control message before the assignment"
  - id: IN-13
    severity: info
    disposition: fixed
    title: "The shader-cache comments claim a separation that the default configuration does not have"
  - id: IN-14
    severity: info
    disposition: fixed
    title: "`Player::open_source` is public and takes any mpv source string"
  - id: IN-15
    severity: info
    disposition: fixed
    title: "Any unrecognised command line starts the service"
  - id: IN-16
    severity: info
    disposition: fixed
    title: "A late `Playing` is only dropped by connection presence, because reports carry no renderer identity"
  - id: IN-17
    severity: info
    disposition: fixed
    title: "The \"all monitors\" measurement row repeats the one-monitor numbers"
  - id: WR-01
    severity: warning
    disposition: fixed
    title: "A Playing report that is processed after its renderer's Exit leaves a stale \"playing\" slot"
  - id: WR-02
    severity: warning
    disposition: fixed
    title: "The follower controller records a speed as sent before `set_speed` succeeded"
  - id: WR-03
    severity: warning
    disposition: fixed
    title: "The video's frame period is cached once, including the fallback default"
  - id: WR-04
    severity: warning
    disposition: fixed
    title: "The order guard removes itself permanently after ten restores in a minute"
  - id: IN-10
    severity: info
    disposition: open
    title: "The recorded GPU regression has an obvious first suspect in the generated options"
open: 4
total: 37
recorded: 2026-11-23T14:01:30.247Z
---

# Phase 02: Code Review Disposition

| Finding | Severity | Disposition | Source |
|---------|----------|-------------|--------|
| WR-15 | warning | fixed | 02-REVIEW-FIX.md |
| WR-16 | warning | fixed | 02-REVIEW-FIX.md |
| WR-17 | warning | fixed | 02-REVIEW-FIX.md |
| IN-07 | info | skipped | 02-REVIEW-FIX.md |
| IN-19 | info | open | - |
| IN-20 | info | open | - |
| IN-21 | info | open | - |
| WR-05 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-06 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-07 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-08 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-09 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-10 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-11 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-12 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-13 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-14 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-01 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-02 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-03 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-04 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-05 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-06 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-08 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-09 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-11 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-12 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-13 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-14 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-15 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-16 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-17 | info | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-01 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-02 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-03 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| WR-04 | warning | fixed | 02-REVIEW-FIX.md (not in the current review) |
| IN-10 | info | open | - (not in the current review) |

Dispositions: `open` (recorded, not yet triaged), `fixed`, `skipped`, `deferred`.
Set `deferred` by hand and put the reason in the Source cell; both are preserved. A `|` in the reason is kept as prose and escaped on the next run.
Re-running the gate keeps every row it can. A row the current review no longer reports is kept and its Source cell flagged, so a finding does not leave this record silently. ONE exception: when a finding id is REUSED by a different finding, the earlier decision cannot keep a row — the id is taken — and it is dropped. A RECORDED decision (anything but `open`) is named on the console when that happens; a row still at `open` is replaced silently, because `open` records no decision to lose.
