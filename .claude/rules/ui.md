---
paths:
  - "apps/sucrose-portal/**"
  - "exp/GPUI/**"
---

# UI (GPUI / gpui-kit)

- Read `docs/GPUI.md` before using an unfamiliar component; it lists known gotchas. When the docs are thin, read the pinned crate's source in the Cargo registry.
- Sucrose has its own design language that looks the same on every OS; use theme tokens, not hard-coded colors, except on image overlays.
- Never do I/O, decoding or network work on the UI thread: use `cx.background_executor()` and update entities when done. Decode thumbnails at card size (see `docs/GPUI.md`).
- Every user-visible string goes through localization (rust-i18n, D-015); keep keys compatible with the 22 existing languages, and remember Arabic and Hebrew are RTL.
- The UI only talks to the service over IPC; it never loads engines.
- Verify visual changes by running a release build without focus and capturing the window (the `port-feature` skill describes how). Record new gotchas in `docs/GPUI.md`.
