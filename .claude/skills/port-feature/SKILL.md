---
name: port-feature
description: Port one feature of the legacy C# Sucrose app (src/) to the Rust workspace and keep docs/FEATURES.md, docs/DECISIONS.md and docs/MEASUREMENTS.md in sync. Use when asked to port, re-implement or reach parity with something the C# app does.
when_to_use: "Asked to port, re-implement or match something the old C# app does, also in Turkish (for example: eski uygulamadaki X'i taşı)."
---

# Port a feature

Run the port inside a GSD command (`/gsd-execute-phase` for planned phase work, `/gsd-quick` for a small port) unless the maintainer asks for a direct change (`CLAUDE.md` → Planning).

1. **Locate.** Find the feature's row in `docs/FEATURES.md` (add one if missing) and its notes in `docs/legacy/`.
2. **Read the C#.** Open the implementation under `src/` (`src/CLAUDE.md` explains the layout; resolve the using-aliases at the top of each file). Write down settings keys and defaults, file formats, Windows APIs, timings and edge cases. If `docs/legacy/` was missing something, add it there.
3. **Design.** Pick the crate (`docs/ARCHITECTURE.md`). OS-specific parts go into `sucrose-platform` behind a capability. Check `docs/DECISIONS.md`; if a new choice is needed, add a Proposed entry and ask the maintainer before anything expensive to undo.
4. **Implement in small steps.** Formats and rules first, with tests. Compatibility tests use fixtures from real files with personal data removed.
5. **Verify.**
   - `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
   - UI: run a release build without focus and capture it (below), then look at the image.
   - Wallpaper behavior: run the service and check it on the desktop; measure RAM/CPU in a release build.
6. **Update the docs in the same change.** `FEATURES.md` row (status, per-OS notes, improvements, behavior changes), `DECISIONS.md` if a choice was made, `GPUI.md` for UI lessons, `MEASUREMENTS.md` for notable numbers. Requirement and phase status in `.planning/` is updated by the GSD workflow.
7. **Report** to the maintainer (in Turkish): what changed, what was verified and how, what was not verified.

## Capturing a window without stealing focus (Windows)

Start the release build with focus disabled (the prototype uses `--no-focus`), wait until it has rendered, then run:

```
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .claude/skills/port-feature/capture.ps1 -Process <exe name without .exe> -Out <png path>
```

From WSL pass Windows paths (`wslpath -w`). The script uses `PrintWindow`, so the window does not need to be in front, and prints the window size, working set and private memory. Stop the app afterwards (`taskkill /IM <exe>.exe /F`). Never move the mouse or send keys to the maintainer's desktop.
