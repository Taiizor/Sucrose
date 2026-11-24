//! The Portal's theme: the Sucrose `ThemeConfig` (light and dark), the stored theme mode (Light, Dark or System, D-21)
//! and the window backdrop (D-22).
//!
//! The colors come from the embedded `themes/sucrose.json`; they are never edited through `Theme::global_mut`, because a
//! direct edit does not reach the components that read semantic tokens (docs/GPUI.md). There is no theme folder and
//! no watcher (D-008): the file is parsed once at start.

use gpui_kit::component::{Theme, ThemeMode as ThemeShade, ThemeRegistry};
use gpui_kit::*;
use sucrose_core::settings::ThemeMode;

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

    use super::{DARK_NAME, LIGHT_NAME, SUCROSE_THEMES, effective_dark_for};

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
