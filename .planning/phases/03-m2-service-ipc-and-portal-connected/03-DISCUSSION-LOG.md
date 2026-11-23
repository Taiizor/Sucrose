# Phase 3: M2 — Service, IPC and Portal connected - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-11-23
**Phase:** 03-m2-service-ipc-and-portal-connected
**Areas discussed:** Settings and migration, Side by side with C#, Service–Portal lifecycle, Portal scope in Phase 3, Theme mode and backdrop, Fullscreen rule and Use, Hover preview and memory, Use transition

The discussion was held in Turkish; options are summarized here in English.

---

## Settings and migration

### Where do the new settings live?

| Option | Description | Selected |
|--------|-------------|----------|
| Single file in Setting/ | One versioned file such as Setting/Sucrose.json; layout unchanged; C# ignores unknown files | ✓ |
| New folder | For example Config/Settings.json; separates legacy and new but adds a top-level folder |  |
| One file per section | Several new files mirroring the legacy split; versioning spread over files |  |

**User's choice:** Single file in Setting/

### Format of the new settings file?

| Option | Description | Selected |
|--------|-------------|----------|
| JSON | serde_json already in the workspace; readable; no comments | ✓ |
| TOML | Comments and hand-editing; new dependency; nested maps less natural |  |
| You decide | Planning picks; readable, versioned, atomic |  |

**User's choice:** JSON

### When and how often does migration run?

| Option | Description | Selected |
|--------|-------------|----------|
| Once, when the new file is missing | Read legacy once, never again; later C# changes are not picked up | ✓ |
| Once + manual re-import | Same, plus regenerating from legacy later (UI in Phase 5) |  |
| Fill missing keys on every start | Keys missing from the new file are filled from legacy each start |  |

**User's choice:** Once, when the new file is missing

### How are keys of features not yet ported stored?

| Option | Description | Selected |
|--------|-------------|----------|
| All typed now | Every key typed with default, range and enum names; fixture-proven | ✓ |
| Phase 3 keys typed, rest raw | Others kept as raw JSON and converted in their phases |  |
| You decide | Planning picks; criterion 2 test must cover every key |  |

**User's choice:** All typed now

### Which legacy values are NOT migrated? (multi-select)

| Option | Description | Selected |
|--------|-------------|----------|
| GitHub PAT (Objectionable.json) | Plaintext secret; GitHub removed by D-020 |  |
| StoreServerType | GitHub/Soferity choice; only Soferity remains |  |
| Aurora.json AppProcessName | Runtime state, not a setting (ARCH-04) |  |
| Kernel.json / System.json | Write-only system information regenerated at runtime | ✓ |

**User's choice:** Kernel.json / System.json

### PAT and AppProcessName conflict with D-017 / ARCH-04: how do they sit in the new file?

| Option | Description | Selected |
|--------|-------------|----------|
| Separate 'legacy' section | Kept but unused by the app; easy to drop later |  |
| Normal fields as-is | Exact parity; needs a note for ARCH-04 and D-017 |  |
| Do not migrate them after all | PAT, StoreServerType and AppProcessName become documented exclusions too | ✓ |

**User's choice:** Do not migrate them after all

**Notes:** First exclusion answer selected only Kernel/System; the follow-up turned PAT, StoreServerType and AppProcessName into exclusions as well. Final exclusions: Donate.json, PAT, StoreServerType, AppProcessName, Kernel.json, System.json.

---

## Side by side with C#

### LIB-09 move while the C# app still points to the old location

| Option | Description | Selected |
|--------|-------------|----------|
| Move like legacy + warning | Legacy move; the confirmation warns when a C# install is detected | ✓ |
| Copy, do not delete, while C# exists | Keeps C# working; doubles disk use; behavior change |  |
| Move like legacy, no warning | Exact parity |  |

**User's choice:** Move like legacy + warning

### Autostart coexistence with the C# entry (maintainer has RunStartup = 3)

| Option | Description | Selected |
|--------|-------------|----------|
| Own Rust entry, never touch C#, warn | Migration creates nothing; entry only when turned on in Rust; warning when a C# entry exists; 2/3 shown as later | ✓ |
| Auto-enable on migration | Register a Normal entry on first run when the migrated value is set |  |
| Same Run value name (take over) | Overwrite the C# Normal entry so only one starts |  |

**User's choice:** Own Rust entry, never touch C#, warn

### How are corrupt themes deleted (shared library)?

| Option | Description | Selected |
|--------|-------------|----------|
| Move to the OS recycle bin | Recoverable if the Rust parser is stricter than Newtonsoft; improvement | ✓ |
| Delete permanently like legacy | Exact parity, parser leniency proven by fixtures |  |
| Delete permanently only when no C# install | Hide and log while C# exists |  |

**User's choice:** Move to the OS recycle bin

### Where do showcase themes come from in Phase 3?

| Option | Description | Selected |
|--------|-------------|----------|
| Showcase/ in the data folder | Legacy location; Rust installer in Phase 5 | ✓ |
| Showcase/ next to the exe | Ships with the app; changes the legacy layout |  |
| Both, data folder first | Flexible, two sources to maintain |  |

**User's choice:** Showcase/ in the data folder

---

## Service–Portal lifecycle

### Who writes the new settings file?

| Option | Description | Selected |
|--------|-------------|----------|
| Single writer: the service | Portal gets a snapshot over IPC and sends changes; service writes atomically and broadcasts | ✓ |
| Both write the file | Atomic rename plus cross-process lock |  |
| Ownership by section | Portal writes UI keys, service the rest |  |

**User's choice:** Single writer: the service

### How does Sucrose start; what if the Portal opens without a running service?

| Option | Description | Selected |
|--------|-------------|----------|
| Legacy rule + Portal starts the service | Legacy Launcher rule; second sucrose opens the Portal; Portal starts the service when needed | ✓ |
| Portal starts the service only on Use | Settings read-only without the service |  |
| Portal always through sucrose | Portal hands off to the service and exits |  |

**User's choice:** Legacy rule + Portal starts the service

### When does the service exit while no wallpaper runs?

| Option | Description | Selected |
|--------|-------------|----------|
| Stays while the tray is visible, else exits | Legacy-like with tray; exits when hidden, idle and no Portal | ✓ |
| Always stays until Exit | Invisible idle process possible |  |
| Always exits without wallpaper and Portal | Leanest; departs from legacy |  |

**User's choice:** Stays while the tray is visible, else exits

### Phase 3 tray menu and Exit semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Open Sucrose + exit per AppExit | Close Applications closes everything; Exit closes only the tray | ✓ |
| Open Sucrose + always close everything | AppExit ignored until Phase 5 |  |
| Only Exit (close everything) | M1 menu; Exit also closes the Portal |  |

**User's choice:** Open Sucrose + exit per AppExit

---

## Portal scope in Phase 3

### How do header actions of later phases appear?

| Option | Description | Selected |
|--------|-------------|----------|
| Only working ones are shown | Search, light/dark, Premium, Help/About; others arrive with their features | ✓ |
| All shown, unavailable ones disabled | Full legacy layout with disabled buttons |  |

**User's choice:** Only working ones are shown

### Where are Phase 3 settings edited?

| Option | Description | Selected |
|--------|-------------|----------|
| Real settings shell, only these keys | General and Personal pages with the Phase 3 keys in legacy places | ✓ |
| Library toolbar + temporary page | Sort and page size in a toolbar, the rest on a temporary page |  |

**User's choice:** Real settings shell, only these keys

### What does Use do on a non-Video theme card?

| Option | Description | Selected |
|--------|-------------|----------|
| Card visible, Use disabled + reason | Library complete; selection and running wallpaper unchanged | ✓ |
| Use works, service says unsupported | Selection saved; the running wallpaper stops |  |

**User's choice:** Card visible, Use disabled + reason

### What does a theme card offer in Phase 3?

| Option | Description | Selected |
|--------|-------------|----------|
| No menu, left click = Use, badge on incompatible | Context menu arrives whole with LIB-05 | ✓ |
| Menu with only Use and Find | Partial menu now |  |

**User's choice:** No menu, left click = Use, badge on incompatible

### What does the status bar memory value show?

| Option | Description | Selected |
|--------|-------------|----------|
| Total of all Sucrose processes | Service + renderers + Portal, breakdown in tooltip; reported by the service over IPC | ✓ |
| Portal only, like legacy | Exact parity |  |

**User's choice:** Total of all Sucrose processes

### How are Phase 3 strings handled?

| Option | Description | Selected |
|--------|-------------|----------|
| English only, through rust-i18n | Keys match the legacy XAML keys; other languages in Phase 5 | ✓ |
| Hard-coded English strings | Faster now, against ui.md |  |

**User's choice:** English only, through rust-i18n

---

## Theme mode and backdrop

### How are theme mode and backdrop stored?

| Option | Description | Selected |
|--------|-------------|----------|
| Two separate settings | ThemeMode {Light, Dark, System} and Backdrop {None, Mica, Acrylic, Tabbed}; Auto → System + Mica | ✓ |
| Keep the legacy coupling | Auto means both backdrop and follow OS |  |

**User's choice:** Two separate settings

### How do unsupported backdrop options appear?

| Option | Description | Selected |
|--------|-------------|----------|
| Visible but disabled + reason | Capability-driven, like legacy on Windows 10 | ✓ |
| Unsupported options hidden | Backdrop row absent on macOS and Linux |  |

**User's choice:** Visible but disabled + reason

---

## Fullscreen rule and Use

### What happens on Use while the fullscreen rule paused/closed the wallpaper?

| Option | Description | Selected |
|--------|-------------|----------|
| Disabled with status suffix like legacy |  (Paused) / (Closed); state from the service over IPC | ✓ |
| Apply and start paused | Selection saved immediately; behavior change |  |

**User's choice:** Disabled with status suffix like legacy

---

## Hover preview and memory

### How is the hover preview GIF played?

| Option | Description | Selected |
|--------|-------------|----------|
| Card size, one card, released on leave | Only the hovered card animates; frames dropped on leave | ✓ |
| Short-lived cache of a few previews | Last few previews kept |  |

**User's choice:** Card size, one card, released on leave

### How is UI-07 memory measured?

| Option | Description | Selected |
|--------|-------------|----------|
| Real library + synthetic large library | 108 real themes and ~500 synthetic, paging rounds and hover | ✓ |
| Real library only | 108 themes, 4 pages |  |

**User's choice:** Real library + synthetic large library

---

## Use transition

### How does switching from one video theme to another work?

| Option | Description | Selected |
|--------|-------------|----------|
| New file into the same renderer | New appended IPC message; no restart or static frame | ✓ |
| Restart like legacy | Close and restart renderers |  |

**User's choice:** New file into the same renderer

---

## Claude's Discretion

- Settings file name, internal structure and atomic-write details
- Portal ↔ service messages, endpoint and authentication
- Run value name and detection of legacy startup entries
- Recycle-bin implementation (platform crate or a maintained crate)
- Library scan threading, search index, progressive card display
- Thumbnail decode size, cache cap, `drop_image`
- Status bar texts, Help/About contents, tray hint dialog, command-line page arguments
- Live OS theme following, the fullscreen scope field from Phase 2 D-05, which optional modules exist in Phase 3

## Deferred Ideas

- Re-importing legacy settings on demand
- Taking over the C# startup entry when the Rust installer replaces the C# app (Phase 5)
- Search telemetry (Phase 5)
- Card context menu and the incompatible-theme Update action (LIB-05, updater, Phase 5)
- Header actions Create Wallpaper, Wallpaper Cycling, Display Preferences (their phases)
- Applying the migrated Culture (UI-06, Phase 5)
