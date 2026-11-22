---
schema_version: 1
open_count: 1
waived_count: 0
fixed_count: 0
total_count: 1
last_updated: 2026-11-22T14:30:11.335Z
---

# Broken Windows Ledger

> Cross-phase defect register. With `workflow.windows_enforce` enabled, `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 2 | unrun-verify | crates/sucrose-platform/src/os_windows/surface.rs |  | Plan 02-06 Task 2 human check: surface fills the monitor at 150 % display scale (DESK-06) not yet run | open |  | 2026-11-22T14:30:11.335Z |  |

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
  }
]
````
