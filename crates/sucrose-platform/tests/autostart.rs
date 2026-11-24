//! Starting with the OS (TRAY-03, D-08, D-034): the text builders run on every OS, the file entries run on a scratch
//! folder, and the real registry runs only in the ignored Windows tests.

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use sucrose_platform::autostart::{
    DESKTOP_FILE_NAME, EntryKind, LAUNCH_AGENT_LABEL, RUN_VALUE_NAME, config_dir_from, desktop_entry, disabled_by_os,
    entry_path, entry_state, entry_text, launch_agent_plist, remove_entry, run_value_data, write_entry,
};

/// A scratch folder of its own, removed when the test ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("sucrose-autostart-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("the scratch folder can be created");
        Self(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn autostart_names_are_the_ones_d034_records() {
    assert_eq!(RUN_VALUE_NAME, "Sucrose.Service");
    assert_eq!(LAUNCH_AGENT_LABEL, "dev.sucrose.Sucrose");
    assert_eq!(DESKTOP_FILE_NAME, "sucrose.desktop");
}

#[test]
fn autostart_is_a_capability_on_every_supported_os() {
    assert!(sucrose_platform::current().capabilities().autostart);
}

#[test]
fn autostart_plist_carries_the_label_the_escaped_path_and_run_at_load() {
    let plist = launch_agent_plist("dev.sucrose.Sucrose", "/Applications/A & B <x>.app/sucrose");

    assert!(plist.contains("<string>dev.sucrose.Sucrose</string>"), "{plist}");
    assert!(
        plist.contains("<string>/Applications/A &amp; B &lt;x&gt;.app/sucrose</string>"),
        "{plist}"
    );
    assert!(!plist.contains("A & B"), "the raw path must not appear: {plist}");
    assert!(plist.contains("<key>ProgramArguments</key>"), "{plist}");
    assert!(plist.contains("<key>RunAtLoad</key>\n\t<true/>"), "{plist}");
}

#[test]
fn autostart_plist_escapes_the_label_too() {
    let plist = launch_agent_plist("a<b", "/x");

    assert!(plist.contains("<string>a&lt;b</string>"), "{plist}");
}

#[test]
fn autostart_desktop_entry_keeps_spaces_in_a_quoted_exec() {
    let entry = desktop_entry("Sucrose", "/opt/my apps/sucrose");

    assert!(entry.starts_with("[Desktop Entry]\n"), "{entry}");
    assert!(entry.contains("\nType=Application\n"), "{entry}");
    assert!(entry.contains("\nName=Sucrose\n"), "{entry}");
    assert!(entry.contains("\nExec=\"/opt/my apps/sucrose\"\n"), "{entry}");
}

#[test]
fn autostart_desktop_entry_leaves_a_plain_path_unquoted() {
    assert!(desktop_entry("Sucrose", "/opt/sucrose/sucrose").contains("\nExec=/opt/sucrose/sucrose\n"));
}

#[test]
fn autostart_desktop_entry_escapes_quote_backslash_backtick_and_dollar() {
    // The quoting rule puts a backslash before each of them, then the string rule doubles every backslash.
    let entry = desktop_entry("Sucrose", "/o/a\"b\\c\u{60}d$e/s");

    assert!(entry.contains("\nExec=\"/o/a\\\\\"b\\\\\\\\c\\\\\u{60}d\\\\$e/s\"\n"), "{entry}");
}

#[test]
fn autostart_desktop_entry_doubles_percent_and_cannot_be_split_by_a_newline() {
    let entry = desktop_entry("Sucrose", "/o/100%/a\nExec=evil");

    assert!(entry.contains("100%%"), "{entry}");
    assert_eq!(entry.matches("\nExec=").count(), 1, "a newline in the path made a second key: {entry}");
}

#[test]
fn autostart_desktop_entry_name_cannot_inject_a_key() {
    let entry = desktop_entry("Su\nExec=evil", "/x");

    assert_eq!(entry.matches("\nExec=").count(), 1, "{entry}");
}

#[test]
fn autostart_run_value_is_the_quoted_path_with_no_arguments() {
    let data = run_value_data(Path::new(r"C:\Program Files\Sucrose\sucrose.exe")).expect("a usable path");

    assert_eq!(data, r#""C:\Program Files\Sucrose\sucrose.exe""#);
}

#[test]
fn autostart_run_value_drops_the_verbatim_prefix_before_a_drive_letter() {
    let data = run_value_data(Path::new(r"\\?\C:\Sucrose\sucrose.exe")).expect("a usable path");

    assert_eq!(data, r#""C:\Sucrose\sucrose.exe""#);
}

#[test]
fn autostart_run_value_refuses_a_quote_an_empty_path_and_control_characters() {
    assert!(run_value_data(Path::new(r#"C:\a"b\sucrose.exe"#)).is_err());
    assert!(run_value_data(Path::new("")).is_err());
    assert!(run_value_data(Path::new("C:\\a\nb\\sucrose.exe")).is_err());
}

#[test]
fn autostart_startup_approved_first_byte_decides() {
    assert!(!disabled_by_os(&[0x02, 0, 0, 0]));
    assert!(!disabled_by_os(&[0x06, 0, 0, 0]));
    assert!(disabled_by_os(&[0x03, 0, 0, 0]));
    assert!(disabled_by_os(&[0x07, 0, 0, 0]));
    assert!(!disabled_by_os(&[]), "a missing value counts as enabled");
}

#[test]
fn autostart_config_dir_prefers_an_absolute_xdg_folder_then_home() {
    let absolute = if cfg!(windows) { r"C:\cfg" } else { "/cfg" };
    assert_eq!(
        config_dir_from(Some(OsStr::new(absolute)), Some(OsStr::new("/home/a"))),
        Some(PathBuf::from(absolute))
    );
    assert_eq!(
        config_dir_from(Some(OsStr::new("relative")), Some(OsStr::new("/home/a"))),
        Some(PathBuf::from("/home/a").join(".config"))
    );
    assert_eq!(
        config_dir_from(Some(OsStr::new("")), Some(OsStr::new("/home/a"))),
        Some(PathBuf::from("/home/a").join(".config"))
    );
    assert_eq!(
        config_dir_from(None, Some(OsStr::new("/home/a"))),
        Some(PathBuf::from("/home/a").join(".config"))
    );
    assert_eq!(config_dir_from(None, None), None);
}

#[test]
fn autostart_entry_paths_follow_the_os_conventions() {
    let home = Path::new("/h");
    assert_eq!(
        entry_path(EntryKind::LaunchAgent, home),
        home.join("Library").join("LaunchAgents").join("dev.sucrose.Sucrose.plist")
    );
    assert_eq!(
        entry_path(EntryKind::Desktop, home),
        home.join("autostart").join("sucrose.desktop")
    );
}

#[test]
fn autostart_launch_agent_is_written_found_and_removed_alone() {
    let scratch = Scratch::new("agent");
    let exe = Path::new("/Applications/Sucrose.app/Contents/MacOS/sucrose");
    let sibling = entry_path(EntryKind::LaunchAgent, &scratch.0).with_file_name("com.other.plist");
    std::fs::create_dir_all(sibling.parent().expect("a parent")).expect("the folder can be created");
    std::fs::write(&sibling, "other").expect("the sibling can be written");

    assert!(!entry_state(EntryKind::LaunchAgent, &scratch.0, exe).entry_present);
    write_entry(EntryKind::LaunchAgent, &scratch.0, exe).expect("the entry is written");
    let written = std::fs::read_to_string(entry_path(EntryKind::LaunchAgent, &scratch.0)).expect("the file exists");
    assert_eq!(written, entry_text(EntryKind::LaunchAgent, exe).expect("text"));
    assert!(entry_state(EntryKind::LaunchAgent, &scratch.0, exe).entry_present);

    remove_entry(EntryKind::LaunchAgent, &scratch.0).expect("the entry is removed");
    assert!(!entry_path(EntryKind::LaunchAgent, &scratch.0).exists());
    assert!(!entry_state(EntryKind::LaunchAgent, &scratch.0, exe).entry_present);
    assert_eq!(std::fs::read_to_string(&sibling).expect("the sibling stays"), "other");
    remove_entry(EntryKind::LaunchAgent, &scratch.0).expect("removing a missing entry is fine");
}

#[test]
fn autostart_desktop_file_is_written_found_and_removed_alone() {
    let scratch = Scratch::new("desktop");
    let exe = Path::new("/opt/my apps/sucrose");
    let sibling = entry_path(EntryKind::Desktop, &scratch.0).with_file_name("other.desktop");
    std::fs::create_dir_all(sibling.parent().expect("a parent")).expect("the folder can be created");
    std::fs::write(&sibling, "other").expect("the sibling can be written");

    write_entry(EntryKind::Desktop, &scratch.0, exe).expect("the entry is written");
    let written = std::fs::read_to_string(entry_path(EntryKind::Desktop, &scratch.0)).expect("the file exists");
    assert!(written.contains("Exec=\"/opt/my apps/sucrose\""), "{written}");
    assert!(entry_state(EntryKind::Desktop, &scratch.0, exe).entry_present);

    remove_entry(EntryKind::Desktop, &scratch.0).expect("the entry is removed");
    assert!(!entry_state(EntryKind::Desktop, &scratch.0, exe).entry_present);
    assert_eq!(std::fs::read_to_string(&sibling).expect("the sibling stays"), "other");
}

#[test]
fn autostart_an_entry_for_another_executable_is_not_this_ones() {
    let scratch = Scratch::new("moved");
    write_entry(EntryKind::Desktop, &scratch.0, Path::new("/old/place/sucrose")).expect("the entry is written");

    let state = entry_state(EntryKind::Desktop, &scratch.0, Path::new("/new/place/sucrose"));

    assert!(!state.entry_present);
    assert!(!state.disabled_by_os);
    assert!(!state.legacy_entry);
}

#[cfg(windows)]
mod registry {
    use std::process::Command;

    use sucrose_platform::{autostart_state, disable_autostart, enable_autostart};

    const RUN: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    const APPROVED: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

    fn reg(args: &[&str]) -> bool {
        Command::new("reg.exe")
            .args(args)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn has(key: &str, name: &str) -> bool {
        reg(&["query", key, "/v", name])
    }

    /// Removes what the test created even when an assertion fails.
    struct Cleanup {
        own: bool,
        legacy: bool,
    }

    impl Drop for Cleanup {
        fn drop(&mut self) {
            if self.own {
                let _ = disable_autostart();
            }
            if self.legacy {
                let _ = reg(&["delete", RUN, "/v", "Sucrose", "/f"]);
            }
        }
    }

    #[test]
    #[ignore = "writes the real HKCU Run key, run it with --ignored"]
    fn autostart_registry_enable_disable_leaves_the_csharp_entry_alone() {
        // The maintainer's own entries are never touched: with one present the test does nothing.
        if has(RUN, "Sucrose.Service") {
            eprintln!("a Sucrose.Service value exists already, the test leaves it alone");
            return;
        }
        let legacy_existed = has(RUN, "Sucrose");
        if !legacy_existed {
            assert!(reg(&["add", RUN, "/v", "Sucrose", "/t", "REG_SZ", "/d", "legacy-test", "/f"]));
        }
        let _cleanup = Cleanup {
            own: true,
            legacy: !legacy_existed,
        };
        let exe = std::env::current_exe().expect("the test executable");

        assert!(!autostart_state(&exe).entry_present);
        enable_autostart(&exe).expect("the entry is written");
        let state = autostart_state(&exe);
        assert!(state.entry_present, "{state:?}");
        assert!(!state.disabled_by_os, "{state:?}");
        assert!(has(RUN, "Sucrose.Service"));

        disable_autostart().expect("the entry is removed");

        assert!(!has(RUN, "Sucrose.Service"));
        assert!(has(RUN, "Sucrose"), "the C# value must still be there");
        assert!(!autostart_state(&exe).entry_present);
    }

    #[test]
    #[ignore = "writes the real HKCU Run key, run it with --ignored"]
    fn autostart_registry_enable_clears_the_task_manager_flag() {
        if has(RUN, "Sucrose.Service") {
            eprintln!("a Sucrose.Service value exists already, the test leaves it alone");
            return;
        }
        let _cleanup = Cleanup {
            own: true,
            legacy: false,
        };
        let exe = std::env::current_exe().expect("the test executable");
        assert!(reg(&[
            "add",
            APPROVED,
            "/v",
            "Sucrose.Service",
            "/t",
            "REG_BINARY",
            "/d",
            "030000000000000000000000",
            "/f"
        ]));
        // The flag alone is not an entry: the OS ignores a flag without its Run value.
        assert!(!autostart_state(&exe).disabled_by_os);

        enable_autostart(&exe).expect("the entry is written");

        assert!(!has(APPROVED, "Sucrose.Service"), "enabling resets Task Manager's flag");
        assert!(!autostart_state(&exe).disabled_by_os);
    }

    #[test]
    #[ignore = "writes the real HKCU Run key, run it with --ignored"]
    fn autostart_registry_reports_an_entry_task_manager_turned_off() {
        if has(RUN, "Sucrose.Service") {
            eprintln!("a Sucrose.Service value exists already, the test leaves it alone");
            return;
        }
        let _cleanup = Cleanup {
            own: true,
            legacy: false,
        };
        let exe = std::env::current_exe().expect("the test executable");
        enable_autostart(&exe).expect("the entry is written");
        assert!(reg(&[
            "add",
            APPROVED,
            "/v",
            "Sucrose.Service",
            "/t",
            "REG_BINARY",
            "/d",
            "030000000000000000000000",
            "/f"
        ]));

        let state = autostart_state(&exe);

        assert!(state.entry_present, "{state:?}");
        assert!(state.disabled_by_os, "{state:?}");
    }
}
