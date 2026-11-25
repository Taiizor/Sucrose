//! The Portal's theme: the Sucrose `ThemeConfig` (light and dark), the stored theme mode (Light, Dark or System, D-21)
//! and the window backdrop (D-22).
//!
//! The colors come from the embedded `themes/sucrose.json`; they are never edited through `Theme::global_mut`, because a
//! direct edit does not reach the components that read semantic tokens (docs/GPUI.md). There is no theme folder and
//! no watcher (D-008): the file is parsed once at start.

use gpui_kit::component::{Theme, ThemeMode as ThemeShade, ThemeRegistry};
use gpui_kit::*;
use sucrose_core::settings::{Backdrop, ThemeMode};

/// The Sucrose light and dark themes, embedded in the executable.
const SUCROSE_THEMES: &str = include_str!("../themes/sucrose.json");
/// The names of the two themes in [`SUCROSE_THEMES`].
const LIGHT_NAME: &str = "Sucrose Light";
const DARK_NAME: &str = "Sucrose Dark";

/// Registers the Sucrose themes as the light and dark theme of the application. Call once after `gpui_kit::init`; a theme
/// file that cannot be read keeps the library's default colors and is logged.
pub fn install(cx: &mut App) {
    let registry = ThemeRegistry::global_mut(cx);
    if let Err(error) = registry.load_themes_from_str(SUCROSE_THEMES) {
        tracing::error!(%error, "the Sucrose themes cannot be read, the default colors stay");
        return;
    }
    let light = registry.themes().get(&SharedString::from(LIGHT_NAME)).cloned();
    let dark = registry.themes().get(&SharedString::from(DARK_NAME)).cloned();
    let (Some(light), Some(dark)) = (light, dark) else {
        tracing::error!("the Sucrose theme file lacks its light or dark theme, the default colors stay");
        return;
    };
    // Only the registered configs are swapped here; the colors reach the components through `Theme::change` in
    // `apply_mode`, which reloads both semantic and component tokens.
    let theme = Theme::global_mut(cx);
    theme.light_theme = light;
    theme.dark_theme = dark;
}

/// Whether the window shows the dark theme under `mode` when the OS reports `appearance`.
pub fn effective_dark_for(mode: ThemeMode, appearance: WindowAppearance) -> bool {
    match mode {
        ThemeMode::Light => false,
        ThemeMode::Dark => true,
        ThemeMode::System => matches!(appearance, WindowAppearance::Dark | WindowAppearance::VibrantDark),
    }
}

/// Whether the window shows the dark theme under `mode` (System follows the window appearance).
pub fn effective_dark(mode: ThemeMode, window: &Window) -> bool {
    effective_dark_for(mode, window.appearance())
}

/// Switches the application theme to `mode`; Light and Dark are explicit, System follows `appearance`.
fn change(mode: ThemeMode, appearance: WindowAppearance, cx: &mut App) {
    let shade = if effective_dark_for(mode, appearance) {
        ThemeShade::Dark
    } else {
        ThemeShade::Light
    };
    Theme::change(shade, None, cx);
    tracing::info!(?mode, dark = shade.is_dark(), "theme applied");
}

/// Applies the stored theme mode before a window exists (the OS appearance stands in for the window's).
pub fn apply_initial(mode: ThemeMode, cx: &mut App) {
    let appearance = cx.window_appearance();
    change(mode, appearance, cx);
}

/// Applies the stored theme mode to the running window. In System mode the window's own appearance decides, so a live
/// OS change only needs this call again.
pub fn apply_mode(mode: ThemeMode, window: &mut Window, cx: &mut App) {
    change(mode, window.appearance(), cx);
}

/// The window background GPUI is told to use for the stored `backdrop`. `supported` is the platform capability: an OS that
/// cannot show a backdrop gets an opaque window and the stored value stays as it is (D-22).
pub fn appearance(backdrop: Backdrop, supported: bool) -> WindowBackgroundAppearance {
    if !supported {
        return WindowBackgroundAppearance::Opaque;
    }
    match backdrop {
        Backdrop::None => WindowBackgroundAppearance::Opaque,
        Backdrop::Mica => WindowBackgroundAppearance::MicaBackdrop,
        Backdrop::Acrylic => WindowBackgroundAppearance::Blurred,
        Backdrop::Tabbed => WindowBackgroundAppearance::MicaAltBackdrop,
    }
}

/// The alpha of the page tint while a backdrop or a background image is active (UI-SPEC Color: dominant).
const TINT_ALPHA: f32 = 0.78;

/// Whether the stored `backdrop` is actually on screen: the OS can show it and it is not None.
pub fn backdrop_active(backdrop: Backdrop, supported: bool) -> bool {
    supported && backdrop != Backdrop::None
}

/// The alpha of the tinted page background: a backdrop or a background image must show through it.
pub fn page_alpha(backdrop_active: bool, image_active: bool) -> f32 {
    if backdrop_active || image_active {
        TINT_ALPHA
    } else {
        1.0
    }
}

/// The fill of the base Root surface under the Portal view: transparent while a backdrop is active, so the DWM
/// backdrop shows through the page tint; None otherwise, which leaves gpui-component's opaque `tokens.background` fill
/// in place (an Opaque window clears to white, which would show under a translucent page or a background image).
pub fn root_fill(_backdrop_active: bool) -> Option<Hsla> {
    None
}

/// Applies [`root_fill`] to the base Root that `gpui_kit::open_window` mounts under the Portal view. Returns false when
/// the window has no base Root yet (it is created after the view's constructor returns); the caller applies it again
/// later. The theme tokens are never touched: only the Root's own style changes.
pub fn apply_root_fill(_backdrop_active: bool, _window: &mut Window, _cx: &mut App) -> bool {
    false
}

/// Calls `on_change` when the OS switches between light and dark, but only while `is_system` says the stored mode is
/// System: an explicit Light or Dark ignores the OS. The window reports the change as an event, so nothing polls.
pub fn follow_os<T: 'static>(
    window: &mut Window,
    cx: &mut Context<T>,
    is_system: impl Fn(&T) -> bool + 'static,
    mut on_change: impl FnMut(&mut T, &mut Window, &mut Context<T>) + 'static,
) -> Subscription {
    cx.observe_window_appearance(window, move |this, window, cx| {
        if is_system(this) {
            on_change(this, window, cx);
        }
    })
}

#[cfg(test)]
mod tests {
    use gpui_kit::WindowAppearance;
    use sucrose_core::settings::ThemeMode;

    use gpui_kit::WindowBackgroundAppearance;
    use sucrose_core::settings::Backdrop;

    use super::{
        DARK_NAME, LIGHT_NAME, SUCROSE_THEMES, appearance, backdrop_active, effective_dark_for, page_alpha, root_fill,
    };

    #[test]
    fn theme_root_fill_is_transparent_only_while_a_backdrop_shows() {
        let fill = root_fill(true).expect("a transparent fill while a backdrop shows");
        assert!(fill.is_transparent());
        assert_eq!(root_fill(false), None);
    }

    #[test]
    fn theme_root_fill_follows_the_backdrop_mapping() {
        for backdrop in Backdrop::ALL {
            for supported in [true, false] {
                let expected = supported && matches!(backdrop, Backdrop::Mica | Backdrop::Acrylic | Backdrop::Tabbed);
                assert_eq!(
                    root_fill(backdrop_active(*backdrop, supported)).is_some(),
                    expected,
                    "{backdrop:?} supported={supported}"
                );
            }
        }
    }

    #[test]
    fn theme_backdrop_maps_to_the_window_background() {
        assert_eq!(appearance(Backdrop::None, true), WindowBackgroundAppearance::Opaque);
        assert_eq!(
            appearance(Backdrop::Mica, true),
            WindowBackgroundAppearance::MicaBackdrop
        );
        assert_eq!(appearance(Backdrop::Acrylic, true), WindowBackgroundAppearance::Blurred);
        assert_eq!(
            appearance(Backdrop::Tabbed, true),
            WindowBackgroundAppearance::MicaAltBackdrop
        );
    }

    #[test]
    fn theme_backdrop_is_opaque_where_the_os_cannot_show_it() {
        for backdrop in Backdrop::ALL {
            assert_eq!(
                appearance(*backdrop, false),
                WindowBackgroundAppearance::Opaque,
                "{backdrop:?}"
            );
        }
    }

    #[test]
    fn theme_page_is_opaque_only_without_a_backdrop_and_an_image() {
        assert!((page_alpha(false, false) - 1.0).abs() < f32::EPSILON);
        assert!((page_alpha(true, false) - 0.78).abs() < f32::EPSILON);
        assert!((page_alpha(false, true) - 0.78).abs() < f32::EPSILON);
        assert!((page_alpha(true, true) - 0.78).abs() < f32::EPSILON);
    }

    #[test]
    fn theme_light_and_dark_ignore_the_os_appearance() {
        for appearance in [WindowAppearance::Light, WindowAppearance::Dark] {
            assert!(!effective_dark_for(ThemeMode::Light, appearance));
            assert!(effective_dark_for(ThemeMode::Dark, appearance));
        }
    }

    #[test]
    fn theme_system_follows_the_os_appearance() {
        assert!(effective_dark_for(ThemeMode::System, WindowAppearance::Dark));
        assert!(effective_dark_for(ThemeMode::System, WindowAppearance::VibrantDark));
        assert!(!effective_dark_for(ThemeMode::System, WindowAppearance::Light));
        assert!(!effective_dark_for(ThemeMode::System, WindowAppearance::VibrantLight));
    }

    #[test]
    fn theme_file_has_the_light_and_the_dark_theme() {
        let set: serde_json::Value = serde_json::from_str(SUCROSE_THEMES).expect("the embedded theme file is JSON");
        let themes = set["themes"].as_array().expect("a themes list");
        let find = |name: &str| themes.iter().find(|theme| theme["name"] == name);
        assert_eq!(find(LIGHT_NAME).expect("light theme")["mode"], "light");
        assert_eq!(find(DARK_NAME).expect("dark theme")["mode"], "dark");
        assert_eq!(find(DARK_NAME).expect("dark theme")["colors"]["background"], "#202020");
        assert_eq!(
            find(LIGHT_NAME).expect("light theme")["colors"]["background"],
            "#F3F3F3"
        );
    }

    #[test]
    fn theme_file_parses_as_a_gpui_component_theme_set() {
        let set: gpui_kit::component::ThemeSet = serde_json::from_str(SUCROSE_THEMES).expect("a ThemeSet");
        assert_eq!(set.themes.len(), 2);
    }
}
