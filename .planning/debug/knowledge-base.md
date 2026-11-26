# GSD Debug Knowledge Base

Resolved debug sessions. Used by `gsd-debugger` to surface known-pattern hypotheses at the start of new investigations.

---

## cef-close-shutdown-dialog — closing a CEF wallpaper opened the Windows Shut Down dialog
- **Date:** 2026-11-26
- **Error patterns:** Shut Down Windows dialog, Windows'u Kapat, theme switch, CEF browser did not close in time, close timeout 5 s, renderer exit 0xffff7003, 0xc0000005, the renderer stopped missing, WM_CLOSE, Progman, DoClose, CloseBrowser
- **Root cause(s):** CEF's default DoClose (false) for a windowed browser makes CloseHostWindow post WM_CLOSE to GetAncestor(browser, GA_ROOT); our life span handler did not override DoClose; the wallpaper browser is a child of the desktop surface under Progman (raised layout), so the WM_CLOSE reached Progman (its WM_CLOSE is the Shut Down Windows dialog); nothing destroyed the browser window, so the close timed out and the renderer crashed in cef::shutdown
- **Fix:** SpanHandler::do_close answers 1 for the wallpaper browser (popups keep the default); CefHost::close destroys the browser's own window once DoClose came (or after 3 s), pumps until OnBeforeClose or 5 s; window.close() from the page is ignored with a WARN; popup OnBeforeClose no longer marks the wallpaper closed (commit 5758edd0)
- **Files changed:** crates/sucrose-engine-cef/src/browser.rs, docs/FEATURES.md, docs/ARCHITECTURE.md, .planning/STATE.md
- **Why not caught:** no gate existed for this class: no test called the life span handler through CEF's function table, review did not check the DoClose contract, and no desktop run switched away from a CEF wallpaper before the 04-20 maintainer check
- **Recurrence guard:** crates/sucrose-engine-cef/src/browser.rs imp::tests::the_life_span_handler_never_lets_cef_close_the_desktop_window and tests::the_wallpaper_browser_never_lets_cef_send_its_close_message (plus the two close_step timing tests); KB pattern: any window hosted under Progman/WorkerW must never let a library send WM_CLOSE to its GA_ROOT
---

