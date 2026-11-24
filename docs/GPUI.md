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
- Windows backdrops: `WindowBackgroundAppearance::{Blurred (acrylic), MicaBackdrop, MicaAltBackdrop, Opaque}`; theme backgrounds need alpha for the backdrop to show. gpui-pre-windows ignores Mica and Tabbed below build 22621 (Windows 11 22H2) and acrylic below 17763, so Sucrose reports the capability itself (`Capabilities.window_backdrop`, read with `RtlGetVersion`) and applies an unsupported stored backdrop as `Opaque`.
- Theme: register a `ThemeConfig` file with `ThemeRegistry::load_themes_from_str`, put its themes into `Theme::light_theme`/`dark_theme` and switch with `Theme::change(mode, None, cx)`; never edit `Theme::global_mut(cx).colors`. `Theme::sync_system_appearance` follows the OS; `cx.observe_window_appearance` reports an OS light/dark change as an event (no polling).
- A background image is a full-window `img(Arc<RenderImage>)` inside an `overflow_hidden` absolute layer, placed with an explicit rectangle and `ObjectFit::Fill` (None, Uniform and UniformToFill need a rectangle the element cannot compute on its own). A multi-frame `RenderImage` animates by itself, so swap in its first frame while `Window::is_visible()` is false and release replaced images with `cx.drop_image(image, None)`.
- Opening a window from automation steals focus and can receive the user's keystrokes. Use `focus: false` (the prototype's `--no-focus`) for automated runs, and capture with `PrintWindow` instead of the screen.
- rust-i18n (Portal, D-20): `rust_i18n::i18n!("locales", fallback = "en")` at the crate root; every `locales/*.<lang>.yml` file loads as that locale (`legacy.en.yml` and `sucrose.en.yml` both merge into `en`), flat dotted keys resolve, and `t!` also takes a runtime `&str` key. A locale file with no entries (comments only) makes every lookup return the key, so never commit one. A key defined in two files of the same locale is ambiguous; `cargo xtask localize` leaves the reworded legacy keys out (`REPLACED_KEYS`) and a test checks the files share no key.

- `gpui_kit::assets::Assets` embeds only the default component icons; an icon such as `ImageOff` or `PackageX` renders as nothing until it is registered. The Portal's `assets.rs` composes `Assets` with `icon_assets!(PageIcons, [...])`; add the icons a page needs there (the full `AllAssets` catalog is not embedded).
- `RenderImage` takes BGRA with premultiplied alpha, so a thumbnail from the `image` crate (RGBA) needs the red/blue swap and a premultiply. Evicted images are freed from the atlas with `cx.drop_image(image, None)` (`Some(window)` only while that window is being updated).
- `InputState` has no max length: `.validate(|text, _| text.chars().count() <= N)` refuses an edit that would pass it (a paste that is too long is refused whole), and `.clean_on_escape()` makes Escape clear the box. `InputState::set_value` emits no `Change`, so it is safe to call while rendering. The same validator can carry shared state (an `Rc<Cell<usize>>`) for a limit that follows the data, such as the digits of the last page number.
- A shortcut that must work wherever the focus is (Ctrl+F for the search box) goes through `cx.observe_keystrokes` on the root entity; an action bound on an element only fires while focus is inside it. Use `modifiers.secondary(cx)` for Ctrl on Windows and Linux and Cmd on macOS.
- `Empty` is `flex_1` and centers its content; for a block that starts 64 px below the top give it `.flex_none()` inside a column with a top padding. `EmptyHeader` is at most 24 rem wide, so a 24 px heading wraps; widen it with `.max_w()`. `gpui-component`'s `Pagination` has no jump field and no fixed slot scheme, so the legacy pagination bar is composed from `Button`s and an `Input`.
- `Skeleton::new()` defaults to full width and 16 px height; override with `.w()`, `.h()`, `.size_full()` and `.rounded()`.

- `App::open_url` and `App::write_to_clipboard` return nothing, so a failure cannot be observed. For the clipboard read the text back with `read_from_clipboard` and compare; for a link only log the attempt.
- Dialogs (`gpui_kit::component::WindowExt`): `window.open_dialog(cx, |dialog, window, cx| dialog.title(..).w(px(..)).child(..).footer(..))`; the builder can run again, so keep it cheap. A plain `Dialog` draws no buttons: compose a `DialogFooter` with a `Button` that calls `window.close_dialog(cx)`. Children go into a scroll container under the title and above the footer, and the surface is at most the viewport height minus 32 px; to keep another clearance give the body its own `max_h` and `overflow_y_scrollbar()` (`ScrollableElement`). `keyboard`, `overlay_closable` and `close_button` are fixed when the dialog opens, and anything that changes while it is open (a countdown) must live in an entity passed as `.footer(entity)` or `.child(entity)`.
- `Button` is a `ParentElement`: a clickable row with an icon and two lines of text is a ghost `Button` with children, which the keyboard reaches for free. `font_semibold()` and the other weights need `gpui_kit::component::StyledExt` in scope.
- `window.on_window_should_close(cx, f)` calls `f` from the platform, outside any entity update; reach the root entity through a `cx.weak_entity()` taken beforehand and `update` it. Returning false keeps the window open.

- Hover preview (library card): `.on_hover(|hovered, window, cx| …)` is a second listener next to `.hover(style)`, and an element takes only one `on_hover`. A multi-frame `RenderImage` in `img()` animates by itself, so the preview is a child above the static thumbnail and costs nothing once it is removed; drop its frames with `cx.drop_image(image, None)` when the hover ends or the page, search or setting changes, because `on_hover(false)` does not fire for a card that leaves the tree. A preview decode shares the thumbnail decode slots (`DecodeQueue::reserve`) and its result is checked against the state machine, so a result for a card the pointer left is released instead of shown. `observe_window_visibility` tells the view to stop when the window hides.

## Patterns

- State lives in entities; long work runs on `cx.background_executor()`, results come back with `this.update(cx, …)`.
- Event plumbing: `cx.subscribe` / `cx.subscribe_in` for component state events (`InputEvent`, `SliderEvent`, `SelectEvent`), `cx.listener` for click handlers.
- Stop a child's click from reaching a clickable parent card: wrap it in a div with `.on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())`.
- gpui-kit has AccessKit roles and a layout direction (RTL) setting; neither is tested in Sucrose yet. Arabic and Hebrew are among the 25 languages, so verify RTL early.