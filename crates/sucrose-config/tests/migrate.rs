//! Fixture tests for the migration of the General, Portal, Library and Warehouse legacy files (FMT-01, D-04, D-21).
//!
//! The `*.real.json` fixtures are copies of a real C# install's `Setting` files with personal data removed (paths and
//! the selected theme); the key sets, value types and enum names are the real ones.

use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use sucrose_config::{
    migrate::{map_theme, migrate},
    model::{DEFAULT_USER_AGENT, MIN_DATE_TIME, Settings},
};
use sucrose_core::settings::{Backdrop, LegacyBackdropType, LegacyThemeType, SortKind, SortMode, Stretch, ThemeMode};

const ROOT: &str = "legacy-root";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// A scratch `Setting` folder under the target folder with the given files; never a real legacy folder.
fn scratch(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("migrate").join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("the scratch folder under target can be created");
    for (file, content) in files {
        fs::write(dir.join(file), content).expect("the scratch file can be written");
    }
    dir
}

/// A scratch folder holding the four real fixtures under their legacy names.
fn real_folder(name: &str) -> PathBuf {
    let read = |fixture: &str| fs::read_to_string(fixtures().join(fixture)).expect("the fixture exists");
    let (general, portal, library, warehouse) = (
        read("General.real.json"),
        read("Portal.real.json"),
        read("Library.real.json"),
        read("Warehouse.real.json"),
    );
    scratch(
        name,
        &[
            ("General.json", &general),
            ("Portal.json", &portal),
            ("Library.json", &library),
            ("Warehouse.json", &warehouse),
        ],
    )
}

fn migrate_files(name: &str, files: &[(&str, &str)]) -> Settings {
    migrate(&scratch(name, files), Path::new(ROOT))
}

fn portal(json: &str, name: &str) -> sucrose_config::model::Portal {
    migrate_files(name, &[("Portal.json", json)]).portal
}

#[test]
fn the_real_portal_look_and_library_state_migrate() {
    let settings = migrate(&real_folder("real"), Path::new(ROOT));

    assert_eq!(settings.portal.theme_mode, ThemeMode::Dark);
    assert_eq!(settings.portal.backdrop, Backdrop::Acrylic);
    assert_eq!(settings.portal.background_opacity, 55);
    assert!(settings.portal.library_preview_hide);
    assert!(!settings.portal.library_preview);
    assert_eq!(settings.portal.adaptive_layout, 6);
    assert_eq!(settings.portal.adaptive_margin, 5);
    assert!(settings.portal.store_preview);
    assert!(!settings.portal.store_preview_hide);
    assert_eq!(settings.portal.store_duration, 1);
    assert_eq!(settings.portal.library_pagination, 30);
    assert_eq!(settings.portal.store_pagination, 30);
    assert!(settings.portal.store_adult);
    assert_eq!(settings.portal.background_image, "");
    assert_eq!(settings.portal.background_stretch, Stretch::UniformToFill);
    assert_eq!(settings.portal.library_sort_kind, SortKind::Descending);
    assert_eq!(settings.portal.library_sort_mode, SortMode::Creation);

    assert_eq!(settings.general.culture.as_deref(), Some("TR"));
    assert_eq!(settings.general.run_startup, 3);
    assert!(settings.general.app_exit);
    assert!(settings.general.app_visible);
    assert!(settings.general.telemetry_data);
    assert!(settings.general.exception_data);
    assert_eq!(settings.general.user_agent, DEFAULT_USER_AGENT);
}

#[test]
fn library_and_warehouse_keep_their_values() {
    let settings = migrate(&real_folder("real-library"), Path::new(ROOT));

    // A path with a non-ASCII character survives byte for byte.
    assert_eq!(settings.library.location, "D:\\S\u{fc}crose K\u{fc}t\u{fc}phane");
    assert_eq!(settings.library.selected, "BBBBBBBBBBBBBBBBBBBBBBBBB");
    assert!(!settings.library.delete_corrupt);
    assert!(settings.library.delete_confirm);
    assert!(settings.library.move_library);

    let expected: Vec<&str> = vec![
        "Alve's Town-1",
        "Bing Wallpaper-1",
        "Floating in Space-1",
        "Fluid Simulation-1",
        "Living Room-1",
        "Media Parallax-1",
        "Music Tunnel-1",
        "Neo Matrix-1",
        "Neomorphic Music Player-1",
        "Particulate Medusae-1",
        "Periodic Table-1",
        "Rainy-1",
        "Simple System-1",
        "Sky Series-1",
        "Sliding Clock Black White-1",
        "Space Train-1",
        "Spirit Tree-1",
        "Triangles and Light-1",
        "Lorem Picsum Wallpaper-1",
        "Ray Music Visualizer-1",
    ];
    assert_eq!(settings.warehouse.showcase, expected);
    assert_eq!(settings.warehouse.showcase.len(), 20);
    assert!(!settings.warehouse.hint_tray_icon);
    // The real file has no time or continue keys: the legacy defaults.
    assert_eq!(settings.warehouse.cef_sharp_time, MIN_DATE_TIME);
    assert_eq!(settings.warehouse.web_view_time, MIN_DATE_TIME);
    assert!(!settings.warehouse.cefsharp_continue);
    assert!(!settings.warehouse.web_view_continue);
}

#[test]
fn the_migrated_settings_survive_a_write_and_read() {
    let settings = migrate(&real_folder("real-round-trip"), Path::new(ROOT));
    let again = Settings::from_slice(&settings.to_bytes().expect("settings serialize")).expect("settings parse");
    assert_eq!(again, settings);
    assert_eq!(again.portal.backdrop, Backdrop::Acrylic);
    assert_eq!(again.library.location, "D:\\S\u{fc}crose K\u{fc}t\u{fc}phane");
}

#[test]
fn warehouse_times_and_continue_flags_migrate_as_raw_strings() {
    let json = r#"{"Properties":{"CefSharpTime":"2026-11-22T01:02:03.1234567+03:00","WebViewTime":"2026-11-23T01:02:03.1234567+03:00","CefsharpContinue":true,"WebViewContinue":true,"HintTrayIcon":true}}"#;
    let warehouse = migrate_files("warehouse-times", &[("Warehouse.json", json)]).warehouse;
    assert_eq!(warehouse.cef_sharp_time, "2026-11-22T01:02:03.1234567+03:00");
    assert_eq!(warehouse.web_view_time, "2026-11-23T01:02:03.1234567+03:00");
    assert!(warehouse.cefsharp_continue);
    assert!(warehouse.web_view_continue);
    assert!(warehouse.hint_tray_icon);
}

#[test]
fn non_string_showcase_items_are_skipped() {
    let json = r#"{"Properties":{"Showcase":["Rainy-1",5,null,"Sky Series-1",{"a":1}]}}"#;
    let warehouse = migrate_files("showcase-mixed", &[("Warehouse.json", json)]).warehouse;
    assert_eq!(warehouse.showcase, ["Rainy-1", "Sky Series-1"]);
}

#[test]
fn map_theme_follows_d21() {
    use LegacyBackdropType as B;
    use LegacyThemeType as T;
    // Auto means follow the system: System plus Mica, whatever ThemeType says.
    assert_eq!(
        map_theme(Some(T::Light), Some(B::Auto)),
        (ThemeMode::System, Backdrop::Mica)
    );
    assert_eq!(
        map_theme(Some(T::Dark), Some(B::Auto)),
        (ThemeMode::System, Backdrop::Mica)
    );
    assert_eq!(map_theme(None, Some(B::Auto)), (ThemeMode::System, Backdrop::Mica));
    // The others map one to one.
    assert_eq!(
        map_theme(Some(T::Dark), Some(B::Acrylic)),
        (ThemeMode::Dark, Backdrop::Acrylic)
    );
    assert_eq!(
        map_theme(Some(T::Light), Some(B::Tabbed)),
        (ThemeMode::Light, Backdrop::Tabbed)
    );
    assert_eq!(
        map_theme(Some(T::Light), Some(B::Mica)),
        (ThemeMode::Light, Backdrop::Mica)
    );
    assert_eq!(
        map_theme(Some(T::Dark), Some(B::None)),
        (ThemeMode::Dark, Backdrop::None)
    );
    assert_eq!(map_theme(Some(T::Dark), None), (ThemeMode::Dark, Backdrop::None));
    // Nothing stored is a fresh install.
    assert_eq!(map_theme(None, None), (ThemeMode::System, Backdrop::None));
    assert_eq!(map_theme(None, Some(B::None)), (ThemeMode::System, Backdrop::None));
}

#[test]
fn backdrop_auto_with_a_light_theme_migrates_to_system_and_mica() {
    let settings = migrate_files(
        "auto-light",
        &[
            ("General.json", r#"{"Properties":{"ThemeType":"Light"}}"#),
            ("Portal.json", r#"{"Properties":{"BackdropType":"Auto"}}"#),
        ],
    );
    assert_eq!(settings.portal.theme_mode, ThemeMode::System);
    assert_eq!(settings.portal.backdrop, Backdrop::Mica);
}

#[test]
fn backdrop_tabbed_and_a_light_theme_migrate_one_to_one() {
    let settings = migrate_files(
        "tabbed-light",
        &[
            ("General.json", r#"{"Properties":{"ThemeType":"Light"}}"#),
            ("Portal.json", r#"{"Properties":{"BackdropType":"Tabbed"}}"#),
        ],
    );
    assert_eq!(settings.portal.theme_mode, ThemeMode::Light);
    assert_eq!(settings.portal.backdrop, Backdrop::Tabbed);
}

#[test]
fn neither_theme_key_gives_system_and_none() {
    let settings = migrate_files("no-theme", &[("General.json", r#"{"Properties":{"Culture":"EN"}}"#)]);
    assert_eq!(settings.portal.theme_mode, ThemeMode::System);
    assert_eq!(settings.portal.backdrop, Backdrop::None);
}

#[test]
fn enum_names_are_exact_and_an_unknown_name_gives_the_default() {
    let settings = migrate_files(
        "enum-case",
        &[
            ("General.json", r#"{"Properties":{"ThemeType":"dark"}}"#),
            ("Portal.json", r#"{"Properties":{"BackdropType":"acrylic"}}"#),
        ],
    );
    assert_eq!(settings.portal.theme_mode, ThemeMode::System);
    assert_eq!(settings.portal.backdrop, Backdrop::None);
}

#[test]
fn numbers_are_clamped_to_their_legacy_ranges() {
    let low = portal(
        r#"{"Properties":{"LibraryPagination":0,"StorePagination":-5,"AdaptiveMargin":2,"AdaptiveLayout":-1,"StoreDuration":0,"BackgroundOpacity":-3}}"#,
        "clamp-low",
    );
    assert_eq!(low.library_pagination, 1);
    assert_eq!(low.store_pagination, 1);
    assert_eq!(low.adaptive_margin, 5);
    assert_eq!(low.adaptive_layout, 0);
    assert_eq!(low.store_duration, 1);
    assert_eq!(low.background_opacity, 0);

    let high = portal(
        r#"{"Properties":{"LibraryPagination":500,"StorePagination":101,"AdaptiveMargin":99,"AdaptiveLayout":300,"StoreDuration":48,"BackgroundOpacity":250}}"#,
        "clamp-high",
    );
    assert_eq!(high.library_pagination, 100);
    assert_eq!(high.store_pagination, 100);
    assert_eq!(high.adaptive_margin, 25);
    assert_eq!(high.adaptive_layout, 100);
    assert_eq!(high.store_duration, 24);
    assert_eq!(high.background_opacity, 100);
}

#[test]
fn run_startup_is_clamped_to_zero_through_ten() {
    let high = migrate_files(
        "startup-high",
        &[("General.json", r#"{"Properties":{"RunStartup":99}}"#)],
    );
    let low = migrate_files(
        "startup-low",
        &[("General.json", r#"{"Properties":{"RunStartup":-2}}"#)],
    );
    assert_eq!(high.general.run_startup, 10);
    assert_eq!(low.general.run_startup, 0);
}

#[test]
fn background_stretch_names_are_case_sensitive() {
    let fill = portal(r#"{"Properties":{"BackgroundStretch":"Fill"}}"#, "stretch-fill");
    let lower = portal(r#"{"Properties":{"BackgroundStretch":"fill"}}"#, "stretch-lower");
    assert_eq!(fill.background_stretch, Stretch::Fill);
    assert_eq!(lower.background_stretch, Stretch::UniformToFill);
}

#[test]
fn sort_names_migrate() {
    let sorted = portal(
        r#"{"Properties":{"LibrarySortKind":"Ascending","LibrarySortMode":"Modification"}}"#,
        "sort",
    );
    assert_eq!(sorted.library_sort_kind, SortKind::Ascending);
    assert_eq!(sorted.library_sort_mode, SortMode::Modification);
}

#[test]
fn a_file_with_a_byte_order_mark_migrates() {
    let json = "\u{feff}{\"Properties\":{\"BackgroundOpacity\":40,\"BackgroundImage\":\"\u{e7}ay.png\"}}";
    let portal = portal(json, "bom");
    assert_eq!(portal.background_opacity, 40);
    assert_eq!(portal.background_image, "\u{e7}ay.png");
}

#[test]
fn an_empty_or_wrapperless_file_gives_that_files_defaults() {
    let settings = migrate_files(
        "empty-and-bare",
        &[
            ("General.json", "  \r\n"),
            ("Portal.json", r#"{"BackgroundOpacity":40}"#),
            ("Warehouse.json", ""),
            ("Library.json", r#"{"Properties":{"Selected":"abc"}}"#),
        ],
    );
    assert_eq!(settings.general, Settings::default().general);
    assert_eq!(settings.portal, Settings::default().portal);
    assert_eq!(settings.warehouse, Settings::default().warehouse);
    // The other file in the same folder is not affected.
    assert_eq!(settings.library.selected, "abc");
}

#[test]
fn a_missing_setting_folder_gives_every_legacy_default() {
    let settings = migrate(&scratch("none", &[]).join("nothing"), Path::new(ROOT));
    assert_eq!(settings, Settings::default());
    assert!(settings.general.app_visible);
    assert_eq!(settings.portal.library_pagination, 30);
    assert!(settings.warehouse.hint_tray_icon);
    assert_eq!(settings.general.run_startup, 0);
    assert_eq!(settings.general.culture, None);
    assert_eq!(settings.portal.theme_mode, ThemeMode::System);
    assert_eq!(settings.portal.backdrop, Backdrop::None);
}

#[test]
fn a_file_that_is_empty_while_the_c_sharp_app_writes_it_is_retried() {
    let dir = scratch("retry", &[("Portal.json", "")]);
    let writer_dir = dir.clone();
    let writer = thread::spawn(move || {
        thread::sleep(Duration::from_millis(60));
        // Written elsewhere and moved over, so a reader never sees half of it.
        let temp = writer_dir.join("Portal.tmp");
        fs::write(&temp, r#"{"Properties":{"BackgroundOpacity":12}}"#).expect("the temp file can be written");
        fs::rename(&temp, writer_dir.join("Portal.json")).expect("the file can be replaced");
    });
    let settings = migrate(&dir, Path::new(ROOT));
    writer.join().expect("the writer thread finishes");
    assert_eq!(settings.portal.background_opacity, 12);
}

/// Reads the maintainer's real legacy folder (`%APPDATA%\Sucrose\Setting`, or `SUCROSE_LEGACY_SETTING_DIR`) and
/// migrates it into a scratch folder. Ignored: it needs the C# app's data on the machine.
#[test]
#[ignore = "reads the real C# install's Setting folder"]
#[allow(
    clippy::print_stdout,
    reason = "the ignored run reports the migrated look for the maintainer to read"
)]
fn real_install_migrates() {
    let legacy = std::env::var_os("SUCROSE_LEGACY_SETTING_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join("Sucrose").join("Setting")))
        .expect("APPDATA or SUCROSE_LEGACY_SETTING_DIR is set");
    assert!(legacy.is_dir(), "{} is not a folder", legacy.display());

    let snapshot = |dir: &Path| -> Vec<(String, Vec<u8>, Option<std::time::SystemTime>)> {
        let mut files: Vec<_> = fs::read_dir(dir)
            .expect("the legacy folder can be listed")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .map(|entry| {
                let path = entry.path();
                (
                    entry.file_name().to_string_lossy().into_owned(),
                    fs::read(&path).expect("a legacy file can be read"),
                    fs::metadata(&path).and_then(|meta| meta.modified()).ok(),
                )
            })
            .collect();
        files.sort_by(|a, b| a.0.cmp(&b.0));
        files
    };
    let before = snapshot(&legacy);

    let target = scratch("real-install", &[]);
    let (store, _outcome) =
        sucrose_config::store::Store::open(&target, Some((&legacy, legacy.parent().unwrap_or(&legacy))))
            .expect("the scratch store opens");
    let settings = store.settings();
    println!(
        "ThemeMode = {:?}, Backdrop = {:?}",
        settings.portal.theme_mode, settings.portal.backdrop
    );
    println!(
        "BackgroundOpacity = {}, Showcase = {}, HintTrayIcon = {}",
        settings.portal.background_opacity,
        settings.warehouse.showcase.len(),
        settings.warehouse.hint_tray_icon
    );

    assert_eq!(snapshot(&legacy), before, "a legacy file changed");
}
