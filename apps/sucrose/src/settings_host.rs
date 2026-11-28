//! The service's only owner of the settings store (D-11, ARCH-04, ARCH-05).
//!
//! Every change the Portal asks for passes through [`SettingsHost::apply`]: it is validated, written atomically and the
//! value that was stored is what the Portal is told. Nothing else in the service writes `Sucrose.json`, and the Portal
//! only ever sees [`SettingsHost::view`] and the `Changed` events. Requests are applied one at a time on the service's
//! main thread, so the last change wins and there is no second writer and no cache.

use std::{ffi::OsStr, path::Path};

use sucrose_config::{
    migrate::migrate,
    model::Settings,
    store::{LoadOutcome, Store},
};
use sucrose_core::settings::EngineType;
use sucrose_engine_web::args::{self, SucroseInputs, Switch, WebEngine};
use sucrose_ipc::portal::{
    MAX_PATH_TEXT, MAX_SWITCH_NAME, MAX_SWITCH_VALUE, MAX_SWITCHES, PortalEvent, PortalSettings, SettingChange,
    SwitchView,
};

use crate::portal_link::clip;

/// What the Portal is told when a change could not be written (UI-SPEC "Setting not saved").
pub const NOT_SAVED: &str = "Your setting couldn't be saved. Try again.";

/// The image types the legacy background picker offers (formats, case-insensitive).
const IMAGE_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "tiff", "webp", "gif"];

/// The settings of the running service and the file they are saved in.
#[derive(Debug)]
pub struct SettingsHost {
    /// The store that owns the file; `None` when the file could not be opened, then nothing can be saved.
    store: Option<Store>,
    /// The settings in use while there is no store.
    unsaved: Settings,
}

impl SettingsHost {
    /// Opens the settings in `setting_dir` (see [`Store::open`] for the load order and `legacy`). When the folder cannot
    /// be opened the legacy files are still migrated, in memory only, so the wallpaper plays; changes are then refused.
    pub fn open(setting_dir: &Path, legacy: Option<(&Path, &Path)>) -> (Self, LoadOutcome) {
        match Store::open(setting_dir, legacy) {
            Ok((store, outcome)) => (
                Self {
                    store: Some(store),
                    unsaved: Settings::default(),
                },
                outcome,
            ),
            Err(error) => {
                tracing::error!(%error, "the settings file cannot be opened; using the legacy settings without saving");
                let (unsaved, outcome) = match legacy {
                    Some((legacy_setting_dir, legacy_root)) => {
                        (migrate(legacy_setting_dir, legacy_root), LoadOutcome::Migrated)
                    }
                    None => (Settings::default(), LoadOutcome::Defaults),
                };
                (Self { store: None, unsaved }, outcome)
            }
        }
    }

    /// A host without a file, for when the OS reports no config folder: the defaults, nothing can be saved.
    pub fn unsaved(settings: Settings) -> Self {
        Self {
            store: None,
            unsaved: settings,
        }
    }

    /// Whether changes are written to a file.
    pub fn persists(&self) -> bool {
        self.store.is_some()
    }

    /// The settings in use.
    pub fn settings(&self) -> &Settings {
        self.store.as_ref().map_or(&self.unsaved, Store::settings)
    }

    /// What the Portal sees of the settings: a copy of the values it shows, never the file (D-11).
    pub fn view(&self) -> PortalSettings {
        view_of(self.settings())
    }

    /// Validates `change`, writes it and returns what was stored (numbers clamped to the legacy ranges). On a refused
    /// value or a failed write the previous settings stay and the error is the sentence the Portal shows.
    pub fn apply(&mut self, change: SettingChange) -> Result<SettingChange, String> {
        let stored = validate(change)?;
        self.update(|settings| write(settings, &stored))?;
        Ok(stored)
    }

    /// Changes the settings for the service itself (the selected theme, the library location and the like) and saves
    /// them. When the save fails the previous settings stay.
    pub fn update(&mut self, change: impl FnOnce(&mut Settings)) -> Result<(), String> {
        let Some(store) = self.store.as_mut() else {
            return Err(NOT_SAVED.to_owned());
        };
        store.update(change).map_err(|error| {
            tracing::warn!(%error, "the settings cannot be saved");
            NOT_SAVED.to_owned()
        })
    }
}

/// Checks `change` and returns the value to store: numbers are clamped to the ranges of the legacy Portal, a
/// background image must be empty or an absolute path of an image type the legacy picker offers, an engine must be
/// one the legacy list of its wallpaper type allows, and a list of CEF or WebView2 switches must hold valid, distinct names (D-17).
pub fn validate(change: SettingChange) -> Result<SettingChange, String> {
    Ok(match change {
        SettingChange::BackgroundOpacity(value) => SettingChange::BackgroundOpacity(value.min(100)),
        SettingChange::LibraryPagination(value) => SettingChange::LibraryPagination(value.clamp(1, 100)),
        SettingChange::AdaptiveLayout(value) => SettingChange::AdaptiveLayout(value.min(100)),
        SettingChange::AdaptiveMargin(value) => SettingChange::AdaptiveMargin(value.clamp(5, 25)),
        SettingChange::BackgroundImage(path) => {
            check_image(&path)?;
            SettingChange::BackgroundImage(path)
        }
        SettingChange::GifEngine(engine) => SettingChange::GifEngine(allowed_engine(engine, EngineType::GIF)?),
        SettingChange::UrlEngine(engine) => SettingChange::UrlEngine(allowed_engine(engine, EngineType::URL)?),
        SettingChange::WebEngine(engine) => SettingChange::WebEngine(allowed_engine(engine, EngineType::WEB)?),
        SettingChange::VideoEngine(engine) => SettingChange::VideoEngine(allowed_engine(engine, EngineType::VIDEO)?),
        SettingChange::YouTubeEngine(engine) => {
            SettingChange::YouTubeEngine(allowed_engine(engine, EngineType::YOUTUBE)?)
        }
        SettingChange::ApplicationEngine(engine) => {
            SettingChange::ApplicationEngine(allowed_engine(engine, EngineType::APPLICATION)?)
        }
        SettingChange::CefArguments(list) => SettingChange::CefArguments(check_switches(list)?),
        SettingChange::WebArguments(list) => SettingChange::WebArguments(check_switches(list)?),
        SettingChange::ScreenDevice(device) => SettingChange::ScreenDevice(check_device(device)?),
        other => other,
    })
}

/// What the Portal is told when a switch name is not letters, digits and dashes (UI-SPEC E4).
pub const SWITCH_NAME_INVALID: &str = "Use letters, digits and dashes only.";

/// What the Portal is told when a switch name occurs twice (UI-SPEC E4).
pub const SWITCH_NAME_REPEATED: &str = "This switch is already in the list.";

/// The switches Sucrose knows: the legacy defaults and the ones it adds itself. Any other valid name is stored all the
/// same and only logged (D-17, D-021).
fn is_known_switch(name: &str) -> bool {
    const SUCROSE_OWN: [&str; 5] = [
        "enable-gpu",
        "enable-gpu-vsync",
        "disable-gpu",
        "disable-gpu-vsync",
        "remote-allow-origins",
    ];
    SUCROSE_OWN.contains(&name) || args::legacy_defaults_cef().contains_key(name)
}

/// Checks a list of switches from the Portal and returns what is stored: names trimmed and without leading dashes,
/// values trimmed with an empty one meaning none. A name must be letters, digits and dashes, must not repeat, and the
/// list and its texts stay within the limits that keep the answer of `GetArguments` inside one frame. Unknown but valid
/// switches pass and are logged by name (D-17); a value is never logged.
fn check_switches(list: Vec<SwitchView>) -> Result<Vec<SwitchView>, String> {
    if list.len() > MAX_SWITCHES {
        return Err(format!("At most {MAX_SWITCHES} arguments can be saved."));
    }
    let mut checked = Vec::with_capacity(list.len());
    for switch in list {
        let name = switch.name.trim().trim_start_matches('-').to_owned();
        if name.len() > MAX_SWITCH_NAME || args::validate_name(&name).is_err() {
            return Err(SWITCH_NAME_INVALID.to_owned());
        }
        let value = switch
            .value
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        if value.as_ref().is_some_and(|value| value.len() > MAX_SWITCH_VALUE) {
            return Err("An argument value is too long.".to_owned());
        }
        checked.push(SwitchView { name, value });
    }
    let as_switches = to_switches(&checked);
    if !args::repeated_names(&as_switches).is_empty() {
        return Err(SWITCH_NAME_REPEATED.to_owned());
    }
    for switch in &checked {
        if !is_known_switch(&switch.name) {
            tracing::info!(name = %switch.name, "an unknown switch is stored");
        }
    }
    Ok(checked)
}

/// The switches of the engine crate for the views of the wire.
fn to_switches(views: &[SwitchView]) -> Vec<Switch> {
    views
        .iter()
        .map(|view| Switch {
            name: view.name.clone(),
            value: view.value.clone(),
        })
        .collect()
}

/// The wire views of switches, cut to what one frame holds (a stored list from a legacy file has no limit).
fn views(switches: Vec<Switch>) -> Vec<SwitchView> {
    switches
        .into_iter()
        .take(MAX_SWITCHES)
        .map(|switch| SwitchView {
            name: clip_to(&switch.name, MAX_SWITCH_NAME),
            value: switch.value.map(|value| clip_to(&value, MAX_SWITCH_VALUE)),
        })
        .collect()
}

/// `text` cut to at most `max` bytes at a character boundary.
fn clip_to(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_owned();
    }
    let mut end = max;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}

/// The answer to a `GetArguments` for `engine`: the user's switches as the engine reads them (an empty stored list
/// means the legacy defaults, as in the classic app), the ones Sucrose adds from `settings` (read-only in the editor)
/// and the legacy defaults for Reset. An engine without switches gets three empty lists.
pub fn arguments_answer(engine: EngineType, settings: &Settings, windows_build: u32) -> PortalEvent {
    let stored = &settings.engine;
    let (web_engine, user, defaults) = match engine {
        EngineType::CefSharpLive => {
            let defaults = args::from_cef_map(&args::legacy_defaults_cef());
            let user = if stored.cef_arguments.is_empty() {
                defaults.clone()
            } else {
                args::from_cef_map(&stored.cef_arguments)
            };
            (WebEngine::Cef, user, defaults)
        }
        EngineType::WebViewLive => {
            let defaults = args::from_web_list(&args::legacy_defaults_web());
            let user = if stored.web_arguments.is_empty() {
                defaults.clone()
            } else {
                args::from_web_list(&stored.web_arguments)
            };
            (WebEngine::WebView2, user, defaults)
        }
        _ => {
            return PortalEvent::Arguments {
                engine,
                user: Vec::new(),
                sucrose: Vec::new(),
                defaults: Vec::new(),
            };
        }
    };
    let sucrose = args::sucrose_switches(&SucroseInputs {
        hardware_acceleration: stored.hardware_acceleration,
        debugging_port: args::debugging_port(stored.developer_port, 0).ok().flatten(),
        windows_build,
        engine: web_engine,
        webview_smartscreen: stored.webview_smartscreen,
    });
    PortalEvent::Arguments {
        engine,
        user: views(user),
        sucrose: views(sucrose),
        defaults: views(defaults),
    }
}

/// `engine` when the legacy list of the wallpaper type (`allowed`) holds it (D-11, T-04-31). An engine that is not
/// installed is allowed: it stays the setting and a fallback plays instead.
fn allowed_engine(engine: EngineType, allowed: &[EngineType]) -> Result<EngineType, String> {
    if allowed.contains(&engine) {
        Ok(engine)
    } else {
        Err("This engine cannot play this type of wallpaper.".to_owned())
    }
}

/// The device name of the monitor the next Use in Per display goes to: empty (the first monitor) or a text of at most
/// [`MAX_PATH_TEXT`] bytes without a NUL. A name that is no connected monitor is stored as it is: the placement falls
/// back to the first monitor (the legacy auto-choose, T-04-93).
fn check_device(device: String) -> Result<String, String> {
    if device.len() > MAX_PATH_TEXT || device.contains('\0') {
        return Err("The monitor name is not valid.".to_owned());
    }
    Ok(device)
}

/// A background image is no image (empty), or an absolute path of at most [`MAX_PATH_TEXT`] bytes that ends in one of
/// [`IMAGE_EXTENSIONS`]. Whether the file exists is the Portal's business when it draws it.
fn check_image(text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }
    if text.len() > MAX_PATH_TEXT {
        return Err("The image path is too long.".to_owned());
    }
    let path = Path::new(text);
    if !path.is_absolute() {
        return Err("The image path must be an absolute path.".to_owned());
    }
    let known = path.extension().and_then(OsStr::to_str).is_some_and(|extension| {
        IMAGE_EXTENSIONS
            .iter()
            .any(|known| known.eq_ignore_ascii_case(extension))
    });
    if known {
        Ok(())
    } else {
        Err("The image must be a PNG, JPG, JPEG, TIFF, WEBP or GIF file.".to_owned())
    }
}

/// Sets the field `change` names.
fn write(settings: &mut Settings, change: &SettingChange) {
    match change {
        SettingChange::ThemeMode(value) => settings.portal.theme_mode = *value,
        SettingChange::Backdrop(value) => settings.portal.backdrop = *value,
        SettingChange::BackgroundImage(value) => settings.portal.background_image.clone_from(value),
        SettingChange::BackgroundStretch(value) => settings.portal.background_stretch = *value,
        SettingChange::BackgroundOpacity(value) => settings.portal.background_opacity = *value,
        SettingChange::LibraryPagination(value) => settings.portal.library_pagination = *value,
        SettingChange::AdaptiveLayout(value) => settings.portal.adaptive_layout = *value,
        SettingChange::AdaptiveMargin(value) => settings.portal.adaptive_margin = *value,
        SettingChange::LibraryPreview(value) => settings.portal.library_preview = *value,
        SettingChange::LibraryPreviewHide(value) => settings.portal.library_preview_hide = *value,
        SettingChange::LibrarySortMode(value) => settings.portal.library_sort_mode = *value,
        SettingChange::LibrarySortKind(value) => settings.portal.library_sort_kind = *value,
        SettingChange::DeleteCorrupt(value) => settings.library.delete_corrupt = *value,
        SettingChange::LibraryMove(value) => settings.library.move_library = *value,
        SettingChange::AppVisible(value) => settings.general.app_visible = *value,
        SettingChange::AppExit(value) => settings.general.app_exit = *value,
        SettingChange::HintTrayIcon(value) => settings.warehouse.hint_tray_icon = *value,
        SettingChange::GifEngine(value) => settings.engine.gif = *value,
        SettingChange::UrlEngine(value) => settings.engine.url = *value,
        SettingChange::WebEngine(value) => settings.engine.web = *value,
        SettingChange::VideoEngine(value) => settings.engine.video = *value,
        SettingChange::YouTubeEngine(value) => settings.engine.youtube = *value,
        SettingChange::ApplicationEngine(value) => settings.engine.application = *value,
        SettingChange::WallpaperLoop(value) => settings.engine.looping = *value,
        SettingChange::WallpaperShuffle(value) => settings.engine.wallpaper_shuffle = *value,
        SettingChange::HardwareAcceleration(value) => settings.engine.hardware_acceleration = *value,
        SettingChange::InputType(value) => settings.engine.input_type = *value,
        SettingChange::InputModuleType(value) => settings.engine.input_module_type = *value,
        SettingChange::InputDesktop(value) => settings.engine.input_desktop = *value,
        SettingChange::AllowInvalidCertificates(value) => settings.engine.allow_invalid_certificates = *value,
        SettingChange::WebViewSmartScreen(value) => settings.engine.webview_smartscreen = *value,
        SettingChange::ScreenDevice(value) => settings.engine.screen_device.clone_from(value),
        SettingChange::CefArguments(list) => settings.engine.cef_arguments = args::to_cef_map(&to_switches(list)),
        SettingChange::WebArguments(list) => settings.engine.web_arguments = args::to_web_list(&to_switches(list)),
        SettingChange::DeveloperMode(value) => settings.engine.developer_mode = *value,
        SettingChange::DeveloperPort(value) => settings.engine.developer_port = *value,
    }
}

/// The change as the log shows it: the variant and its value, never a path (D-08).
pub fn describe(change: &SettingChange) -> String {
    match change {
        SettingChange::BackgroundImage(path) => {
            format!("BackgroundImage({})", if path.is_empty() { "none" } else { "set" })
        }
        other => format!("{other:?}"),
    }
}

fn view_of(settings: &Settings) -> PortalSettings {
    PortalSettings {
        app_visible: settings.general.app_visible,
        app_exit: settings.general.app_exit,
        run_startup: settings.general.run_startup,
        theme_mode: settings.portal.theme_mode,
        backdrop: settings.portal.backdrop,
        background_image: clip(&settings.portal.background_image),
        background_stretch: settings.portal.background_stretch,
        background_opacity: settings.portal.background_opacity,
        library_pagination: settings.portal.library_pagination,
        adaptive_layout: settings.portal.adaptive_layout,
        adaptive_margin: settings.portal.adaptive_margin,
        library_preview: settings.portal.library_preview,
        library_preview_hide: settings.portal.library_preview_hide,
        library_sort_mode: settings.portal.library_sort_mode,
        library_sort_kind: settings.portal.library_sort_kind,
        library_location: clip(&settings.library.location),
        library_move: settings.library.move_library,
        delete_corrupt: settings.library.delete_corrupt,
        hint_tray_icon: settings.warehouse.hint_tray_icon,
        gif_engine: settings.engine.gif,
        url_engine: settings.engine.url,
        web_engine: settings.engine.web,
        video_engine: settings.engine.video,
        youtube_engine: settings.engine.youtube,
        application_engine: settings.engine.application,
        wallpaper_loop: settings.engine.looping,
        wallpaper_shuffle: settings.engine.wallpaper_shuffle,
        hardware_acceleration: settings.engine.hardware_acceleration,
        input_type: settings.engine.input_type,
        input_module_type: settings.engine.input_module_type,
        input_desktop: settings.engine.input_desktop,
        allow_invalid_certificates: settings.engine.allow_invalid_certificates,
        developer_mode: settings.engine.developer_mode,
        developer_port: settings.engine.developer_port,
        webview_smartscreen: settings.engine.webview_smartscreen,
        screen_device: clip(&settings.engine.screen_device),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use sucrose_core::settings::{EngineType, InputModuleType, InputType, ThemeMode};
    use sucrose_ipc::portal::MAX_PATH_TEXT;

    use super::*;

    /// A settings folder of its own, removed when the test ends.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("sucrose-settings-host-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            Self(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// An absolute path with `file_name` that exists on no machine, in the form of this OS.
    fn absolute(file_name: &str) -> String {
        std::env::temp_dir().join(file_name).to_string_lossy().into_owned()
    }

    #[test]
    fn validate_clamps_numbers_to_the_legacy_ranges() {
        let cases = [
            (
                SettingChange::BackgroundOpacity(150),
                SettingChange::BackgroundOpacity(100),
            ),
            (
                SettingChange::BackgroundOpacity(40),
                SettingChange::BackgroundOpacity(40),
            ),
            (SettingChange::LibraryPagination(0), SettingChange::LibraryPagination(1)),
            (
                SettingChange::LibraryPagination(250),
                SettingChange::LibraryPagination(100),
            ),
            (SettingChange::AdaptiveLayout(101), SettingChange::AdaptiveLayout(100)),
            (SettingChange::AdaptiveMargin(2), SettingChange::AdaptiveMargin(5)),
            (SettingChange::AdaptiveMargin(30), SettingChange::AdaptiveMargin(25)),
            (SettingChange::AdaptiveMargin(12), SettingChange::AdaptiveMargin(12)),
        ];
        for (asked, stored) in cases {
            assert_eq!(validate(asked.clone()), Ok(stored), "{asked:?}");
        }
    }

    #[test]
    fn validate_accepts_the_engines_the_legacy_list_of_the_type_allows() {
        for change in [
            SettingChange::WebEngine(EngineType::CefSharpLive),
            SettingChange::WebEngine(EngineType::WebViewLive),
            SettingChange::UrlEngine(EngineType::CefSharpLive),
            SettingChange::YouTubeEngine(EngineType::WebViewLive),
            SettingChange::VideoEngine(EngineType::MpvPlayerLive),
            SettingChange::VideoEngine(EngineType::NebulaLive),
            SettingChange::GifEngine(EngineType::VexanaLive),
            SettingChange::ApplicationEngine(EngineType::AuroraLive),
        ] {
            assert_eq!(validate(change.clone()), Ok(change));
        }
    }

    #[test]
    fn validate_refuses_an_engine_outside_the_legacy_list_of_the_type() {
        for change in [
            SettingChange::WebEngine(EngineType::MpvPlayerLive),
            SettingChange::WebEngine(EngineType::AuroraLive),
            SettingChange::UrlEngine(EngineType::VlcPlayerLive),
            SettingChange::YouTubeEngine(EngineType::MpvPlayerLive),
            SettingChange::VideoEngine(EngineType::AuroraLive),
            SettingChange::GifEngine(EngineType::NebulaLive),
            SettingChange::ApplicationEngine(EngineType::CefSharpLive),
        ] {
            assert!(validate(change.clone()).is_err(), "{change:?}");
        }
    }

    #[test]
    fn an_accepted_engine_is_written_and_shown_and_a_refused_one_changes_nothing() {
        let scratch = Scratch::new("engine");
        let (mut host, _) = SettingsHost::open(&scratch.0, None);

        let stored = host.apply(SettingChange::WebEngine(EngineType::WebViewLive));
        let refused = host.apply(SettingChange::VideoEngine(EngineType::AuroraLive));

        assert_eq!(stored, Ok(SettingChange::WebEngine(EngineType::WebViewLive)));
        assert!(refused.is_err());
        let view = host.view();
        assert_eq!(view.web_engine, EngineType::WebViewLive);
        assert_eq!(
            view.video_engine,
            EngineType::MpvPlayerLive,
            "the refused change left the default"
        );
        let file = fs::read(scratch.0.join(sucrose_config::store::SETTINGS_FILE)).expect("the settings file exists");
        let json: serde_json::Value = serde_json::from_slice(&file).expect("the file is JSON");
        assert_eq!(json["Engine"]["Web"], "WebViewLive");
    }

    #[test]
    fn validate_accepts_every_change_of_the_wallpaper_group() {
        for change in [
            SettingChange::WallpaperLoop(false),
            SettingChange::WallpaperShuffle(false),
            SettingChange::HardwareAcceleration(false),
            SettingChange::InputType(InputType::Close),
            SettingChange::InputType(InputType::OnlyMouse),
            SettingChange::InputType(InputType::OnlyKeyboard),
            SettingChange::InputType(InputType::MouseKeyboard),
            SettingChange::InputModuleType(InputModuleType::RawInput),
            SettingChange::InputDesktop(true),
        ] {
            assert_eq!(validate(change.clone()), Ok(change));
        }
    }

    #[test]
    fn a_wallpaper_group_change_is_written_to_the_engine_section_and_shown() {
        let scratch = Scratch::new("wallpaper-group");
        let (mut host, _) = SettingsHost::open(&scratch.0, None);
        let defaults = host.view();
        assert!(defaults.wallpaper_loop && defaults.wallpaper_shuffle && defaults.hardware_acceleration);
        assert_eq!(defaults.input_type, InputType::MouseKeyboard);
        assert!(!defaults.input_desktop);

        for change in [
            SettingChange::WallpaperLoop(false),
            SettingChange::WallpaperShuffle(false),
            SettingChange::HardwareAcceleration(false),
            SettingChange::InputType(InputType::OnlyKeyboard),
            SettingChange::InputModuleType(InputModuleType::RawInput),
            SettingChange::InputDesktop(true),
        ] {
            assert_eq!(host.apply(change.clone()), Ok(change));
        }

        let view = host.view();
        assert!(!view.wallpaper_loop && !view.wallpaper_shuffle && !view.hardware_acceleration);
        assert_eq!(view.input_type, InputType::OnlyKeyboard);
        assert_eq!(view.input_module_type, InputModuleType::RawInput);
        assert!(view.input_desktop);
        let file = fs::read(scratch.0.join(sucrose_config::store::SETTINGS_FILE)).expect("the settings file exists");
        let json: serde_json::Value = serde_json::from_slice(&file).expect("the file is JSON");
        assert_eq!(json["Engine"]["WallpaperLoop"], false);
        assert_eq!(json["Engine"]["WallpaperShuffle"], false);
        assert_eq!(json["Engine"]["HardwareAcceleration"], false);
        assert_eq!(json["Engine"]["InputType"], "OnlyKeyboard");
        assert_eq!(json["Engine"]["InputModuleType"], "RawInput");
        assert_eq!(json["Engine"]["InputDesktop"], true);
    }

    #[test]
    fn a_host_that_cannot_save_refuses_a_wallpaper_group_change_and_keeps_the_value() {
        let mut host = SettingsHost::unsaved(Settings::default());
        let before = host.view();

        let answer = host.apply(SettingChange::WallpaperLoop(false));

        assert_eq!(answer, Err(NOT_SAVED.to_owned()));
        assert_eq!(host.view(), before);
    }

    #[test]
    fn validate_accepts_no_image_and_an_absolute_image_path_in_any_case() {
        for path in [
            "".to_owned(),
            absolute("wall.PNG"),
            absolute("wall.jpeg"),
            absolute("a.WebP"),
        ] {
            let change = SettingChange::BackgroundImage(path);
            assert_eq!(validate(change.clone()), Ok(change));
        }
    }

    #[test]
    fn validate_refuses_a_relative_path_an_unknown_extension_and_a_long_text() {
        let long = format!("{}.png", absolute(&"a".repeat(MAX_PATH_TEXT)));
        for path in [
            "wall.png".to_owned(),
            "..\\wall.png".to_owned(),
            absolute("wall.bmp"),
            absolute("wall"),
            long,
        ] {
            let change = SettingChange::BackgroundImage(path.clone());
            assert!(validate(change).is_err(), "{path}");
        }
    }

    #[test]
    fn the_same_change_twice_leaves_the_same_file_bytes() {
        let scratch = Scratch::new("twice");
        let (mut host, _) = SettingsHost::open(&scratch.0, None);
        let file = scratch.0.join("Sucrose.json");

        let first = host.apply(SettingChange::LibraryPagination(33));
        let after_first = fs::read(&file).expect("the settings file exists");
        let second = host.apply(SettingChange::LibraryPagination(33));
        let after_second = fs::read(&file).expect("the settings file exists");

        assert_eq!(first, Ok(SettingChange::LibraryPagination(33)));
        assert_eq!(second, first);
        assert_eq!(after_first, after_second);
    }

    #[test]
    fn apply_returns_the_clamped_value_and_that_is_what_the_view_shows() {
        let scratch = Scratch::new("clamped");
        let (mut host, _) = SettingsHost::open(&scratch.0, None);

        let stored = host.apply(SettingChange::LibraryPagination(250));

        assert_eq!(stored, Ok(SettingChange::LibraryPagination(100)));
        assert_eq!(host.view().library_pagination, 100);
    }

    #[test]
    fn a_refused_value_changes_nothing() {
        let scratch = Scratch::new("refused");
        let (mut host, _) = SettingsHost::open(&scratch.0, None);
        let before = host.view();

        let answer = host.apply(SettingChange::BackgroundImage("relative.png".to_owned()));

        assert!(answer.is_err());
        assert_eq!(host.view(), before);
    }

    #[test]
    fn a_host_that_cannot_save_answers_with_the_not_saved_text_and_keeps_the_value() {
        let mut host = SettingsHost::unsaved(Settings::default());
        let before = host.view();

        let answer = host.apply(SettingChange::LibraryPagination(7));

        assert_eq!(answer, Err(NOT_SAVED.to_owned()));
        assert_eq!(host.view(), before);
    }

    #[test]
    fn a_host_whose_settings_file_cannot_be_read_saves_nothing() {
        let scratch = Scratch::new("unreadable");
        // A directory where the settings file belongs: it exists and cannot be read as a file, on every OS.
        fs::create_dir_all(scratch.0.join(sucrose_config::store::SETTINGS_FILE)).unwrap();

        let (mut host, _) = SettingsHost::open(&scratch.0, None);
        let before = host.view();
        let answer = host.apply(SettingChange::LibraryPagination(7));

        assert!(!host.persists());
        assert_eq!(answer, Err(NOT_SAVED.to_owned()));
        assert_eq!(host.view(), before);
        let names: Vec<String> = fs::read_dir(&scratch.0)
            .unwrap()
            .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
            .collect();
        assert_eq!(
            names,
            [sucrose_config::store::SETTINGS_FILE],
            "the host wrote and migrated nothing"
        );
        assert!(scratch.0.join(sucrose_config::store::SETTINGS_FILE).is_dir());
    }

    #[test]
    fn the_view_carries_the_values_the_portal_shows() {
        let mut settings = Settings::default();
        settings.portal.library_pagination = 17;
        settings.portal.theme_mode = ThemeMode::Dark;
        settings.general.app_visible = false;
        settings.library.location = "D:\\Themes".to_owned();

        let view = SettingsHost::unsaved(settings).view();

        assert_eq!(view.library_pagination, 17);
        assert_eq!(view.theme_mode, ThemeMode::Dark);
        assert!(!view.app_visible);
        assert_eq!(view.library_location, "D:\\Themes");
    }

    #[test]
    fn long_paths_are_cut_at_a_character() {
        let mut settings = Settings::default();
        settings.portal.background_image = "\u{20ac}".repeat(MAX_PATH_TEXT);

        let view = SettingsHost::unsaved(settings).view();

        assert!(view.background_image.len() <= MAX_PATH_TEXT);
        assert!(view.background_image.chars().all(|c| c == '\u{20ac}'));
    }

    fn switch(name: &str, value: Option<&str>) -> SwitchView {
        SwitchView {
            name: name.to_owned(),
            value: value.map(str::to_owned),
        }
    }

    #[test]
    fn the_certificate_switch_is_written_to_the_engine_section_and_shown() {
        let scratch = Scratch::new("certificates");
        let (mut host, _) = SettingsHost::open(&scratch.0, None);
        assert!(!host.view().allow_invalid_certificates, "off by default (D-12)");

        let stored = host.apply(SettingChange::AllowInvalidCertificates(true));

        assert_eq!(stored, Ok(SettingChange::AllowInvalidCertificates(true)));
        assert!(host.view().allow_invalid_certificates);
        let file = fs::read(scratch.0.join(sucrose_config::store::SETTINGS_FILE)).expect("the settings file exists");
        let json: serde_json::Value = serde_json::from_slice(&file).expect("the file is JSON");
        assert_eq!(json["Engine"]["AllowInvalidCertificates"], true);
    }

    #[test]
    fn a_webview2_smartscreen_change_is_stored_and_shown() {
        let scratch = Scratch::new("smartscreen");
        let (mut host, _) = SettingsHost::open(&scratch.0, None);
        assert!(!host.view().webview_smartscreen, "off by default (D-041)");

        let stored = host.apply(SettingChange::WebViewSmartScreen(true));

        assert_eq!(stored, Ok(SettingChange::WebViewSmartScreen(true)));
        assert!(host.view().webview_smartscreen);
        let file = fs::read(scratch.0.join(sucrose_config::store::SETTINGS_FILE)).expect("the settings file exists");
        let json: serde_json::Value = serde_json::from_slice(&file).expect("the file is JSON");
        assert_eq!(json["Engine"]["WebViewSmartScreen"], true);
    }

    #[test]
    fn a_screen_device_change_is_stored_shown_and_bounded() {
        let scratch = Scratch::new("screendevice");
        let (mut host, _) = SettingsHost::open(&scratch.0, None);
        assert_eq!(host.view().screen_device, "", "the first monitor by default");

        let device = "\\\\.\\DISPLAY9".to_owned();
        let stored = host.apply(SettingChange::ScreenDevice(device.clone()));

        assert_eq!(stored, Ok(SettingChange::ScreenDevice(device.clone())));
        assert_eq!(host.view().screen_device, device);
        let file = fs::read(scratch.0.join(sucrose_config::store::SETTINGS_FILE)).expect("the settings file exists");
        let json: serde_json::Value = serde_json::from_slice(&file).expect("the file is JSON");
        assert_eq!(json["Engine"]["ScreenDevice"], device.as_str());

        assert_eq!(
            host.apply(SettingChange::ScreenDevice(String::new())),
            Ok(SettingChange::ScreenDevice(String::new())),
            "empty means the first monitor"
        );
        assert!(validate(SettingChange::ScreenDevice("d".repeat(MAX_PATH_TEXT + 1))).is_err());
        assert!(validate(SettingChange::ScreenDevice("a\0b".to_owned())).is_err());
        assert!(validate(SettingChange::ScreenDevice("d".repeat(MAX_PATH_TEXT))).is_ok());
        assert_eq!(host.view().screen_device, "", "a refused change stored nothing");
    }

    #[test]
    fn settings_host_validate_accepts_the_whole_port_range_and_both_developer_changes() {
        for change in [
            SettingChange::DeveloperPort(0),
            SettingChange::DeveloperPort(1024),
            SettingChange::DeveloperPort(65535),
            SettingChange::DeveloperMode(true),
            SettingChange::DeveloperMode(false),
        ] {
            assert_eq!(validate(change.clone()), Ok(change));
        }
    }

    #[test]
    fn settings_host_developer_mode_and_port_are_written_to_the_engine_section_and_shown() {
        let scratch = Scratch::new("developer");
        let (mut host, _) = SettingsHost::open(&scratch.0, None);
        let defaults = host.view();
        assert!(!defaults.developer_mode, "developer tools are off by default");
        assert_eq!(defaults.developer_port, 0, "0 means no debugging port");

        assert_eq!(
            host.apply(SettingChange::DeveloperMode(true)),
            Ok(SettingChange::DeveloperMode(true))
        );
        assert_eq!(
            host.apply(SettingChange::DeveloperPort(9222)),
            Ok(SettingChange::DeveloperPort(9222))
        );

        let view = host.view();
        assert!(view.developer_mode);
        assert_eq!(view.developer_port, 9222);
        let file = fs::read(scratch.0.join(sucrose_config::store::SETTINGS_FILE)).expect("the settings file exists");
        let json: serde_json::Value = serde_json::from_slice(&file).expect("the file is JSON");
        assert_eq!(json["Engine"]["DeveloperMode"], true);
        assert_eq!(json["Engine"]["DeveloperPort"], 9222);
    }

    #[test]
    fn a_cef_argument_list_is_cleaned_and_stored_as_the_legacy_dictionary() {
        let scratch = Scratch::new("cef-arguments");
        let (mut host, _) = SettingsHost::open(&scratch.0, None);

        let stored = host.apply(SettingChange::CefArguments(vec![
            switch(" --disable-extensions ", None),
            switch("autoplay-policy", Some(" no-user-gesture-required ")),
            switch("my-own-switch", Some("  ")),
        ]));

        assert_eq!(
            stored,
            Ok(SettingChange::CefArguments(vec![
                switch("disable-extensions", None),
                switch("autoplay-policy", Some("no-user-gesture-required")),
                switch("my-own-switch", None),
            ])),
            "dashes and blanks are removed, an unknown switch is stored"
        );
        let file = fs::read(scratch.0.join(sucrose_config::store::SETTINGS_FILE)).expect("the settings file exists");
        let json: serde_json::Value = serde_json::from_slice(&file).expect("the file is JSON");
        let map = &json["Engine"]["CefArguments"];
        assert_eq!(map["disable-extensions"], "");
        assert_eq!(map["autoplay-policy"], "no-user-gesture-required");
        assert_eq!(map["my-own-switch"], "");
        assert_eq!(map.as_object().map(|map| map.len()), Some(3));
    }

    #[test]
    fn a_cef_argument_list_with_a_bad_or_repeated_name_is_refused_and_nothing_is_stored() {
        let scratch = Scratch::new("cef-arguments-refused");
        let (mut host, _) = SettingsHost::open(&scratch.0, None);
        let before = host.settings().engine.cef_arguments.clone();

        for (list, reason) in [
            (vec![switch("has space", None)], SWITCH_NAME_INVALID),
            (vec![switch("", Some("x"))], SWITCH_NAME_INVALID),
            (vec![switch("a=b", None)], SWITCH_NAME_INVALID),
            (vec![switch("--", None)], SWITCH_NAME_INVALID),
            (
                vec![switch("same", None), switch("--same", Some("1"))],
                SWITCH_NAME_REPEATED,
            ),
        ] {
            let answer = host.apply(SettingChange::CefArguments(list.clone()));
            assert_eq!(answer, Err(reason.to_owned()), "{list:?}");
        }
        assert_eq!(host.settings().engine.cef_arguments, before);
    }

    #[test]
    fn a_cef_argument_list_over_the_limits_is_refused() {
        let too_many: Vec<SwitchView> = (0..=MAX_SWITCHES)
            .map(|index| switch(&format!("s{index}"), None))
            .collect();
        assert!(validate(SettingChange::CefArguments(too_many)).is_err());
        let long_name = "n".repeat(MAX_SWITCH_NAME + 1);
        assert!(validate(SettingChange::CefArguments(vec![switch(&long_name, None)])).is_err());
        let long_value = "v".repeat(MAX_SWITCH_VALUE + 1);
        assert!(validate(SettingChange::CefArguments(vec![switch("x", Some(&long_value))])).is_err());
        let longest = "v".repeat(MAX_SWITCH_VALUE);
        assert!(validate(SettingChange::CefArguments(vec![switch("x", Some(&longest))])).is_ok());
    }

    #[test]
    fn a_webview2_argument_list_is_cleaned_and_stored_as_the_legacy_list() {
        let scratch = Scratch::new("web-arguments");
        let (mut host, _) = SettingsHost::open(&scratch.0, None);

        let stored = host.apply(SettingChange::WebArguments(vec![
            switch("disable-web-security", None),
            switch(" --autoplay-policy ", Some(" no-user-gesture-required ")),
        ]));

        assert_eq!(
            stored,
            Ok(SettingChange::WebArguments(vec![
                switch("disable-web-security", None),
                switch("autoplay-policy", Some("no-user-gesture-required")),
            ]))
        );
        assert_eq!(
            host.settings().engine.web_arguments,
            vec![
                "--disable-web-security".to_owned(),
                "--autoplay-policy=no-user-gesture-required".to_owned()
            ]
        );
        let file = fs::read(scratch.0.join(sucrose_config::store::SETTINGS_FILE)).expect("the settings file exists");
        let json: serde_json::Value = serde_json::from_slice(&file).expect("the file is JSON");
        assert_eq!(
            json["Engine"]["WebArguments"],
            serde_json::json!(["--disable-web-security", "--autoplay-policy=no-user-gesture-required"])
        );
        assert!(
            host.settings().engine.cef_arguments.is_empty(),
            "the CEF list is untouched"
        );
    }

    #[test]
    fn a_webview2_argument_list_that_is_bad_repeated_or_too_long_is_refused_and_nothing_is_stored() {
        let scratch = Scratch::new("web-arguments-refused");
        let (mut host, _) = SettingsHost::open(&scratch.0, None);
        let before = host.settings().engine.web_arguments.clone();

        for (list, reason) in [
            (vec![switch("has space", None)], SWITCH_NAME_INVALID),
            (
                vec![switch("same", None), switch("--same", Some("1"))],
                SWITCH_NAME_REPEATED,
            ),
        ] {
            let answer = host.apply(SettingChange::WebArguments(list.clone()));
            assert_eq!(answer, Err(reason.to_owned()), "{list:?}");
        }
        let too_many: Vec<SwitchView> = (0..=MAX_SWITCHES)
            .map(|index| switch(&format!("s{index}"), None))
            .collect();
        assert!(host.apply(SettingChange::WebArguments(too_many)).is_err());
        let long_name = "n".repeat(MAX_SWITCH_NAME + 1);
        assert!(
            host.apply(SettingChange::WebArguments(vec![switch(&long_name, None)]))
                .is_err()
        );
        let long_value = "v".repeat(MAX_SWITCH_VALUE + 1);
        assert!(
            host.apply(SettingChange::WebArguments(vec![switch("x", Some(&long_value))]))
                .is_err()
        );
        assert_eq!(host.settings().engine.web_arguments, before);
    }

    #[test]
    fn the_webview2_arguments_answer_is_the_stored_list_or_the_legacy_defaults_when_it_is_empty() {
        let mut settings = Settings::default();
        let PortalEvent::Arguments { user, defaults, .. } = arguments_answer(EngineType::WebViewLive, &settings, 22631)
        else {
            panic!("not an Arguments answer");
        };
        assert_eq!(user, defaults, "an empty stored list means the legacy defaults");
        assert!(!defaults.is_empty());
        assert!(
            defaults
                .iter()
                .all(|switch| switch.name != "off-screen-rendering-enabled")
        );

        settings.engine.web_arguments = vec!["--my-own-switch".to_owned(), "--x=1".to_owned()];
        let PortalEvent::Arguments { user, .. } = arguments_answer(EngineType::WebViewLive, &settings, 22631) else {
            panic!("not an Arguments answer");
        };
        assert_eq!(user, vec![switch("my-own-switch", None), switch("x", Some("1"))]);
    }

    #[test]
    fn the_webview2_arguments_answer_lists_the_smartscreen_switch_only_while_smartscreen_is_off() {
        let mut settings = Settings::default();
        let sucrose_of = |settings: &Settings| {
            let PortalEvent::Arguments { sucrose, .. } = arguments_answer(EngineType::WebViewLive, settings, 22631)
            else {
                panic!("not an Arguments answer");
            };
            sucrose
        };

        assert!(
            sucrose_of(&settings).contains(&switch("disable-features", Some("msSmartScreenProtection"))),
            "D-041: SmartScreen is off by default"
        );

        settings.engine.webview_smartscreen = true;

        assert!(
            sucrose_of(&settings).iter().all(|view| view.name != "disable-features"),
            "with SmartScreen on the editor lists no disable-features switch of Sucrose's"
        );
    }

    #[test]
    fn the_arguments_answer_gives_the_stored_switches_sucrose_own_and_the_legacy_defaults() {
        let mut settings = Settings::default();
        settings.engine.cef_arguments = args::to_cef_map(&[Switch::flag("my-own-switch")]);
        settings.engine.hardware_acceleration = true;

        let PortalEvent::Arguments {
            engine,
            user,
            sucrose,
            defaults,
        } = arguments_answer(EngineType::CefSharpLive, &settings, 22631)
        else {
            panic!("an Arguments event");
        };

        assert_eq!(engine, EngineType::CefSharpLive);
        assert_eq!(user, vec![switch("my-own-switch", None)]);
        assert_eq!(
            sucrose.iter().map(|switch| switch.name.as_str()).collect::<Vec<_>>(),
            ["enable-gpu", "enable-gpu-vsync"]
        );
        assert_eq!(defaults.len(), args::legacy_defaults_cef().len());
        assert!(defaults.iter().any(|switch| switch.name == "autoplay-policy"));
    }

    #[test]
    fn an_empty_stored_list_reads_as_the_legacy_defaults_and_an_engine_without_switches_as_nothing() {
        let settings = Settings::default();
        let mut with_defaults = settings.clone();
        with_defaults.engine.cef_arguments.clear();

        let PortalEvent::Arguments { user, defaults, .. } =
            arguments_answer(EngineType::CefSharpLive, &with_defaults, 0)
        else {
            panic!("an Arguments event");
        };
        assert_eq!(
            user, defaults,
            "an empty dictionary means the defaults, as in the classic app"
        );
        assert!(!user.is_empty());

        let PortalEvent::Arguments {
            user: none,
            sucrose,
            defaults: nothing,
            ..
        } = arguments_answer(EngineType::MpvPlayerLive, &settings, 0)
        else {
            panic!("an Arguments event");
        };
        assert!(none.is_empty() && sucrose.is_empty() && nothing.is_empty());
    }

    #[test]
    fn the_arguments_answer_stays_inside_the_limits_of_one_frame() {
        let mut settings = Settings::default();
        settings.engine.cef_arguments = (0..200)
            .map(|index| (format!("switch-{index}"), "v".repeat(5000)))
            .collect();

        let PortalEvent::Arguments { user, .. } = arguments_answer(EngineType::CefSharpLive, &settings, 0) else {
            panic!("an Arguments event");
        };

        assert_eq!(user.len(), MAX_SWITCHES);
        assert!(
            user.iter()
                .all(|switch| switch.value.as_ref().is_some_and(|v| v.len() <= MAX_SWITCH_VALUE))
        );
    }
}
