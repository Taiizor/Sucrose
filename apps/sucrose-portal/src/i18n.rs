//! Localization helpers over rust-i18n (D-20, D-015).
//!
//! Every user-visible Portal string is a key of `locales/legacy.en.yml` (generated, legacy key names) or of
//! `locales/sucrose.en.yml` (keys new in Phase 3). Pages call these helpers; a key added later goes into
//! `sucrose.en.yml` and into [`REQUIRED_KEYS`].

use gpui_kit::SharedString;

/// The text of a key in the current locale.
pub fn tr(key: &str) -> SharedString {
    SharedString::from(rust_i18n::t!(key).into_owned())
}

/// The text of a key with its `%{name}` placeholders filled from `args` (name, value).
pub fn tr_with(key: &str, args: &[(&str, &str)]) -> SharedString {
    let text = rust_i18n::t!(key);
    let names: Vec<&str> = args.iter().map(|(name, _)| *name).collect();
    let values: Vec<String> = args.iter().map(|(_, value)| (*value).to_owned()).collect();
    SharedString::from(rust_i18n::replace_patterns(&text, &names, &values))
}

/// The text of a counted message: `key` for any count but one, `<key>One` for exactly one; `%{count}` is filled in.
pub fn tr_count(key: &str, count: usize) -> SharedString {
    tr_count_with(key, count, &[])
}

/// Like [`tr_count`], with further `%{name}` placeholders filled from `args`.
pub fn tr_count_with(key: &str, count: usize, args: &[(&str, &str)]) -> SharedString {
    let number = count.to_string();
    let mut all = vec![("count", number.as_str())];
    all.extend_from_slice(args);
    if count == 1 {
        tr_with(&format!("{key}One"), &all)
    } else {
        tr_with(key, &all)
    }
}

/// Every key the Portal uses in Phase 3: the legacy keys of the UI-SPEC String Inventory that are shown, and all new keys.
/// A test fails when one does not resolve, and when `sucrose.en.yml` has a key that is not listed here.
pub const REQUIRED_KEYS: &[&str] = &[
    // Legacy keys (legacy.en.yml).
    "Portal.Release",
    "Portal.Copyright",
    "Portal.MainWindow.Title",
    "Portal.MainWindow.MenuLibrary",
    "Portal.MainWindow.MenuGeneral",
    "Portal.MainWindow.MenuPersonal",
    "Portal.MainWindow.MenuBackTip",
    "Portal.MainWindow.MenuSettingTip",
    "Portal.MainWindow.MenuHelp",
    "Portal.MainWindow.MenuAbout",
    "Portal.MainWindow.MenuHelpTip",
    "Portal.MainWindow.MenuAboutTip",
    "Portal.MainWindow.SearchBoxHint",
    "Portal.MainWindow.SearchBoxTip",
    "Portal.MainWindow.MenuDarkTip",
    "Portal.MainWindow.MenuLightTip",
    "Portal.MainWindow.MenuPremiumStoreTip",
    "Portal.MainWindow.MenuOtherOptionsTip",
    "Portal.EmptyLibraryPage.TitleText",
    "Portal.EmptyLibraryPage.DescriptionText",
    "Portal.SearchLibraryPage.TitleText",
    "Portal.SearchLibraryPage.DescriptionText",
    "Portal.LibraryCard.VersionTip",
    "Portal.LibraryCard.MenuUse",
    "Portal.LibraryCard.Closed",
    "Portal.LibraryCard.Paused",
    "Portal.LibraryCard.Selected",
    "Portal.LibraryCard.Incompatible",
    "Portal.Pagination.PageJump",
    "Portal.TrayIcon.Title",
    "Portal.TrayIcon.Message",
    "Portal.TrayIcon.Close",
    "Portal.OtherHelp.Title",
    "Portal.OtherHelp.Close",
    "Portal.OtherHelp.WebPage",
    "Portal.OtherHelp.WebPage.Description",
    "Portal.OtherHelp.Documentation",
    "Portal.OtherHelp.Documentation.Description",
    "Portal.OtherHelp.Repository",
    "Portal.OtherHelp.Repository.Description",
    "Portal.OtherHelp.Discussions",
    "Portal.OtherHelp.Discussions.Description",
    "Portal.OtherHelp.Report",
    "Portal.OtherHelp.Report.Description",
    "Portal.OtherAbout.Title",
    "Portal.OtherAbout.About",
    "Portal.OtherAbout.Close",
    "Portal.OtherAbout.Update",
    "Portal.OtherAbout.Update.Notes",
    "Portal.OtherAbout.Contributors",
    "Portal.OtherAbout.TermsOfService",
    "Portal.OtherAbout.PrivacyPolicy",
    "Portal.OtherAbout.CodeOfConduct",
    "Portal.GeneralSettingPage.ApplicationStartup",
    "Portal.GeneralSettingPage.ApplicationStartup.Description",
    "Portal.GeneralSettingPage.ApplicationStartup.Startup.None",
    "Portal.GeneralSettingPage.ApplicationStartup.Startup.Normal",
    "Portal.GeneralSettingPage.ApplicationStartup.Startup.Priority",
    "Portal.GeneralSettingPage.ApplicationStartup.Startup.Scheduler",
    "Portal.GeneralSettingPage.NotifyIcon",
    "Portal.GeneralSettingPage.NotifyIcon.Description",
    "Portal.GeneralSettingPage.NotifyIcon.Notify.Show",
    "Portal.GeneralSettingPage.NotifyIcon.Notify.Hide",
    "Portal.GeneralSettingPage.NotifyIcon.NotifyExit",
    "Portal.GeneralSettingPage.WindowBackdrop",
    "Portal.GeneralSettingPage.WindowBackdrop.Description",
    "Portal.GeneralSettingPage.WindowBackdrop.BackdropOpacity",
    "Portal.GeneralSettingPage.WindowBackdrop.BackdropStretch",
    "Portal.GeneralSettingPage.WindowBackdrop.BackgroundImage.Select",
    "Portal.GeneralSettingPage.WindowBackdrop.BackgroundImage.Access",
    "Portal.GeneralSettingPage.WindowBackdrop.BackgroundImage.Lock",
    "Portal.GeneralSettingPage.PrivateLibrary",
    "Portal.GeneralSettingPage.PrivateLibrary.Description",
    "Portal.GeneralSettingPage.PrivateLibrary.LibraryMove",
    "Portal.GeneralSettingPage.PrivateLibrary.LibraryLocation.Empty",
    "Portal.GeneralSettingPage.PrivateLibrary.LibraryLocation.Access",
    "Portal.GeneralSettingPage.PrivateLibrary.LibraryLocation.Move",
    "Portal.PersonalSettingPage.Delete",
    "Portal.PersonalSettingPage.Preview",
    "Portal.PersonalSettingPage.Preview.Description",
    "Portal.PersonalSettingPage.Preview.LibraryPreview",
    "Portal.PersonalSettingPage.Preview.LibraryPreviewHide",
    "Portal.PersonalSettingPage.Adaptive",
    "Portal.PersonalSettingPage.Adaptive.Description",
    "Portal.PersonalSettingPage.Adaptive.AdaptiveMargin",
    "Portal.PersonalSettingPage.Adaptive.AdaptiveLayout",
    "Portal.PersonalSettingPage.Sort",
    "Portal.PersonalSettingPage.Sort.Description",
    "Portal.PersonalSettingPage.Sort.SortMode",
    "Portal.PersonalSettingPage.Sort.SortKind",
    "Portal.PersonalSettingPage.Library",
    "Portal.PersonalSettingPage.Library.Description",
    "Portal.Enum.WindowBackdropType.None",
    "Portal.Enum.WindowBackdropType.Mica",
    "Portal.Enum.WindowBackdropType.Acrylic",
    "Portal.Enum.WindowBackdropType.Tabbed",
    "Portal.Enum.Stretch.None",
    "Portal.Enum.Stretch.Fill",
    "Portal.Enum.Stretch.Uniform",
    "Portal.Enum.Stretch.UniformToFill",
    "Portal.Enum.SortModeType.Name",
    "Portal.Enum.SortModeType.Creation",
    "Portal.Enum.SortModeType.Modification",
    "Portal.Enum.SortKindType.Ascending",
    "Portal.Enum.SortKindType.Descending",
    // New in Phase 3 (sucrose.en.yml).
    "Portal.LibraryPage.LoadError.Title",
    "Portal.LibraryPage.LoadError.Body",
    "Portal.LibraryPage.TryAgain",
    "Portal.LibraryPage.ChangeLocation",
    "Portal.Pagination.Previous",
    "Portal.Pagination.Next",
    "Portal.LibraryPage.Preparing",
    "Portal.LibraryPage.CorruptMoved",
    "Portal.LibraryPage.CorruptMovedOne",
    "Portal.Connection.Connecting",
    "Portal.Connection.Lost",
    "Portal.Connection.Failed",
    "Portal.Connection.TryAgain",
    "Portal.Connection.Unavailable",
    "Portal.LibraryCard.UseFailed",
    "Portal.LibraryCard.UnsupportedType",
    "Portal.LibraryCard.NoEngine",
    "Portal.MainWindow.MenuWallpaper",
    "Portal.Area.Extensions",
    "Portal.Area.Engines",
    "Portal.WallpaperSettingPage.VideoPlayer",
    "Portal.WallpaperSettingPage.VideoPlayer.Description",
    "Portal.WallpaperSettingPage.WebPlayer",
    "Portal.WallpaperSettingPage.WebPlayer.Description",
    "Portal.StatusBar.Copied",
    "Portal.StatusBar.CopyFailed",
    "Portal.StatusBar.MemoryTotal",
    "Portal.StatusBar.MemoryService",
    "Portal.StatusBar.MemoryRenderer",
    "Portal.StatusBar.MemoryPortal",
    "Portal.GeneralSettingPage.ThemeMode",
    "Portal.GeneralSettingPage.ThemeMode.Description",
    "Portal.GeneralSettingPage.ThemeMode.Light",
    "Portal.GeneralSettingPage.ThemeMode.Dark",
    "Portal.GeneralSettingPage.ThemeMode.System",
    "Portal.GeneralSettingPage.PrivateLibrary.ConfirmTitle",
    "Portal.GeneralSettingPage.PrivateLibrary.ConfirmBody",
    "Portal.GeneralSettingPage.PrivateLibrary.ConfirmLegacy",
    "Portal.GeneralSettingPage.PrivateLibrary.Confirm",
    "Portal.GeneralSettingPage.PrivateLibrary.KeepLocation",
    "Portal.GeneralSettingPage.PrivateLibrary.NoAccess",
    "Portal.GeneralSettingPage.PrivateLibrary.Nested",
    "Portal.GeneralSettingPage.PrivateLibrary.Missing",
    "Portal.GeneralSettingPage.PrivateLibrary.Busy",
    "Portal.GeneralSettingPage.PrivateLibrary.MoveFailed",
    "Portal.GeneralSettingPage.ApplicationStartup.LegacyEntry",
    "Portal.GeneralSettingPage.ApplicationStartup.NotYet",
    "Portal.GeneralSettingPage.ApplicationStartup.LaterTip",
    "Portal.GeneralSettingPage.ApplicationStartup.DisabledByOs",
    "Portal.GeneralSettingPage.ApplicationStartup.NoEntry",
    "Portal.GeneralSettingPage.WindowBackdrop.RequiresWin11",
    "Portal.GeneralSettingPage.WindowBackdrop.Unavailable",
    "Portal.GeneralSettingPage.WindowBackdrop.ImageMissing",
    "Portal.PersonalSettingPage.Delete.Description",
    "Portal.PersonalSettingPage.Delete.DeleteHint",
    "Portal.Setting.SaveFailed",
    "Portal.SettingsPage.GroupBehavior",
    "Portal.SettingsPage.GroupAppearance",
    "Portal.SettingsPage.GroupLibrary",
    "Portal.SettingsPage.GroupWallpaper",
    "Portal.WallpaperSettingPage.Engine.NotInstalled",
    "Portal.Enum.EngineType.BuiltInGif",
    "Portal.Enum.EngineType.BuiltInVideo",
    "Portal.Enum.EngineType.WebView2",
    "Portal.Enum.EngineType.Cef",
    "Portal.Enum.EngineType.MpvPlayer",
    "Portal.Enum.EngineType.VlcPlayer",
    "Portal.Enum.EngineType.Aurora",
    "Portal.GeneralSettingPage.WindowBackdrop.BackgroundImage.Remove",
    "Portal.GeneralSettingPage.WindowBackdrop.BackgroundImage.None",
    "Portal.GeneralSettingPage.PrivateLibrary.OpenFolder",
    "Portal.TrayIcon.CloseCountdown",
];

#[cfg(test)]
mod tests {
    use super::*;

    const LEGACY: &str = include_str!("../locales/legacy.en.yml");
    const NEW: &str = include_str!("../locales/sucrose.en.yml");

    /// The keys of a flat locale file: every line that starts with a quoted key.
    fn keys(file: &str) -> Vec<&str> {
        file.lines()
            .filter_map(|line| line.strip_prefix('"')?.split_once("\":"))
            .map(|(key, _)| key)
            .collect()
    }

    fn english(key: &str) -> String {
        rust_i18n::t!(key, locale = "en").into_owned()
    }

    #[test]
    fn every_required_key_resolves_to_text() {
        let missing: Vec<&str> = REQUIRED_KEYS
            .iter()
            .copied()
            .filter(|key| english(key) == *key)
            .collect();
        assert!(missing.is_empty(), "keys without English text: {missing:?}");
    }

    #[test]
    fn every_quoting_resolves_to_text() {
        for index in 0..=85 {
            let key = format!("Portal.Quoting{index}");
            assert_ne!(english(&key), key, "{key} has no text");
        }
    }

    #[test]
    fn every_new_key_is_required() {
        let unlisted: Vec<&str> = keys(NEW)
            .into_iter()
            .filter(|key| !REQUIRED_KEYS.contains(key))
            .collect();
        assert!(
            unlisted.is_empty(),
            "keys of sucrose.en.yml missing from REQUIRED_KEYS: {unlisted:?}"
        );
    }

    #[test]
    fn the_two_locale_files_share_no_key() {
        let legacy = keys(LEGACY);
        let shared: Vec<&str> = keys(NEW).into_iter().filter(|key| legacy.contains(key)).collect();
        assert!(shared.is_empty(), "keys in both locale files: {shared:?}");
    }

    #[test]
    fn tr_count_picks_the_singular_text_for_one() {
        let one = tr_count_with("Portal.LibraryPage.CorruptMoved", 1, &[("bin", "Recycle Bin")]);
        assert_eq!(&*one, "1 corrupt theme was moved to the Recycle Bin.");
        let three = tr_count_with("Portal.LibraryPage.CorruptMoved", 3, &[("bin", "Trash")]);
        assert_eq!(&*three, "3 corrupt themes were moved to the Trash.");
        let zero = tr_count("Portal.LibraryPage.CorruptMoved", 0);
        assert!(zero.starts_with("0 corrupt themes were moved to the "), "{zero}");
    }

    #[test]
    fn tr_with_fills_a_named_placeholder() {
        let hint = tr_with(
            "Portal.PersonalSettingPage.Delete.DeleteHint",
            &[("bin", "Recycle Bin")],
        );
        assert_eq!(
            &*hint,
            "Note: Themes are moved to the Recycle Bin, so you can restore them."
        );
        assert_eq!(
            &*tr("Portal.Setting.SaveFailed"),
            "Your setting couldn't be saved. Try again."
        );
    }
}
