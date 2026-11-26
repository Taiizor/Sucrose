---
schema_version: 1
open_count: 6
waived_count: 0
fixed_count: 0
total_count: 6
last_updated: 2026-11-26T22:42:41.123Z
---

# Broken Windows Ledger

> Cross-phase defect register. With `workflow.windows_enforce` enabled, `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 2 | unrun-verify | crates/sucrose-platform/src/os_windows/surface.rs |  | Plan 02-06 Task 2 human check: surface fills the monitor at 150 % display scale (DESK-06) not yet run | open |  | 2026-11-22T14:30:11.335Z |  |
| 2 | 03 | unrun-verify | apps/sucrose-portal/src/dialogs/move_library.rs |  | Plan 22 human-check: move a synthetic library from the release Portal (confirmation, progress text, reload, non-empty refusal) is left for the end of the phase | open |  | 2026-11-24T11:28:21.457Z |  |
| 3 | 04 | unrun-verify | crates/sucrose-engine-cef/src/browser.rs |  | DevTools window from Inspect, F12, Ctrl+Shift+I and OpenDevTools not seen live (needs input or the 04-17 Portal button); covered by key_action and queue tests | open |  | 2026-11-26T03:43:51.646Z |  |
| 4 | 04 | deviation | crates/sucrose-engine-web/src/js.rs |  | Compatible calls follow the legacy code run order, not CompatibleKey::ALL; ThemeType System is sent as Dark until the OS theme is read | open |  | 2026-11-26T04:16:32.376Z |  |
| 5 | 4 | stub | crates/sucrose-engine-webview2/src/engine.rs |  | WebView2 pump only logs pause, volume, loop, shuffle and stretch commands until 04-24 applies them | open |  | 2026-11-26T22:42:40.983Z |  |
| 6 | 4 | stub | crates/sucrose-engine-webview2/src/host.rs |  | WebView2 answers a Range request with the whole file and 200 until 04-23 adds partial replies | open |  | 2026-11-26T22:42:41.123Z |  |

````json
[
  {
    "id": 1,
    "kind": "unrun-verify",
    "phase": "2",
    "file": "crates/sucrose-platform/src/os_windows/surface.rs",
    "line": null,
    "description": "Plan 02-06 Task 2 human check: surface fills the monitor at 150 % display scale (DESK-06) not yet run",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-22T14:30:11.335Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 2,
    "kind": "unrun-verify",
    "phase": "03",
    "file": "apps/sucrose-portal/src/dialogs/move_library.rs",
    "line": null,
    "description": "Plan 22 human-check: move a synthetic library from the release Portal (confirmation, progress text, reload, non-empty refusal) is left for the end of the phase",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-24T11:28:21.457Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 3,
    "kind": "unrun-verify",
    "phase": "04",
    "file": "crates/sucrose-engine-cef/src/browser.rs",
    "line": null,
    "description": "DevTools window from Inspect, F12, Ctrl+Shift+I and OpenDevTools not seen live (needs input or the 04-17 Portal button); covered by key_action and queue tests",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-26T03:43:51.646Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 4,
    "kind": "deviation",
    "phase": "04",
    "file": "crates/sucrose-engine-web/src/js.rs",
    "line": null,
    "description": "Compatible calls follow the legacy code run order, not CompatibleKey::ALL; ThemeType System is sent as Dark until the OS theme is read",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-26T04:16:32.376Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 5,
    "kind": "stub",
    "phase": "4",
    "file": "crates/sucrose-engine-webview2/src/engine.rs",
    "line": null,
    "description": "WebView2 pump only logs pause, volume, loop, shuffle and stretch commands until 04-24 applies them",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-26T22:42:40.983Z",
    "resolved_at": null,
    "milestone": null
  },
  {
    "id": 6,
    "kind": "stub",
    "phase": "4",
    "file": "crates/sucrose-engine-webview2/src/host.rs",
    "line": null,
    "description": "WebView2 answers a Range request with the whole file and 200 until 04-23 adds partial replies",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-11-26T22:42:41.123Z",
    "resolved_at": null,
    "milestone": null
  }
]
````
