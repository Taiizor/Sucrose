---
phase: 03
review: 03-REVIEW.md
titles: json
findings:
  - id: CR-01
    severity: critical
    disposition: open
    title: "Progressive JPEG bypasses the decode limits (memory bomb from a theme thumbnail or background image)"
  - id: CR-02
    severity: critical
    disposition: open
    title: "A transient read error makes the settings store restore an older backup, or migrate and overwrite, a good `Sucrose.json`"
  - id: WR-01
    severity: warning
    disposition: open
    title: "MaintainLibrary is accepted while a library move runs"
  - id: WR-02
    severity: warning
    disposition: open
    title: "Portal requests are still handled once the service is quitting, so a move can be cut off by process exit"
  - id: WR-03
    severity: warning
    disposition: open
    title: "The library move does not fully quiesce the fleet (monitor changes are dropped, a queued respawn can reopen the old folder)"
  - id: WR-04
    severity: warning
    disposition: open
    title: "A Portal dropped for a full queue keeps its pipe open, the Portal is never told, and the service may never exit"
  - id: WR-05
    severity: warning
    disposition: open
    title: "Turning startup off can fail half-way and leave the setting and the registry disagreeing"
  - id: WR-06
    severity: warning
    disposition: open
    title: "`synth-library` guard against the real data folder is lexical and bypassed by a verbatim path"
  - id: WR-07
    severity: warning
    disposition: open
    title: "Theme folder names bypass the control-character filter on the tray and log path"
  - id: WR-08
    severity: warning
    disposition: open
    title: "A queued restart of the old file can be taken for the first frame of the new source"
  - id: IN-01
    severity: info
    disposition: open
    title: "No limit on connections and no read timeout on the first message"
  - id: IN-02
    severity: info
    disposition: open
    title: "Endpoint hardening is Windows-only while Linux gets an abstract socket"
  - id: IN-03
    severity: info
    disposition: open
    title: "Duplicated code"
  - id: IN-04
    severity: info
    disposition: open
    title: "Stale `#[allow(dead_code)]` with outdated reasons"
  - id: IN-05
    severity: info
    disposition: open
    title: "`StartActions.start_wallpaper` is read only by tests"
  - id: IN-06
    severity: info
    disposition: open
    title: "Generality without instantiation, the `Restart` use transition"
  - id: IN-07
    severity: info
    disposition: open
    title: "Paths in info-level log lines, against the \"values only\" rule"
  - id: IN-08
    severity: info
    disposition: open
    title: "The Portal re-derives the default library folder instead of using the service's answer"
  - id: IN-09
    severity: info
    disposition: open
    title: "A refresh answer can cause a false \"could not be saved\" toast"
  - id: IN-10
    severity: info
    disposition: open
    title: "An invalid `Version` is read as current and then overwritten"
  - id: IN-11
    severity: info
    disposition: open
    title: "Background image path check accepts NUL and UNC paths"
  - id: IN-12
    severity: info
    disposition: open
    title: "Timing assertion in the memory integration test is tight"
---

# Phase 03: Code Review Disposition

| Finding | Severity | Disposition | Source |
|---------|----------|-------------|--------|
| CR-01 | critical | open | - |
| CR-02 | critical | open | - |
| WR-01 | warning | open | - |
| WR-02 | warning | open | - |
| WR-03 | warning | open | - |
| WR-04 | warning | open | - |
| WR-05 | warning | open | - |
| WR-06 | warning | open | - |
| WR-07 | warning | open | - |
| WR-08 | warning | open | - |
| IN-01 | info | open | - |
| IN-02 | info | open | - |
| IN-03 | info | open | - |
| IN-04 | info | open | - |
| IN-05 | info | open | - |
| IN-06 | info | open | - |
| IN-07 | info | open | - |
| IN-08 | info | open | - |
| IN-09 | info | open | - |
| IN-10 | info | open | - |
| IN-11 | info | open | - |
| IN-12 | info | open | - |
