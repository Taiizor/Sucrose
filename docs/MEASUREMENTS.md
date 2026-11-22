# Measurements

RAM and CPU numbers for user-visible changes, measured on release builds and compared with the C# app where possible (`CLAUDE.md`, non-negotiable 6). Add a row for every notable measurement; MEAS-01 and MEAS-02 in `.planning/REQUIREMENTS.md` depend on this table.

| Date | Scenario | C# app | Rust | Notes |
|---|---|---|---|---|
| 2026-11-21 | Portal prototype, settings page | – | ~90 MB working set | GPUI prototype, Windows, release build |
| 2026-11-21 | Portal prototype, 30 thumbnails | 173–201 MB (README screenshots, not same machine) | ~142 MB | thumbnails decoded at 480x270 |
| 2026-11-21 | Portal prototype, window appears | – | ~0.3 s | |
