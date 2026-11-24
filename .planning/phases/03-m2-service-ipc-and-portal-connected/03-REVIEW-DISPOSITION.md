---
phase: 03
review: 03-REVIEW.md
titles: json
findings:
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
  - id: IN-02
    severity: info
    disposition: deferred
    title: "Endpoint hardening is Windows-only while Linux gets an abstract socket"
  - id: IN-03
    severity: info
    disposition: deferred
    title: "Duplicated code"
  - id: IN-04
    severity: info
    disposition: deferred
    title: "Stale `#[allow(dead_code)]` with outdated reasons"
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
  - id: IN-10
    severity: info
    disposition: deferred
    title: "An invalid `Version` is read as current and then overwritten"
  - id: IN-11
    severity: info
    disposition: deferred
    title: "Background image path check accepts NUL and UNC paths"
  - id: IN-12
    severity: info
    disposition: deferred
    title: "Timing assertion in the memory integration test is tight"
---

# Phase 03: Code Review Disposition

| Finding | Severity | Disposition | Source |
|---------|----------|-------------|--------|
| CR-01 | critical | fixed | 03-29 |
| CR-02 | critical | fixed | 03-30 |
| WR-01 | warning | fixed | 03-31 |
| WR-02 | warning | fixed | 03-32 |
| WR-03 | warning | fixed | 03-31 |
| WR-04 | warning | fixed | 03-33 |
| WR-05 | warning | fixed | 03-35 |
| WR-06 | warning | fixed | 03-36 |
| WR-07 | warning | fixed | 03-31 |
| WR-08 | warning | fixed | 03-34 |
| IN-01 | info | deferred | Same-user only (single-SID pipe DACL); connection cap and first-read deadline belong with the next IPC hardening pass |
| IN-02 | info | deferred | Latent until the Linux service exists (EventLoop::new fails there today); fix before Linux support is enabled |
| IN-03 | info | deferred | Fixed in part by 03-29 (one decode_jpeg_scaled for thumbnails and the background image, with CR-01); the bin_name duplicate remains: no behavior impact, next Portal cleanup |
| IN-04 | info | deferred | Lint hygiene only, no behavior impact; next Portal cleanup |
| IN-05 | info | deferred | Test-only field, no behavior impact; next lifecycle change |
| IN-06 | info | deferred | Deliberate documented D-26 fallback; remove after the LoadSource path has run longer |
| IN-07 | info | deferred | Log hygiene (paths in info lines), no functional impact; with the logging and backup work |
| IN-08 | info | deferred | Both derivations agree today; move to a service-sent resolved folder with the full settings pages (UI-04) |
| IN-09 | info | deferred | Rare transient toast in a refresh/SetStartup race; fix with the full settings pages (UI-04) |
| IN-10 | info | deferred | Needs a hand-edited or damaged Version; fix with the next settings format change |
| IN-11 | info | deferred | Same-user settings and IPC only; share one path validator when the background image picker grows (UI-04) |
| IN-12 | info | deferred | Test robustness; passes today; widen the deadline if it flakes on CI |
