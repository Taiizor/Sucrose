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
use sucrose_ipc::portal::{MAX_PATH_TEXT, PortalSettings, SettingChange};

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
/// background image must be empty or an absolute path of an image type the legacy picker offers.
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
        other => other,
    })
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
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use sucrose_core::settings::ThemeMode;
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
}
