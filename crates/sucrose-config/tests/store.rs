//! Tests of the settings store (ARCH-04, FMT-01, FMT-06): load order, the `.bak`, the newer-version guard, unknown keys,
//! stale temporary files and a writer that is killed in the middle of its writes.
//!
//! Every test works in its own folder under the target folder, never in a real data folder.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use sucrose_config::{
    model::{SETTINGS_VERSION, Settings},
    store::{BACKUP_FILE, LoadOutcome, SETTINGS_FILE, Store, StoreError},
};

const ROOT: &str = "legacy-root";
/// Set by the kill test for the copy of this test binary it starts; no app reads it.
const KILL_DIR_ENV: &str = "SUCROSE_CONFIG_KILL_DIR";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// An empty scratch folder for one test.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("store").join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("the scratch folder under target can be created");
    dir
}

/// A scratch folder holding copies of the four legacy fixtures the migration reads.
fn legacy_copy(name: &str) -> PathBuf {
    let dir = scratch(name);
    for file in ["Library.json", "General.json", "Engine.json", "Backgroundog.json"] {
        fs::copy(fixtures().join(file), dir.join(file)).expect("the fixture can be copied");
    }
    dir
}

fn settings_with(change: impl FnOnce(&mut Settings)) -> Settings {
    let mut settings = Settings::default();
    change(&mut settings);
    settings
}

fn write_settings(dir: &Path, file: &str, settings: &Settings) {
    fs::write(dir.join(file), settings.to_bytes().expect("settings serialize")).expect("the file can be written");
}

fn read_settings(path: &Path) -> Settings {
    Settings::from_slice(&fs::read(path).expect("the file exists")).expect("the file parses")
}

fn temporary_files(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .expect("the folder can be listed")
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter(|name| name.starts_with("Sucrose.json.") && name.ends_with(".tmp"))
        .collect()
}

fn legacy_snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files: Vec<(String, Vec<u8>)> = fs::read_dir(dir)
        .expect("the folder can be listed")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            Some((entry.file_name().into_string().ok()?, fs::read(entry.path()).ok()?))
        })
        .collect();
    files.sort();
    files
}

#[test]
fn a_valid_file_is_loaded_without_reading_the_legacy_files() {
    let setting = scratch("valid-setting");
    let legacy = scratch("valid-legacy");
    fs::write(legacy.join("Library.json"), "this is not json").unwrap();
    write_settings(
        &setting,
        SETTINGS_FILE,
        &settings_with(|s| s.library.selected = "chosen".to_owned()),
    );

    let (store, outcome) = Store::open(&setting, Some((&legacy, Path::new(ROOT)))).unwrap();

    assert_eq!(outcome, LoadOutcome::Loaded);
    assert_eq!(store.settings().library.selected, "chosen");
}

#[test]
fn a_corrupt_file_with_a_good_backup_is_restored_not_migrated() {
    let setting = scratch("restore-setting");
    let legacy = legacy_copy("restore-legacy");
    fs::write(setting.join(SETTINGS_FILE), b"{\"Version\":1,\"Library\":{\"Sel").unwrap();
    write_settings(
        &setting,
        BACKUP_FILE,
        &settings_with(|s| s.library.selected = "from-backup".to_owned()),
    );

    let (store, outcome) = Store::open(&setting, Some((&legacy, Path::new(ROOT)))).unwrap();

    assert_eq!(outcome, LoadOutcome::RestoredFromBackup);
    assert_eq!(store.settings().library.selected, "from-backup");
    assert_eq!(
        read_settings(&setting.join(SETTINGS_FILE)).library.selected,
        "from-backup"
    );
}

#[test]
fn a_missing_file_with_a_good_backup_is_restored() {
    let setting = scratch("missing-setting");
    write_settings(&setting, BACKUP_FILE, &settings_with(|s| s.engine.volume = 42));

    let (store, outcome) = Store::open(&setting, None).unwrap();

    assert_eq!(outcome, LoadOutcome::RestoredFromBackup);
    assert_eq!(store.settings().engine.volume, 42);
    assert!(setting.join(SETTINGS_FILE).is_file());
}

#[test]
fn nothing_at_all_gives_the_defaults_in_a_new_setting_folder() {
    let setting = scratch("nothing").join("Setting");

    let (store, outcome) = Store::open(&setting, None).unwrap();

    assert_eq!(outcome, LoadOutcome::Defaults);
    assert_eq!(store.settings(), &Settings::default());
    assert_eq!(
        store.settings().library_location(Path::new(ROOT)),
        Path::new(ROOT).join("Library")
    );
    let names: Vec<String> = fs::read_dir(&setting)
        .unwrap()
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .collect();
    assert_eq!(
        names,
        [SETTINGS_FILE],
        "the only file the app adds is the settings file"
    );
}

#[test]
fn an_update_keeps_the_previous_file_as_the_backup() {
    let setting = scratch("backup");
    let (mut store, _) = Store::open(&setting, None).unwrap();

    store.update(|s| s.engine.volume = 11).unwrap();
    store.update(|s| s.engine.volume = 22).unwrap();

    assert_eq!(read_settings(&setting.join(SETTINGS_FILE)).engine.volume, 22);
    assert_eq!(read_settings(&setting.join(BACKUP_FILE)).engine.volume, 11);
    assert_eq!(store.settings().engine.volume, 22);
}

#[test]
fn a_newer_version_is_read_but_never_overwritten() {
    let setting = scratch("newer");
    let text = br#"{"Version":2,"Engine":{"WallpaperVolume":5},"Surprise":{"X":1}}"#;
    fs::write(setting.join(SETTINGS_FILE), text).unwrap();

    let (mut store, outcome) = Store::open(&setting, None).unwrap();

    assert_eq!(outcome, LoadOutcome::Loaded);
    assert_eq!(store.settings().engine.volume, 5);
    assert_eq!(store.settings().version, SETTINGS_VERSION + 1);
    let result = store.update(|s| s.engine.volume = 6);
    assert!(
        matches!(result, Err(StoreError::NewerVersion { found: 2 })),
        "{result:?}"
    );
    assert_eq!(fs::read(setting.join(SETTINGS_FILE)).unwrap(), text);
    assert!(!setting.join(BACKUP_FILE).exists());
}

#[test]
fn unknown_keys_survive_load_update_load() {
    let setting = scratch("unknown");
    fs::write(
        setting.join(SETTINGS_FILE),
        br#"{"Version":1,"Surprise":{"X":1},"Library":{"Selected":"a","Later":[1,2]}}"#,
    )
    .unwrap();

    let (mut store, _) = Store::open(&setting, None).unwrap();
    store.update(|s| s.engine.volume = 9).unwrap();
    drop(store);
    let (store, outcome) = Store::open(&setting, None).unwrap();

    assert_eq!(outcome, LoadOutcome::Loaded);
    assert_eq!(store.settings().engine.volume, 9);
    assert_eq!(store.settings().extra["Surprise"]["X"], 1);
    assert_eq!(store.settings().library.extra["Later"][1], 2);
}

#[test]
fn stale_temporary_files_are_removed_at_open() {
    let setting = scratch("stale");
    write_settings(&setting, SETTINGS_FILE, &Settings::default());
    fs::write(setting.join("Sucrose.json.4242.tmp"), b"{\"Version\":").unwrap();
    fs::write(setting.join("Sucrose.json.7.tmp"), b"").unwrap();
    fs::write(setting.join("notes.tmp"), b"keep me").unwrap();

    let (_store, _) = Store::open(&setting, None).unwrap();

    assert!(temporary_files(&setting).is_empty(), "{:?}", temporary_files(&setting));
    assert!(setting.join("notes.tmp").is_file(), "a file that is not ours stays");
    assert!(setting.join(SETTINGS_FILE).is_file());
}

#[test]
fn migration_runs_once_and_never_writes_a_legacy_file() {
    let setting = scratch("migrate-setting");
    let legacy = legacy_copy("migrate-legacy");
    let before = legacy_snapshot(&legacy);

    let (store, outcome) = Store::open(&setting, Some((&legacy, Path::new(ROOT)))).unwrap();
    assert_eq!(outcome, LoadOutcome::Migrated);
    assert_eq!(store.settings().library.selected, "AAAAAAAAAAAAAAAAAAAAAAAAA");
    drop(store);
    // The second start reads only the new file: break the legacy files to prove they are not read.
    fs::write(legacy.join("Library.json"), "this is not json").unwrap();
    let broken = legacy_snapshot(&legacy);
    let (store, outcome) = Store::open(&setting, Some((&legacy, Path::new(ROOT)))).unwrap();

    assert_eq!(outcome, LoadOutcome::Loaded);
    assert_eq!(store.settings().library.selected, "AAAAAAAAAAAAAAAAAAAAAAAAA");
    assert_eq!(legacy_snapshot(&legacy), broken);
    assert_ne!(before, broken);
}

/// The writer of the kill test: it only does anything in the copy of this binary the kill test starts.
#[test]
fn write_loop_child() {
    let Some(dir) = std::env::var_os(KILL_DIR_ENV) else {
        return;
    };
    let dir = PathBuf::from(dir);
    let (mut store, _) = Store::open(&dir, None).expect("the child can open the store");
    let mut volume: u8 = 0;
    store
        .update(|s| s.engine.volume = volume)
        .expect("the first update works");
    fs::write(dir.join("ready"), b"").expect("the marker can be written");
    loop {
        volume = (volume + 1) % 101;
        store
            .update(|s| s.engine.volume = volume)
            .expect("the update works until the process is killed");
    }
}

fn parses(path: &Path) -> bool {
    fs::read(path).is_ok_and(|bytes| Settings::from_slice(&bytes).is_ok())
}

#[test]
fn kill_during_write_never_leaves_a_corrupt_file() {
    const ROUNDS: usize = 60;
    let dir = scratch("kill");
    let exe = std::env::current_exe().expect("the test binary has a path");
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |d| d.as_nanos() as u64)
        | 1;
    let mut random = move || {
        // xorshift64
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut left_a_temporary_file = 0;
    for round in 0..ROUNDS {
        let _ = fs::remove_file(dir.join("ready"));
        let mut child = Command::new(&exe)
            .args(["--exact", "write_loop_child", "--nocapture"])
            .env(KILL_DIR_ENV, &dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the writer can be started");
        // Kill only once the writer rewrites the file in its loop, so the kill lands inside the writes.
        let started = Instant::now();
        while !dir.join("ready").exists() {
            assert!(started.elapsed() < Duration::from_secs(30), "the writer never started");
            thread::sleep(Duration::from_millis(1));
        }
        thread::sleep(Duration::from_millis(random() % 41));
        child.kill().expect("the writer can be killed");
        child.wait().expect("the writer can be reaped");

        let main = dir.join(SETTINGS_FILE);
        let backup = dir.join(BACKUP_FILE);
        assert!(
            parses(&main) || (!main.exists() && parses(&backup)),
            "round {round}: {SETTINGS_FILE} is corrupt and there is no good backup"
        );
        if !temporary_files(&dir).is_empty() {
            left_a_temporary_file += 1;
        }
        let (_store, _) = Store::open(&dir, None).expect("the store opens after a kill");
        assert!(
            temporary_files(&dir).is_empty(),
            "round {round}: a stale temporary file survived open"
        );
    }
    println!("{left_a_temporary_file} of {ROUNDS} kills left a temporary file behind");
}
