# Phase 03 — UI Review

**Audited:** 2026-11-24
**Baseline:** 03-UI-SPEC.md (approved 2026-11-23)
**Screenshots:** not captured (native GPUI app; no dev server or URL. Per project constraints the auditor does not launch sucrose.exe or sucrose-portal.exe. Existing PrintWindow captures named in SUMMARYs under target/ were not available to this audit, so visual findings are code-derived.)
**Interaction captures:** off (workflow.ui_interaction_capture is false)

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Copywriting | 3/4 | Connection, load-error, toast and banner strings are present in sucrose.en.yml and wired through tr(); the RTL and translated variants are not yet exercised |
| 2. Visuals | 3/4 | Card, header, pagination and dialog structure follow the layout contract in code; no runtime capture confirms focal point or hierarchy |
| 3. Color | 3/4 | Theme tokens are used; only 7 hard-coded black/white occurrences in card.rs (image overlays, allowed by UI-SPEC); the 60/30/10 split cannot be verified from code |
| 4. Typography | 3/4 | Sizes resolve to the declared scale (12, 16, 24 px plus text_xs/text_sm/text_2xl tokens); the text_xs/text_sm mapping to 12/14 px was not verified against the gpui-component source |
| 5. Spacing | 3/4 | Explicit px values are 4-multiples except the spec-declared card gap, 1 px strokes and 20 px divider height; spacing is mostly inherited from component defaults |
| 6. Experience Design | 3/4 | Loading (150 ms skeleton delay), connection banner states, Use disabled reasons, toasts and load-error state are all implemented; keyboard focus and reduced-motion paths are not verified |

**Overall: 18/24**

---

## Top 3 Priority Fixes

1. **RTL layout is unverified** — UI-SPEC E4 requires a mirrored (Arabic/Hebrew) capture, and 03-18-SUMMARY defers it to Phase 5. Shipping Phase 3 with no RTL check leaves the header, status bar and pagination direction unproven — **Fix:** run the release build without focus, capture one RTL pass, and record the result in this file or in the Phase 5 plan.
2. **No runtime capture of the final state** — none of the scores above is backed by a screenshot of the finished window at 900×600, 1440×900 or the Dark theme. Findings in Visuals, Color and Typography are therefore provisional — **Fix:** capture the default size, Dark and Acrylic states with PrintWindow (no synthetic input) and attach them under target/ with the SUMMARY reference.
3. **Hard-coded overlay colors outside the image-overlay rule** — card.rs:323 uses `black().opacity(0.35)` on the card spinner backdrop, which is not an image overlay and so falls outside the UI-SPEC exception (the spec allows hard-coded colors only on image overlays, `ui.md`). It will not follow the Dark/Light token change — **Fix:** replace it with a theme token (for example the page background at alpha) or document it as an overlay exception in UI-SPEC.

---

## Detailed Findings

### Pillar 1: Copywriting (3/4)
- Connection strings are present and keyed as UI-SPEC declares: `Portal.Connection.{Connecting,Lost,Failed,TryAgain,Unavailable}` in sucrose.en.yml; banner.rs maps each state to its key (banner.rs:19–37).
- Load-error, preparing, use-failed and setting-save-failed copy matches the spec wording (sucrose.en.yml lines for LibraryPage.LoadError, LibraryCard.UseFailed, Setting.SaveFailed).
- Gap: `Portal.GeneralSettingPage.PrivateLibrary.Busy` exists in the locale file but is not in the spec's new-keys list. Not a defect, but the contract should be updated (docs must stay true to code).
- Gap: the spec's `Portal.Setting.SaveFailed` path is shown as a toast, but i18n.rs:284 keeps a literal English string at a second location. Check that the literal is not user-visible in another language build.

### Pillar 2: Visuals (3/4)
- Structure follows the layout contract: header row 48 px (header.rs:166, 213), search 320 px shrinking to 200 px (header.rs:231–232), status bar and pagination as specified.
- Card hierarchy: title over description over a 56 px gradient strip, pills above the strip (card.rs:214–342). Matches the contract.
- No runtime capture; focal point and visual balance cannot be confirmed from code.

### Pillar 3: Color (3/4)
- Hard-coded colors found (grep of `rgb|rgba|hsl|hex|black()|white()`):
  - card.rs:223–224 gradient black at 30% to 75% — allowed (image overlay).
  - card.rs:232, 242 white and white at 82% — allowed (image overlay text).
  - card.rs:323 black at 35% on the card spinner backdrop — **not** an image overlay (see Fix 3).
  - theme.rs:188–191 hex values are in the test and the theme fixture only; the production palette comes from ThemeConfig.
- Accent usage (theme.primary) is limited to the Selected pill, the spinner and the focus/hover paths, which matches the reserved list; the 10% share cannot be measured without a capture.
- Backdrop alpha (0.78 floor for Muted text contrast) is not asserted in code; the 4.5:1 contrast requirement is untested.

### Pillar 4: Typography (3/4)
- Sizes in use: `text_size(px(16.))` x2, `text_size(px(12.))` x2, `text_size(px(24.))` x1, plus theme tokens `text_xs()` x11, `text_sm()` x8, `text_2xl()` x1, `font_semibold()` x4.
- The spec allows exactly four sizes (14, 12, 16, 24) and two weights (400, 600). The token-to-size mapping (text_xs, text_sm) was not checked against gpui-component 0.10.0; if text_xs is 11 px the scale is violated.
- Card title/description use 16 px and 12 px with 20 and 16 px line heights (card.rs:229–241), consistent with the Heading and Label roles.

### Pillar 5: Spacing (3/4)
- Explicit spacing values found: gap 24 (1 use), height 24 (2), height 20 (1, divider), width 72 (1), width 40 (1), height 32 (1), width 1 (border). Values are 4-multiples except the 20 px divider and the 72/40 px fixed widths, which should be mapped to spec tokens or noted as exceptions.
- Most spacing comes from component defaults; a rhythm check needs a capture.
- Card gap (AdaptiveMargin, 5–25 px) is the declared exception and is user-driven.

### Pillar 6: Experience Design (3/4)
- Loading: skeleton after a 150 ms delay (library/mod.rs:43–44, 445–452), skeleton cards in card.rs:350.
- Connection: banner for Connecting, Lost, Failed with a Try Again action (banner.rs); reconnect cap via MAX_RECONNECTS (ipc.rs:87–91).
- Use: spinner on the clicked card, Selected pill follows the service broadcast (library/mod.rs:316–334), failure toast — matches D-11.
- Empty and error states: LoadError title/body and Try Again/Change Location present; the spider-web empty copy is keyed in i18n.rs:61–64.
- Not verified from code: keyboard focus ring (2 px accent), reduced-motion handling, arrow-key card navigation and accessible labels for icon-only controls. These are required by UI-SPEC's Interaction Contract and need a runtime check.

---

## Files Audited
- .planning/phases/03-m2-service-ipc-and-portal-connected/03-UI-SPEC.md
- .planning/phases/03-m2-service-ipc-and-portal-connected/03-CONTEXT.md
- apps/sucrose-portal/src/{app,banner,header,toast,status_bar,ipc,i18n,main,background,atlas,assets}.rs
- apps/sucrose-portal/src/library/{card,grid,layout,mod,pagination,preview,thumbs}.rs
- apps/sucrose-portal/src/settings/{general,personal,row,mod}.rs
- apps/sucrose-portal/src/dialogs/{about,help,mod,move_library,tray_hint}.rs
- apps/sucrose-portal/locales/sucrose.en.yml
- docs/GPUI.md
- SUMMARY references in 03-08, 03-14, 03-17, 03-18, 03-20 (captures named under target/; not present for this audit)
