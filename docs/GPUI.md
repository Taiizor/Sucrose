# GPUI notes

Practical notes for the UI (`gpui-kit`). GPUI documentation is thin; read the source of the pinned crates in the Cargo registry (`$CARGO_HOME/registry/src/…/gpui-kit-*`, `gpui-component-*`, `gpui-base-*`, `gpui-pre-*`) when an API is unclear.

## Versions

- `gpui-kit 0.10` re-exports GPUI and pins a Zed snapshot published as `gpui-pre 0.3.12`. This is **not** the official `gpui` 0.2.x on crates.io. Always depend on `gpui-kit` only, with an exact version, and upgrade deliberately (read its changelog, rebuild the prototype, compare screenshots).
- `use gpui_kit::*;` is GPUI. Components: `gpui_kit::component::…`. Icons: `gpui_kit::assets::IconName` (Lucide). `gpui_kit::open_window` mounts the base `Root`; call `gpui_kit::init(cx)` first.

## Prototype

`exp/GPUI/Sucrose.Portal` is a throwaway experiment that partly recreates the current Portal (library, store, settings, status bar) with real data, read-only. It is not the base of `apps/sucrose-portal`. It never evicts decoded thumbnails or releases them from GPUI's image atlas, so memory keeps growing while paging through the store: the real Portal needs a bounded thumbnail cache and must release images it no longer shows (check the `drop_image` API in the pinned GPUI). Flags: `--page=library|store|settings`, `--section=general|personal|performance|other`, `--light`, `--no-focus`. Measured: window appears in ~0.3 s, ~90 MB on the settings page, ~142 MB with 30 thumbnails (release build, Windows).

## Gotchas found so far

- `Tab::icon()` makes an icon-only tab. Use `.prefix(icon)` together with `.label()`.
- `Select`'s outer container takes the full row width and collapses `flex_1` siblings (text renders one letter per line). Wrap it in a fixed-width `div`.
- Inside scroll containers give content a definite width when flex children must share space.
- Changing `Theme::global_mut(cx)` colors (`primary`, `slider_bar`, …) does not reach components that read semantic tokens (Switch, Slider). A real accent color needs a theme config, not direct color edits.
- `img(path)` decodes at full resolution and keeps it cached. For thumbnails decode yourself on a background thread at card size (`jpeg-decoder` DCT scaling, limited concurrency), build a `RenderImage` (BGRA), and pass `Arc<RenderImage>`. This took the prototype from ~230 MB to ~142 MB.
- Windows backdrops: `WindowBackgroundAppearance::{Blurred (acrylic), MicaBackdrop, MicaAltBackdrop, Opaque}`; theme backgrounds need alpha for the backdrop to show.
- Opening a window from automation steals focus and can receive the user's keystrokes. Use `focus: false` (the prototype's `--no-focus`) for automated runs, and capture with `PrintWindow` instead of the screen.

## Patterns

- State lives in entities; long work runs on `cx.background_executor()`, results come back with `this.update(cx, …)`.
- Event plumbing: `cx.subscribe` / `cx.subscribe_in` for component state events (`InputEvent`, `SliderEvent`, `SelectEvent`), `cx.listener` for click handlers.
- Stop a child's click from reaching a clickable parent card: wrap it in a div with `.on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())`.
- gpui-kit has AccessKit roles and a layout direction (RTL) setting; neither is tested in Sucrose yet. Arabic and Hebrew are among the 22 languages, so verify RTL early.